//! Python binding for the `store` module: a `Store` class over any backend.
//!
//! Which backends are reachable depends on the `store-*` features enabled when
//! the extension is built (e.g. build with `--features python,store-s3` to use
//! `s3://` URLs directly from Python).

use pyo3::exceptions::{PyIOError, PyKeyError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::store::{self, Store, StoreError};

fn map_err(e: StoreError) -> PyErr {
    match e {
        StoreError::NotFound(k) => PyKeyError::new_err(k),
        other => PyIOError::new_err(other.to_string()),
    }
}

/// A Python exception raised inside a callback becomes a store error.
fn to_store_err(e: PyErr) -> StoreError {
    StoreError::Other(e.to_string())
}

/// A byte store keyed by `/`-delimited paths. Construct with `Store.open(url)`.
#[pyclass(name = "Store")]
pub struct PyStore {
    inner: Box<dyn Store>,
}

#[pymethods]
impl PyStore {
    /// Open a store from a URL (`mem://`, `file:///path`, `s3://bucket/prefix`,
    /// `redis://…`, `postgres://…` — subject to enabled features).
    #[staticmethod]
    fn open(url: &str) -> PyResult<Self> {
        Ok(Self {
            inner: store::open(url).map_err(map_err)?,
        })
    }

    /// Fetch `key`, or `None` if absent.
    fn get<'py>(&self, py: Python<'py>, key: &str) -> PyResult<Option<Bound<'py, PyBytes>>> {
        match self.inner.get(key).map_err(map_err)? {
            Some(bytes) => Ok(Some(PyBytes::new(py, &bytes))),
            None => Ok(None),
        }
    }

    /// Store `data` at `key`, overwriting.
    fn put(&self, key: &str, data: &[u8]) -> PyResult<()> {
        self.inner.put(key, data).map_err(map_err)
    }

    /// Whether `key` exists.
    fn exists(&self, key: &str) -> PyResult<bool> {
        self.inner.exists(key).map_err(map_err)
    }

    /// Remove `key` (idempotent).
    fn delete(&self, key: &str) -> PyResult<()> {
        self.inner.delete(key).map_err(map_err)
    }

    /// Keys under `prefix`.
    fn list(&self, prefix: &str) -> PyResult<Vec<String>> {
        self.inner.list(prefix).map_err(map_err)
    }

    /// Raise if the store is not usable.
    fn health(&self) -> PyResult<()> {
        self.inner.health().map_err(map_err)
    }

    /// Atomically write `data` at `key` only if it does not already exist.
    /// Returns `True` if written, `False` if the key was already present.
    fn put_if_absent(&self, key: &str, data: &[u8]) -> PyResult<bool> {
        self.inner.put_if_absent(key, data).map_err(map_err)
    }

    /// A keyset page of keys under `prefix`, sorted, starting strictly after
    /// `after`, at most `limit`. Returns `(keys, next_cursor)`; `next_cursor` is
    /// `None` once exhausted (pass it back as `after` to continue).
    #[pyo3(signature = (prefix, after=None, limit=1000))]
    fn list_paginated(
        &self,
        prefix: &str,
        after: Option<&str>,
        limit: usize,
    ) -> PyResult<(Vec<String>, Option<String>)> {
        self.inner
            .list_paginated(prefix, after, limit)
            .map_err(map_err)
    }

    /// Build a store backed by Python callables. `handlers` is any object with
    /// synchronous methods: `get(key) -> bytes | None`, `put(key, bytes)`,
    /// `has(key) -> bool`, `delete(key)`, `list(prefix) -> list[str]`, `health()`.
    /// A Python exception raised in any handler surfaces as a store error.
    #[staticmethod]
    fn from_callbacks(handlers: Py<PyAny>) -> PyResult<Self> {
        let attr = |name: &str| -> PyResult<Py<PyAny>> {
            Python::with_gil(|py| Ok(handlers.bind(py).getattr(name)?.unbind()))
        };
        Ok(Self {
            inner: Box::new(PyCallbackStore {
                get: attr("get")?,
                put: attr("put")?,
                has: attr("has")?,
                delete: attr("delete")?,
                list: attr("list")?,
                health: attr("health")?,
            }),
        })
    }
}

/// A [`Store`] backed by Python callables (see [`PyStore::from_callbacks`]).
/// `Py<PyAny>` is `Send + Sync`, so no unsafe is needed; each call re-acquires
/// the GIL. Mirrors the WASM `fromCallbacks` JS-handler store.
struct PyCallbackStore {
    get: Py<PyAny>,
    put: Py<PyAny>,
    has: Py<PyAny>,
    delete: Py<PyAny>,
    list: Py<PyAny>,
    health: Py<PyAny>,
}

impl Store for PyCallbackStore {
    fn get(&self, key: &str) -> crate::store::Result<Option<Vec<u8>>> {
        Python::with_gil(|py| {
            let r = self.get.bind(py).call1((key,)).map_err(to_store_err)?;
            if r.is_none() {
                Ok(None)
            } else {
                Ok(Some(r.extract::<Vec<u8>>().map_err(to_store_err)?))
            }
        })
    }
    fn put(&self, key: &str, bytes: &[u8]) -> crate::store::Result<()> {
        Python::with_gil(|py| {
            self.put
                .bind(py)
                .call1((key, PyBytes::new(py, bytes)))
                .map_err(to_store_err)?;
            Ok(())
        })
    }
    fn exists(&self, key: &str) -> crate::store::Result<bool> {
        Python::with_gil(|py| {
            self.has
                .bind(py)
                .call1((key,))
                .map_err(to_store_err)?
                .extract::<bool>()
                .map_err(to_store_err)
        })
    }
    fn delete(&self, key: &str) -> crate::store::Result<()> {
        Python::with_gil(|py| {
            self.delete.bind(py).call1((key,)).map_err(to_store_err)?;
            Ok(())
        })
    }
    fn list(&self, prefix: &str) -> crate::store::Result<Vec<String>> {
        Python::with_gil(|py| {
            self.list
                .bind(py)
                .call1((prefix,))
                .map_err(to_store_err)?
                .extract::<Vec<String>>()
                .map_err(to_store_err)
        })
    }
    fn health(&self) -> crate::store::Result<()> {
        Python::with_gil(|py| {
            self.health.bind(py).call0().map_err(to_store_err)?;
            Ok(())
        })
    }
}
