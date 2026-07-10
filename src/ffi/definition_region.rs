use super::schematic::CBoundingBox;
use super::*;

// --- Schematic Definition Region Methods ---

pub struct DefinitionRegionWrapper(pub(crate) DefinitionRegion);

#[no_mangle]
pub extern "C" fn schematic_add_definition_region(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    region: *const DefinitionRegionWrapper,
) -> c_int {
    if schematic.is_null() || name.is_null() || region.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let r = unsafe { &(*region).0 };
    s.definition_regions.insert(n, r.clone());
    0
}

#[no_mangle]
pub extern "C" fn schematic_get_definition_region(
    schematic: *const SchematicWrapper,
    name: *const c_char,
) -> *mut DefinitionRegionWrapper {
    if schematic.is_null() || name.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy() };
    match s.definition_regions.get(n.as_ref()) {
        Some(region) => Box::into_raw(Box::new(DefinitionRegionWrapper(region.clone()))),
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn schematic_remove_definition_region(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
) -> c_int {
    if schematic.is_null() || name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy() };
    if s.definition_regions.remove(n.as_ref()).is_some() {
        0
    } else {
        -2
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_definition_region_names(
    schematic: *const SchematicWrapper,
) -> StringArray {
    if schematic.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let names: Vec<String> = s.definition_regions.keys().cloned().collect();
    vec_string_to_string_array(names)
}

#[no_mangle]
pub extern "C" fn schematic_create_definition_region(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
) -> c_int {
    if schematic.is_null() || name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    s.definition_regions.insert(n, DefinitionRegion::new());
    0
}

#[no_mangle]
pub extern "C" fn schematic_create_definition_region_from_point(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if schematic.is_null() || name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let mut region = DefinitionRegion::new();
    region.add_point(x, y, z);
    s.definition_regions.insert(n, region);
    0
}

#[no_mangle]
pub extern "C" fn schematic_create_definition_region_from_bounds(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> c_int {
    if schematic.is_null() || name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let mut region = DefinitionRegion::new();
    region.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
    s.definition_regions.insert(n, region);
    0
}

#[no_mangle]
pub extern "C" fn schematic_create_region(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> *mut DefinitionRegionWrapper {
    if schematic.is_null() || name.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let mut region = DefinitionRegion::new();
    region.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
    s.definition_regions.insert(n, region.clone());
    Box::into_raw(Box::new(DefinitionRegionWrapper(region)))
}

#[no_mangle]
pub extern "C" fn schematic_update_region(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    region: *const DefinitionRegionWrapper,
) -> c_int {
    if schematic.is_null() || name.is_null() || region.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let r = unsafe { &(*region).0 };
    s.definition_regions.insert(n, r.clone());
    0
}

#[no_mangle]
pub extern "C" fn schematic_definition_region_add_bounds(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> c_int {
    if schematic.is_null() || name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy() };
    match s.definition_regions.get_mut(n.as_ref()) {
        Some(region) => {
            region.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
            0
        }
        None => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_definition_region_add_point(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if schematic.is_null() || name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy() };
    match s.definition_regions.get_mut(n.as_ref()) {
        Some(region) => {
            region.add_point(x, y, z);
            0
        }
        None => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_definition_region_set_metadata(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    key: *const c_char,
    value: *const c_char,
) -> c_int {
    if schematic.is_null() || name.is_null() || key.is_null() || value.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy() };
    let k = unsafe { CStr::from_ptr(key).to_string_lossy().into_owned() };
    let v = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };
    match s.definition_regions.get_mut(n.as_ref()) {
        Some(region) => {
            region.metadata.insert(k, v);
            0
        }
        None => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_definition_region_shift(
    schematic: *mut SchematicWrapper,
    name: *const c_char,
    dx: c_int,
    dy: c_int,
    dz: c_int,
) -> c_int {
    if schematic.is_null() || name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy() };
    match s.definition_regions.get_mut(n.as_ref()) {
        Some(region) => {
            region.shift(dx, dy, dz);
            0
        }
        None => -2,
    }
}

// --- DefinitionRegion Wrapper ---

#[no_mangle]
pub extern "C" fn definitionregion_new() -> *mut DefinitionRegionWrapper {
    Box::into_raw(Box::new(DefinitionRegionWrapper(DefinitionRegion::new())))
}

#[no_mangle]
pub extern "C" fn definitionregion_free(ptr: *mut DefinitionRegionWrapper) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_from_bounds(
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> *mut DefinitionRegionWrapper {
    Box::into_raw(Box::new(DefinitionRegionWrapper(
        DefinitionRegion::from_bounds((min_x, min_y, min_z), (max_x, max_y, max_z)),
    )))
}

#[no_mangle]
pub extern "C" fn definitionregion_add_bounds(
    ptr: *mut DefinitionRegionWrapper,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    r.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_add_point(
    ptr: *mut DefinitionRegionWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    r.add_point(x, y, z);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_set_metadata(
    ptr: *mut DefinitionRegionWrapper,
    key: *const c_char,
    value: *const c_char,
) -> c_int {
    if ptr.is_null() || key.is_null() || value.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    let k = unsafe { CStr::from_ptr(key).to_string_lossy().into_owned() };
    let v = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };
    r.set_metadata(k, v);
    0
}

/// Fluent alias for `definitionregion_set_metadata`. Same semantics —
/// mutates in place and returns 0 on success.
#[no_mangle]
pub extern "C" fn definitionregion_with_metadata(
    ptr: *mut DefinitionRegionWrapper,
    key: *const c_char,
    value: *const c_char,
) -> c_int {
    definitionregion_set_metadata(ptr, key, value)
}

#[no_mangle]
pub extern "C" fn definitionregion_get_metadata(
    ptr: *const DefinitionRegionWrapper,
    key: *const c_char,
) -> *mut c_char {
    if ptr.is_null() || key.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    let k = unsafe { CStr::from_ptr(key).to_string_lossy() };
    match r.get_metadata(&k) {
        Some(v) => CString::new(v.clone()).unwrap().into_raw(),
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_is_empty(ptr: *const DefinitionRegionWrapper) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    if r.is_empty() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_volume(ptr: *const DefinitionRegionWrapper) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    r.volume() as c_int
}

#[no_mangle]
pub extern "C" fn definitionregion_contains(
    ptr: *const DefinitionRegionWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    if r.contains(x, y, z) {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_shift(
    ptr: *mut DefinitionRegionWrapper,
    dx: c_int,
    dy: c_int,
    dz: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    r.shift(dx, dy, dz);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_expand(
    ptr: *mut DefinitionRegionWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    r.expand(x, y, z);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_contract(
    ptr: *mut DefinitionRegionWrapper,
    amount: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    r.contract(amount);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_intersect(
    a: *const DefinitionRegionWrapper,
    b: *const DefinitionRegionWrapper,
) -> *mut DefinitionRegionWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let ra = unsafe { &(*a).0 };
    let rb = unsafe { &(*b).0 };
    let result = ra.intersected(rb);
    Box::into_raw(Box::new(DefinitionRegionWrapper(result)))
}

#[no_mangle]
pub extern "C" fn definitionregion_union(
    a: *const DefinitionRegionWrapper,
    b: *const DefinitionRegionWrapper,
) -> *mut DefinitionRegionWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let ra = unsafe { &(*a).0 };
    let rb = unsafe { &(*b).0 };
    let result = ra.union(rb);
    Box::into_raw(Box::new(DefinitionRegionWrapper(result)))
}

#[no_mangle]
pub extern "C" fn definitionregion_subtract(
    a: *const DefinitionRegionWrapper,
    b: *const DefinitionRegionWrapper,
) -> *mut DefinitionRegionWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let ra = unsafe { &(*a).0 };
    let rb = unsafe { &(*b).0 };
    let result = ra.subtracted(rb);
    Box::into_raw(Box::new(DefinitionRegionWrapper(result)))
}

#[no_mangle]
pub extern "C" fn definitionregion_get_bounds(ptr: *const DefinitionRegionWrapper) -> IntArray {
    let empty = IntArray {
        data: ptr::null_mut(),
        len: 0,
    };
    if ptr.is_null() {
        return empty;
    }
    let r = unsafe { &(*ptr).0 };
    match r.get_bounds() {
        Some(bbox) => {
            let vals = vec![
                bbox.min.0, bbox.min.1, bbox.min.2, bbox.max.0, bbox.max.1, bbox.max.2,
            ];
            let mut boxed = vals.into_boxed_slice();
            let p = boxed.as_mut_ptr();
            let len = boxed.len();
            std::mem::forget(boxed);
            IntArray { data: p, len }
        }
        None => empty,
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_dimensions(ptr: *const DefinitionRegionWrapper) -> IntArray {
    if ptr.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    let (w, h, l) = r.dimensions();
    let dims = vec![w, h, l];
    let mut boxed = dims.into_boxed_slice();
    let p = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    IntArray { data: p, len }
}

#[no_mangle]
pub extern "C" fn definitionregion_center(ptr: *const DefinitionRegionWrapper) -> IntArray {
    if ptr.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    match r.center() {
        Some((x, y, z)) => {
            let vals = vec![x, y, z];
            let mut boxed = vals.into_boxed_slice();
            let p = boxed.as_mut_ptr();
            let len = boxed.len();
            std::mem::forget(boxed);
            IntArray { data: p, len }
        }
        None => IntArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_positions(ptr: *const DefinitionRegionWrapper) -> IntArray {
    if ptr.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    let mut flat: Vec<c_int> = Vec::new();
    for (x, y, z) in r.iter_positions() {
        flat.push(x);
        flat.push(y);
        flat.push(z);
    }
    let mut boxed = flat.into_boxed_slice();
    let p = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    IntArray { data: p, len }
}

#[no_mangle]
pub extern "C" fn definitionregion_filter_by_block(
    ptr: *const DefinitionRegionWrapper,
    schematic: *const SchematicWrapper,
    block_name: *const c_char,
) -> *mut DefinitionRegionWrapper {
    if ptr.is_null() || schematic.is_null() || block_name.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    let s = unsafe { &*(*schematic).0 };
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy() };
    let mut filtered = r.clone();
    filtered.filter_by_block(s, &name);
    Box::into_raw(Box::new(DefinitionRegionWrapper(filtered)))
}

// --- Additional DefinitionRegion Methods ---

#[no_mangle]
pub extern "C" fn definitionregion_from_positions(
    positions: *const c_int,
    positions_len: usize,
) -> *mut DefinitionRegionWrapper {
    if positions.is_null() || !positions_len.is_multiple_of(3) {
        return ptr::null_mut();
    }
    let slice = unsafe { std::slice::from_raw_parts(positions, positions_len) };
    let pts: Vec<(i32, i32, i32)> = slice.chunks(3).map(|c| (c[0], c[1], c[2])).collect();
    Box::into_raw(Box::new(DefinitionRegionWrapper(
        DefinitionRegion::from_positions(&pts),
    )))
}

#[no_mangle]
pub extern "C" fn definitionregion_from_bounding_boxes(
    boxes: *const c_int,
    boxes_len: usize,
) -> *mut DefinitionRegionWrapper {
    if boxes.is_null() || !boxes_len.is_multiple_of(6) {
        return ptr::null_mut();
    }
    let slice = unsafe { std::slice::from_raw_parts(boxes, boxes_len) };
    let bbs: Vec<((i32, i32, i32), (i32, i32, i32))> = slice
        .chunks(6)
        .map(|c| ((c[0], c[1], c[2]), (c[3], c[4], c[5])))
        .collect();
    Box::into_raw(Box::new(DefinitionRegionWrapper(
        DefinitionRegion::from_bounding_boxes(bbs),
    )))
}

#[no_mangle]
pub extern "C" fn definitionregion_box_count(ptr: *const DefinitionRegionWrapper) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    r.box_count() as c_int
}

#[no_mangle]
pub extern "C" fn definitionregion_get_box(
    ptr: *const DefinitionRegionWrapper,
    index: usize,
) -> CBoundingBox {
    let empty = CBoundingBox {
        min_x: 0,
        min_y: 0,
        min_z: 0,
        max_x: 0,
        max_y: 0,
        max_z: 0,
    };
    if ptr.is_null() {
        return empty;
    }
    let r = unsafe { &(*ptr).0 };
    match r.get_box(index) {
        Some((min, max)) => CBoundingBox {
            min_x: min.0,
            min_y: min.1,
            min_z: min.2,
            max_x: max.0,
            max_y: max.1,
            max_z: max.2,
        },
        None => empty,
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_get_boxes(ptr: *const DefinitionRegionWrapper) -> IntArray {
    if ptr.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    let boxes = r.get_boxes();
    let mut flat: Vec<c_int> = Vec::with_capacity(boxes.len() * 6);
    for (min, max) in boxes {
        flat.push(min.0);
        flat.push(min.1);
        flat.push(min.2);
        flat.push(max.0);
        flat.push(max.1);
        flat.push(max.2);
    }
    let mut boxed = flat.into_boxed_slice();
    let p = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    IntArray { data: p, len }
}

#[no_mangle]
pub extern "C" fn definitionregion_get_all_metadata(
    ptr: *const DefinitionRegionWrapper,
) -> StringArray {
    if ptr.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    let pairs: Vec<String> = r
        .metadata
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    vec_string_to_string_array(pairs)
}

#[no_mangle]
pub extern "C" fn definitionregion_metadata_keys(
    ptr: *const DefinitionRegionWrapper,
) -> StringArray {
    if ptr.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    let keys: Vec<String> = r.metadata_keys().into_iter().cloned().collect();
    vec_string_to_string_array(keys)
}

#[no_mangle]
pub extern "C" fn definitionregion_is_contiguous(ptr: *const DefinitionRegionWrapper) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    if r.is_contiguous() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_connected_components(
    ptr: *const DefinitionRegionWrapper,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    r.connected_components() as c_int
}

#[no_mangle]
pub extern "C" fn definitionregion_filter_by_properties(
    ptr: *const DefinitionRegionWrapper,
    schematic: *const SchematicWrapper,
    properties_json: *const c_char,
) -> *mut DefinitionRegionWrapper {
    if ptr.is_null() || schematic.is_null() || properties_json.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    let s = unsafe { &*(*schematic).0 };
    let json_str = unsafe { CStr::from_ptr(properties_json).to_string_lossy() };
    let props: HashMap<String, String> = serde_json::from_str(&json_str).unwrap_or_default();
    let result = r.filter_by_properties(s, &props);
    Box::into_raw(Box::new(DefinitionRegionWrapper(result)))
}

#[no_mangle]
pub extern "C" fn definitionregion_intersected(
    a: *const DefinitionRegionWrapper,
    b: *const DefinitionRegionWrapper,
) -> *mut DefinitionRegionWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let ra = unsafe { &(*a).0 };
    let rb = unsafe { &(*b).0 };
    Box::into_raw(Box::new(DefinitionRegionWrapper(ra.intersected(rb))))
}

#[no_mangle]
pub extern "C" fn definitionregion_subtracted(
    a: *const DefinitionRegionWrapper,
    b: *const DefinitionRegionWrapper,
) -> *mut DefinitionRegionWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let ra = unsafe { &(*a).0 };
    let rb = unsafe { &(*b).0 };
    Box::into_raw(Box::new(DefinitionRegionWrapper(ra.subtracted(rb))))
}

#[no_mangle]
pub extern "C" fn definitionregion_simplify(ptr: *mut DefinitionRegionWrapper) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    r.simplify();
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_positions_sorted(
    ptr: *const DefinitionRegionWrapper,
) -> IntArray {
    if ptr.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    let sorted = r.iter_positions_sorted();
    let mut flat: Vec<c_int> = Vec::with_capacity(sorted.len() * 3);
    for (x, y, z) in sorted {
        flat.push(x);
        flat.push(y);
        flat.push(z);
    }
    let mut boxed = flat.into_boxed_slice();
    let p = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    IntArray { data: p, len }
}

#[no_mangle]
pub extern "C" fn definitionregion_shifted(
    ptr: *const DefinitionRegionWrapper,
    dx: c_int,
    dy: c_int,
    dz: c_int,
) -> *mut DefinitionRegionWrapper {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    Box::into_raw(Box::new(DefinitionRegionWrapper(r.shifted(dx, dy, dz))))
}

#[no_mangle]
pub extern "C" fn definitionregion_expanded(
    ptr: *const DefinitionRegionWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut DefinitionRegionWrapper {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    Box::into_raw(Box::new(DefinitionRegionWrapper(r.expanded(x, y, z))))
}

#[no_mangle]
pub extern "C" fn definitionregion_contracted(
    ptr: *const DefinitionRegionWrapper,
    amount: c_int,
) -> *mut DefinitionRegionWrapper {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    Box::into_raw(Box::new(DefinitionRegionWrapper(r.contracted(amount))))
}

#[no_mangle]
pub extern "C" fn definitionregion_copy(
    ptr: *const DefinitionRegionWrapper,
) -> *mut DefinitionRegionWrapper {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    Box::into_raw(Box::new(DefinitionRegionWrapper(r.copy())))
}

#[no_mangle]
pub extern "C" fn definitionregion_clone_region(
    ptr: *const DefinitionRegionWrapper,
) -> *mut DefinitionRegionWrapper {
    definitionregion_copy(ptr)
}

#[no_mangle]
pub extern "C" fn definitionregion_set_color(
    ptr: *mut DefinitionRegionWrapper,
    color: u32,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    r.set_color(color);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_merge(
    ptr: *mut DefinitionRegionWrapper,
    other: *const DefinitionRegionWrapper,
) -> c_int {
    if ptr.is_null() || other.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    let o = unsafe { &(*other).0 };
    r.merge(o);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_union_into(
    ptr: *mut DefinitionRegionWrapper,
    other: *const DefinitionRegionWrapper,
) -> c_int {
    if ptr.is_null() || other.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    let o = unsafe { &(*other).0 };
    r.union_into(o);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_intersects_bounds(
    ptr: *const DefinitionRegionWrapper,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    if r.intersects_bounds((min_x, min_y, min_z), (max_x, max_y, max_z)) {
        1
    } else {
        0
    }
}

#[repr(C)]
pub struct CFloatArray {
    pub(super) data: *mut c_float,
    pub(super) len: usize,
}

#[no_mangle]
pub extern "C" fn free_float_array(array: CFloatArray) {
    if !array.data.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(array.data, array.len, array.len);
        }
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_center_f32(ptr: *const DefinitionRegionWrapper) -> CFloatArray {
    if ptr.is_null() {
        return CFloatArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let r = unsafe { &(*ptr).0 };
    match r.center_f32() {
        Some((x, y, z)) => {
            let vals = vec![x, y, z];
            let mut boxed = vals.into_boxed_slice();
            let p = boxed.as_mut_ptr();
            let len = boxed.len();
            std::mem::forget(boxed);
            CFloatArray { data: p, len }
        }
        None => CFloatArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

#[no_mangle]
pub extern "C" fn definitionregion_exclude_block(
    ptr: *mut DefinitionRegionWrapper,
    schematic: *const SchematicWrapper,
    block_name: *const c_char,
) -> c_int {
    if ptr.is_null() || schematic.is_null() || block_name.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    let s = unsafe { &*(*schematic).0 };
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy() };
    r.exclude_block(s, &name);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_add_filter(
    ptr: *mut DefinitionRegionWrapper,
    filter: *const c_char,
) -> c_int {
    if ptr.is_null() || filter.is_null() {
        return -1;
    }
    let r = unsafe { &mut (*ptr).0 };
    let f = unsafe { CStr::from_ptr(filter).to_string_lossy().into_owned() };
    r.set_metadata("filter", f);
    0
}

#[no_mangle]
pub extern "C" fn definitionregion_set_metadata_mut(
    ptr: *mut DefinitionRegionWrapper,
    key: *const c_char,
    value: *const c_char,
) -> c_int {
    definitionregion_set_metadata(ptr, key, value)
}

#[no_mangle]
pub extern "C" fn definitionregion_blocks(
    ptr: *const DefinitionRegionWrapper,
    schematic: *const SchematicWrapper,
) -> CBlockArray {
    let empty = CBlockArray {
        data: ptr::null_mut(),
        len: 0,
    };
    if ptr.is_null() || schematic.is_null() {
        return empty;
    }
    let r = unsafe { &(*ptr).0 };
    let s = unsafe { &*(*schematic).0 };
    let blocks: Vec<CBlock> = r
        .iter_positions()
        .filter_map(|(x, y, z)| {
            s.get_block(x, y, z).map(|block| {
                let props_json = serde_json::to_string(&block.properties).unwrap_or_default();
                CBlock {
                    x,
                    y,
                    z,
                    name: CString::new(block.name.as_str()).unwrap().into_raw(),
                    properties_json: CString::new(props_json).unwrap().into_raw(),
                }
            })
        })
        .collect();
    let mut blocks = blocks;
    let p = blocks.as_mut_ptr();
    let len = blocks.len();
    std::mem::forget(blocks);
    CBlockArray { data: p, len }
}

#[no_mangle]
pub extern "C" fn definitionregion_sync(
    ptr: *const DefinitionRegionWrapper,
    schematic: *mut SchematicWrapper,
    name: *const c_char,
) -> c_int {
    if ptr.is_null() || schematic.is_null() || name.is_null() {
        return -1;
    }
    let r = unsafe { &(*ptr).0 };
    let s = unsafe { &mut *(*schematic).0 };
    let n = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    s.definition_regions.insert(n, r.clone());
    0
}
