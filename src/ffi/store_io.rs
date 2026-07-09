use super::*;

// --- Store-backed (transparent) I/O ---

/// Open a schematic from a URI: a local path, `file://...`, or `s3://bucket/key.schem`.
/// The format is auto-detected from the URI's extension.
///
/// Single-string URIs for `redis://`, `postgres://`, and `mem://` are rejected by
/// the core resolver; use [`schematic_open_from_store`] with an explicit store
/// handle for those backends.
///
/// Returns a new wrapper on success (free it with `schematic_free`), or null on
/// error (call `schematic_last_error`).
#[no_mangle]
pub extern "C" fn schematic_open(uri: *const c_char) -> *mut SchematicWrapper {
    if uri.is_null() {
        set_last_error("schematic_open: null uri".into());
        return ptr::null_mut();
    }
    let uri_str = unsafe { CStr::from_ptr(uri).to_string_lossy().into_owned() };
    match UniversalSchematic::open(&uri_str) {
        Ok(schematic) => {
            let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            Box::into_raw(Box::new(wrapper))
        }
        Err(e) => {
            set_last_error(format!("schematic_open: {}", e));
            ptr::null_mut()
        }
    }
}

/// Save a schematic to a URI: a local path, `file://...`, or `s3://bucket/key.schem`.
/// The format is auto-detected from the URI's extension. `version` may be NULL.
///
/// Single-string URIs for `redis://`, `postgres://`, and `mem://` are rejected by
/// the core resolver; use [`schematic_save_to_store`] with an explicit store
/// handle for those backends.
///
/// Returns 0 on success, -1 on null arguments, -2 on error (call `schematic_last_error`).
#[no_mangle]
pub extern "C" fn schematic_save(
    handle: *const SchematicWrapper,
    uri: *const c_char,
    version: *const c_char,
) -> c_int {
    if handle.is_null() || uri.is_null() {
        set_last_error("schematic_save: null schematic or uri".into());
        return -1;
    }
    let s = unsafe { &*(*handle).0 };
    let uri_str = unsafe { CStr::from_ptr(uri).to_string_lossy().into_owned() };
    let ver = if version.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(version).to_string_lossy().into_owned() })
    };
    match s.save(&uri_str, ver.as_deref()) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(format!("schematic_save: {}", e));
            -2
        }
    }
}

/// Open a schematic from an explicit store handle (from `nuc_store_open`) at `key`.
/// Works for every backend, including `redis://`/`postgres://`/`mem://` that the
/// single-string URI form rejects.
///
/// Returns a new wrapper on success (free it with `schematic_free`), or null on
/// error (call `schematic_last_error`).
#[no_mangle]
pub extern "C" fn schematic_open_from_store(
    store: *const crate::store::ffi::StoreHandle,
    key: *const c_char,
) -> *mut SchematicWrapper {
    let store = unsafe { store.as_ref() };
    let (Some(store), false) = (store, key.is_null()) else {
        set_last_error("schematic_open_from_store: null store or key".into());
        return ptr::null_mut();
    };
    let key_str = unsafe { CStr::from_ptr(key).to_string_lossy().into_owned() };
    match UniversalSchematic::from_store(store.inner(), &key_str) {
        Ok(schematic) => {
            let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            Box::into_raw(Box::new(wrapper))
        }
        Err(e) => {
            set_last_error(format!("schematic_open_from_store: {}", e));
            ptr::null_mut()
        }
    }
}

/// Save a schematic to an explicit store handle (from `nuc_store_open`) at `key`.
/// Works for every backend, including `redis://`/`postgres://`/`mem://` that the
/// single-string URI form rejects. `version` may be NULL.
///
/// Returns 0 on success, -1 on null arguments, -2 on error (call `schematic_last_error`).
#[no_mangle]
pub extern "C" fn schematic_save_to_store(
    handle: *const SchematicWrapper,
    store: *const crate::store::ffi::StoreHandle,
    key: *const c_char,
    version: *const c_char,
) -> c_int {
    let store = unsafe { store.as_ref() };
    if handle.is_null() || store.is_none() || key.is_null() {
        set_last_error("schematic_save_to_store: null schematic, store, or key".into());
        return -1;
    }
    let s = unsafe { &*(*handle).0 };
    let store = store.unwrap();
    let key_str = unsafe { CStr::from_ptr(key).to_string_lossy().into_owned() };
    let ver = if version.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(version).to_string_lossy().into_owned() })
    };
    match s.save_to_store(store.inner(), &key_str, ver.as_deref()) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(format!("schematic_save_to_store: {}", e));
            -2
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_export_settings_schema(format: *const c_char) -> *mut c_char {
    if format.is_null() {
        return ptr::null_mut();
    }
    let fmt = unsafe { CStr::from_ptr(format).to_string_lossy() };
    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => return ptr::null_mut(),
    };
    match manager.get_export_settings_schema(&fmt) {
        Some(schema) => CString::new(schema)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut()),
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_import_settings_schema(format: *const c_char) -> *mut c_char {
    if format.is_null() {
        return ptr::null_mut();
    }
    let fmt = unsafe { CStr::from_ptr(format).to_string_lossy() };
    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => return ptr::null_mut(),
    };
    match manager.get_import_settings_schema(&fmt) {
        Some(schema) => CString::new(schema)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut()),
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_supported_import_formats() -> StringArray {
    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            }
        }
    };
    let formats = manager.list_importers();
    vec_string_to_string_array(formats)
}

#[no_mangle]
pub extern "C" fn schematic_get_supported_export_formats() -> StringArray {
    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            }
        }
    };
    let formats = manager.list_exporters();
    vec_string_to_string_array(formats)
}

#[no_mangle]
pub extern "C" fn schematic_get_format_versions(format: *const c_char) -> StringArray {
    if format.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let fmt = unsafe { CStr::from_ptr(format).to_string_lossy() };
    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            }
        }
    };
    let versions = manager.get_exporter_versions(&fmt).unwrap_or_default();
    vec_string_to_string_array(versions)
}

#[no_mangle]
pub extern "C" fn schematic_get_default_format_version(format: *const c_char) -> *mut c_char {
    if format.is_null() {
        return ptr::null_mut();
    }
    let fmt = unsafe { CStr::from_ptr(format).to_string_lossy() };
    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => return ptr::null_mut(),
    };
    match manager.get_exporter_default_version(&fmt) {
        Some(v) => CString::new(v).unwrap().into_raw(),
        None => ptr::null_mut(),
    }
}

#[cfg(test)]
mod store_io_tests {
    use super::*;

    fn cs(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    /// Round-trip a schematic through `schematic_save` / `schematic_open` over a
    /// local temp `.schem` file (no external services).
    #[test]
    fn schematic_open_save_roundtrip_local_file() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("nuc_ffi_roundtrip_{}.schem", std::process::id()));
        let path_c = cs(path.to_str().unwrap());

        // Build a non-empty schematic.
        let handle = schematic_new();
        assert!(!handle.is_null());
        let block = cs("minecraft:stone");
        assert_eq!(
            schematic_set_block(handle, 0, 0, 0, block.as_ptr()),
            0,
            "set_block failed"
        );

        // Save to a local file URI.
        assert_eq!(
            schematic_save(handle, path_c.as_ptr(), ptr::null()),
            0,
            "schematic_save failed: {:?}",
            unsafe { CStr::from_ptr(schematic_last_error()) }
        );
        assert!(path.exists(), "save did not produce a file");

        // Re-open from the same path.
        let reopened = schematic_open(path_c.as_ptr());
        assert!(
            !reopened.is_null(),
            "schematic_open returned null: {:?}",
            unsafe { CStr::from_ptr(schematic_last_error()) }
        );

        // The re-opened schematic must still contain the block we placed.
        let name = schematic_get_block(reopened, 0, 0, 0);
        assert!(!name.is_null(), "block missing after round-trip");
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
        free_string(name);
        assert!(
            name_str.contains("stone"),
            "unexpected block after round-trip: {}",
            name_str
        );

        schematic_free(handle);
        schematic_free(reopened);
        let _ = std::fs::remove_file(&path);
    }

    /// A single-string `mem://` URI must be rejected by the core resolver.
    #[test]
    fn schematic_open_rejects_mem_uri() {
        let uri = cs("mem://some/key.schem");
        let result = schematic_open(uri.as_ptr());
        assert!(
            result.is_null(),
            "mem:// single-string URI should be rejected"
        );
        let err = schematic_last_error();
        assert!(!err.is_null(), "expected an error message");
    }
}
