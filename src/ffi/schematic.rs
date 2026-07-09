use super::*;
use crate::formats::schematic;

// --- Schematic Lifecycle ---

/// Creates a new, empty schematic.
/// The returned pointer must be freed with `schematic_free`.
#[no_mangle]
pub extern "C" fn schematic_new() -> *mut SchematicWrapper {
    let schematic = UniversalSchematic::new("Default".to_string());
    let wrapper = SchematicWrapper(Box::into_raw(Box::new(schematic)));
    Box::into_raw(Box::new(wrapper))
}

/// Frees the memory associated with a SchematicWrapper.
#[no_mangle]
pub extern "C" fn schematic_free(schematic: *mut SchematicWrapper) {
    if !schematic.is_null() {
        unsafe {
            let wrapper = Box::from_raw(schematic);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

// --- Data I/O ---

/// Populates a schematic from raw byte data, auto-detecting the format.
/// Supports Litematic, Sponge Schematic, and McStructure (Bedrock) formats.
/// Returns 0 on success, -1 for null pointers, -2 for parse error, -3 for unknown format.
#[no_mangle]
pub extern "C" fn schematic_from_data(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };

    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => return -2,
    };
    match manager.read(data_slice) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(e) => {
            let detected = manager.detect_format(data_slice);
            set_last_error(format!(
                "schematic_from_data: detected={:?}, len={}, error={}",
                detected, data_len, e
            ));
            // Check if any format was detected but failed to parse
            if detected.is_some() {
                -2 // Parse error
            } else {
                -3 // Unknown format
            }
        }
    }
}

/// Populates a schematic from Litematic data.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_from_litematic(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match litematic::from_litematic(data_slice) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Converts the schematic to Litematic format.
/// The returned ByteArray must be freed with `free_byte_array`.
#[no_mangle]
pub extern "C" fn schematic_to_litematic(schematic: *const SchematicWrapper) -> ByteArray {
    if schematic.is_null() {
        return ByteArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    match litematic::to_litematic(s) {
        Ok(data) => {
            let mut data = data;
            let ptr = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            ByteArray { data: ptr, len }
        }
        Err(_) => ByteArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

/// Populates a schematic from classic `.schematic` data.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_from_schematic(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match schematic::from_schematic(data_slice) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Converts the schematic to classic `.schematic` format.
/// The returned ByteArray must be freed with `free_byte_array`.
#[no_mangle]
pub extern "C" fn schematic_to_schematic(schematic: *const SchematicWrapper) -> ByteArray {
    if schematic.is_null() {
        return ByteArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    match schematic::to_schematic(s) {
        Ok(data) => {
            let mut data = data;
            let ptr = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            ByteArray { data: ptr, len }
        }
        Err(_) => ByteArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

/// Populates a schematic from snapshot (fast binary) data.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_from_snapshot(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match crate::formats::snapshot::from_snapshot(data_slice) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Converts the schematic to snapshot (fast binary) format.
/// The returned ByteArray must be freed with `free_byte_array`.
#[no_mangle]
pub extern "C" fn schematic_to_snapshot(schematic: *const SchematicWrapper) -> ByteArray {
    if schematic.is_null() {
        return ByteArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    match crate::formats::snapshot::to_snapshot(s) {
        Ok(mut data) => {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            ByteArray { data: ptr, len }
        }
        Err(_) => ByteArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

/// Populates a schematic from McStructure (Bedrock) data.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_from_mcstructure(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match mcstructure::from_mcstructure(data_slice) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Converts the schematic to McStructure (Bedrock) format.
/// The returned ByteArray must be freed with `free_byte_array`.
#[no_mangle]
pub extern "C" fn schematic_to_mcstructure(schematic: *const SchematicWrapper) -> ByteArray {
    if schematic.is_null() {
        return ByteArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    match mcstructure::to_mcstructure(s) {
        Ok(data) => {
            let mut data = data;
            let ptr = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            ByteArray { data: ptr, len }
        }
        Err(_) => ByteArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

// --- MCA / World Import/Export ---

/// C-compatible file entry for world export.
#[repr(C)]
pub struct CFileEntry {
    path: *mut c_char,
    data: *mut c_uchar,
    data_len: usize,
}

/// C-compatible file map for world export.
#[repr(C)]
pub struct CFileMap {
    entries: *mut CFileEntry,
    len: usize,
}

/// Frees a CFileMap returned by schematic_to_world.
#[no_mangle]
pub extern "C" fn free_file_map(map: CFileMap) {
    if map.entries.is_null() {
        return;
    }
    let entries = unsafe { Vec::from_raw_parts(map.entries, map.len, map.len) };
    for entry in entries {
        if !entry.path.is_null() {
            unsafe {
                let _ = CString::from_raw(entry.path);
            }
        }
        if !entry.data.is_null() {
            unsafe {
                let _ = Vec::from_raw_parts(entry.data, entry.data_len, entry.data_len);
            }
        }
    }
}

/// Import from a single MCA region file.
#[no_mangle]
pub extern "C" fn schematic_from_mca(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match crate::formats::world::from_mca(data_slice) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Import from MCA with coordinate bounds.
#[no_mangle]
pub extern "C" fn schematic_from_mca_bounded(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match crate::formats::world::from_mca_bounded(
        data_slice, min_x, min_y, min_z, max_x, max_y, max_z,
    ) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Import from a zipped world folder.
#[no_mangle]
pub extern "C" fn schematic_from_world_zip(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match crate::formats::world::from_world_zip(data_slice) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Import from zipped world with coordinate bounds.
#[no_mangle]
pub extern "C" fn schematic_from_world_zip_bounded(
    schematic: *mut SchematicWrapper,
    data: *const c_uchar,
    data_len: usize,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> c_int {
    if schematic.is_null() || data.is_null() {
        return -1;
    }
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
    let s = unsafe { &mut *(*schematic).0 };
    match crate::formats::world::from_world_zip_bounded(
        data_slice, min_x, min_y, min_z, max_x, max_y, max_z,
    ) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Import from a Minecraft world directory path.
#[no_mangle]
pub extern "C" fn schematic_from_world_directory(
    schematic: *mut SchematicWrapper,
    path: *const c_char,
) -> c_int {
    if schematic.is_null() || path.is_null() {
        return -1;
    }
    let path_str = unsafe { CStr::from_ptr(path) };
    let path_str = match path_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let s = unsafe { &mut *(*schematic).0 };
    match crate::formats::world::from_world_directory(std::path::Path::new(path_str)) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Import from world directory with coordinate bounds.
#[no_mangle]
pub extern "C" fn schematic_from_world_directory_bounded(
    schematic: *mut SchematicWrapper,
    path: *const c_char,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> c_int {
    if schematic.is_null() || path.is_null() {
        return -1;
    }
    let path_str = unsafe { CStr::from_ptr(path) };
    let path_str = match path_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let s = unsafe { &mut *(*schematic).0 };
    match crate::formats::world::from_world_directory_bounded(
        std::path::Path::new(path_str),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
    ) {
        Ok(res) => {
            *s = res;
            0
        }
        Err(_) => -2,
    }
}

/// Export schematic as a Minecraft world. Returns CFileMap.
/// The returned CFileMap must be freed with `free_file_map`.
#[no_mangle]
pub extern "C" fn schematic_to_world(
    schematic: *const SchematicWrapper,
    options_json: *const c_char,
) -> CFileMap {
    let empty = CFileMap {
        entries: ptr::null_mut(),
        len: 0,
    };

    if schematic.is_null() {
        return empty;
    }

    let s = unsafe { &*(*schematic).0 };

    let options = if options_json.is_null() {
        None
    } else {
        let json_str = unsafe { CStr::from_ptr(options_json) };
        match json_str.to_str() {
            Ok(json) => {
                match serde_json::from_str::<crate::formats::world::WorldExportOptions>(json) {
                    Ok(opts) => Some(opts),
                    Err(_) => return empty,
                }
            }
            Err(_) => return empty,
        }
    };

    let files = match crate::formats::world::to_world(s, options) {
        Ok(f) => f,
        Err(_) => return empty,
    };

    let mut entries: Vec<CFileEntry> = Vec::with_capacity(files.len());
    for (path, data) in files {
        let c_path = match CString::new(path) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let mut data = data;
        let data_ptr = data.as_mut_ptr();
        let data_len = data.len();
        std::mem::forget(data);

        entries.push(CFileEntry {
            path: c_path.into_raw(),
            data: data_ptr,
            data_len,
        });
    }

    let len = entries.len();
    let ptr = entries.as_mut_ptr();
    std::mem::forget(entries);

    CFileMap { entries: ptr, len }
}

/// Export and write world files to a directory.
#[no_mangle]
pub extern "C" fn schematic_save_world(
    schematic: *const SchematicWrapper,
    directory: *const c_char,
    options_json: *const c_char,
) -> c_int {
    if schematic.is_null() || directory.is_null() {
        return -1;
    }

    let s = unsafe { &*(*schematic).0 };

    let dir_str = unsafe { CStr::from_ptr(directory) };
    let dir_str = match dir_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let options = if options_json.is_null() {
        None
    } else {
        let json_str = unsafe { CStr::from_ptr(options_json) };
        match json_str.to_str() {
            Ok(json) => {
                match serde_json::from_str::<crate::formats::world::WorldExportOptions>(json) {
                    Ok(opts) => Some(opts),
                    Err(_) => return -3,
                }
            }
            Err(_) => return -1,
        }
    };

    match crate::formats::world::save_world(s, std::path::Path::new(dir_str), options) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Export a schematic as a zipped Minecraft world. Returns a ByteArray.
/// Caller must free the returned ByteArray.
#[no_mangle]
pub extern "C" fn schematic_to_world_zip(
    schematic: *const SchematicWrapper,
    options_json: *const c_char,
) -> ByteArray {
    if schematic.is_null() {
        return ByteArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }

    let s = unsafe { &*(*schematic).0 };

    let options = if options_json.is_null() {
        None
    } else {
        let json_str = unsafe { CStr::from_ptr(options_json) };
        match json_str.to_str() {
            Ok(json) => {
                serde_json::from_str::<crate::formats::world::WorldExportOptions>(json).ok()
            }
            Err(_) => None,
        }
    };

    match crate::formats::world::to_world_zip(s, options) {
        Ok(bytes) => {
            let mut bytes = bytes;
            let ptr = bytes.as_mut_ptr();
            let len = bytes.len();
            std::mem::forget(bytes);
            ByteArray { data: ptr, len }
        }
        Err(_) => ByteArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

// --- Block Manipulation ---

/// Sets a block at a given position with just a block name (no properties).
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_set_block(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let block_name_str = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };

    let block_state = BlockState::new(block_name_str);
    s.set_block(x, y, z, &block_state);
    0
}

/// Sets a block at a given position with properties.
/// The properties array is a list of key-value pairs.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_set_block_with_properties(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    block_name: *const c_char,
    properties: *const CProperty,
    properties_len: usize,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let block_name_str = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };

    let mut props = Vec::new();
    if !properties.is_null() {
        let props_slice = unsafe { std::slice::from_raw_parts(properties, properties_len) };
        for prop in props_slice {
            let key = unsafe { CStr::from_ptr(prop.key).to_string_lossy().into_owned() };
            let value = unsafe { CStr::from_ptr(prop.value).to_string_lossy().into_owned() };
            props.push((key.into(), value.into()));
        }
    }

    let block_state = BlockState {
        name: block_name_str.into(),
        properties: props,
    };
    s.set_block(x, y, z, &block_state);
    0
}

/// Sets a block from a full block string, e.g., "minecraft:chest[facing=north]{Items:[...]}".
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_set_block_from_string(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    block_string: *const c_char,
) -> c_int {
    if schematic.is_null() || block_string.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let block_str = unsafe { CStr::from_ptr(block_string).to_string_lossy() };
    match s.set_block_from_string(x, y, z, &block_str) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Batch-sets blocks at multiple positions to the same block name.
/// `positions` is a flat array of [x0,y0,z0, x1,y1,z1, ...] with `positions_len` elements (must be a multiple of 3).
/// Pre-resolve a plain block name to a palette index for use with
/// `schematic_place`. Pair them in hot loops with many unique block
/// names to skip the per-call name → palette lookup.
///
/// Returns the palette index, or negative on error.
#[no_mangle]
pub extern "C" fn schematic_prepare_block(
    schematic: *mut SchematicWrapper,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() {
        set_last_error("schematic_prepare_block: schematic pointer is null".into());
        return -1;
    }
    if block_name.is_null() {
        set_last_error("schematic_prepare_block: block_name pointer is null".into());
        return -2;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy() };
    s.default_region.get_or_insert_palette_by_name(&name) as c_int
}

/// Place a block by pre-resolved palette index. Pair with
/// `schematic_prepare_block`. Returns 0 on success, negative on error.
/// On error, call `schematic_last_error` for details.
#[no_mangle]
pub extern "C" fn schematic_place(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    palette_index: c_int,
) -> c_int {
    if schematic.is_null() {
        set_last_error("schematic_place: schematic pointer is null".into());
        return -1;
    }
    if palette_index < 0 {
        set_last_error(format!(
            "schematic_place: palette_index must be >= 0, got {}",
            palette_index
        ));
        return -2;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let region = &mut s.default_region;
    if (palette_index as usize) >= region.palette.len() {
        set_last_error(format!(
            "schematic_place: palette_index {} out of range (palette size {})",
            palette_index,
            region.palette.len()
        ));
        return -3;
    }
    if !region.is_in_region(x, y, z) {
        region.expand_to_fit(x, y, z);
    }
    region.set_block_at_index_unchecked(palette_index as usize, x, y, z);
    0
}

/// Returns the number of blocks set, or negative on error.
#[no_mangle]
pub extern "C" fn schematic_set_blocks(
    schematic: *mut SchematicWrapper,
    positions: *const c_int,
    positions_len: usize,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() || positions.is_null() || block_name.is_null() {
        return -1;
    }
    if !positions_len.is_multiple_of(3) {
        return -2;
    }
    let count = positions_len / 3;
    if count == 0 {
        return 0;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let block_name_str = unsafe { CStr::from_ptr(block_name).to_string_lossy() };
    let pos_slice = unsafe { std::slice::from_raw_parts(positions, positions_len) };

    // Complex block strings: parse once, apply many.
    if block_name_str.contains('[') || block_name_str.ends_with('}') {
        let (mut block_state, nbt_data) =
            match crate::UniversalSchematic::parse_block_string(&block_name_str) {
                Ok(p) => p,
                Err(e) => {
                    set_last_error(format!("parse_block_string: {}", e));
                    return -3;
                }
            };
        if block_state.name.contains("jukebox") {
            if let Some(ref nbt) = nbt_data {
                let has_record = nbt.contains_key("RecordItem");
                block_state.set_property("has_record", has_record.to_string());
            }
        }

        let (mut min_x, mut min_y, mut min_z) = (pos_slice[0], pos_slice[1], pos_slice[2]);
        let (mut max_x, mut max_y, mut max_z) = (min_x, min_y, min_z);
        for i in 1..count {
            let (x, y, z) = (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]);
            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if z < min_z {
                min_z = z;
            }
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
            if z > max_z {
                max_z = z;
            }
        }

        let block_name_owned = block_state.name.to_string();
        let proto: Option<crate::block_entity::BlockEntity> = nbt_data.as_ref().map(|nbt| {
            let mut be = crate::block_entity::BlockEntity::new(block_name_owned.clone(), (0, 0, 0));
            for (k, v) in nbt {
                be = be.with_nbt_data(k.clone(), v.clone());
            }
            be
        });

        let region = &mut s.default_region;
        region.ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
        let palette_index = region.get_or_insert_palette_by_state(&block_state);
        for i in 0..count {
            let (x, y, z) = (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]);
            region.set_block_at_index_unchecked(palette_index, x, y, z);
        }

        if let Some(ref template) = proto {
            for i in 0..count {
                let (x, y, z) = (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]);
                let mut be = template.clone();
                be.position = (x, y, z);
                s.set_block_entity(crate::block_position::BlockPosition { x, y, z }, be);
            }
        }
        return count as c_int;
    }

    // Pre-expand to fit all positions
    let (mut min_x, mut min_y, mut min_z) = (pos_slice[0], pos_slice[1], pos_slice[2]);
    let (mut max_x, mut max_y, mut max_z) = (min_x, min_y, min_z);
    for i in 1..count {
        let (x, y, z) = (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]);
        if x < min_x {
            min_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if z < min_z {
            min_z = z;
        }
        if x > max_x {
            max_x = x;
        }
        if y > max_y {
            max_y = y;
        }
        if z > max_z {
            max_z = z;
        }
    }

    let region = &mut s.default_region;
    region.ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
    let palette_index = region.get_or_insert_palette_by_name(&block_name_str);

    for i in 0..count {
        let (x, y, z) = (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]);
        region.set_block_at_index_unchecked(palette_index, x, y, z);
    }

    count as c_int
}

/// Batch-gets block names at multiple positions.
/// `positions` is a flat array of [x0,y0,z0, x1,y1,z1, ...] with `positions_len` elements (must be a multiple of 3).
/// Returns a StringArray where each entry is the block name (or NULL for empty/out-of-bounds).
/// The returned StringArray must be freed with `free_string_array`.
#[no_mangle]
pub extern "C" fn schematic_get_blocks(
    schematic: *const SchematicWrapper,
    positions: *const c_int,
    positions_len: usize,
) -> StringArray {
    if schematic.is_null() || positions.is_null() || !positions_len.is_multiple_of(3) {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let count = positions_len / 3;
    if count == 0 {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let pos_slice = unsafe { std::slice::from_raw_parts(positions, positions_len) };
    let region = &s.default_region;

    let mut results: Vec<*mut c_char> = Vec::with_capacity(count);
    for i in 0..count {
        let (x, y, z) = (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]);
        let name = if region.is_in_region(x, y, z) {
            region.get_block_name(x, y, z)
        } else {
            s.get_block(x, y, z).map(|bs| bs.name.as_str())
        };
        match name {
            Some(n) => results.push(CString::new(n).unwrap().into_raw()),
            None => results.push(ptr::null_mut()),
        }
    }

    let mut results = std::mem::ManuallyDrop::new(results);
    StringArray {
        data: results.as_mut_ptr(),
        len: count,
    }
}

/// Copies a region from a source schematic to a target schematic.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn schematic_copy_region(
    target: *mut SchematicWrapper,
    source: *const SchematicWrapper,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
    target_x: c_int,
    target_y: c_int,
    target_z: c_int,
    excluded_blocks: *const *const c_char,
    excluded_blocks_len: usize,
) -> c_int {
    if target.is_null() || source.is_null() {
        return -1;
    }
    let target_s = unsafe { &mut *(*target).0 };
    let source_s = unsafe { &*(*source).0 };
    let bounds = BoundingBox::new((min_x, min_y, min_z), (max_x, max_y, max_z));

    let mut excluded = Vec::new();
    if !excluded_blocks.is_null() {
        let excluded_slice =
            unsafe { std::slice::from_raw_parts(excluded_blocks, excluded_blocks_len) };
        for &block_ptr in excluded_slice {
            let block_str = unsafe { CStr::from_ptr(block_ptr).to_string_lossy() };
            match UniversalSchematic::parse_block_string(&block_str) {
                Ok((bs, _)) => excluded.push(bs),
                Err(_) => return -3,
            }
        }
    }

    match target_s.copy_region(source_s, &bounds, (target_x, target_y, target_z), &excluded) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

// --- Block & Entity Accessors ---

/// Gets the block name at a given position. Returns NULL if no block is found.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_get_block(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    s.get_block(x, y, z).map_or(ptr::null_mut(), |block_state| {
        CString::new(block_state.name.as_str()).unwrap().into_raw()
    })
}

/// Gets the block at a given position. Returns a BlockStateWrapper.
/// The returned pointer must be freed with `blockstate_free`. Returns NULL if no block is found.
#[no_mangle]
pub extern "C" fn schematic_get_block_with_properties(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut BlockStateWrapper {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    s.get_block(x, y, z).cloned().map_or(ptr::null_mut(), |bs| {
        Box::into_raw(Box::new(BlockStateWrapper(Box::into_raw(Box::new(bs)))))
    })
}

/// Gets the block entity at a given position.
/// The returned CBlockEntity pointer must be freed by calling `free_block_entity_array` on a CBlockEntityArray of length 1.
/// Returns NULL if no block entity is found.
#[no_mangle]
pub extern "C" fn schematic_get_block_entity(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut CBlockEntity {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let pos = BlockPosition { x, y, z };

    s.get_block_entity(pos).map_or(ptr::null_mut(), |be| {
        let nbt_json = serde_json::to_string(&be.nbt).unwrap_or_default();
        let entity = CBlockEntity {
            id: CString::new(be.id.clone()).unwrap().into_raw(),
            x: be.position.0,
            y: be.position.1,
            z: be.position.2,
            nbt_json: CString::new(nbt_json).unwrap().into_raw(),
        };
        Box::into_raw(Box::new(entity))
    })
}

/// Gets a list of all block entities in the schematic.
/// The returned CBlockEntityArray must be freed with `free_block_entity_array`.
#[no_mangle]
pub extern "C" fn schematic_get_all_block_entities(
    schematic: *const SchematicWrapper,
) -> CBlockEntityArray {
    if schematic.is_null() {
        return CBlockEntityArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let block_entities = s.get_block_entities_as_list();

    let mut c_entities = Vec::with_capacity(block_entities.len());
    for be in block_entities {
        let nbt_json = serde_json::to_string(&be.nbt).unwrap_or_default();
        c_entities.push(CBlockEntity {
            id: CString::new(be.id).unwrap().into_raw(),
            x: be.position.0,
            y: be.position.1,
            z: be.position.2,
            nbt_json: CString::new(nbt_json).unwrap().into_raw(),
        });
    }

    let mut c_entities = c_entities;
    let ptr = c_entities.as_mut_ptr();
    let len = c_entities.len();
    std::mem::forget(c_entities);
    CBlockEntityArray { data: ptr, len }
}

/// Gets the number of mobile entities (not block entities).
#[no_mangle]
pub extern "C" fn schematic_entity_count(schematic: *const SchematicWrapper) -> usize {
    if schematic.is_null() {
        return 0;
    }
    let s = unsafe { &*(*schematic).0 };
    s.default_region.entities.len()
}

/// Gets all mobile entities as a CEntityArray.
/// The returned CEntityArray must be freed with `free_entity_array`.
#[no_mangle]
pub extern "C" fn schematic_get_entities(schematic: *const SchematicWrapper) -> CEntityArray {
    if schematic.is_null() {
        return CEntityArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let entities = &s.default_region.entities;

    let mut c_entities = Vec::with_capacity(entities.len());
    for entity in entities {
        let nbt_json = serde_json::to_string(&entity.nbt).unwrap_or_default();
        c_entities.push(CEntity {
            id: CString::new(entity.id.clone()).unwrap().into_raw(),
            x: entity.position.0,
            y: entity.position.1,
            z: entity.position.2,
            nbt_json: CString::new(nbt_json).unwrap().into_raw(),
        });
    }

    let ptr = c_entities.as_mut_ptr();
    let len = c_entities.len();
    std::mem::forget(c_entities);
    CEntityArray { data: ptr, len }
}

/// Add a mobile entity to the schematic. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn schematic_add_entity(
    schematic: *mut SchematicWrapper,
    id: *const c_char,
    x: f64,
    y: f64,
    z: f64,
    nbt_json: *const c_char,
) -> c_int {
    if schematic.is_null() || id.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let id_str = unsafe { CStr::from_ptr(id) }.to_string_lossy().to_string();
    let mut entity = crate::entity::Entity::new(id_str, (x, y, z));

    if !nbt_json.is_null() {
        let json = unsafe { CStr::from_ptr(nbt_json) }
            .to_string_lossy()
            .to_string();
        if !json.is_empty() {
            if let Ok(nbt_map) = serde_json::from_str(&json) {
                entity.nbt = nbt_map;
            }
        }
    }

    s.add_entity(entity);
    0
}

/// Remove a mobile entity by index. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn schematic_remove_entity(schematic: *mut SchematicWrapper, index: usize) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    if s.remove_entity(index).is_some() {
        0
    } else {
        -1
    }
}

// --- Data-version conversion (datafixers) ---

/// The canonical in-memory data version (the forward-conversion target).
#[no_mangle]
pub extern "C" fn schematic_canonical_data_version() -> c_int {
    crate::dataconverter::CANONICAL_DATA_VERSION
}

/// Convert block/item/entity data between Minecraft data versions. Forward
/// (`target >= source`) is lossless; reverse is lossy. Returns a freshly
/// allocated JSON loss report string (`[]` when lossless) that the caller must
/// free with `free_string`; returns NULL on a null pointer.
#[no_mangle]
pub extern "C" fn schematic_convert_to_data_version(
    schematic: *mut SchematicWrapper,
    target_data_version: c_int,
    source_data_version: c_int,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &mut *(*schematic).0 };
    let json = if target_data_version == source_data_version {
        "[]".to_string()
    } else if target_data_version > source_data_version {
        crate::dataconverter::convert_schematic(s, source_data_version, target_data_version);
        "[]".to_string()
    } else {
        crate::dataconverter::convert_schematic_reverse(s, source_data_version, target_data_version)
            .to_json()
    };
    CString::new(json)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Convert to `target_data_version` using the schematic's captured source
/// version (else `mc_version`, else canonical) as origin, updating metadata to
/// the target. Returns a freshly allocated JSON loss report (`[]` when
/// lossless), freed with `free_string`; NULL on a null pointer.
#[no_mangle]
pub extern "C" fn schematic_convert_to_version(
    schematic: *mut SchematicWrapper,
    target_data_version: c_int,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &mut *(*schematic).0 };
    let json = s.convert_to_data_version(target_data_version).to_json();
    CString::new(json)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// The Minecraft data version of the file this schematic was loaded from, or
/// `-1` if none was captured (versionless / freshly built).
#[no_mangle]
pub extern "C" fn schematic_get_source_data_version(schematic: *const SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &*(*schematic).0 };
    s.metadata.source_data_version.unwrap_or(-1)
}

/// Override the source data version for formats that carry no Java data version,
/// so the converter knows what to convert *from*.
#[no_mangle]
pub extern "C" fn schematic_set_source_data_version(
    schematic: *mut SchematicWrapper,
    version: c_int,
) {
    if schematic.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.metadata.source_data_version = Some(version);
}

/// Serialize a `.litematic` targeting a specific Minecraft data version. A COPY
/// is converted to `target_data_version` and the matching schematic Version
/// header written; the schematic is left unchanged. The byte payload is
/// returned (free with `free_byte_array`); the JSON loss report is written to
/// `*out_loss` (free with `free_string`, may be set to NULL on error).
#[no_mangle]
pub extern "C" fn schematic_to_litematic_for_version(
    schematic: *const SchematicWrapper,
    target_data_version: c_int,
    out_loss: *mut *mut c_char,
) -> ByteArray {
    if !out_loss.is_null() {
        unsafe { *out_loss = ptr::null_mut() };
    }
    if schematic.is_null() {
        return ByteArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    match litematic::to_litematic_for_data_version(s, target_data_version) {
        Ok((mut data, report)) => {
            if !out_loss.is_null() {
                if let Ok(c) = CString::new(report.to_json()) {
                    unsafe { *out_loss = c.into_raw() };
                }
            }
            let ptr = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            ByteArray { data: ptr, len }
        }
        Err(_) => ByteArray {
            data: ptr::null_mut(),
            len: 0,
        },
    }
}

// --- Faithful (SNBT) block-entity / entity access ---

/// The block entity's NBT as a typed SNBT string, or NULL if none. Round-trips
/// losslessly. Caller frees with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_get_block_entity_snbt(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    match s.get_block_entity(BlockPosition { x, y, z }) {
        Some(be) => {
            let snbt = quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt();
            CString::new(snbt)
                .map(|c| c.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        None => ptr::null_mut(),
    }
}

/// Set (or replace) a block entity at a position from a typed SNBT string.
/// Returns 0 on success, -1 on error (null pointer or invalid SNBT).
#[no_mangle]
pub extern "C" fn schematic_set_block_entity(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    id: *const c_char,
    snbt: *const c_char,
) -> c_int {
    if schematic.is_null() || id.is_null() || snbt.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let id_str = unsafe { CStr::from_ptr(id) }.to_string_lossy().to_string();
    let snbt_str = unsafe { CStr::from_ptr(snbt) }
        .to_string_lossy()
        .to_string();
    let compound = match quartz_nbt::snbt::parse(&snbt_str) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let nbt = crate::nbt::NbtMap::from_quartz_nbt(&compound);
    let mut be = crate::block_entity::BlockEntity::new(id_str, (x, y, z));
    be.set_nbt(nbt);
    s.set_block_entity(BlockPosition { x, y, z }, be);
    0
}

/// Remove the block entity at a position. Returns 0 if one was removed, -1 otherwise.
#[no_mangle]
pub extern "C" fn schematic_remove_block_entity(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    if s.remove_block_entity((x, y, z)).is_some() {
        0
    } else {
        -1
    }
}

/// Every block entity as a JSON array of `{id, position:[x,y,z], snbt}`. The
/// `snbt` is the inner data only (no `Id`/`Pos`). Caller frees with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_get_all_block_entities_snbt(
    schematic: *const SchematicWrapper,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let items: Vec<serde_json::Value> = s
        .get_block_entities_as_list()
        .into_iter()
        .map(|be| {
            let snbt = quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt();
            serde_json::json!({
                "id": be.id,
                "position": [be.position.0, be.position.1, be.position.2],
                "snbt": snbt,
            })
        })
        .collect();
    let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
    CString::new(json)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Every mobile entity as a JSON array of typed SNBT strings (full compound
/// incl. `id`/`Pos`). Caller frees with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_get_entities_snbt(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let snbts: Vec<String> = s
        .get_entities_as_list()
        .iter()
        .map(|entity| entity.to_nbt().to_snbt())
        .collect();
    let json = serde_json::to_string(&snbts).unwrap_or_else(|_| "[]".to_string());
    CString::new(json)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Add a mobile entity from a full SNBT entity compound (must contain `id` and
/// `Pos`). Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn schematic_add_entity_from_snbt(
    schematic: *mut SchematicWrapper,
    snbt: *const c_char,
) -> c_int {
    if schematic.is_null() || snbt.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let snbt_str = unsafe { CStr::from_ptr(snbt) }
        .to_string_lossy()
        .to_string();
    let compound = match quartz_nbt::snbt::parse(&snbt_str) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    match crate::entity::Entity::from_nbt(&compound) {
        Ok(entity) => {
            s.add_entity(entity);
            0
        }
        Err(_) => -1,
    }
}

/// Frees a CEntityArray returned by `schematic_get_entities`.
#[no_mangle]
pub extern "C" fn free_entity_array(arr: CEntityArray) {
    if arr.data.is_null() || arr.len == 0 {
        return;
    }
    let entities = unsafe { Vec::from_raw_parts(arr.data, arr.len, arr.len) };
    for entity in entities {
        unsafe {
            if !entity.id.is_null() {
                drop(CString::from_raw(entity.id));
            }
            if !entity.nbt_json.is_null() {
                drop(CString::from_raw(entity.nbt_json));
            }
        }
    }
}

/// Gets a list of all non-air blocks in the schematic.
/// The returned CBlockArray must be freed with `free_block_array`.
#[no_mangle]
pub extern "C" fn schematic_get_all_blocks(schematic: *const SchematicWrapper) -> CBlockArray {
    if schematic.is_null() {
        return CBlockArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let blocks: Vec<CBlock> = s
        .iter_blocks()
        .map(|(pos, block)| {
            let props_json = serde_json::to_string(&block.properties).unwrap_or_default();
            CBlock {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                name: CString::new(block.name.as_str()).unwrap().into_raw(),
                properties_json: CString::new(props_json).unwrap().into_raw(),
            }
        })
        .collect();

    let mut blocks = blocks;
    let ptr = blocks.as_mut_ptr();
    let len = blocks.len();
    std::mem::forget(blocks);
    CBlockArray { data: ptr, len }
}

/// Gets all blocks within a specific sub-region (chunk) of the schematic.
/// The returned CBlockArray must be freed with `free_block_array`.
#[no_mangle]
pub extern "C" fn schematic_get_chunk_blocks(
    schematic: *const SchematicWrapper,
    offset_x: c_int,
    offset_y: c_int,
    offset_z: c_int,
    width: c_int,
    height: c_int,
    length: c_int,
) -> CBlockArray {
    if schematic.is_null() {
        return CBlockArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };

    let blocks: Vec<CBlock> = s
        .iter_blocks()
        .filter(|(pos, _)| {
            pos.x >= offset_x
                && pos.x < offset_x + width
                && pos.y >= offset_y
                && pos.y < offset_y + height
                && pos.z >= offset_z
                && pos.z < offset_z + length
        })
        .map(|(pos, block)| {
            let props_json = serde_json::to_string(&block.properties).unwrap_or_default();
            CBlock {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                name: CString::new(block.name.as_str()).unwrap().into_raw(),
                properties_json: CString::new(props_json).unwrap().into_raw(),
            }
        })
        .collect();

    let mut blocks = blocks;
    let ptr = blocks.as_mut_ptr();
    let len = blocks.len();
    std::mem::forget(blocks);
    CBlockArray { data: ptr, len }
}

// --- Chunking ---

/// Splits the schematic into chunks with basic bottom-up strategy.
/// The returned CChunkArray must be freed with `free_chunk_array`.
#[no_mangle]
pub extern "C" fn schematic_get_chunks(
    schematic: *const SchematicWrapper,
    chunk_width: c_int,
    chunk_height: c_int,
    chunk_length: c_int,
) -> CChunkArray {
    schematic_get_chunks_with_strategy(
        schematic,
        chunk_width,
        chunk_height,
        chunk_length,
        ptr::null(), // Use default strategy
        0.0,
        0.0,
        0.0, // Camera position not used for default
    )
}

/// Splits the schematic into chunks with a specified loading strategy.
/// The returned CChunkArray must be freed with `free_chunk_array`.
#[no_mangle]
pub extern "C" fn schematic_get_chunks_with_strategy(
    schematic: *const SchematicWrapper,
    chunk_width: c_int,
    chunk_height: c_int,
    chunk_length: c_int,
    strategy: *const c_char,
    camera_x: c_float,
    camera_y: c_float,
    camera_z: c_float,
) -> CChunkArray {
    if schematic.is_null() {
        return CChunkArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let strategy_str = if strategy.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(strategy).to_str().unwrap_or("") }
    };

    let strategy_enum = match strategy_str {
        "distance_to_camera" => Some(ChunkLoadingStrategy::DistanceToCamera(
            camera_x, camera_y, camera_z,
        )),
        "top_down" => Some(ChunkLoadingStrategy::TopDown),
        "bottom_up" => Some(ChunkLoadingStrategy::BottomUp),
        "center_outward" => Some(ChunkLoadingStrategy::CenterOutward),
        "random" => Some(ChunkLoadingStrategy::Random),
        _ => Some(ChunkLoadingStrategy::BottomUp), // Default strategy
    };

    let chunks: Vec<CChunk> = s
        .iter_chunks(chunk_width, chunk_height, chunk_length, strategy_enum)
        .map(|chunk| {
            let blocks: Vec<CBlock> = chunk
                .positions
                .into_iter()
                .filter_map(|pos| s.get_block(pos.x, pos.y, pos.z).map(|b| (pos, b)))
                .map(|(pos, block)| {
                    let props_json = serde_json::to_string(&block.properties).unwrap_or_default();
                    CBlock {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        name: CString::new(block.name.as_str()).unwrap().into_raw(),
                        properties_json: CString::new(props_json).unwrap().into_raw(),
                    }
                })
                .collect();

            let mut blocks_vec = blocks;
            let blocks_ptr = blocks_vec.as_mut_ptr();
            let blocks_len = blocks_vec.len();
            std::mem::forget(blocks_vec);

            CChunk {
                chunk_x: chunk.chunk_x,
                chunk_y: chunk.chunk_y,
                chunk_z: chunk.chunk_z,
                blocks: CBlockArray {
                    data: blocks_ptr,
                    len: blocks_len,
                },
            }
        })
        .collect();

    let mut chunks = chunks;
    let ptr = chunks.as_mut_ptr();
    let len = chunks.len();
    std::mem::forget(chunks);
    CChunkArray { data: ptr, len }
}

// --- Metadata & Info ---

/// Gets the schematic's dimensions [width, height, length].
/// The returned IntArray must be freed with `free_int_array`.
#[no_mangle]
pub extern "C" fn schematic_get_dimensions(schematic: *const SchematicWrapper) -> IntArray {
    if schematic.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let (x, y, z) = s.get_dimensions();
    let dims = vec![x, y, z];
    let mut boxed_slice = dims.into_boxed_slice();
    let ptr = boxed_slice.as_mut_ptr();
    let len = boxed_slice.len();
    std::mem::forget(boxed_slice);
    IntArray { data: ptr, len }
}

/// Gets the total number of non-air blocks in the schematic.
#[no_mangle]
pub extern "C" fn schematic_get_block_count(schematic: *const SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return 0;
    }
    unsafe { (*(*schematic).0).total_blocks() }
}

/// Gets the total volume of the schematic's bounding box.
#[no_mangle]
pub extern "C" fn schematic_get_volume(schematic: *const SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return 0;
    }
    unsafe { (*(*schematic).0).total_volume() }
}

/// Gets the names of all regions in the schematic.
/// The returned StringArray must be freed with `free_string_array`.
#[no_mangle]
pub extern "C" fn schematic_get_region_names(schematic: *const SchematicWrapper) -> StringArray {
    if schematic.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let names = s.get_region_names();
    let c_names: Vec<*mut c_char> = names
        .into_iter()
        .map(|n| CString::new(n).unwrap().into_raw())
        .collect();

    let mut c_names = c_names;
    let ptr = c_names.as_mut_ptr();
    let len = c_names.len();
    std::mem::forget(c_names);
    StringArray { data: ptr, len }
}

// --- BlockState Wrapper ---

/// Creates a new BlockState.
/// The returned pointer must be freed with `blockstate_free`.
#[no_mangle]
pub extern "C" fn blockstate_new(name: *const c_char) -> *mut BlockStateWrapper {
    if name.is_null() {
        return ptr::null_mut();
    }
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let bs = BlockState::new(name_str);
    Box::into_raw(Box::new(BlockStateWrapper(Box::into_raw(Box::new(bs)))))
}

/// Frees a BlockStateWrapper.
#[no_mangle]
pub extern "C" fn blockstate_free(bs: *mut BlockStateWrapper) {
    if !bs.is_null() {
        unsafe {
            let wrapper = Box::from_raw(bs);
            let _ = Box::from_raw(wrapper.0);
        }
    }
}

/// Sets a property on a BlockState, returning a new BlockStateWrapper.
/// The original `block_state` is NOT modified. The new instance must be freed with `blockstate_free`.
#[no_mangle]
pub extern "C" fn blockstate_with_property(
    block_state: *mut BlockStateWrapper,
    key: *const c_char,
    value: *const c_char,
) -> *mut BlockStateWrapper {
    if block_state.is_null() || key.is_null() || value.is_null() {
        return ptr::null_mut();
    }
    let state = unsafe { &*(*block_state).0 };
    let key_str = unsafe { CStr::from_ptr(key).to_string_lossy().into_owned() };
    let value_str = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };

    let new_state = state.clone().with_property(key_str, value_str);
    Box::into_raw(Box::new(BlockStateWrapper(Box::into_raw(Box::new(
        new_state,
    )))))
}

/// Gets the name of a BlockState.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn blockstate_get_name(block_state: *const BlockStateWrapper) -> *mut c_char {
    if block_state.is_null() {
        return ptr::null_mut();
    }
    let state = unsafe { &*(*block_state).0 };
    CString::new(state.name.as_str()).unwrap().into_raw()
}

/// Gets all properties of a BlockState.
/// The returned CPropertyArray must be freed with `free_property_array`.
#[no_mangle]
pub extern "C" fn blockstate_get_properties(
    block_state: *const BlockStateWrapper,
) -> CPropertyArray {
    if block_state.is_null() {
        return CPropertyArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let state = unsafe { &*(*block_state).0 };

    let props: Vec<CProperty> = state
        .properties
        .iter()
        .map(|(k, v)| CProperty {
            key: CString::new(k.as_str()).unwrap().into_raw(),
            value: CString::new(v.as_str()).unwrap().into_raw(),
        })
        .collect();

    let mut props = props;
    let ptr = props.as_mut_ptr();
    let len = props.len();
    std::mem::forget(props);
    CPropertyArray { data: ptr, len }
}

// --- Debugging & Utility Functions ---

/// Returns a string with basic debug info about the schematic.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_debug_info(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let info = format!(
        "Schematic name: {}, Regions: {}",
        s.metadata.name.as_ref().unwrap_or(&"Unnamed".to_string()),
        s.other_regions.len() + 1
    ); // +1 for the main region
    CString::new(info).unwrap().into_raw()
}

/// Returns a formatted schematic layout string.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_print(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let output = format_schematic(s);
    CString::new(output).unwrap().into_raw()
}

/// Returns a detailed debug string, including a visual layout.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn debug_schematic(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let debug_info = format!(
        "Schematic name: {}, Regions: {}",
        s.metadata.name.as_ref().unwrap_or(&"Unnamed".to_string()),
        s.other_regions.len() + 1
    ); // +1 for the main region
    let info = format!("{}\n{}", debug_info, format_schematic(s));
    CString::new(info).unwrap().into_raw()
}

/// Returns a detailed debug string in JSON format.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn debug_json_schematic(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let debug_info = format!(
        "Schematic name: {}, Regions: {}",
        s.metadata.name.as_ref().unwrap_or(&"Unnamed".to_string()),
        s.other_regions.len() + 1
    ); // +1 for the main region
    let info = format!("{}\n{}", debug_info, format_json_schematic(s));
    CString::new(info).unwrap().into_raw()
}

// --- Metadata Accessors ---

/// Get the schematic name. Returns null if not set.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_get_name(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    match &s.metadata.name {
        Some(name) => CString::new(name.as_str()).unwrap_or_default().into_raw(),
        None => ptr::null_mut(),
    }
}

/// Set the schematic name.
#[no_mangle]
pub extern "C" fn schematic_set_name(schematic: *mut SchematicWrapper, name: *const c_char) {
    if schematic.is_null() || name.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    s.metadata.name = Some(name);
}

/// Get the schematic author. Returns null if not set.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_get_author(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    match &s.metadata.author {
        Some(author) => CString::new(author.as_str()).unwrap_or_default().into_raw(),
        None => ptr::null_mut(),
    }
}

/// Set the schematic author.
#[no_mangle]
pub extern "C" fn schematic_set_author(schematic: *mut SchematicWrapper, author: *const c_char) {
    if schematic.is_null() || author.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let author = unsafe { CStr::from_ptr(author).to_string_lossy().into_owned() };
    s.metadata.author = Some(author);
}

/// Get the schematic description. Returns null if not set.
/// The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn schematic_get_description(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    match &s.metadata.description {
        Some(desc) => CString::new(desc.as_str()).unwrap_or_default().into_raw(),
        None => ptr::null_mut(),
    }
}

/// Set the schematic description.
#[no_mangle]
pub extern "C" fn schematic_set_description(
    schematic: *mut SchematicWrapper,
    description: *const c_char,
) {
    if schematic.is_null() || description.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let desc = unsafe { CStr::from_ptr(description).to_string_lossy().into_owned() };
    s.metadata.description = Some(desc);
}

/// Get the creation timestamp (milliseconds since epoch). Returns -1 if not set.
#[no_mangle]
pub extern "C" fn schematic_get_created(schematic: *const SchematicWrapper) -> i64 {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &*(*schematic).0 };
    s.metadata.created.map(|v| v as i64).unwrap_or(-1)
}

/// Set the creation timestamp (milliseconds since epoch).
#[no_mangle]
pub extern "C" fn schematic_set_created(schematic: *mut SchematicWrapper, created: u64) {
    if schematic.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.metadata.created = Some(created);
}

/// Get the modification timestamp (milliseconds since epoch). Returns -1 if not set.
#[no_mangle]
pub extern "C" fn schematic_get_modified(schematic: *const SchematicWrapper) -> i64 {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &*(*schematic).0 };
    s.metadata.modified.map(|v| v as i64).unwrap_or(-1)
}

/// Set the modification timestamp (milliseconds since epoch).
#[no_mangle]
pub extern "C" fn schematic_set_modified(schematic: *mut SchematicWrapper, modified: u64) {
    if schematic.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.metadata.modified = Some(modified);
}

/// Get the Litematic format version. Returns -1 if not set.
#[no_mangle]
pub extern "C" fn schematic_get_lm_version(schematic: *const SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &*(*schematic).0 };
    s.metadata.lm_version.unwrap_or(-1)
}

/// Set the Litematic format version.
#[no_mangle]
pub extern "C" fn schematic_set_lm_version(schematic: *mut SchematicWrapper, version: c_int) {
    if schematic.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.metadata.lm_version = Some(version);
}

/// Get the Minecraft data version. Returns -1 if not set.
#[no_mangle]
pub extern "C" fn schematic_get_mc_version(schematic: *const SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &*(*schematic).0 };
    s.metadata.mc_version.unwrap_or(-1)
}

/// Set the Minecraft data version.
#[no_mangle]
pub extern "C" fn schematic_set_mc_version(schematic: *mut SchematicWrapper, version: c_int) {
    if schematic.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.metadata.mc_version = Some(version);
}

/// Get the WorldEdit version. Returns -1 if not set.
#[no_mangle]
pub extern "C" fn schematic_get_we_version(schematic: *const SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &*(*schematic).0 };
    s.metadata.we_version.unwrap_or(-1)
}

/// Set the WorldEdit version.
#[no_mangle]
pub extern "C" fn schematic_set_we_version(schematic: *mut SchematicWrapper, version: c_int) {
    if schematic.is_null() {
        return;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.metadata.we_version = Some(version);
}

// --- C-Compatible Bounding Box ---

#[repr(C)]
pub struct CBoundingBox {
    pub(super) min_x: c_int,
    pub(super) min_y: c_int,
    pub(super) min_z: c_int,
    pub(super) max_x: c_int,
    pub(super) max_y: c_int,
    pub(super) max_z: c_int,
}

// --- Schematic Transformation Methods ---

#[no_mangle]
pub extern "C" fn schematic_flip_x(schematic: *mut SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.flip_x();
    0
}

#[no_mangle]
pub extern "C" fn schematic_flip_y(schematic: *mut SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.flip_y();
    0
}

#[no_mangle]
pub extern "C" fn schematic_flip_z(schematic: *mut SchematicWrapper) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.flip_z();
    0
}

#[no_mangle]
pub extern "C" fn schematic_rotate_x(schematic: *mut SchematicWrapper, degrees: c_int) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.rotate_x(degrees);
    0
}

#[no_mangle]
pub extern "C" fn schematic_rotate_y(schematic: *mut SchematicWrapper, degrees: c_int) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.rotate_y(degrees);
    0
}

#[no_mangle]
pub extern "C" fn schematic_rotate_z(schematic: *mut SchematicWrapper, degrees: c_int) -> c_int {
    if schematic.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    s.rotate_z(degrees);
    0
}

#[no_mangle]
pub extern "C" fn schematic_flip_region_x(
    schematic: *mut SchematicWrapper,
    region_name: *const c_char,
) -> c_int {
    if schematic.is_null() || region_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    match s.flip_region_x(&name) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_flip_region_y(
    schematic: *mut SchematicWrapper,
    region_name: *const c_char,
) -> c_int {
    if schematic.is_null() || region_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    match s.flip_region_y(&name) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_flip_region_z(
    schematic: *mut SchematicWrapper,
    region_name: *const c_char,
) -> c_int {
    if schematic.is_null() || region_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    match s.flip_region_z(&name) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_rotate_region_x(
    schematic: *mut SchematicWrapper,
    region_name: *const c_char,
    degrees: c_int,
) -> c_int {
    if schematic.is_null() || region_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    match s.rotate_region_x(&name, degrees) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_rotate_region_y(
    schematic: *mut SchematicWrapper,
    region_name: *const c_char,
    degrees: c_int,
) -> c_int {
    if schematic.is_null() || region_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    match s.rotate_region_y(&name, degrees) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_rotate_region_z(
    schematic: *mut SchematicWrapper,
    region_name: *const c_char,
    degrees: c_int,
) -> c_int {
    if schematic.is_null() || region_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    match s.rotate_region_z(&name, degrees) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

// --- Schematic Building Methods ---

#[no_mangle]
pub extern "C" fn schematic_fill_cuboid(
    schematic: *mut SchematicWrapper,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let block = BlockState::new(name);
    let shape = ShapeEnum::Cuboid(Cuboid::new((min_x, min_y, min_z), (max_x, max_y, max_z)));
    let brush = SolidBrush::new(block);
    let mut tool = BuildingTool::new(s);
    tool.fill(&shape, &brush);
    0
}

#[no_mangle]
pub extern "C" fn schematic_fill_sphere(
    schematic: *mut SchematicWrapper,
    cx: c_float,
    cy: c_float,
    cz: c_float,
    radius: c_float,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let block = BlockState::new(name);
    let shape = ShapeEnum::Sphere(Sphere::new(
        (cx as i32, cy as i32, cz as i32),
        radius as f64,
    ));
    let brush = SolidBrush::new(block);
    let mut tool = BuildingTool::new(s);
    tool.fill(&shape, &brush);
    0
}

// --- Schematic Format Management ---

#[no_mangle]
pub extern "C" fn schematic_save_as(
    schematic: *const SchematicWrapper,
    format: *const c_char,
    version: *const c_char,
    settings: *const c_char,
) -> ByteArray {
    let empty = ByteArray {
        data: ptr::null_mut(),
        len: 0,
    };
    if schematic.is_null() || format.is_null() {
        return empty;
    }
    let s = unsafe { &*(*schematic).0 };
    let fmt = unsafe { CStr::from_ptr(format).to_string_lossy() };
    let ver = if version.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(version).to_string_lossy().into_owned() })
    };
    let settings_str = if settings.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(settings).to_string_lossy().into_owned() })
    };

    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => return empty,
    };
    match manager.write_with_settings(&fmt, s, ver.as_deref(), settings_str.as_deref()) {
        Ok(mut data) => {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            ByteArray { data: ptr, len }
        }
        Err(_) => empty,
    }
}

/// Save a schematic to a file. If `format` is NULL, the format is auto-detected from the file
/// extension. Returns 0 on success, -1 on null arguments, -2 on serialization error, -3 on IO error.
#[no_mangle]
pub extern "C" fn schematic_save_to_file(
    schematic: *const SchematicWrapper,
    path: *const c_char,
    format: *const c_char,
    version: *const c_char,
) -> c_int {
    if schematic.is_null() || path.is_null() {
        set_last_error("schematic_save_to_file: null schematic or path".into());
        return -1;
    }
    let s = unsafe { &*(*schematic).0 };
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };
    let ver = if version.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(version).to_string_lossy().into_owned() })
    };

    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(e) => {
            set_last_error(format!("schematic_save_to_file: lock error: {}", e));
            return -2;
        }
    };

    let bytes = if format.is_null() {
        manager.write_auto_with_settings(&path_str, s, ver.as_deref(), None)
    } else {
        let fmt = unsafe { CStr::from_ptr(format).to_string_lossy() };
        manager.write_with_settings(&fmt, s, ver.as_deref(), None)
    };

    let bytes = match bytes {
        Ok(b) => b,
        Err(e) => {
            set_last_error(format!("schematic_save_to_file: serialize error: {}", e));
            return -2;
        }
    };

    match std::fs::write(path_str.as_ref(), &bytes) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(format!("schematic_save_to_file: IO error: {}", e));
            -3
        }
    }
}

// --- Schematic Block Methods ---

#[no_mangle]
pub extern "C" fn schematic_set_block_with_nbt(
    schematic: *mut SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
    block_name: *const c_char,
    nbt_json: *const c_char,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let nbt: HashMap<String, String> = if nbt_json.is_null() {
        HashMap::new()
    } else {
        let json_str = unsafe { CStr::from_ptr(nbt_json).to_string_lossy() };
        serde_json::from_str(&json_str).unwrap_or_default()
    };
    match s.set_block_with_nbt(x, y, z, &name, nbt) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_set_block_in_region(
    schematic: *mut SchematicWrapper,
    region_name: *const c_char,
    x: c_int,
    y: c_int,
    z: c_int,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() || region_name.is_null() || block_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    let block = unsafe { CStr::from_ptr(block_name).to_string_lossy() };
    if s.set_block_in_region_str(&name, x, y, z, &block) {
        0
    } else {
        -2
    }
}

// --- Schematic Palette/Bounding Box/Info ---

#[no_mangle]
pub extern "C" fn schematic_get_bounding_box(schematic: *const SchematicWrapper) -> IntArray {
    if schematic.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let bbox = s.get_bounding_box();
    let vals = vec![
        bbox.min.0, bbox.min.1, bbox.min.2, bbox.max.0, bbox.max.1, bbox.max.2,
    ];
    let mut boxed = vals.into_boxed_slice();
    let p = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    IntArray { data: p, len }
}

#[no_mangle]
pub extern "C" fn schematic_get_region_bounding_box(
    schematic: *const SchematicWrapper,
    region_name: *const c_char,
) -> IntArray {
    let empty = IntArray {
        data: ptr::null_mut(),
        len: 0,
    };
    if schematic.is_null() || region_name.is_null() {
        return empty;
    }
    let s = unsafe { &*(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    let bbox = if name == "default" || name == "Default" {
        s.default_region.get_bounding_box()
    } else {
        match s.other_regions.get(name.as_ref()) {
            Some(region) => region.get_bounding_box(),
            None => return empty,
        }
    };
    let vals = vec![
        bbox.min.0, bbox.min.1, bbox.min.2, bbox.max.0, bbox.max.1, bbox.max.2,
    ];
    let mut boxed = vals.into_boxed_slice();
    let p = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    IntArray { data: p, len }
}

#[no_mangle]
pub extern "C" fn schematic_get_palette(schematic: *const SchematicWrapper) -> StringArray {
    if schematic.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let merged = s.get_merged_region();
    let names: Vec<String> = merged
        .palette
        .iter()
        .map(|bs| bs.name.to_string())
        .collect();
    vec_string_to_string_array(names)
}

#[no_mangle]
pub extern "C" fn schematic_get_tight_dimensions(schematic: *const SchematicWrapper) -> IntArray {
    if schematic.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let (x, y, z) = s.get_tight_dimensions();
    let dims = vec![x, y, z];
    let mut boxed = dims.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    IntArray { data: ptr, len }
}

#[no_mangle]
pub extern "C" fn schematic_get_allocated_dimensions(
    schematic: *const SchematicWrapper,
) -> IntArray {
    // Same as schematic_get_dimensions but named for parity
    schematic_get_dimensions(schematic)
}

#[no_mangle]
pub extern "C" fn schematic_extract_signs(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let signs = crate::insign::extract_signs(s);
    // SignInput doesn't derive Serialize, manually build JSON
    let json_array: Vec<String> = signs
        .iter()
        .map(|sign| {
            format!(
                "{{\"pos\":[{},{},{}],\"text\":{}}}",
                sign.pos[0],
                sign.pos[1],
                sign.pos[2],
                serde_json::to_string(&sign.text).unwrap_or_default()
            )
        })
        .collect();
    let json = format!("[{}]", json_array.join(","));
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn schematic_compile_insign(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    match crate::insign::compile_schematic_insign(s) {
        Ok(data) => {
            let json = serde_json::to_string(&data).unwrap_or_default();
            CString::new(json).unwrap().into_raw()
        }
        Err(_) => ptr::null_mut(),
    }
}

// --- Schematic Print ---

#[no_mangle]
pub extern "C" fn schematic_print_schematic(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    CString::new(format_schematic(s)).unwrap().into_raw()
}

// --- Schematic Get/Set for format-aware methods ---

#[no_mangle]
pub extern "C" fn schematic_get_block_string(
    schematic: *const SchematicWrapper,
    x: c_int,
    y: c_int,
    z: c_int,
) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    s.get_block(x, y, z).map_or(ptr::null_mut(), |bs| {
        CString::new(bs.to_string()).unwrap().into_raw()
    })
}

// --- Schematic Format Conversion Methods ---

#[no_mangle]
pub extern "C" fn schematic_to_schematic_version(
    schematic: *const SchematicWrapper,
    version: *const c_char,
) -> ByteArray {
    let empty = ByteArray {
        data: ptr::null_mut(),
        len: 0,
    };
    if schematic.is_null() || version.is_null() {
        return empty;
    }
    let s = unsafe { &*(*schematic).0 };
    let ver = unsafe { CStr::from_ptr(version).to_string_lossy() };
    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => return empty,
    };
    match manager.write("sponge", s, Some(&ver)) {
        Ok(mut data) => {
            let p = data.as_mut_ptr();
            let len = data.len();
            std::mem::forget(data);
            ByteArray { data: p, len }
        }
        Err(_) => empty,
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_available_schematic_versions() -> StringArray {
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
    let versions = manager.get_exporter_versions("sponge").unwrap_or_default();
    vec_string_to_string_array(versions)
}

// --- Schematic Palette Methods ---

#[no_mangle]
pub extern "C" fn schematic_get_all_palettes(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let mut palettes: HashMap<String, Vec<String>> = HashMap::new();
    let default_blocks: Vec<String> = s
        .default_region
        .palette
        .iter()
        .map(|bs| bs.name.to_string())
        .collect();
    palettes.insert("default".to_string(), default_blocks);
    for (name, region) in &s.other_regions {
        let blocks: Vec<String> = region
            .palette
            .iter()
            .map(|bs| bs.name.to_string())
            .collect();
        palettes.insert(name.clone(), blocks);
    }
    let json = serde_json::to_string(&palettes).unwrap_or_default();
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn schematic_get_default_region_palette(
    schematic: *const SchematicWrapper,
) -> StringArray {
    if schematic.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    let names: Vec<String> = s
        .default_region
        .palette
        .iter()
        .map(|bs| bs.name.to_string())
        .collect();
    vec_string_to_string_array(names)
}

#[no_mangle]
pub extern "C" fn schematic_get_palette_from_region(
    schematic: *const SchematicWrapper,
    region_name: *const c_char,
) -> StringArray {
    let empty = StringArray {
        data: ptr::null_mut(),
        len: 0,
    };
    if schematic.is_null() || region_name.is_null() {
        return empty;
    }
    let s = unsafe { &*(*schematic).0 };
    let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
    let region = if name == "default" || name == "Default" {
        &s.default_region
    } else {
        match s.other_regions.get(name.as_ref()) {
            Some(r) => r,
            None => return empty,
        }
    };
    let names: Vec<String> = region
        .palette
        .iter()
        .map(|bs| bs.name.to_string())
        .collect();
    vec_string_to_string_array(names)
}

// --- Schematic Tight Bounds ---

#[no_mangle]
pub extern "C" fn schematic_get_tight_bounds_min(schematic: *const SchematicWrapper) -> IntArray {
    if schematic.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    match s.get_tight_bounds() {
        Some(bbox) => {
            let vals = vec![bbox.min.0, bbox.min.1, bbox.min.2];
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
pub extern "C" fn schematic_get_tight_bounds_max(schematic: *const SchematicWrapper) -> IntArray {
    if schematic.is_null() {
        return IntArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let s = unsafe { &*(*schematic).0 };
    match s.get_tight_bounds() {
        Some(bbox) => {
            let vals = vec![bbox.max.0, bbox.max.1, bbox.max.2];
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
