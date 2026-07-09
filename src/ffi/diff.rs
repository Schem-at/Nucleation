use super::*;

// =============================================================================
// Diff + Fingerprint FFI
// =============================================================================

/// Boxes a schematic into a `*mut SchematicWrapper` using the same nested-box
/// ownership convention as `schematic_new`. Free with `schematic_free`.
fn box_schematic(schematic: UniversalSchematic) -> *mut SchematicWrapper {
    let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
    Box::into_raw(Box::new(wrapper))
}

/// Computes the fingerprint of a schematic for the given preset and returns it
/// as a hex string. Returns null on null pointer or unknown preset.
/// The returned pointer must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_fingerprint(
    schematic: *const SchematicWrapper,
    preset: *const c_char,
) -> *mut c_char {
    if schematic.is_null() || preset.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let preset_str = unsafe { CStr::from_ptr(preset) }.to_string_lossy();
    let spec = match crate::fingerprint::FingerprintSpec::from_preset(&preset_str) {
        Some(spec) => spec,
        None => {
            set_last_error(format!(
                "schematic_fingerprint: unknown preset '{}'",
                preset_str
            ));
            return ptr::null_mut();
        }
    };
    let hex = crate::fingerprint::fingerprint(s, &spec).to_hex();
    CString::new(hex).unwrap_or_default().into_raw()
}

/// Computes the structural signature (JSON) of a schematic for the given preset.
/// Returns null on null pointer or unknown preset.
/// The returned pointer must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_signature(
    schematic: *const SchematicWrapper,
    preset: *const c_char,
) -> *mut c_char {
    if schematic.is_null() || preset.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let preset_str = unsafe { CStr::from_ptr(preset) }.to_string_lossy();
    let spec = match crate::fingerprint::FingerprintSpec::from_preset(&preset_str) {
        Some(spec) => spec,
        None => {
            set_last_error(format!(
                "schematic_signature: unknown preset '{}'",
                preset_str
            ));
            return ptr::null_mut();
        }
    };
    let json = crate::fingerprint::signature(s, &spec).to_json();
    CString::new(json).unwrap_or_default().into_raw()
}

/// Translation-invariant fuzzy distance between two builds' footprints.
/// Returns -1.0 on null pointer or unknown preset.
#[no_mangle]
pub extern "C" fn schematic_footprint_distance(
    a: *const SchematicWrapper,
    b: *const SchematicWrapper,
    preset: *const c_char,
) -> c_float {
    if a.is_null() || b.is_null() || preset.is_null() {
        return -1.0;
    }
    let sa = unsafe { &*(*a).0 };
    let sb = unsafe { &*(*b).0 };
    let preset_str = unsafe { CStr::from_ptr(preset) }.to_string_lossy();
    let spec = match crate::fingerprint::FingerprintSpec::from_preset(&preset_str) {
        Some(spec) => spec,
        None => {
            set_last_error(format!(
                "schematic_footprint_distance: unknown preset '{}'",
                preset_str
            ));
            return -1.0;
        }
    };
    crate::fingerprint::footprint_distance(sa, sb, &spec)
}

/// The schematic's translation/scale-invariant FFT shape footprint as a JSON
/// array of floats. Returns NULL on null pointer or unknown preset. Free the
/// returned string with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_footprint(
    ptr: *const SchematicWrapper,
    preset: *const c_char,
) -> *mut c_char {
    if ptr.is_null() || preset.is_null() {
        return std::ptr::null_mut();
    }
    let s = unsafe { &*(*ptr).0 };
    let preset_str = unsafe { CStr::from_ptr(preset) }.to_string_lossy();
    let spec = match crate::fingerprint::FingerprintSpec::from_preset(&preset_str) {
        Some(spec) => spec,
        None => {
            set_last_error(format!(
                "schematic_footprint: unknown preset '{}'",
                preset_str
            ));
            return std::ptr::null_mut();
        }
    };
    let v = crate::fingerprint::footprint(s, &spec).0;
    match serde_json::to_string(&v) {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Returns true if two schematics share the same fingerprint for the given
/// preset. Returns false on null pointer or unknown preset.
#[no_mangle]
pub extern "C" fn schematic_is_duplicate_of(
    a: *const SchematicWrapper,
    b: *const SchematicWrapper,
    preset: *const c_char,
) -> bool {
    if a.is_null() || b.is_null() || preset.is_null() {
        return false;
    }
    let sa = unsafe { &*(*a).0 };
    let sb = unsafe { &*(*b).0 };
    let preset_str = unsafe { CStr::from_ptr(preset) }.to_string_lossy();
    let spec = match crate::fingerprint::FingerprintSpec::from_preset(&preset_str) {
        Some(spec) => spec,
        None => {
            set_last_error(format!(
                "schematic_is_duplicate_of: unknown preset '{}'",
                preset_str
            ));
            return false;
        }
    };
    crate::fingerprint::is_duplicate(sa, sb, &spec)
}

/// Opaque handle to a computed diff. Free with `diff_free`.
pub struct DiffWrapper(*mut crate::diff::Diff);

/// Computes the diff between two schematics using the given preset (default
/// cost model). Returns null on null pointer or unknown preset.
/// The returned pointer must be freed with `diff_free`.
#[no_mangle]
pub extern "C" fn schematic_diff(
    a: *const SchematicWrapper,
    b: *const SchematicWrapper,
    preset: *const c_char,
) -> *mut DiffWrapper {
    schematic_diff_opts(a, b, preset, -1, -1, -1, -1, ptr::null())
}

/// Computes the diff between two schematics using the given preset with
/// optional cost/symmetry overrides. Negative cost ints mean "unset" (use the
/// preset default); a null `symmetry` means "unset". Returns null on null
/// pointer or unknown preset/symmetry name.
/// The returned pointer must be freed with `diff_free`.
#[no_mangle]
pub extern "C" fn schematic_diff_opts(
    a: *const SchematicWrapper,
    b: *const SchematicWrapper,
    preset: *const c_char,
    cost_add: c_int,
    cost_delete: c_int,
    cost_change: c_int,
    cost_swap: c_int,
    symmetry: *const c_char,
) -> *mut DiffWrapper {
    if a.is_null() || b.is_null() || preset.is_null() {
        return ptr::null_mut();
    }
    let sa = unsafe { &*(*a).0 };
    let sb = unsafe { &*(*b).0 };
    let preset_str = unsafe { CStr::from_ptr(preset) }.to_string_lossy();

    let mut overrides = crate::diff::SpecOverrides::default();
    if cost_add >= 0 {
        overrides.cost_add = Some(cost_add as u32);
    }
    if cost_delete >= 0 {
        overrides.cost_delete = Some(cost_delete as u32);
    }
    if cost_change >= 0 {
        overrides.cost_change = Some(cost_change as u32);
    }
    if cost_swap >= 0 {
        overrides.cost_swap = Some(cost_swap as u32);
    }
    if !symmetry.is_null() {
        let sym_str = unsafe { CStr::from_ptr(symmetry) }.to_string_lossy();
        match crate::fingerprint::symmetry::Symmetry::from_name(&sym_str) {
            Some(sym) => overrides.symmetry = Some(sym),
            None => {
                set_last_error(format!(
                    "schematic_diff_opts: unknown symmetry '{}'",
                    sym_str
                ));
                return ptr::null_mut();
            }
        }
    }

    let spec = match crate::diff::DiffSpec::resolve(&preset_str, &overrides) {
        Some(spec) => spec,
        None => {
            set_last_error(format!(
                "schematic_diff_opts: unknown preset '{}'",
                preset_str
            ));
            return ptr::null_mut();
        }
    };
    let diff = crate::diff::diff(sa, sb, &spec);
    let wrapper = DiffWrapper(Box::into_raw(Box::new(diff)));
    Box::into_raw(Box::new(wrapper))
}

/// Frees the memory associated with a `DiffWrapper`.
#[no_mangle]
pub extern "C" fn diff_free(diff: *mut DiffWrapper) {
    if !diff.is_null() {
        unsafe {
            let wrapper = Box::from_raw(diff);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

/// Returns the edit distance of a diff (0 on null pointer).
#[no_mangle]
pub extern "C" fn diff_distance(diff: *const DiffWrapper) -> u64 {
    if diff.is_null() {
        return 0;
    }
    unsafe { (*(*diff).0).distance }
}

/// Returns the support (alignment confidence) of a diff (0.0 on null pointer).
#[no_mangle]
pub extern "C" fn diff_support(diff: *const DiffWrapper) -> c_float {
    if diff.is_null() {
        return 0.0;
    }
    unsafe { (*(*diff).0).support }
}

/// Serializes a diff to its full JSON representation. Returns null on null
/// pointer. The returned pointer must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn diff_to_json(diff: *const DiffWrapper) -> *mut c_char {
    if diff.is_null() {
        return ptr::null_mut();
    }
    let json = unsafe { (*(*diff).0).to_json() };
    CString::new(json).unwrap_or_default().into_raw()
}

/// Serializes a diff to its compact summary JSON. Returns null on null pointer.
/// The returned pointer must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn diff_summary_json(diff: *const DiffWrapper) -> *mut c_char {
    if diff.is_null() {
        return ptr::null_mut();
    }
    let json = unsafe { (*(*diff).0).summary_json() };
    CString::new(json).unwrap_or_default().into_raw()
}

/// Reconstructs a diff from its JSON representation. Returns null on null
/// pointer or parse error. The returned pointer must be freed with `diff_free`.
#[no_mangle]
pub extern "C" fn diff_from_json(json: *const c_char) -> *mut DiffWrapper {
    if json.is_null() {
        return ptr::null_mut();
    }
    let json_str = unsafe { CStr::from_ptr(json) }.to_string_lossy();
    match crate::diff::Diff::from_json(&json_str) {
        Ok(diff) => {
            let wrapper = DiffWrapper(Box::into_raw(Box::new(diff)));
            Box::into_raw(Box::new(wrapper))
        }
        Err(e) => {
            set_last_error(format!("diff_from_json: {}", e.0));
            ptr::null_mut()
        }
    }
}

/// Returns a new schematic containing only the blocks added in this diff.
/// Returns null on null pointer. Free with `schematic_free`.
#[no_mangle]
pub extern "C" fn diff_added(diff: *const DiffWrapper) -> *mut SchematicWrapper {
    if diff.is_null() {
        return ptr::null_mut();
    }
    box_schematic(unsafe { (*(*diff).0).added() })
}

/// Returns a new schematic containing only the blocks removed in this diff.
/// Returns null on null pointer. Free with `schematic_free`.
#[no_mangle]
pub extern "C" fn diff_removed(diff: *const DiffWrapper) -> *mut SchematicWrapper {
    if diff.is_null() {
        return ptr::null_mut();
    }
    box_schematic(unsafe { (*(*diff).0).removed() })
}

/// Returns a new schematic containing only the blocks changed in this diff.
/// Returns null on null pointer. Free with `schematic_free`.
#[no_mangle]
pub extern "C" fn diff_changed(diff: *const DiffWrapper) -> *mut SchematicWrapper {
    if diff.is_null() {
        return ptr::null_mut();
    }
    box_schematic(unsafe { (*(*diff).0).changed() })
}

/// Returns a new schematic containing only the blocks swapped in this diff.
/// Returns null on null pointer. Free with `schematic_free`.
#[no_mangle]
pub extern "C" fn diff_swapped(diff: *const DiffWrapper) -> *mut SchematicWrapper {
    if diff.is_null() {
        return ptr::null_mut();
    }
    box_schematic(unsafe { (*(*diff).0).swapped() })
}

/// Returns a new schematic with marker blocks summarizing this diff.
/// Returns null on null pointer. Free with `schematic_free`.
#[no_mangle]
pub extern "C" fn diff_markers(diff: *const DiffWrapper) -> *mut SchematicWrapper {
    if diff.is_null() {
        return ptr::null_mut();
    }
    box_schematic(unsafe { (*(*diff).0).markers() })
}

/// Renders a diff overlay on top of an "after" GLB buffer, returning a new GLB
/// buffer. On success, writes the buffer length to `out_len` and returns a heap
/// pointer that must be freed with `diff_free_glb`. Returns null on error (and
/// sets `out_len` to 0 when non-null).
#[cfg(feature = "meshing")]
#[no_mangle]
pub extern "C" fn diff_to_overlay_glb(
    diff: *const DiffWrapper,
    after_glb: *const c_uchar,
    after_len: usize,
    out_len: *mut usize,
) -> *mut c_uchar {
    if !out_len.is_null() {
        unsafe { *out_len = 0 };
    }
    if diff.is_null() || after_glb.is_null() {
        return ptr::null_mut();
    }
    let after_slice = unsafe { std::slice::from_raw_parts(after_glb, after_len) };
    let opts = crate::diff::OverlayOptions::default();
    match unsafe { (*(*diff).0).to_overlay_glb(after_slice, &opts) } {
        Ok(mut data) => {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            if !out_len.is_null() {
                unsafe { *out_len = len };
            }
            ptr
        }
        Err(e) => {
            set_last_error(format!("diff_to_overlay_glb: {}", e.0));
            ptr::null_mut()
        }
    }
}

/// Frees a GLB buffer returned by `diff_to_overlay_glb`.
#[cfg(feature = "meshing")]
#[no_mangle]
pub extern "C" fn diff_free_glb(data: *mut c_uchar, len: usize) {
    if !data.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(data, len, len);
        }
    }
}
