// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// A wrapped [`HashMap`], that can store multiple types.
/// this feature is realized with the [`Any`] type. Retrieval
/// of any data of any type requires to use a reference of the given type,
/// otherwise the retrieval will fail. Values to be inserted must be wrapped
/// inside a [`Box`]. This offers a more flexible way to store multi types
/// in a single [`HashMap`] with little overhead, but incurs some performance
/// issues. This type is **not thread-safe**!
///
/// # Example
/// ```
/// use policyengine::types::AnyMap;
///
/// let mut map = AnyMap::default();
/// map.insert("key0", Box::new(0usize));
/// map.insert("key1", Box::new("hello"));
///
/// let n = match map.get::<&usize>("key0") {
///     Some(v) => v,
///     _ => panic!("No value present"),
/// };
/// let s = match map.get::<&&str>("key1") {
///     Some(v) => v,
///     _ => panic!("No value present"),
/// };
/// assert_eq!(*n, &0usize);
/// assert_eq!(*s, &"hello");
/// ```
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash,
};

#[derive(Default)]
pub struct AnyMap<K>
where
    K: Eq + Hash,
{
    /// Data storage for any type, that will be evaluated at runtime.
    data: HashMap<K, Box<dyn Any>>,
}

impl<K> AnyMap<K>
where
    K: Eq + Hash,
{
    /// Inserts some data into the map using the key of type `K`
    pub fn insert(&mut self, key: K, value: Box<dyn Any>) {
        self.data.insert(key, value);
    }

    /// Retrieves the stored generic value. Accessing the data
    /// must use a reference to get the generic data type
    pub fn get<T>(&self, key: K) -> Option<&T>
    where
        T: 'static,
    {
        if let Some(v) = self.data.get(&key) {
            if TypeId::of::<Box<dyn Any>>() == v.type_id() {
                // cast to target type
                let out = unsafe { &*(v as *const dyn Any as *const T) };

                // check if casted type is target type
                if TypeId::of::<T>() == out.type_id() {
                    return Some(out);
                }
            }
        }
        None
    }

    /// Clears all data inside the map
    pub fn clear(&mut self) {
        self.data.clear()
    }
}
