use super::*;

// ---------------------------------------------------------------------------
// Auto-stack: periodicity detection + resize
// ---------------------------------------------------------------------------
/// Detect repeating structures (region coverage). Returns a JSON array string
/// (free with `free_string`); each element has `mode`, `vectors`, `coverage`,
/// `region_min`/`region_max`, `cell_min`/`cell_max`, `label`.
#[cfg(feature = "autostack")]
#[no_mangle]
pub extern "C" fn schematic_detect_structures(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let json = crate::autostack::detect_structures_json(s);
    CString::new(json).unwrap_or_default().into_raw()
}

/// Graph-based detection: recovers diagonal lattice periods (e.g. a diagonal
/// carry-chain adder) that `schematic_detect_structures` misses, via the redstone
/// logic graph. Returns a JSON array string (free with `free_string`); `"[]"` for
/// non-redstone builds. Requires the `simulation` feature.
#[cfg(all(feature = "autostack", feature = "simulation"))]
#[no_mangle]
pub extern "C" fn schematic_detect_structures_graph(
    schematic: *const SchematicWrapper,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let json = crate::autostack::detect_structures_graph_json(s);
    CString::new(json).unwrap_or_default().into_raw()
}

/// Resize a 1D / diagonal structure along its period vector. Returns a new
/// schematic (free with `schematic_free`), or null on error.
#[cfg(feature = "autostack")]
#[no_mangle]
pub extern "C" fn schematic_autostack_resize_1d(
    schematic: *const SchematicWrapper,
    vx: c_int,
    vy: c_int,
    vz: c_int,
    units: u32,
) -> *mut SchematicWrapper {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    match crate::autostack::resize_1d(s, [vx, vy, vz], units as usize) {
        Ok(new) => Box::into_raw(Box::new(SchematicWrapper(Box::into_raw(Box::new(new))))),
        Err(_) => ptr::null_mut(),
    }
}

/// Resize a 2D structure to `n1`×`n2` cells along the two period vectors.
/// Returns a new schematic (free with `schematic_free`), or null on error.
#[cfg(feature = "autostack")]
#[no_mangle]
pub extern "C" fn schematic_autostack_resize_2d(
    schematic: *const SchematicWrapper,
    v1x: c_int,
    v1y: c_int,
    v1z: c_int,
    v2x: c_int,
    v2y: c_int,
    v2z: c_int,
    n1: u32,
    n2: u32,
) -> *mut SchematicWrapper {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    match crate::autostack::resize_2d(
        s,
        [v1x, v1y, v1z],
        [v2x, v2y, v2z],
        n1 as usize,
        n2 as usize,
    ) {
        Ok(new) => Box::into_raw(Box::new(SchematicWrapper(Box::into_raw(Box::new(new))))),
        Err(_) => ptr::null_mut(),
    }
}
