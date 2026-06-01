//! A [`Store`] whose operations delegate to caller-supplied closures.
//!
//! This is the substrate for "host owns IO": an FFI host (C function pointers)
//! or a WASM host (JS functions) provides the actual storage, and Nucleation
//! drives it through this adapter. In pure Rust you usually don't need it —
//! just pass your own `Box<dyn Store>` — but it's the bridge when the backend
//! lives on the other side of a language boundary.

use std::sync::Arc;

use super::{Result, Store, StoreError};

type GetFn = dyn Fn(&str) -> Result<Option<Vec<u8>>> + Send + Sync;
type PutFn = dyn Fn(&str, &[u8]) -> Result<()> + Send + Sync;
type ExistsFn = dyn Fn(&str) -> Result<bool> + Send + Sync;
type DeleteFn = dyn Fn(&str) -> Result<()> + Send + Sync;
type ListFn = dyn Fn(&str) -> Result<Vec<String>> + Send + Sync;
type HealthFn = dyn Fn() -> Result<()> + Send + Sync;

struct Callbacks {
    get: Box<GetFn>,
    put: Box<PutFn>,
    exists: Box<ExistsFn>,
    delete: Box<DeleteFn>,
    list: Box<ListFn>,
    health: Box<HealthFn>,
}

/// A store backed by closures. Build one with [`CallbackStore::builder`].
/// Cloning shares the same callbacks.
#[derive(Clone)]
pub struct CallbackStore {
    inner: Arc<Callbacks>,
}

/// Builder for [`CallbackStore`]; every operation must be supplied.
#[derive(Default)]
pub struct CallbackStoreBuilder {
    get: Option<Box<GetFn>>,
    put: Option<Box<PutFn>>,
    exists: Option<Box<ExistsFn>>,
    delete: Option<Box<DeleteFn>>,
    list: Option<Box<ListFn>>,
    health: Option<Box<HealthFn>>,
}

impl CallbackStore {
    /// Start building a callback-backed store.
    pub fn builder() -> CallbackStoreBuilder {
        CallbackStoreBuilder::default()
    }
}

impl CallbackStoreBuilder {
    pub fn on_get(
        mut self,
        f: impl Fn(&str) -> Result<Option<Vec<u8>>> + Send + Sync + 'static,
    ) -> Self {
        self.get = Some(Box::new(f));
        self
    }
    pub fn on_put(mut self, f: impl Fn(&str, &[u8]) -> Result<()> + Send + Sync + 'static) -> Self {
        self.put = Some(Box::new(f));
        self
    }
    pub fn on_exists(mut self, f: impl Fn(&str) -> Result<bool> + Send + Sync + 'static) -> Self {
        self.exists = Some(Box::new(f));
        self
    }
    pub fn on_delete(mut self, f: impl Fn(&str) -> Result<()> + Send + Sync + 'static) -> Self {
        self.delete = Some(Box::new(f));
        self
    }
    pub fn on_list(
        mut self,
        f: impl Fn(&str) -> Result<Vec<String>> + Send + Sync + 'static,
    ) -> Self {
        self.list = Some(Box::new(f));
        self
    }
    pub fn on_health(mut self, f: impl Fn() -> Result<()> + Send + Sync + 'static) -> Self {
        self.health = Some(Box::new(f));
        self
    }

    /// Finalize. Errors if any operation was not supplied.
    pub fn build(self) -> Result<CallbackStore> {
        macro_rules! require {
            ($field:ident) => {
                self.$field.ok_or_else(|| {
                    StoreError::Other(
                        concat!("CallbackStore missing `", stringify!($field), "` handler").into(),
                    )
                })?
            };
        }
        Ok(CallbackStore {
            inner: Arc::new(Callbacks {
                get: require!(get),
                put: require!(put),
                exists: require!(exists),
                delete: require!(delete),
                list: require!(list),
                health: require!(health),
            }),
        })
    }
}

impl Store for CallbackStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        (self.inner.get)(key)
    }
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        (self.inner.put)(key, bytes)
    }
    fn exists(&self, key: &str) -> Result<bool> {
        (self.inner.exists)(key)
    }
    fn delete(&self, key: &str) -> Result<()> {
        (self.inner.delete)(key)
    }
    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        (self.inner.list)(prefix)
    }
    fn health(&self) -> Result<()> {
        (self.inner.health)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemStore;

    /// A `CallbackStore` whose handlers delegate to a shared `MemStore` must
    /// satisfy the same contract as any other backend.
    #[test]
    fn callback_store_delegating_to_mem_satisfies_contract() {
        let backing = MemStore::new();
        let (g, p, e, d, l, h) = (
            backing.clone(),
            backing.clone(),
            backing.clone(),
            backing.clone(),
            backing.clone(),
            backing.clone(),
        );
        let store = CallbackStore::builder()
            .on_get(move |k| g.get(k))
            .on_put(move |k, b| p.put(k, b))
            .on_exists(move |k| e.exists(k))
            .on_delete(move |k| d.delete(k))
            .on_list(move |pre| l.list(pre))
            .on_health(move || h.health())
            .build()
            .expect("build");

        crate::store::contract::run_contract(&store);
    }

    #[test]
    fn build_fails_when_handler_missing() {
        let r = CallbackStore::builder().on_get(|_| Ok(None)).build();
        assert!(r.is_err(), "build must fail if not all handlers are set");
    }
}
