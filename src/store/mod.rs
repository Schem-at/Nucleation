//! Pluggable byte storage — the abstraction every higher Nucleation layer
//! (codecs, the ingestion pipeline, the stitch frontier) reads and writes
//! through, independent of *where* the bytes live.
//!
//! `Store` answers only one question: *given a key, move these bytes in or
//! out.* It deliberately knows nothing about schematics, images, or meshes —
//! that "what format" concern belongs to codecs. Keys are UTF-8, `/`-delimited
//! logical paths; `list(prefix)` enumerates under a prefix.
//!
//! Backends are batteries-included but feature-gated: [`MemStore`] is always
//! available (and the WASM default), [`FsStore`] ships by default on native
//! targets, and networked backends (S3 / Redis / Postgres) live behind their
//! own Cargo features so you only pull the dependencies you enable.
//!
//! The trait is **synchronous** and its implementors are `Send + Sync` cheap
//! handles, so a single store clones freely across worker threads. Backends
//! built on async SDKs manage their own runtime internally.

use std::io::{Read, Write};

mod error;
pub use error::StoreError;

pub mod mem;
pub use mem::MemStore;

#[cfg(all(feature = "store-fs", not(target_arch = "wasm32")))]
pub mod fs;
#[cfg(all(feature = "store-fs", not(target_arch = "wasm32")))]
pub use fs::FsStore;

#[cfg(feature = "store-callback")]
pub mod callback;
#[cfg(feature = "store-callback")]
pub use callback::{CallbackStore, CallbackStoreBuilder};

#[cfg(feature = "store-s3")]
pub mod s3;
#[cfg(feature = "store-s3")]
pub use s3::{S3Config, S3Store};

#[cfg(feature = "store-redis")]
pub mod redis;
#[cfg(feature = "store-redis")]
pub use redis::{RedisConfig, RedisStore};

#[cfg(feature = "store-pg")]
pub mod pg;
#[cfg(feature = "store-pg")]
pub use pg::{PgConfig, PgStore};


/// Backend-agnostic behavioural contract suite. Public when `store-testkit` is
/// enabled so integration tests (e.g. testcontainers) can exercise any backend.
#[cfg(any(test, feature = "store-testkit"))]
pub mod contract;

/// Convenience alias for store results.
pub type Result<T> = std::result::Result<T, StoreError>;

/// A location-agnostic byte store keyed by `/`-delimited UTF-8 paths.
///
/// Semantics every backend must honour (enforced by the shared contract
/// suite in [`contract`]):
/// - `put` overwrites an existing key.
/// - `get` on a missing key returns `Ok(None)`; `reader` returns
///   [`StoreError::NotFound`].
/// - `delete` of a missing key is idempotent (`Ok`).
/// - `list(prefix)` returns every key for which `key.starts_with(prefix)`.
pub trait Store: Send + Sync {
    /// Fetch the bytes at `key`, or `None` if absent.
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Store `bytes` at `key`, overwriting any existing object.
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()>;

    /// Whether an object exists at `key`.
    fn exists(&self, key: &str) -> Result<bool>;

    /// Remove `key`. Removing a missing key succeeds (idempotent).
    fn delete(&self, key: &str) -> Result<()>;

    /// Every key under `prefix` (i.e. `key.starts_with(prefix)`). Order is
    /// unspecified.
    fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// Cheap reachability/credentials check. `Ok(())` means usable.
    fn health(&self) -> Result<()>;

    /// Atomically write `bytes` at `key` only if no object exists there.
    /// Returns `true` if it was written, `false` if the key already existed.
    ///
    /// The default is a non-atomic `exists`-then-`put`, safe only without
    /// concurrent writers. Networked backends override it with a genuinely
    /// atomic operation (S3 `If-None-Match`, Redis `SET NX`, Postgres
    /// `ON CONFLICT DO NOTHING`).
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<bool> {
        if self.exists(key)? {
            Ok(false)
        } else {
            self.put(key, bytes)?;
            Ok(true)
        }
    }

    /// A keyset page of keys under `prefix`, sorted ascending, starting strictly
    /// after `after` (exclusive), at most `limit` keys. Returns the page plus a
    /// cursor to pass as the next `after` when more keys may remain (`None` once
    /// exhausted). `limit == 0` yields an empty page.
    ///
    /// The default lists everything and slices in memory; S3 and Postgres
    /// override it with native keyset pagination (`start-after` / `key > $after`).
    fn list_paginated(
        &self,
        prefix: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<(Vec<String>, Option<String>)> {
        let mut all = self.list(prefix)?;
        all.sort();
        let start = match after {
            Some(a) => all.partition_point(|k| k.as_str() <= a),
            None => 0,
        };
        let page: Vec<String> = all.into_iter().skip(start).take(limit).collect();
        let next = if limit > 0 && page.len() == limit {
            page.last().cloned()
        } else {
            None
        };
        Ok((page, next))
    }

    /// Streaming read of `key`. The default buffers via [`Store::get`];
    /// backends with native streaming may override.
    fn reader(&self, key: &str) -> Result<Box<dyn Read + '_>> {
        match self.get(key)? {
            Some(bytes) => Ok(Box::new(std::io::Cursor::new(bytes))),
            None => Err(StoreError::NotFound(key.to_string())),
        }
    }

    /// Streaming write of `key`. The default buffers in memory and commits via
    /// [`Store::put`] on `flush`/drop; backends with native streaming may
    /// override. Object-safe: callable on `&dyn Store`.
    fn writer(&self, key: &str) -> Result<Box<dyn Write + '_>> {
        Ok(Box::new(BufferingWriter::new(self, key.to_string())))
    }
}

/// Default [`Store::writer`]: accumulate in memory, commit on `flush`/drop.
struct BufferingWriter<'a, S: Store + ?Sized> {
    store: &'a S,
    key: String,
    buf: Vec<u8>,
    committed: bool,
}

impl<'a, S: Store + ?Sized> BufferingWriter<'a, S> {
    fn new(store: &'a S, key: String) -> Self {
        Self {
            store,
            key,
            buf: Vec::new(),
            committed: false,
        }
    }

    fn commit(&mut self) -> std::io::Result<()> {
        self.store
            .put(&self.key, &self.buf)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        self.committed = true;
        Ok(())
    }
}

impl<S: Store + ?Sized> Write for BufferingWriter<'_, S> {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.commit()
    }
}

impl<S: Store + ?Sized> Drop for BufferingWriter<'_, S> {
    fn drop(&mut self) {
        if !self.committed {
            if let Err(e) = self.commit() {
                // `Drop` can't return a `Result`, so a commit-on-drop failure
                // would otherwise be silent data loss. Make it loud, and trip in
                // debug builds — callers who need to handle errors should
                // `flush()` explicitly before dropping the writer.
                eprintln!(
                    "nucleation::store: writer for key {:?} failed to commit on drop \
                     ({e}); data was NOT persisted. Call flush() to handle this error.",
                    self.key
                );
                debug_assert!(false, "store writer failed to commit on drop: {e}");
            }
        }
    }
}

/// Drive a future to completion, blocking the current thread.
///
/// Safe to call from *within* a Tokio runtime: if it detects an active runtime
/// it offloads to a scoped thread, so `Runtime::block_on` never panics with
/// "Cannot start a runtime from within a runtime". Used by the async-SDK
/// backends (`s3`/`redis`/`pg`) to present a sync `Store` API.
#[cfg(any(feature = "store-s3", feature = "store-redis", feature = "store-pg"))]
pub(crate) fn block_on<F>(rt: &tokio::runtime::Runtime, fut: F) -> F::Output
where
    F: std::future::Future + Send,
    F::Output: Send,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        // We're on a runtime thread — blocking here would panic. Run the future
        // on a separate (non-runtime) thread and wait for it.
        std::thread::scope(|s| s.spawn(|| rt.block_on(fut)).join().unwrap())
    } else {
        rt.block_on(fut)
    }
}

#[cfg(test)]
#[cfg(any(feature = "store-s3", feature = "store-redis", feature = "store-pg"))]
mod block_on_tests {
    #[test]
    fn block_on_from_within_a_runtime_does_not_panic() {
        let outer = tokio::runtime::Runtime::new().unwrap();
        let inner = tokio::runtime::Runtime::new().unwrap();
        // Inside `outer`'s context, the guarded block_on must offload rather than
        // panic with "Cannot start a runtime from within a runtime".
        let result = outer.block_on(async { super::block_on(&inner, async { 21 + 21 }) });
        assert_eq!(result, 42);
    }
}

/// Construct a store from a URL/DSN.
///
/// Supported schemes depend on enabled features:
/// `mem://`, `file:///abs/path` (with `store-fs`), and — once their features
/// are enabled — `s3://`, `redis://`, `postgres://`.
pub fn open(url: &str) -> Result<Box<dyn Store>> {
    if url == "mem://" || url.starts_with("mem://") {
        return Ok(Box::new(MemStore::new()));
    }

    #[cfg(all(feature = "store-fs", not(target_arch = "wasm32")))]
    if let Some(path) = url.strip_prefix("file://") {
        return Ok(Box::new(FsStore::new(path)));
    }

    #[cfg(feature = "store-s3")]
    if let Some(rest) = url.strip_prefix("s3://") {
        let (bucket, prefix) = match rest.split_once('/') {
            Some((b, p)) => (b.to_string(), p.to_string()),
            None => (rest.to_string(), String::new()),
        };
        let cfg = s3::S3Config {
            prefix,
            region: std::env::var("AWS_REGION").ok(),
            endpoint: std::env::var("AWS_ENDPOINT_URL").ok(),
            force_path_style: matches!(
                std::env::var("AWS_S3_FORCE_PATH_STYLE").as_deref(),
                Ok("true") | Ok("1")
            ),
            ..s3::S3Config::new(bucket)
        };
        return Ok(Box::new(s3::S3Store::connect(cfg)?));
    }

    #[cfg(feature = "store-redis")]
    if url.starts_with("redis://") || url.starts_with("rediss://") {
        return Ok(Box::new(redis::RedisStore::connect(
            redis::RedisConfig::new(url),
        )?));
    }

    #[cfg(feature = "store-pg")]
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        let table =
            std::env::var("NUC_STORE_PG_TABLE").unwrap_or_else(|_| "nucleation_store".to_string());
        return Ok(Box::new(pg::PgStore::connect(pg::PgConfig::new(
            url, table,
        ))?));
    }

    let scheme = url.split("://").next().unwrap_or(url);
    Err(StoreError::Unsupported(format!(
        "no store backend for scheme `{scheme}` (is its feature enabled?)"
    )))
}

#[cfg(test)]
mod open_tests {
    use super::*;

    #[test]
    fn mem_scheme_opens_a_working_store() {
        let store = open("mem://").expect("open mem");
        store.put("k", b"v").expect("put");
        assert_eq!(store.get("k").expect("get"), Some(b"v".to_vec()));
    }

    #[test]
    fn unknown_scheme_is_unsupported() {
        match open("ftp://host/path") {
            Err(StoreError::Unsupported(_)) => {}
            Err(other) => panic!("expected Unsupported, got {other:?}"),
            Ok(_) => panic!("expected Unsupported, got Ok"),
        }
    }

    #[cfg(all(feature = "store-fs", not(target_arch = "wasm32")))]
    #[test]
    fn file_scheme_opens_an_fs_store() {
        let dir = std::env::temp_dir().join(format!("nucleation-open-fs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let url = format!("file://{}", dir.display());
        let store = open(&url).expect("open file");
        store.put("a/b", b"xy").expect("put");
        assert_eq!(store.get("a/b").expect("get"), Some(b"xy".to_vec()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
