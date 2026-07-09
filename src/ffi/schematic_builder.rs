use super::*;

// --- SchematicBuilder FFI ---

pub struct SchematicBuilderWrapper(SchematicBuilder);

#[no_mangle]
pub extern "C" fn schematicbuilder_new() -> *mut SchematicBuilderWrapper {
    Box::into_raw(Box::new(SchematicBuilderWrapper(SchematicBuilder::new())))
}

#[no_mangle]
pub extern "C" fn schematicbuilder_free(ptr: *mut SchematicBuilderWrapper) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn schematicbuilder_name(
    ptr: *mut SchematicBuilderWrapper,
    name: *const c_char,
) -> c_int {
    if ptr.is_null() || name.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    // SchematicBuilder::name consumes self, so we need to swap
    let old = std::mem::take(builder);
    *builder = old.name(n);
    0
}

#[no_mangle]
pub extern "C" fn schematicbuilder_map(
    ptr: *mut SchematicBuilderWrapper,
    ch: c_char,
    block: *const c_char,
) -> c_int {
    if ptr.is_null() || block.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let b = unsafe { CStr::from_ptr(block).to_string_lossy().into_owned() };
    let old = std::mem::take(builder);
    *builder = old.map(ch as u8 as char, &b);
    0
}

#[no_mangle]
pub extern "C" fn schematicbuilder_layers(
    ptr: *mut SchematicBuilderWrapper,
    layers_json: *const c_char,
) -> c_int {
    if ptr.is_null() || layers_json.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let json_str = unsafe { CStr::from_ptr(layers_json).to_string_lossy() };
    let layers: Vec<Vec<String>> = match serde_json::from_str(&json_str) {
        Ok(l) => l,
        Err(_) => return -2,
    };
    let layer_refs: Vec<Vec<&str>> = layers
        .iter()
        .map(|l| l.iter().map(|s| s.as_str()).collect())
        .collect();
    let layer_slice_refs: Vec<&[&str]> = layer_refs.iter().map(|v| v.as_slice()).collect();
    let old = std::mem::take(builder);
    *builder = old.layers(&layer_slice_refs);
    0
}

#[no_mangle]
pub extern "C" fn schematicbuilder_build(
    ptr: *mut SchematicBuilderWrapper,
) -> *mut SchematicWrapper {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let wrapper = unsafe { Box::from_raw(ptr) };
    match wrapper.0.build() {
        Ok(schematic) => {
            let w = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            Box::into_raw(Box::new(w))
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Builds the schematic and returns an error message if it fails.
/// On success, sets `*out` to the new schematic and returns NULL.
/// On failure, returns an error string (must be freed with `free_string`) and sets `*out` to NULL.
#[no_mangle]
pub extern "C" fn schematicbuilder_build_with_error(
    ptr: *mut SchematicBuilderWrapper,
    out: *mut *mut SchematicWrapper,
) -> *mut c_char {
    if ptr.is_null() || out.is_null() {
        return CString::new("null pointer").unwrap().into_raw();
    }
    let wrapper = unsafe { Box::from_raw(ptr) };
    match wrapper.0.build() {
        Ok(schematic) => {
            let w = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            unsafe { *out = Box::into_raw(Box::new(w)) };
            ptr::null_mut()
        }
        Err(e) => {
            unsafe { *out = ptr::null_mut() };
            CString::new(e).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn schematicbuilder_from_template(
    template: *const c_char,
) -> *mut SchematicBuilderWrapper {
    if template.is_null() {
        return ptr::null_mut();
    }
    let t = unsafe { CStr::from_ptr(template).to_string_lossy() };
    match SchematicBuilder::from_template(&t) {
        Ok(builder) => Box::into_raw(Box::new(SchematicBuilderWrapper(builder))),
        Err(_) => ptr::null_mut(),
    }
}

/// Append a single layer of rows. `rows_json` is a JSON array of strings,
/// e.g. `["abc", "def"]`. Equivalent to a one-element layers array.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematicbuilder_layer(
    ptr: *mut SchematicBuilderWrapper,
    rows_json: *const c_char,
) -> c_int {
    if ptr.is_null() || rows_json.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let json_str = unsafe { CStr::from_ptr(rows_json).to_string_lossy() };
    let rows: Vec<String> = match serde_json::from_str(&json_str) {
        Ok(r) => r,
        Err(_) => return -2,
    };
    let row_refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
    let old = std::mem::take(builder);
    *builder = old.layer(&row_refs);
    0
}

/// Bulk-register palette characters. `pairs_json` is a JSON array of
/// `[char, block]` two-element arrays, e.g.
/// `[["c", "minecraft:gray_concrete"], [" ", "minecraft:air"]]`.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematicbuilder_palette(
    ptr: *mut SchematicBuilderWrapper,
    pairs_json: *const c_char,
) -> c_int {
    if ptr.is_null() || pairs_json.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let json_str = unsafe { CStr::from_ptr(pairs_json).to_string_lossy() };
    let raw: Vec<(String, String)> = match serde_json::from_str(&json_str) {
        Ok(p) => p,
        Err(_) => return -2,
    };
    let pairs: Vec<(char, &str)> = raw
        .iter()
        .filter_map(|(k, v)| k.chars().next().map(|c| (c, v.as_str())))
        .collect();
    let old = std::mem::take(builder);
    *builder = old.palette(&pairs);
    0
}

#[no_mangle]
pub extern "C" fn schematicbuilder_offset(
    ptr: *mut SchematicBuilderWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let old = std::mem::take(builder);
    *builder = old.offset(x, y, z);
    0
}

#[no_mangle]
pub extern "C" fn schematicbuilder_use_standard_palette(
    ptr: *mut SchematicBuilderWrapper,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let old = std::mem::take(builder);
    *builder = old.use_standard_palette();
    0
}

#[no_mangle]
pub extern "C" fn schematicbuilder_use_minimal_palette(ptr: *mut SchematicBuilderWrapper) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let old = std::mem::take(builder);
    *builder = old.use_minimal_palette();
    0
}

#[no_mangle]
pub extern "C" fn schematicbuilder_use_compact_palette(ptr: *mut SchematicBuilderWrapper) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let builder = unsafe { &mut (*ptr).0 };
    let old = std::mem::take(builder);
    *builder = old.use_compact_palette();
    0
}

/// Run pre-build validation. Returns NULL on success or a malloc-d
/// error string (caller frees with `free_string`) on failure.
#[no_mangle]
pub extern "C" fn schematicbuilder_validate(ptr: *const SchematicBuilderWrapper) -> *mut c_char {
    if ptr.is_null() {
        return CString::new("null pointer").unwrap().into_raw();
    }
    let builder = unsafe { &(*ptr).0 };
    match builder.validate() {
        Ok(()) => ptr::null_mut(),
        Err(e) => CString::new(e).unwrap().into_raw(),
    }
}

/// Serialize the builder back into the canonical template format.
/// Returns a malloc-d C string (caller frees with `free_string`).
#[no_mangle]
pub extern "C" fn schematicbuilder_to_template(ptr: *const SchematicBuilderWrapper) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let builder = unsafe { &(*ptr).0 };
    CString::new(builder.to_template())
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}
