//! WASM binding for the `store` module.
//!
//! On `wasm32` only the always-available `mem://` backend compiles (networked
//! backends are gated out); the API surface is identical to the other bindings.
//!
//! A **JS-callback store** (host owns IO) is also available on `wasm32` via
//! [`StoreWrapper::from_callbacks`]: the host supplies synchronous `get/put/
//! has/delete/list/health` functions and Nucleation drives them. This is the
//! WASM analogue of the native `CallbackStore`.

use wasm_bindgen::prelude::*;

use crate::store::{self, Store, StoreError};

fn to_js(e: StoreError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// A byte store keyed by `/`-delimited paths. Construct with `new Store(url)`.
#[wasm_bindgen]
pub struct StoreWrapper {
    inner: Box<dyn Store>,
}

impl StoreWrapper {
    /// Borrow the underlying store so other in-crate WASM bindings (e.g. the
    /// schematic binding) can perform store-backed open/save. Not exposed to JS.
    pub(crate) fn as_store(&self) -> &dyn Store {
        self.inner.as_ref()
    }
}

#[wasm_bindgen]
impl StoreWrapper {
    /// Open a store from a URL. On the web, use `mem://`.
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<StoreWrapper, JsValue> {
        Ok(Self {
            inner: store::open(url).map_err(to_js)?,
        })
    }

    /// Open a store from a URL (canonical name, matching Python/FFI `open`).
    /// Equivalent to `new Store(url)`.
    #[wasm_bindgen(js_name = open)]
    pub fn open(url: &str) -> Result<StoreWrapper, JsValue> {
        StoreWrapper::new(url)
    }

    /// Atomically write `data` at `key` only if it does not already exist.
    /// Returns `true` if written, `false` if the key already existed.
    #[wasm_bindgen(js_name = putIfAbsent)]
    pub fn put_if_absent(&self, key: &str, data: &[u8]) -> Result<bool, JsValue> {
        self.inner.put_if_absent(key, data).map_err(to_js)
    }

    /// A keyset page of keys under `prefix`, sorted, starting strictly after
    /// `after` (pass `null`/`undefined` for the first page), at most `limit`.
    /// Returns `{ keys: string[], next: string | null }`; pass `next` back as
    /// `after` to continue, or stop when it is `null`.
    #[wasm_bindgen(js_name = listPaginated)]
    pub fn list_paginated(
        &self,
        prefix: &str,
        after: Option<String>,
        limit: usize,
    ) -> Result<JsValue, JsValue> {
        let (keys, next) = self
            .inner
            .list_paginated(prefix, after.as_deref(), limit)
            .map_err(to_js)?;
        let arr = js_sys::Array::new();
        for k in keys {
            arr.push(&JsValue::from_str(&k));
        }
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("keys"), &arr)?;
        let next_val = match next {
            Some(n) => JsValue::from_str(&n),
            None => JsValue::NULL,
        };
        js_sys::Reflect::set(&obj, &JsValue::from_str("next"), &next_val)?;
        Ok(obj.into())
    }

    /// Fetch `key`, or `undefined` if absent.
    #[wasm_bindgen(js_name = get)]
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, JsValue> {
        self.inner.get(key).map_err(to_js)
    }

    /// Store `data` at `key`, overwriting.
    #[wasm_bindgen(js_name = put)]
    pub fn put(&self, key: &str, data: &[u8]) -> Result<(), JsValue> {
        self.inner.put(key, data).map_err(to_js)
    }

    /// Whether `key` exists.
    #[wasm_bindgen(js_name = has)]
    pub fn exists(&self, key: &str) -> Result<bool, JsValue> {
        self.inner.exists(key).map_err(to_js)
    }

    /// Remove `key` (idempotent).
    #[wasm_bindgen(js_name = delete)]
    pub fn delete(&self, key: &str) -> Result<(), JsValue> {
        self.inner.delete(key).map_err(to_js)
    }

    /// Keys under `prefix`.
    #[wasm_bindgen(js_name = list)]
    pub fn list(&self, prefix: &str) -> Result<Vec<String>, JsValue> {
        self.inner.list(prefix).map_err(to_js)
    }

    /// Reject if the store is not usable.
    #[wasm_bindgen(js_name = health)]
    pub fn health(&self) -> Result<(), JsValue> {
        self.inner.health().map_err(to_js)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl StoreWrapper {
    /// Build a store backed by host-supplied JS functions ("host owns IO").
    ///
    /// `handlers` is a JS object with **synchronous** methods:
    /// `get(key) -> Uint8Array | null`, `put(key, Uint8Array)`,
    /// `has(key) -> bool`, `delete(key)`, `list(prefix) -> string[]`,
    /// `health()`. Any thrown JS error surfaces as a store error.
    #[wasm_bindgen(js_name = fromCallbacks)]
    pub fn from_callbacks(handlers: &JsValue) -> Result<StoreWrapper, JsValue> {
        Ok(StoreWrapper {
            inner: Box::new(js_callback::JsCallbackStore::from_object(handlers)?),
        })
    }
}

/// JS-callback-backed store. `wasm32`-only: the `unsafe Send + Sync` is sound
/// because WASM is single-threaded, and the trait requires those bounds.
#[cfg(target_arch = "wasm32")]
mod js_callback {
    use js_sys::{Array, Function, Reflect, Uint8Array};
    use wasm_bindgen::{JsCast, JsValue};

    use crate::store::{Result, Store, StoreError};

    /// Sound only under WASM's single-threaded model.
    struct SendWrap<T>(T);
    // SAFETY: WASM has no threads, so these are never shared across threads.
    unsafe impl<T> Send for SendWrap<T> {}
    unsafe impl<T> Sync for SendWrap<T> {}

    pub struct JsCallbackStore {
        get: SendWrap<Function>,
        put: SendWrap<Function>,
        has: SendWrap<Function>,
        delete: SendWrap<Function>,
        list: SendWrap<Function>,
        health: SendWrap<Function>,
    }

    fn js_err(e: JsValue) -> StoreError {
        StoreError::Other(e.as_string().unwrap_or_else(|| "JS callback error".into()))
    }

    fn fetch_fn(obj: &JsValue, name: &str) -> std::result::Result<Function, JsValue> {
        let val = Reflect::get(obj, &JsValue::from_str(name))?;
        val.dyn_into::<Function>()
            .map_err(|_| JsValue::from_str(&format!("handler `{name}` is not a function")))
    }

    impl JsCallbackStore {
        pub fn from_object(obj: &JsValue) -> std::result::Result<Self, JsValue> {
            Ok(Self {
                get: SendWrap(fetch_fn(obj, "get")?),
                put: SendWrap(fetch_fn(obj, "put")?),
                has: SendWrap(fetch_fn(obj, "has")?),
                delete: SendWrap(fetch_fn(obj, "delete")?),
                list: SendWrap(fetch_fn(obj, "list")?),
                health: SendWrap(fetch_fn(obj, "health")?),
            })
        }
    }

    impl Store for JsCallbackStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
            let res = self
                .get
                .0
                .call1(&JsValue::NULL, &JsValue::from_str(key))
                .map_err(js_err)?;
            if res.is_null() || res.is_undefined() {
                return Ok(None);
            }
            let arr = res
                .dyn_into::<Uint8Array>()
                .map_err(|_| StoreError::Other("get() must return a Uint8Array or null".into()))?;
            Ok(Some(arr.to_vec()))
        }

        fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
            let data = Uint8Array::from(bytes);
            self.put
                .0
                .call2(&JsValue::NULL, &JsValue::from_str(key), &data)
                .map_err(js_err)?;
            Ok(())
        }

        fn exists(&self, key: &str) -> Result<bool> {
            let res = self
                .has
                .0
                .call1(&JsValue::NULL, &JsValue::from_str(key))
                .map_err(js_err)?;
            Ok(res.is_truthy())
        }

        fn delete(&self, key: &str) -> Result<()> {
            self.delete
                .0
                .call1(&JsValue::NULL, &JsValue::from_str(key))
                .map_err(js_err)?;
            Ok(())
        }

        fn list(&self, prefix: &str) -> Result<Vec<String>> {
            let res = self
                .list
                .0
                .call1(&JsValue::NULL, &JsValue::from_str(prefix))
                .map_err(js_err)?;
            let arr = Array::from(&res);
            Ok(arr.iter().filter_map(|v| v.as_string()).collect())
        }

        fn health(&self) -> Result<()> {
            self.health.0.call0(&JsValue::NULL).map_err(js_err)?;
            Ok(())
        }
    }
}
