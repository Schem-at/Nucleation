use super::*;

// ============================================================================
// SDF generation
// ============================================================================

/// Builds a schematic by sampling an SDF JSON tree with material rules JSON.
/// When `has_bounds` is 0 the tree's own AABB is used (fails for unbounded
/// trees). Returns null on parse/sampling error.
#[no_mangle]
pub extern "C" fn schematic_from_sdf(
    sdf_json: *const c_char,
    rules_json: *const c_char,
    has_bounds: c_int,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> *mut SchematicWrapper {
    if sdf_json.is_null() || rules_json.is_null() {
        return ptr::null_mut();
    }
    let sdf_str = unsafe { CStr::from_ptr(sdf_json) }.to_string_lossy();
    let rules_str = unsafe { CStr::from_ptr(rules_json) }.to_string_lossy();

    let node = match crate::sdf::SdfNode::from_json(&sdf_str) {
        Ok(n) => n,
        Err(_) => return ptr::null_mut(),
    };
    let rules = match crate::sdf::MaterialRules::from_json(&rules_str) {
        Ok(r) => r,
        Err(_) => return ptr::null_mut(),
    };
    let bounds = if has_bounds != 0 {
        Some(crate::sdf::SampleBounds {
            min: [min_x, min_y, min_z],
            max: [max_x, max_y, max_z],
        })
    } else {
        None
    };
    match crate::sdf::sample_to_schematic(&node, &rules, bounds, "sdf") {
        Ok(schematic) => {
            let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
            Box::into_raw(Box::new(wrapper))
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Evaluates an SDF JSON tree at a point. Returns 0 and writes the signed
/// distance to `out_distance` on success, -1 on parse error / null args.
#[no_mangle]
pub extern "C" fn sdf_eval(
    sdf_json: *const c_char,
    x: f32,
    y: f32,
    z: f32,
    out_distance: *mut f32,
) -> c_int {
    if sdf_json.is_null() || out_distance.is_null() {
        return -1;
    }
    let sdf_str = unsafe { CStr::from_ptr(sdf_json) }.to_string_lossy();
    match crate::sdf::SdfNode::from_json(&sdf_str) {
        Ok(node) => {
            unsafe { *out_distance = node.eval(x, y, z) };
            0
        }
        Err(_) => -1,
    }
}
