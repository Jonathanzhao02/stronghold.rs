// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Secure Client Actor State

use crate::{actors::VaultError, internals, line_error, state::key_store::KeyStore, utils::LoadFromPath, Location};
use engine::{
    store::Cache,
    vault::{ClientId, DbView, Key, RecordId, VaultId},
};
use std::{collections::HashSet, time::Duration};

/// Cache type definition
pub type Store = Cache<Vec<u8>, Vec<u8>>;

pub struct SecureClient {
    // A keystore
    pub(crate) keystore: KeyStore<internals::Provider>,
    // A view on the vault entries
    pub(crate) db: DbView<internals::Provider>,
    // The id of this client
    pub client_id: ClientId,
    // Contains the vault ids and the record ids with their associated indexes.
    pub vaults: HashSet<VaultId>,
    // Contains the Record Ids for the most recent Record in each vault.
    pub store: Store,
}

impl SecureClient {
    /// Creates a new Client given a `ClientID` and `ChannelRef<SHResults>`
    pub fn new(client_id: ClientId) -> Self {
        let vaults = HashSet::new();

        let store = Cache::new();

        Self {
            client_id,
            vaults,
            store,
            keystore: KeyStore::new(),
            db: DbView::new(),
        }
    }

    /// Write unencrypted data to the store.  Returns [`None`] if the key didn't already exist and [`Some(Vec<u8>)`] if
    /// the key was updated.
    pub fn write_to_store(&mut self, key: Vec<u8>, data: Vec<u8>, lifetime: Option<Duration>) -> Option<Vec<u8>> {
        self.store.insert(key, data, lifetime)
    }

    /// Attempts to read the data from the store.  Returns [`Some(Vec<u8>)`] if the key exists and [`None`] if it
    /// doesn't.
    pub fn read_from_store(&mut self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.store.get(&key).map(|v| v.to_vec())
    }

    /// Deletes an item from the store by the given key.
    pub fn store_delete_item(&mut self, key: Vec<u8>) {
        self.store.remove(&key);
    }

    /// Checks to see if the key exists in the store.
    pub fn store_key_exists(&mut self, key: Vec<u8>) -> bool {
        self.store.contains_key(&key)
    }

    /// Sets the client id to swap from one client to another.
    pub fn set_client_id(&mut self, client_id: ClientId) {
        self.client_id = client_id
    }

    /// Adds a new vault to the client.  If the [`VaultId`] already exists, will just use that existing Vault.
    pub fn add_new_vault(&mut self, vid: VaultId) {
        self.vaults.insert(vid);
    }

    /// Empty the Client Cache.
    pub fn clear_cache(&mut self) -> Option<()> {
        self.vaults = HashSet::default();

        Some(())
    }

    /// Rebuilds the cache using the parameters.
    pub fn rebuild_cache(&mut self, id: ClientId, vaults: HashSet<VaultId>, store: Store) {
        self.client_id = id;
        self.vaults = vaults;
        self.store = store;
    }

    /// Resolves a location to a `VaultId` and a `RecordId`
    pub fn resolve_location<L: AsRef<Location>>(l: L) -> (VaultId, RecordId) {
        match l.as_ref() {
            Location::Generic {
                vault_path,
                record_path,
            } => {
                let vid = Self::derive_vault_id(vault_path);
                let rid = RecordId::load_from_path(vid.as_ref(), record_path).expect(line_error!(""));
                (vid, rid)
            }
            Location::Counter { vault_path, counter } => {
                let vid = Self::derive_vault_id(vault_path);
                let rid = Self::derive_record_id(vault_path, *counter);

                (vid, rid)
            }
        }
    }

    /// Gets the [`VaultId`] from a specified path.
    pub fn derive_vault_id<P: AsRef<Vec<u8>>>(path: P) -> VaultId {
        VaultId::load_from_path(path.as_ref(), path.as_ref()).expect(line_error!(""))
    }

    /// Derives the counter [`RecordId`] from the given vault path and the counter value.
    pub fn derive_record_id<P: AsRef<Vec<u8>>>(vault_path: P, ctr: usize) -> RecordId {
        let vault_path = vault_path.as_ref();

        let path = if ctr == 0 {
            format!("{:?}{}", vault_path, "first_record")
        } else {
            format!("{:?}{}", vault_path, ctr)
        };

        RecordId::load_from_path(path.as_bytes(), path.as_bytes()).expect(line_error!())
    }

    /// Gets the client string.
    pub fn get_client_str(&self) -> String {
        self.client_id.into()
    }

    /// Checks to see if the vault exists in the client.
    pub fn vault_exist(&self, vid: VaultId) -> Option<&VaultId> {
        self.vaults.get(&vid)
    }

    /// Gets the current index of a record if its a counter.
    pub fn get_index_from_record_id<P: AsRef<Vec<u8>>>(&self, vault_path: P, record_id: RecordId) -> usize {
        let mut ctr = 0;
        let vault_path = vault_path.as_ref();

        while ctr <= 32_000_000 {
            let rid = Self::derive_record_id(vault_path, ctr);
            if record_id == rid {
                break;
            }
            ctr += 1;
        }

        ctr
    }

    pub fn get_key(&mut self, vault_id: VaultId) -> Result<Key<internals::Provider>, anyhow::Error> {
        let key = self
            .keystore
            .get_key(vault_id)
            .ok_or_else(|| anyhow::anyhow!(VaultError::NotExisting))?;
        self.keystore.insert_key(vault_id, key.clone());
        Ok(key)
    }

    pub fn get_or_create_key(&mut self, vault_id: VaultId) -> Result<Key<internals::Provider>, anyhow::Error> {
        let key = if !self.keystore.vault_exists(vault_id) {
            let k = self.keystore.create_key(vault_id);
            self.db.init_vault(&k, vault_id)?;
            k
        } else {
            self.keystore
                .get_key(vault_id)
                .ok_or_else(|| anyhow::anyhow!(crate::Error::KeyStoreError("".into())))?
        };
        self.keystore.insert_key(vault_id, key.clone());
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Provider;

    #[test]
    fn test_add() {
        let vid = VaultId::random::<Provider>().expect(line_error!());

        let mut cache: SecureClient = SecureClient::new(ClientId::random::<Provider>().expect(line_error!()));

        cache.add_new_vault(vid);

        assert_eq!(cache.vaults.get(&vid), Some(&vid));
    }

    #[test]
    fn test_rid_internals() {
        let clientid = ClientId::random::<Provider>().expect(line_error!());

        let vid = VaultId::random::<Provider>().expect(line_error!());
        let vid2 = VaultId::random::<Provider>().expect(line_error!());
        let vault_path = b"some_vault".to_vec();

        let mut client: SecureClient = SecureClient::new(clientid);
        let mut ctr = 0;
        let mut ctr2 = 0;

        let _rid = SecureClient::derive_record_id(vault_path.clone(), ctr);
        let _rid2 = SecureClient::derive_record_id(vault_path.clone(), ctr2);

        client.add_new_vault(vid);

        client.add_new_vault(vid2);

        ctr += 1;
        ctr2 += 1;

        let _rid = SecureClient::derive_record_id(vault_path.clone(), ctr);
        let _rid2 = SecureClient::derive_record_id(vault_path.clone(), ctr2);

        ctr += 1;

        let rid = SecureClient::derive_record_id(vault_path.clone(), ctr);

        let test_rid = SecureClient::derive_record_id(vault_path.clone(), 2);
        let ctr = client.get_index_from_record_id(vault_path, rid);

        assert_eq!(test_rid, rid);
        assert_eq!(2, ctr);
    }

    #[test]
    fn test_location_counter_api() {
        let clientid = ClientId::random::<Provider>().expect(line_error!());

        let vidlochead = Location::counter::<_, usize>("some_vault", 0);
        let vidlochead2 = Location::counter::<_, usize>("some_vault 2", 0);

        let mut client: SecureClient = SecureClient::new(clientid);

        let (vid, rid) = SecureClient::resolve_location(&vidlochead);
        let (vid2, rid2) = SecureClient::resolve_location(&vidlochead2);

        client.add_new_vault(vid);
        client.add_new_vault(vid2);

        let (_, rid_head) = SecureClient::resolve_location(&vidlochead);
        let (_, rid_head_2) = SecureClient::resolve_location(&vidlochead2);

        assert_eq!(rid, rid_head);
        assert_eq!(rid2, rid_head_2);
    }
}
