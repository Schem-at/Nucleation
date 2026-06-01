//! In-memory [`Store`] backed by a shared `HashMap`. Always available
//! (including WASM); the default backend for tests.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::{Result, Store, StoreError};

/// A `HashMap`-backed store. Cloning shares the same underlying data, so a
/// `MemStore` can be handed to many workers cheaply.
#[derive(Clone, Default)]
pub struct MemStore {
    inner: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl MemStore {
    fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, HashMap<String, Vec<u8>>>> {
        self.inner
            .read()
            .map_err(|_| StoreError::Other("MemStore lock poisoned".into()))
    }

    fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, HashMap<String, Vec<u8>>>> {
        self.inner
            .write()
            .map_err(|_| StoreError::Other("MemStore lock poisoned".into()))
    }
}

impl Store for MemStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.read()?.get(key).cloned())
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        self.write()?.insert(key.to_string(), bytes.to_vec());
        Ok(())
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.read()?.contains_key(key))
    }

    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<bool> {
        let mut map = self.write()?;
        if map.contains_key(key) {
            Ok(false)
        } else {
            map.insert(key.to_string(), bytes.to_vec());
            Ok(true)
        }
    }

    fn delete(&self, key: &str) -> Result<()> {
        self.write()?.remove(key);
        Ok(())
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        Ok(self
            .read()?
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    fn health(&self) -> Result<()> {
        // Reachable iff the lock isn't poisoned.
        self.read().map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_store_satisfies_contract() {
        crate::store::contract::run_contract(&MemStore::new());
    }
}
