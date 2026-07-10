//! Store-backed I/O: the `store` module's C surface (`src/store/ffi.rs`) plus the
//! schematic-level transparent open/save and format-manager queries
//! (`ffi/store_io.rs`), unified in one module.
//!
//! Omitted from port: `nuc_store_free` — destructor is generated.
//! Omitted from port: `nuc_store_last_error` — error transport is generated.
//! Omitted from port: `nuc_store_string_free`, `nuc_store_bytes_free` — buffer
//! memory management is obsolete (strings cross via `DiplomatWrite`, bytes as
//! base64 per PORTING rule 6).

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use base64::Engine;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// A key/value store opened from a URL (e.g. `mem://`, `file:///path`,
    /// `s3://bucket/prefix`, `redis://…`, `postgres://…`).
    #[diplomat::opaque]
    pub struct Store(pub(crate) Box<dyn crate::store::Store>);

    impl Store {
        fn utf8(s: &[u8]) -> Result<&str, NucleationError> {
            std::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)
        }

        /// Open a store from a URL. Errors with `Store` on an unknown scheme or
        /// connection failure.
        pub fn open(url: &DiplomatStr) -> Result<Box<Store>, NucleationError> {
            let url = Self::utf8(url)?;
            crate::store::open(url)
                .map(|s| Box::new(Store(s)))
                .map_err(|_| NucleationError::Store)
        }

        /// Fetch `key`, writing the value as base64 (PORTING rule 6). Errors with
        /// `NotFound` when the key is absent.
        pub fn get_b64(
            &self,
            key: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let key = Self::utf8(key)?;
            match self.0.get(key).map_err(|_| NucleationError::Store)? {
                Some(bytes) => {
                    let _ = write!(
                        out,
                        "{}",
                        base64::engine::general_purpose::STANDARD.encode(&bytes)
                    );
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        /// Store `data` at `key`.
        pub fn put(&self, key: &DiplomatStr, data: &[u8]) -> Result<(), NucleationError> {
            let key = Self::utf8(key)?;
            self.0.put(key, data).map_err(|_| NucleationError::Store)
        }

        /// Whether `key` exists.
        pub fn exists(&self, key: &DiplomatStr) -> Result<bool, NucleationError> {
            let key = Self::utf8(key)?;
            self.0.exists(key).map_err(|_| NucleationError::Store)
        }

        /// Delete `key` (idempotent).
        pub fn delete(&self, key: &DiplomatStr) -> Result<(), NucleationError> {
            let key = Self::utf8(key)?;
            self.0.delete(key).map_err(|_| NucleationError::Store)
        }

        /// List keys under `prefix`, written as a JSON array string.
        pub fn list(
            &self,
            prefix: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let prefix = Self::utf8(prefix)?;
            let keys = self.0.list(prefix).map_err(|_| NucleationError::Store)?;
            let json = serde_json::to_string(&keys).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Atomically write `data` at `key` only if it does not already exist.
        /// Returns `true` if written, `false` if the key existed.
        pub fn put_if_absent(
            &self,
            key: &DiplomatStr,
            data: &[u8],
        ) -> Result<bool, NucleationError> {
            let key = Self::utf8(key)?;
            self.0
                .put_if_absent(key, data)
                .map_err(|_| NucleationError::Store)
        }

        /// A keyset page of keys under `prefix`. `after` is the exclusive cursor
        /// (empty string for the first page); at most `limit` keys are returned.
        /// Writes a JSON object string `{"keys":[...],"next":"…"|null}`.
        pub fn list_paginated(
            &self,
            prefix: &DiplomatStr,
            after: &DiplomatStr,
            limit: u32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let prefix = Self::utf8(prefix)?;
            let after = Self::utf8(after)?;
            let after = if after.is_empty() { None } else { Some(after) };
            let (keys, next) = self
                .0
                .list_paginated(prefix, after, limit as usize)
                .map_err(|_| NucleationError::Store)?;
            let json = serde_json::to_string(&serde_json::json!({ "keys": keys, "next": next }))
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Health check: `Ok` when the store is usable.
        pub fn health(&self) -> Result<(), NucleationError> {
            self.0.health().map_err(|_| NucleationError::Store)
        }

        /// Open a schematic stored at `key` in this store. Works for every
        /// backend, including `redis://`/`postgres://`/`mem://` that the
        /// single-string URI form (`StoreIo::open`) rejects.
        pub fn open_schematic(&self, key: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let key = Self::utf8(key)?;
            crate::UniversalSchematic::from_store(self.0.as_ref(), key)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Store)
        }

        /// Save a schematic at `key` in this store. `version` selects the format
        /// version (empty string = format default). Works for every backend,
        /// including `redis://`/`postgres://`/`mem://` that the single-string URI
        /// form (`StoreIo::save`) rejects.
        pub fn save_schematic(
            &self,
            schematic: &Schematic,
            key: &DiplomatStr,
            version: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let key = Self::utf8(key)?;
            let version = Self::utf8(version)?;
            let version = if version.is_empty() { None } else { Some(version) };
            schematic
                .0
                .save_to_store(self.0.as_ref(), key, version)
                .map_err(|_| NucleationError::Store)
        }
    }

    /// Namespace type for the URI-based transparent I/O and format-manager
    /// queries (PORTING rule 12).
    #[diplomat::opaque]
    pub struct StoreIo;

    impl StoreIo {
        fn utf8(s: &[u8]) -> Result<&str, NucleationError> {
            std::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)
        }

        /// Open a schematic from a URI: a local path, `file://...`, or
        /// `s3://bucket/key.schem`. The format is auto-detected from the URI's
        /// extension. Single-string URIs for `redis://`, `postgres://`, and
        /// `mem://` are rejected by the core resolver; use `Store::open_schematic`
        /// with an explicit store for those backends.
        pub fn open(uri: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let uri = Self::utf8(uri)?;
            crate::UniversalSchematic::open(uri)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Store)
        }

        /// Save a schematic to a URI: a local path, `file://...`, or
        /// `s3://bucket/key.schem`. The format is auto-detected from the URI's
        /// extension; `version` selects the format version (empty string =
        /// format default). Single-string URIs for `redis://`, `postgres://`, and
        /// `mem://` are rejected by the core resolver; use `Store::save_schematic`
        /// with an explicit store for those backends.
        pub fn save(
            schematic: &Schematic,
            uri: &DiplomatStr,
            version: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let uri = Self::utf8(uri)?;
            let version = Self::utf8(version)?;
            let version = if version.is_empty() { None } else { Some(version) };
            schematic
                .0
                .save(uri, version)
                .map_err(|_| NucleationError::Store)
        }

        /// The JSON schema describing the export settings of `format`. Errors
        /// with `NotFound` for an unknown format.
        pub fn export_settings_schema(
            format: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let format = Self::utf8(format)?;
            let manager = crate::formats::manager::get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let schema = manager
                .get_export_settings_schema(format)
                .ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{}", schema);
            Ok(())
        }

        /// The JSON schema describing the import settings of `format`. Errors
        /// with `NotFound` for an unknown format.
        pub fn import_settings_schema(
            format: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let format = Self::utf8(format)?;
            let manager = crate::formats::manager::get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let schema = manager
                .get_import_settings_schema(format)
                .ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{}", schema);
            Ok(())
        }

        /// The supported import formats, written as a JSON array string.
        pub fn supported_import_formats(out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let manager = crate::formats::manager::get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let json = serde_json::to_string(&manager.list_importers())
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// The supported export formats, written as a JSON array string.
        pub fn supported_export_formats(out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let manager = crate::formats::manager::get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let json = serde_json::to_string(&manager.list_exporters())
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// The known versions of an export format, written as a JSON array string
        /// (empty array for an unknown format, matching the old ABI).
        pub fn format_versions(
            format: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let format = Self::utf8(format)?;
            let manager = crate::formats::manager::get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let versions = manager.get_exporter_versions(format).unwrap_or_default();
            let json = serde_json::to_string(&versions).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// The default version of an export format. Errors with `NotFound` for an
        /// unknown format.
        pub fn default_format_version(
            format: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let format = Self::utf8(format)?;
            let manager = crate::formats::manager::get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let version = manager
                .get_exporter_default_version(format)
                .ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{}", version);
            Ok(())
        }
    }
}
