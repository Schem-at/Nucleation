use super::*;

// --- Scripting FFI Entry Points ---

/// Run a Lua script file. Returns a new SchematicWrapper pointer on success
/// (if the script assigns to `result`), or null on failure / no result.
/// Caller must free the returned pointer with `schematic_free`.
#[cfg(feature = "scripting-lua")]
#[no_mangle]
pub extern "C" fn run_lua_script(path: *const c_char) -> *mut SchematicWrapper {
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid path string: {}", e));
            return ptr::null_mut();
        }
    };
    match crate::scripting::lua_engine::run_lua_script(path) {
        Ok(Some(ss)) => {
            let schematic = ss.inner;
            let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            Box::into_raw(Box::new(wrapper))
        }
        Ok(None) => ptr::null_mut(),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Run a JS script file. Returns a new SchematicWrapper pointer on success
/// (if the script assigns to `result`), or null on failure / no result.
/// Caller must free the returned pointer with `schematic_free`.
#[cfg(feature = "scripting-js")]
#[no_mangle]
pub extern "C" fn run_js_script(path: *const c_char) -> *mut SchematicWrapper {
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid path string: {}", e));
            return ptr::null_mut();
        }
    };
    match crate::scripting::js_engine::run_js_script(path) {
        Ok(Some(ss)) => {
            let schematic = ss.inner;
            let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            Box::into_raw(Box::new(wrapper))
        }
        Ok(None) => ptr::null_mut(),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Run a script file, auto-detecting engine by extension (.lua or .js).
/// Returns a new SchematicWrapper pointer on success, or null on failure / no result.
/// Caller must free the returned pointer with `schematic_free`.
#[cfg(any(feature = "scripting-lua", feature = "scripting-js"))]
#[no_mangle]
pub extern "C" fn run_script(path: *const c_char) -> *mut SchematicWrapper {
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid path string: {}", e));
            return ptr::null_mut();
        }
    };
    match crate::scripting::run_script(path) {
        Ok(Some(ss)) => {
            let schematic = ss.inner;
            let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            Box::into_raw(Box::new(wrapper))
        }
        Ok(None) => ptr::null_mut(),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}
