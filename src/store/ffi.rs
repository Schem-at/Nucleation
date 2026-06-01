//! C ABI for the `store` module: open a store from a URL and drive it over the
//! language boundary. Opaque handle, explicit frees, thread-local last-error —
//! matching the conventions in the crate's main `ffi` module.
//!
//! Status codes (`c_int`): `0` success, `1` not-found (`get`/`exists`-style),
//! `-1` error (call [`nuc_store_last_error`]).

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use super::Store;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_err(msg: impl Into<String>) {
    let c = CString::new(msg.into()).unwrap_or_default();
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(c));
}

/// Opaque handle wrapping a boxed [`Store`].
pub struct StoreHandle(Box<dyn Store>);

impl StoreHandle {
    /// Borrow the underlying store as a trait object. Used by the schematic FFI
    /// (`schematic_open_from_store` / `schematic_save_to_store`) to bridge a
    /// store handle into the core store-backed open/save paths.
    pub(crate) fn inner(&self) -> &dyn Store {
        self.0.as_ref()
    }
}

unsafe fn as_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok()
}

fn cstring_raw(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

/// Open a store from a URL (e.g. `mem://`, `file:///path`, `s3://bucket/prefix`).
/// Returns null on error.
///
/// # Safety
/// `url` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_open(url: *const c_char) -> *mut StoreHandle {
    let Some(url) = as_str(url) else {
        set_err("nuc_store_open: url is null or not UTF-8");
        return std::ptr::null_mut();
    };
    match super::open(url) {
        Ok(store) => Box::into_raw(Box::new(StoreHandle(store))),
        Err(e) => {
            set_err(e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Free a store handle.
///
/// # Safety
/// `handle` must come from [`nuc_store_open`] and not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_free(handle: *mut StoreHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Fetch `key`. On success (`0`), `*out_ptr`/`*out_len` receive a buffer the
/// caller must free with [`nuc_store_bytes_free`]. Returns `1` if absent, `-1`
/// on error.
///
/// # Safety
/// `handle` must be valid; `key` a valid C string; `out_ptr`/`out_len` non-null.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_get(
    handle: *const StoreHandle,
    key: *const c_char,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    let (Some(h), Some(key)) = (handle.as_ref(), as_str(key)) else {
        set_err("nuc_store_get: invalid argument");
        return -1;
    };
    match h.0.get(key) {
        Ok(Some(bytes)) => {
            let mut boxed = bytes.into_boxed_slice();
            *out_len = boxed.len();
            *out_ptr = boxed.as_mut_ptr();
            std::mem::forget(boxed);
            0
        }
        Ok(None) => 1,
        Err(e) => {
            set_err(e.to_string());
            -1
        }
    }
}

/// Store `len` bytes at `key`. Returns `0` / `-1`.
///
/// # Safety
/// `handle` valid; `key` a valid C string; `data` points to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_put(
    handle: *const StoreHandle,
    key: *const c_char,
    data: *const u8,
    len: usize,
) -> c_int {
    let (Some(h), Some(key)) = (handle.as_ref(), as_str(key)) else {
        set_err("nuc_store_put: invalid argument");
        return -1;
    };
    let bytes = if len == 0 {
        &[][..]
    } else {
        std::slice::from_raw_parts(data, len)
    };
    match h.0.put(key, bytes) {
        Ok(()) => 0,
        Err(e) => {
            set_err(e.to_string());
            -1
        }
    }
}

/// Whether `key` exists. Returns `1` yes, `0` no, `-1` error.
///
/// # Safety
/// `handle` valid; `key` a valid C string.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_exists(handle: *const StoreHandle, key: *const c_char) -> c_int {
    let (Some(h), Some(key)) = (handle.as_ref(), as_str(key)) else {
        set_err("nuc_store_exists: invalid argument");
        return -1;
    };
    match h.0.exists(key) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            set_err(e.to_string());
            -1
        }
    }
}

/// Delete `key` (idempotent). Returns `0` / `-1`.
///
/// # Safety
/// `handle` valid; `key` a valid C string.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_delete(handle: *const StoreHandle, key: *const c_char) -> c_int {
    let (Some(h), Some(key)) = (handle.as_ref(), as_str(key)) else {
        set_err("nuc_store_delete: invalid argument");
        return -1;
    };
    match h.0.delete(key) {
        Ok(()) => 0,
        Err(e) => {
            set_err(e.to_string());
            -1
        }
    }
}

/// List keys under `prefix` as a JSON array string (caller frees with
/// [`nuc_store_string_free`]). Returns null on error.
///
/// # Safety
/// `handle` valid; `prefix` a valid C string.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_list(
    handle: *const StoreHandle,
    prefix: *const c_char,
) -> *mut c_char {
    let (Some(h), Some(prefix)) = (handle.as_ref(), as_str(prefix)) else {
        set_err("nuc_store_list: invalid argument");
        return std::ptr::null_mut();
    };
    match h.0.list(prefix) {
        Ok(keys) => match serde_json::to_string(&keys) {
            Ok(json) => cstring_raw(json),
            Err(e) => {
                set_err(e.to_string());
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            set_err(e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Atomically write `len` bytes at `key` only if it does not already exist.
/// Returns `1` if written, `0` if the key existed, `-1` on error.
///
/// # Safety
/// `handle` valid; `key` a valid C string; `data` points to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_put_if_absent(
    handle: *const StoreHandle,
    key: *const c_char,
    data: *const u8,
    len: usize,
) -> c_int {
    let (Some(h), Some(key)) = (handle.as_ref(), as_str(key)) else {
        set_err("nuc_store_put_if_absent: invalid argument");
        return -1;
    };
    let bytes = if len == 0 {
        &[][..]
    } else {
        std::slice::from_raw_parts(data, len)
    };
    match h.0.put_if_absent(key, bytes) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            set_err(e.to_string());
            -1
        }
    }
}

/// A keyset page of keys under `prefix`. `after` is the exclusive cursor (NULL
/// for the first page); at most `limit` keys are returned. Result is a JSON
/// object string `{"keys":[...],"next":"…"|null}`, freed with
/// [`nuc_store_string_free`]; NULL on error.
///
/// # Safety
/// `handle` valid; `prefix` a valid C string; `after` NULL or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_list_paginated(
    handle: *const StoreHandle,
    prefix: *const c_char,
    after: *const c_char,
    limit: usize,
) -> *mut c_char {
    let (Some(h), Some(prefix)) = (handle.as_ref(), as_str(prefix)) else {
        set_err("nuc_store_list_paginated: invalid argument");
        return std::ptr::null_mut();
    };
    let after = as_str(after); // None when NULL
    match h.0.list_paginated(prefix, after, limit) {
        Ok((keys, next)) => {
            match serde_json::to_string(&serde_json::json!({ "keys": keys, "next": next })) {
                Ok(json) => cstring_raw(json),
                Err(e) => {
                    set_err(e.to_string());
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            set_err(e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Health check. Returns `0` usable, `-1` otherwise.
///
/// # Safety
/// `handle` valid.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_health(handle: *const StoreHandle) -> c_int {
    let Some(h) = handle.as_ref() else {
        set_err("nuc_store_health: null handle");
        return -1;
    };
    match h.0.health() {
        Ok(()) => 0,
        Err(e) => {
            set_err(e.to_string());
            -1
        }
    }
}

/// The last error on this thread, or null. Caller frees with
/// [`nuc_store_string_free`].
#[no_mangle]
pub extern "C" fn nuc_store_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| match &*e.borrow() {
        Some(c) => c.clone().into_raw(),
        None => std::ptr::null_mut(),
    })
}

/// Free a string returned by this module.
///
/// # Safety
/// `s` must come from a `nuc_store_*` function returning `*mut c_char`.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Free a byte buffer returned by [`nuc_store_get`].
///
/// # Safety
/// `ptr`/`len` must be exactly what `nuc_store_get` produced.
#[no_mangle]
pub unsafe extern "C" fn nuc_store_bytes_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len != 0 {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cs(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn ffi_roundtrip_over_mem_store() {
        unsafe {
            let h = nuc_store_open(cs("mem://").as_ptr());
            assert!(!h.is_null());

            let key = cs("a/b");
            let data = b"hello ffi";
            assert_eq!(nuc_store_put(h, key.as_ptr(), data.as_ptr(), data.len()), 0);
            assert_eq!(nuc_store_exists(h, key.as_ptr()), 1);

            let mut ptr: *mut u8 = std::ptr::null_mut();
            let mut len: usize = 0;
            assert_eq!(nuc_store_get(h, key.as_ptr(), &mut ptr, &mut len), 0);
            let got = std::slice::from_raw_parts(ptr, len).to_vec();
            assert_eq!(got, data);
            nuc_store_bytes_free(ptr, len);

            let list = nuc_store_list(h, cs("a/").as_ptr());
            assert!(!list.is_null());
            let json = CStr::from_ptr(list).to_str().unwrap().to_string();
            assert_eq!(json, "[\"a/b\"]");
            nuc_store_string_free(list);

            assert_eq!(nuc_store_delete(h, key.as_ptr()), 0);
            assert_eq!(nuc_store_exists(h, key.as_ptr()), 0);

            // Missing key -> status 1.
            let mut p2: *mut u8 = std::ptr::null_mut();
            let mut l2: usize = 0;
            assert_eq!(nuc_store_get(h, key.as_ptr(), &mut p2, &mut l2), 1);

            assert_eq!(nuc_store_health(h), 0);
            nuc_store_free(h);
        }
    }

    #[test]
    fn ffi_open_bad_scheme_sets_error() {
        unsafe {
            let h = nuc_store_open(cs("ftp://nope").as_ptr());
            assert!(h.is_null());
            let err = nuc_store_last_error();
            assert!(!err.is_null());
            nuc_store_string_free(err);
        }
    }
}
