//! Transparent schematic IO over the [`store`](crate::store) layer.
//!
//! Lets every binding open/save a schematic from a local path, a store URI
//! (`s3://bucket/key.schem`), or an explicit [`Store`] + key — with the format
//! inferred from the key/path extension. Implemented once here so all bindings
//! share identical scheme handling, errors, and format detection.

use std::error::Error;
use std::path::PathBuf;

use crate::store::{self, Store, StoreError};
use crate::universal_schematic::UniversalSchematic;

/// What a URI resolves to.
pub enum Target {
    /// A local filesystem path (plain paths and `file://`).
    Local(PathBuf),
    /// A remote store plus the object key within it.
    Remote(Box<dyn Store>, String),
}

/// Resolve a URI/path into a [`Target`].
///
/// - no scheme, or `file://` → [`Target::Local`]
/// - `s3://bucket/key` → [`Target::Remote`] (the whole path after the bucket is
///   the key)
/// - `redis://` / `postgres://` / `mem://` single-strings are rejected: their
///   URL has no slot for an object key, so open them with an explicit store
///   (`UniversalSchematic::from_store`).
pub fn resolve(uri: &str) -> Result<Target, StoreError> {
    match uri.split_once("://") {
        None => Ok(Target::Local(PathBuf::from(uri))),
        Some(("file", path)) => Ok(Target::Local(PathBuf::from(path))),
        Some(("s3", rest)) => {
            let (bucket, key) = rest
                .split_once('/')
                .filter(|(_, k)| !k.is_empty())
                .ok_or_else(|| {
                    StoreError::InvalidKey(format!(
                        "S3 URI needs an object key: s3://{rest}/key.schem"
                    ))
                })?;
            let store = store::open(&format!("s3://{bucket}"))?;
            Ok(Target::Remote(store, key.to_string()))
        }
        Some((scheme @ ("redis" | "rediss" | "postgres" | "postgresql" | "mem"), _)) => {
            Err(StoreError::Unsupported(format!(
                "{scheme}:// URIs can't carry an object key in one string; \
                 open with an explicit store (from_store / save_to_store)"
            )))
        }
        Some((scheme, _)) => Err(StoreError::Unsupported(format!(
            "unknown URI scheme `{scheme}`"
        ))),
    }
}

fn read_manager(bytes: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
    let arc = crate::formats::manager::get_manager();
    let manager = arc
        .lock()
        .map_err(|_| "format manager lock poisoned".to_string())?;
    Ok(manager.read(bytes)?)
}

fn write_manager(
    schematic: &UniversalSchematic,
    key_or_path: &str,
    version: Option<&str>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let arc = crate::formats::manager::get_manager();
    let manager = arc
        .lock()
        .map_err(|_| "format manager lock poisoned".to_string())?;
    Ok(manager.write_auto(key_or_path, schematic, version)?)
}

impl UniversalSchematic {
    /// Load a schematic from a file path or store URI. Format is inferred from
    /// the extension.
    pub fn open(uri: &str) -> Result<Self, Box<dyn Error>> {
        match resolve(uri)? {
            Target::Local(path) => read_manager(&std::fs::read(path)?),
            Target::Remote(store, key) => Self::from_store(store.as_ref(), &key),
        }
    }

    /// Load a schematic from an explicit store at `key` (works for any backend).
    pub fn from_store(store: &dyn Store, key: &str) -> Result<Self, Box<dyn Error>> {
        let bytes = store
            .get(key)?
            .ok_or_else(|| StoreError::NotFound(key.to_string()))?;
        read_manager(&bytes)
    }

    /// Save to a file path or store URI. Format is inferred from the extension;
    /// `version` is the format version (or `None` for the default).
    pub fn save(&self, uri: &str, version: Option<&str>) -> Result<(), Box<dyn Error>> {
        match resolve(uri)? {
            Target::Local(path) => {
                let bytes = write_manager(self, &path.to_string_lossy(), version)?;
                std::fs::write(path, bytes)?;
                Ok(())
            }
            Target::Remote(store, key) => self.save_to_store(store.as_ref(), &key, version),
        }
    }

    /// Save to an explicit store at `key`. Format inferred from the key's
    /// extension.
    pub fn save_to_store(
        &self,
        store: &dyn Store,
        key: &str,
        version: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = write_manager(self, key, version)?;
        store.put(key, &bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_state::BlockState;
    use crate::store::MemStore;

    fn sample() -> UniversalSchematic {
        let mut s = UniversalSchematic::new("x".to_string());
        s.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
        s
    }

    #[test]
    fn explicit_store_roundtrip() {
        let store = MemStore::new();
        sample()
            .save_to_store(&store, "builds/x.schem", None)
            .unwrap();
        assert!(store.exists("builds/x.schem").unwrap());
        let loaded = UniversalSchematic::from_store(&store, "builds/x.schem").unwrap();
        assert!(
            loaded.get_tight_bounds().is_some(),
            "loaded schematic non-empty"
        );
    }

    #[test]
    fn local_file_roundtrip() {
        let dir = std::env::temp_dir().join(format!("nuc-storeio-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("x.litematic");
        let p = path.to_str().unwrap();
        sample().save(p, None).unwrap();
        let loaded = UniversalSchematic::open(p).unwrap();
        assert!(loaded.get_tight_bounds().is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_rules() {
        assert!(matches!(resolve("build.schem").unwrap(), Target::Local(_)));
        assert!(matches!(
            resolve("file:///tmp/a.schem").unwrap(),
            Target::Local(_)
        ));
        assert!(resolve("redis://h/x").is_err());
        assert!(resolve("postgres://h/db").is_err());
        assert!(resolve("ftp://h/x").is_err());
    }
}
