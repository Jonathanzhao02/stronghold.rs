// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! macro crate for policy engine
//!
//! ~the macro to implement policies is `#[policy(..)]` with two mandatory (`allow=[".."]`, and `deny=[""]`),
//! and one optional (`name=""`) attribute. This macro can be used to either decorate structs or function-calls.
//! decorating types with the attribute macro allows the user to disallow execution of certain types,
//! as their function calls will be intercepted and resulting in an error, but not in a runtime panic.
//! Functions may also be decorated allowing, or disallowing certain input types (TODO DEFINE!), as well as
//! passing a function for more finegrained control, on what values are allowed and what not.~
//!
//! All impl. should be moved to somewhere else, since the macros implemented here are more of general use

use proc_macro::*;
use quote::quote;
use std::str::FromStr;

/// Creates a Hashmap
///
/// Convenience macro like vec![] to create a hashmap.
#[proc_macro]
pub fn map(input: TokenStream) -> TokenStream {
    let tokens = input.to_string();
    let tokens_str = tokens.as_str();

    // json spec expects an object
    let wrapped = format!("{{ {} }}", tokens_str);

    let result = quote! {
        serde_json::from_str(#wrapped)
            .unwrap_or_else(|e|
                panic!("Error: {}, Input: {}", e, #wrapped))
    };
    result.into()
}

#[proc_macro]
pub fn impl_count_tuples(input: proc_macro::TokenStream) -> TokenStream {
    let count = input.to_string().parse().unwrap();

    (1..count)
        .into_iter()
        .map(|n| {
            let mut generics = alphabetical(n).collect::<Vec<String>>().join(",");
            if n == 1 {
                generics.push(',')
            };
            let generics_tokens = proc_macro2::TokenStream::from_str(&generics).unwrap();

            quote! {
                impl<#generics_tokens> Count for (#generics_tokens) {
                    fn count(&self) -> usize {
                        #n
                    }
                }
            }
        })
        .collect::<proc_macro2::TokenStream>()
        .into()
}

/// Returns an iterator of [`String`]s of the alphabet (A-Z). If the last character
/// is 'Z', the next string returned will be 'AA' and so on..
fn alphabetical(count: usize) -> impl Iterator<Item = String> {
    let radix = 26;
    let end = ('A' as usize + radix) as u8 as char;
    let digits = (count as f64).log(radix as f64) as usize;
    let alpha = ('A'..end).collect::<Vec<char>>();

    (0..count).map(move |i| {
        let mut div = i;
        let mut index = 0;

        let mut result = vec!['A'; digits + 1];

        while div != 0 {
            if index > 0 {
                div -= 1;
            }
            let a_index = div % radix;
            result[digits - index] = alpha[a_index];
            index += 1;
            div /= radix;
        }

        result.iter().collect::<String>()
    })
}

/// todo move outside
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_alphabet_iterator() {
        let mut iter = alphabetical(32).skip(28);

        assert_eq!(iter.next(), Some("AC".to_string()));
        assert_eq!(iter.next(), Some("AD".to_string()));
        assert_eq!(iter.next(), Some("AE".to_string()));
        assert_eq!(iter.next(), Some("AF".to_string()));
        assert_eq!(iter.next(), None);
    }
}