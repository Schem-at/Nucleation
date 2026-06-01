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
}
