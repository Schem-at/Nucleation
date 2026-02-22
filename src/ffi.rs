// src/ffi.rs
#![cfg(feature = "ffi")]
use crate::{
    block_position::BlockPosition,
    bounding_box::BoundingBox,
    building::{
        BilinearGradientBrush, BrushEnum, BuildingTool, ColorBrush, Cuboid, InterpolationSpace,
        LinearGradientBrush, PointGradientBrush, ShadedBrush, ShapeEnum, SolidBrush, Sphere,
    },
    definition_region::DefinitionRegion,
    formats::{litematic, manager::get_manager, mcstructure, schematic},
    print_utils::{format_json_schematic, format_schematic},
    universal_schematic::ChunkLoadingStrategy,
    BlockState, SchematicBuilder, UniversalSchematic,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_uchar};
use std::ptr;

// --- Last Error ---

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_last_error(msg: String) {
    eprintln!("[nucleation] {}", &msg);
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

/// Returns the last error message as a C string.
/// Caller must free the returned pointer with `schematic_free_string`.
/// Returns null if no error.
#[no_mangle]
pub extern "C" fn schematic_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| {
        let err = e.borrow();
        match err.as_ref() {
            Some(msg) => CString::new(msg.as_str()).unwrap_or_default().into_raw(),
            None => ptr::null_mut(),
        }
    })
}

// --- C-Compatible Data Structures ---

#[repr(C)]
pub struct ByteArray {
    data: *mut c_uchar,
    len: usize,
}

#[repr(C)]
pub struct StringArray {
    data: *mut *mut c_char,
    len: usize,
}

#[repr(C)]
pub struct IntArray {
    data: *mut c_int,
    len: usize,
}

#[repr(C)]
pub struct CProperty {
    key: *mut c_char,
    value: *mut c_char,
}

#[repr(C)]
pub struct CPropertyArray {
    data: *mut CProperty,
    len: usize,
}

#[repr(C)]
pub struct CBlock {
    x: c_int,
    y: c_int,
    z: c_int,
    name: *mut c_char,
    properties_json: *mut c_char,
}

#[repr(C)]
#[derive(Clone)]
pub struct CBlockArray {
    data: *mut CBlock,
    len: usize,
}

#[repr(C)]
pub struct CBlockEntity {
    id: *mut c_char,
    x: c_int,
    y: c_int,
    z: c_int,
    nbt_json: *mut c_char,
}

#[repr(C)]
pub struct CBlockEntityArray {
    data: *mut CBlockEntity,
    len: usize,
}

#[repr(C)]
pub struct CEntity {
    id: *mut c_char,
    x: f64,
    y: f64,
    z: f64,
    nbt_json: *mut c_char,
}

#[repr(C)]
pub struct CEntityArray {
    data: *mut CEntity,
    len: usize,
}

#[repr(C)]
pub struct CChunk {
    chunk_x: c_int,
    chunk_y: c_int,
    chunk_z: c_int,
    blocks: CBlockArray,
}

#[repr(C)]
pub struct CChunkArray {
    data: *mut CChunk,
    len: usize,
}

// --- Wrapper Structs with Opaque Pointers ---

pub struct SchematicWrapper(*mut UniversalSchematic);
pub struct BlockStateWrapper(*mut BlockState);

#[cfg(feature = "simulation")]
pub struct MchprsWorldWrapper(*mut crate::simulation::MchprsWorld);

// --- Memory Management ---

/// Frees a ByteArray returned by the library.
#[no_mangle]
pub extern "C" fn free_byte_array(array: ByteArray) {
    if !array.data.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(array.data, array.len, array.len);
        }
    }
}

/// Frees a StringArray returned by the library.
#[no_mangle]
pub extern "C" fn free_string_array(array: StringArray) {
    if !array.data.is_null() {
        unsafe {
            let strings = Vec::from_raw_parts(array.data, array.len, array.len);
            for s in strings {
                let _ = CString::from_raw(s);
            }
        }
    }
}

/// Frees an IntArray returned by the library.
#[no_mangle]
pub extern "C" fn free_int_array(array: IntArray) {
    if !array.data.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(array.data, array.len, array.len);
        }
    }
}

/// Frees a C string returned by the library.
#[no_mangle]
pub extern "C" fn free_string(string: *mut c_char) {
    if !string.is_null() {
        unsafe {
            let _ = CString::from_raw(string);
        }
    }
}

/// Frees a CPropertyArray returned by `blockstate_get_properties`.
#[no_mangle]
pub extern "C" fn free_property_array(array: CPropertyArray) {
    if !array.data.is_null() {
        unsafe {
            let props = Vec::from_raw_parts(array.data, array.len, array.len);
            for prop in props {
                free_string(prop.key);
                free_string(prop.value);
            }
        }
    }
}

/// Frees a single CBlock. Used as a helper by `free_block_array`.
fn free_single_block(block: &mut CBlock) {
    free_string(block.name);
    free_string(block.properties_json);
}

/// Frees a CBlockArray returned by functions like `schematic_get_all_blocks`.
#[no_mangle]
pub extern "C" fn free_block_array(array: CBlockArray) {
    if !array.data.is_null() {
        unsafe {
            let mut blocks = Vec::from_raw_parts(array.data, array.len, array.len);
            for block in &mut blocks {
                free_single_block(block);
            }
        }
    }
}

/// Frees a single CBlockEntity. Used as a helper by `free_block_entity_array`.
fn free_single_block_entity(entity: &mut CBlockEntity) {
    free_string(entity.id);
    free_string(entity.nbt_json);
}

/// Frees a CBlockEntityArray returned by `schematic_get_all_block_entities`.
#[no_mangle]
pub extern "C" fn free_block_entity_array(array: CBlockEntityArray) {
    if !array.data.is_null() {
        unsafe {
            let mut entities = Vec::from_raw_parts(array.data, array.len, array.len);
            for entity in &mut entities {
                free_single_block_entity(entity);
            }
        }
    }
}

/// Frees a single CChunk. Used as a helper by `free_chunk_array`.
fn free_single_chunk(chunk: &mut CChunk) {
    free_block_array(chunk.blocks.clone());
}

/// Frees a CChunkArray returned by `schematic_get_chunks`.
#[no_mangle]
pub extern "C" fn free_chunk_array(array: CChunkArray) {
    if !array.data.is_null() {
        unsafe {
            let mut chunks = Vec::from_raw_parts(array.data, array.len, array.len);
            for chunk in &mut chunks {
                free_single_chunk(chunk);
            }
        }
    }
}

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
    if positions_len % 3 != 0 {
        return -2;
    }
    let count = positions_len / 3;
    if count == 0 {
        return 0;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let block_name_str = unsafe { CStr::from_ptr(block_name).to_string_lossy() };
    let pos_slice = unsafe { std::slice::from_raw_parts(positions, positions_len) };

    // Complex block strings fall back to per-call path
    if block_name_str.contains('[') || block_name_str.ends_with('}') {
        let mut set = 0i32;
        for i in 0..count {
            let (x, y, z) = (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]);
            if s.set_block_str(x, y, z, &block_name_str) {
                set += 1;
            }
        }
        return set;
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
    if schematic.is_null() || positions.is_null() || positions_len % 3 != 0 {
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
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
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

fn vec_string_to_string_array(strings: Vec<String>) -> StringArray {
    let c_strings: Vec<*mut c_char> = strings
        .into_iter()
        .map(|s| CString::new(s).unwrap().into_raw())
        .collect();
    let mut c_strings = c_strings;
    let ptr = c_strings.as_mut_ptr();
    let len = c_strings.len();
    std::mem::forget(c_strings);
    StringArray { data: ptr, len }
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

// --- Schematic Print ---

#[no_mangle]
pub extern "C" fn schematic_print_schematic(schematic: *const SchematicWrapper) -> *mut c_char {
    if schematic.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    CString::new(format_schematic(s)).unwrap().into_raw()
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

// --- Additional DefinitionRegion Methods ---

#[no_mangle]
pub extern "C" fn definitionregion_from_positions(
    positions: *const c_int,
    positions_len: usize,
) -> *mut DefinitionRegionWrapper {
    if positions.is_null() || positions_len % 3 != 0 {
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
    if boxes.is_null() || boxes_len % 6 != 0 {
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
    data: *mut c_float,
    len: usize,
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

// --- Shape/Brush/BuildingTool FFI ---

pub struct ShapeWrapper(ShapeEnum);
pub struct BrushWrapper(BrushEnum);

#[no_mangle]
pub extern "C" fn shape_sphere(
    cx: c_float,
    cy: c_float,
    cz: c_float,
    radius: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Sphere(Sphere::new(
        (cx as i32, cy as i32, cz as i32),
        radius as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_cuboid(
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Cuboid(Cuboid::new(
        (min_x, min_y, min_z),
        (max_x, max_y, max_z),
    )))))
}

#[no_mangle]
pub extern "C" fn shape_free(ptr: *mut ShapeWrapper) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn brush_solid(block_name: *const c_char) -> *mut BrushWrapper {
    if block_name.is_null() {
        return ptr::null_mut();
    }
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let block = BlockState::new(name);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Solid(SolidBrush::new(
        block,
    )))))
}

#[no_mangle]
pub extern "C" fn brush_color(r: c_uchar, g: c_uchar, b: c_uchar) -> *mut BrushWrapper {
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Color(ColorBrush::new(
        r, g, b,
    )))))
}

#[no_mangle]
pub extern "C" fn brush_linear_gradient(
    x1: c_int,
    y1: c_int,
    z1: c_int,
    r1: c_uchar,
    g1: c_uchar,
    b1: c_uchar,
    x2: c_int,
    y2: c_int,
    z2: c_int,
    r2: c_uchar,
    g2: c_uchar,
    b2: c_uchar,
    space: c_int,
) -> *mut BrushWrapper {
    let interp = if space == 1 {
        InterpolationSpace::Oklab
    } else {
        InterpolationSpace::Rgb
    };
    let brush = LinearGradientBrush::new((x1, y1, z1), (r1, g1, b1), (x2, y2, z2), (r2, g2, b2))
        .with_space(interp);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Linear(brush))))
}

#[no_mangle]
pub extern "C" fn brush_shaded(
    r: c_uchar,
    g: c_uchar,
    b: c_uchar,
    lx: c_float,
    ly: c_float,
    lz: c_float,
) -> *mut BrushWrapper {
    let brush = ShadedBrush::new((r, g, b), (lx as f64, ly as f64, lz as f64));
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Shaded(brush))))
}

#[no_mangle]
pub extern "C" fn brush_bilinear_gradient(
    ox: c_int,
    oy: c_int,
    oz: c_int,
    ux: c_int,
    uy: c_int,
    uz: c_int,
    vx: c_int,
    vy: c_int,
    vz: c_int,
    r00: c_uchar,
    g00: c_uchar,
    b00: c_uchar,
    r10: c_uchar,
    g10: c_uchar,
    b10: c_uchar,
    r01: c_uchar,
    g01: c_uchar,
    b01: c_uchar,
    r11: c_uchar,
    g11: c_uchar,
    b11: c_uchar,
    space: c_int,
) -> *mut BrushWrapper {
    let interp = if space == 1 {
        InterpolationSpace::Oklab
    } else {
        InterpolationSpace::Rgb
    };
    let brush = BilinearGradientBrush::new(
        (ox, oy, oz),
        (ux, uy, uz),
        (vx, vy, vz),
        (r00, g00, b00),
        (r10, g10, b10),
        (r01, g01, b01),
        (r11, g11, b11),
    )
    .with_space(interp);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Bilinear(brush))))
}

#[no_mangle]
pub extern "C" fn brush_point_gradient(
    positions: *const c_int,
    colors: *const c_uchar,
    count: usize,
    falloff: c_float,
    space: c_int,
) -> *mut BrushWrapper {
    if positions.is_null() || colors.is_null() || count == 0 {
        return ptr::null_mut();
    }
    let pos_slice = unsafe { std::slice::from_raw_parts(positions, count * 3) };
    let col_slice = unsafe { std::slice::from_raw_parts(colors, count * 3) };
    let points: Vec<((i32, i32, i32), (u8, u8, u8))> = (0..count)
        .map(|i| {
            (
                (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]),
                (col_slice[i * 3], col_slice[i * 3 + 1], col_slice[i * 3 + 2]),
            )
        })
        .collect();
    let interp = if space == 1 {
        InterpolationSpace::Oklab
    } else {
        InterpolationSpace::Rgb
    };
    let brush = PointGradientBrush::new(points)
        .with_space(interp)
        .with_falloff(falloff as f64);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Point(brush))))
}

#[no_mangle]
pub extern "C" fn brush_free(ptr: *mut BrushWrapper) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn buildingtool_fill(
    schematic: *mut SchematicWrapper,
    shape: *const ShapeWrapper,
    brush: *const BrushWrapper,
) -> c_int {
    if schematic.is_null() || shape.is_null() || brush.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let sh = unsafe { &(*shape).0 };
    let br = unsafe { &(*brush).0 };
    let mut tool = BuildingTool::new(s);
    tool.fill(sh, br);
    0
}

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
    let old = std::mem::replace(builder, SchematicBuilder::new());
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
    let old = std::mem::replace(builder, SchematicBuilder::new());
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
    let old = std::mem::replace(builder, SchematicBuilder::new());
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

// =============================================================================
// Simulation FFI Bindings (feature-gated)
// =============================================================================

#[cfg(feature = "simulation")]
pub mod simulation_ffi {
    use super::*;
    use crate::simulation::circuit_builder::CircuitBuilder;
    use crate::simulation::typed_executor::{
        ExecutionMode, ExecutionResult, IoLayout, IoLayoutBuilder, IoType, LayoutFunction,
        LayoutInfo, OutputCondition, SortStrategy, StateMode, TypedCircuitExecutor, Value,
    };
    use crate::simulation::CustomIoChange;
    use crate::simulation::{MchprsWorld, SimulationOptions};
    use mchprs_blocks::BlockPos;

    // --- Wrapper Structs ---

    pub struct ValueWrapper(Value);
    pub struct IoTypeWrapper(IoType);
    pub struct LayoutFunctionWrapper(LayoutFunction);
    pub struct OutputConditionWrapper(OutputCondition);
    pub struct ExecutionModeWrapper(ExecutionMode);
    pub struct SortStrategyWrapper(SortStrategy);
    pub struct IoLayoutBuilderWrapper(Option<IoLayoutBuilder>);
    pub struct IoLayoutWrapper(IoLayout);
    pub struct CircuitBuilderWrapper(Option<CircuitBuilder>);
    pub struct TypedCircuitExecutorWrapper(*mut TypedCircuitExecutor);

    /// Creates a new MchprsWorld from a schematic with default options.
    /// Returns null on error. Caller must free with `mchprs_world_free`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_new(
        schematic: *const SchematicWrapper,
    ) -> *mut MchprsWorldWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        match MchprsWorld::new(s.clone()) {
            Ok(world) => {
                let w = Box::into_raw(Box::new(world));
                Box::into_raw(Box::new(MchprsWorldWrapper(w)))
            }
            Err(_) => ptr::null_mut(),
        }
    }

    /// Creates a new MchprsWorld from a schematic with options.
    /// `optimize`: 0=false, non-zero=true
    /// `io_only`: 0=false, non-zero=true
    /// Returns null on error. Caller must free with `mchprs_world_free`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_new_with_options(
        schematic: *const SchematicWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> *mut MchprsWorldWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        match MchprsWorld::with_options(s.clone(), options) {
            Ok(world) => {
                let w = Box::into_raw(Box::new(world));
                Box::into_raw(Box::new(MchprsWorldWrapper(w)))
            }
            Err(_) => ptr::null_mut(),
        }
    }

    /// Creates a new MchprsWorld from a Schematic with custom IO positions.
    ///
    /// `custom_io_positions`: pointer to array of [x, y, z, x, y, z, ...] i32 values
    /// `custom_io_count`: number of positions (i.e., array length / 3)
    /// Returns null on error. Caller must free with `mchprs_world_free`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_new_with_custom_io(
        schematic: *const SchematicWrapper,
        optimize: c_int,
        io_only: c_int,
        custom_io_positions: *const c_int,
        custom_io_count: c_int,
    ) -> *mut MchprsWorldWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let mut custom_io = Vec::new();
        if !custom_io_positions.is_null() && custom_io_count > 0 {
            let coords = unsafe {
                std::slice::from_raw_parts(custom_io_positions, custom_io_count as usize * 3)
            };
            for chunk in coords.chunks_exact(3) {
                custom_io.push(BlockPos::new(chunk[0], chunk[1], chunk[2]));
            }
        }
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io,
        };
        match MchprsWorld::with_options(s.clone(), options) {
            Ok(world) => {
                let w = Box::into_raw(Box::new(world));
                Box::into_raw(Box::new(MchprsWorldWrapper(w)))
            }
            Err(_) => ptr::null_mut(),
        }
    }

    /// Frees a MchprsWorld.
    #[no_mangle]
    pub extern "C" fn mchprs_world_free(world: *mut MchprsWorldWrapper) {
        if !world.is_null() {
            unsafe {
                let wrapper = Box::from_raw(world);
                let _ = Box::from_raw(wrapper.0);
            }
        }
    }

    /// Advances the simulation by the specified number of ticks.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_tick(world: *mut MchprsWorldWrapper, ticks: u32) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.tick(ticks);
        0
    }

    /// Flushes pending changes from the compiler to the world.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_flush(world: *mut MchprsWorldWrapper) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.flush();
        0
    }

    /// Sets the power state of a lever.
    /// `powered`: 0=off, non-zero=on. Uses c_int instead of bool for ABI safety.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_set_lever_power(
        world: *mut MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
        powered: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.set_lever_power(BlockPos::new(x, y, z), powered != 0);
        0
    }

    /// Gets the power state of a lever.
    /// Returns 1 if powered, 0 if not, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_lever_power(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &*(*world).0 };
        if w.get_lever_power(BlockPos::new(x, y, z)) {
            1
        } else {
            0
        }
    }

    /// Checks if a redstone lamp is lit at the given position.
    /// Returns 1 if lit, 0 if not, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_is_lit(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &*(*world).0 };
        if w.is_lit(BlockPos::new(x, y, z)) {
            1
        } else {
            0
        }
    }

    /// Sets the signal strength at a position (for custom IO).
    /// `strength` is 0-15.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_set_signal_strength(
        world: *mut MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
        strength: u8,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.set_signal_strength(BlockPos::new(x, y, z), strength);
        0
    }

    /// Gets the signal strength at a position.
    /// Returns 0-15 signal strength, or 0 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_signal_strength(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> u8 {
        if world.is_null() {
            return 0;
        }
        let w = unsafe { &*(*world).0 };
        w.get_signal_strength(BlockPos::new(x, y, z))
    }

    /// Simulates a right-click on a block (typically a lever).
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_on_use_block(
        world: *mut MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.on_use_block(BlockPos::new(x, y, z));
        0
    }

    /// Syncs the simulation state back to the schematic.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_sync_to_schematic(world: *mut MchprsWorldWrapper) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.sync_to_schematic();
        0
    }

    /// Gets a clone of the schematic from the world.
    /// Caller must free the returned SchematicWrapper with `schematic_free`.
    /// Returns null on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_schematic(
        world: *const MchprsWorldWrapper,
    ) -> *mut SchematicWrapper {
        if world.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let cloned = w.get_schematic().clone();
        let boxed = Box::into_raw(Box::new(cloned));
        Box::into_raw(Box::new(SchematicWrapper(boxed)))
    }

    // =========================================================================
    // Value FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn value_from_u32(v: u32) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::U32(v))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_i32(v: i32) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::I32(v))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_f32(v: f32) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::F32(v))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_bool(v: c_int) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::Bool(v != 0))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_string(s: *const c_char) -> *mut ValueWrapper {
        if s.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned();
        Box::into_raw(Box::new(ValueWrapper(Value::String(s))))
    }

    #[no_mangle]
    pub extern "C" fn value_as_u32(v: *const ValueWrapper) -> u32 {
        if v.is_null() {
            return 0;
        }
        unsafe { &*v }.0.as_u32().unwrap_or(0)
    }

    #[no_mangle]
    pub extern "C" fn value_as_i32(v: *const ValueWrapper) -> i32 {
        if v.is_null() {
            return 0;
        }
        unsafe { &*v }.0.as_i32().unwrap_or(0)
    }

    #[no_mangle]
    pub extern "C" fn value_as_f32(v: *const ValueWrapper) -> f32 {
        if v.is_null() {
            return 0.0;
        }
        unsafe { &*v }.0.as_f32().unwrap_or(0.0)
    }

    #[no_mangle]
    pub extern "C" fn value_as_bool(v: *const ValueWrapper) -> c_int {
        if v.is_null() {
            return 0;
        }
        if unsafe { &*v }.0.as_bool().unwrap_or(false) {
            1
        } else {
            0
        }
    }

    /// Returns the string value. Caller must free with `schematic_free_string`.
    /// Returns null if not a string value.
    #[no_mangle]
    pub extern "C" fn value_as_string(v: *const ValueWrapper) -> *mut c_char {
        if v.is_null() {
            return ptr::null_mut();
        }
        match unsafe { &*v }.0.as_str() {
            Ok(s) => CString::new(s).unwrap_or_default().into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }

    /// Returns the type name (e.g. "u32", "bool", "string").
    /// Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn value_type_name(v: *const ValueWrapper) -> *mut c_char {
        if v.is_null() {
            return ptr::null_mut();
        }
        let name = match &unsafe { &*v }.0 {
            Value::U32(_) => "u32",
            Value::U64(_) => "u64",
            Value::I32(_) => "i32",
            Value::I64(_) => "i64",
            Value::F32(_) => "f32",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::BitArray(_) => "bit_array",
            Value::Bytes(_) => "bytes",
            Value::Array(_) => "array",
            Value::Struct(_) => "struct",
        };
        CString::new(name).unwrap_or_default().into_raw()
    }

    #[no_mangle]
    pub extern "C" fn value_free(ptr: *mut ValueWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // IoType FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn io_type_unsigned_int(bits: usize) -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::UnsignedInt { bits })))
    }

    #[no_mangle]
    pub extern "C" fn io_type_signed_int(bits: usize) -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::SignedInt { bits })))
    }

    #[no_mangle]
    pub extern "C" fn io_type_float32() -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::Float32)))
    }

    #[no_mangle]
    pub extern "C" fn io_type_boolean() -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::Boolean)))
    }

    #[no_mangle]
    pub extern "C" fn io_type_ascii(chars: usize) -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::Ascii { chars })))
    }

    #[no_mangle]
    pub extern "C" fn io_type_free(ptr: *mut IoTypeWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // LayoutFunction FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn layout_function_one_to_one() -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::OneToOne)))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_packed4() -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::Packed4)))
    }

    /// `mapping` is a pointer to an array of usize values of length `len`.
    #[no_mangle]
    pub extern "C" fn layout_function_custom(
        mapping: *const usize,
        len: usize,
    ) -> *mut LayoutFunctionWrapper {
        if mapping.is_null() || len == 0 {
            return ptr::null_mut();
        }
        let mapping = unsafe { std::slice::from_raw_parts(mapping, len) }.to_vec();
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::Custom(
            mapping,
        ))))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_row_major(
        rows: usize,
        cols: usize,
        bits_per_element: usize,
    ) -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::RowMajor {
            rows,
            cols,
            bits_per_element,
        })))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_column_major(
        rows: usize,
        cols: usize,
        bits_per_element: usize,
    ) -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(
            LayoutFunction::ColumnMajor {
                rows,
                cols,
                bits_per_element,
            },
        )))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_scanline(
        width: usize,
        height: usize,
        bits_per_pixel: usize,
    ) -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::Scanline {
            width,
            height,
            bits_per_pixel,
        })))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_free(ptr: *mut LayoutFunctionWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // OutputCondition FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn output_condition_equals(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(OutputCondition::Equals(v))))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_not_equals(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(
            OutputCondition::NotEquals(v),
        )))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_greater_than(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(
            OutputCondition::GreaterThan(v),
        )))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_less_than(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(OutputCondition::LessThan(
            v,
        ))))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_bitwise_and(mask: u32) -> *mut OutputConditionWrapper {
        Box::into_raw(Box::new(OutputConditionWrapper(
            OutputCondition::BitwiseAnd(mask as u64),
        )))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_free(ptr: *mut OutputConditionWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // ExecutionMode FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn execution_mode_fixed_ticks(ticks: u32) -> *mut ExecutionModeWrapper {
        Box::into_raw(Box::new(ExecutionModeWrapper(ExecutionMode::FixedTicks {
            ticks,
        })))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_until_condition(
        output_name: *const c_char,
        condition: *const OutputConditionWrapper,
        max_ticks: u32,
        check_interval: u32,
    ) -> *mut ExecutionModeWrapper {
        if output_name.is_null() || condition.is_null() {
            return ptr::null_mut();
        }
        let name = unsafe { CStr::from_ptr(output_name) }
            .to_string_lossy()
            .into_owned();
        let cond = unsafe { &*condition }.0.clone();
        Box::into_raw(Box::new(ExecutionModeWrapper(
            ExecutionMode::UntilCondition {
                output_name: name,
                condition: cond,
                max_ticks,
                check_interval,
            },
        )))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_until_change(
        max_ticks: u32,
        check_interval: u32,
    ) -> *mut ExecutionModeWrapper {
        Box::into_raw(Box::new(ExecutionModeWrapper(ExecutionMode::UntilChange {
            max_ticks,
            check_interval,
        })))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_until_stable(
        stable_ticks: u32,
        max_ticks: u32,
    ) -> *mut ExecutionModeWrapper {
        Box::into_raw(Box::new(ExecutionModeWrapper(ExecutionMode::UntilStable {
            stable_ticks,
            max_ticks,
        })))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_free(ptr: *mut ExecutionModeWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // SortStrategy FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn sort_strategy_yxz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::YXZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_xyz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::XYZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_zyx() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::ZYX)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_y_desc_xz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::YDescXZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_x_desc_yz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::XDescYZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_z_desc_yx() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::ZDescYX)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_descending() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::YXZDesc)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_distance_from(
        x: i32,
        y: i32,
        z: i32,
    ) -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::DistanceFrom {
            reference: (x, y, z),
        })))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_distance_from_desc(
        x: i32,
        y: i32,
        z: i32,
    ) -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(
            SortStrategy::DistanceFromDesc {
                reference: (x, y, z),
            },
        )))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_preserve() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::Preserve)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_reverse() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::Reverse)))
    }

    /// Parse a sort strategy from a string (e.g. "yxz", "descending", "preserve").
    /// Returns null if the string is not recognized.
    #[no_mangle]
    pub extern "C" fn sort_strategy_from_string(s: *const c_char) -> *mut SortStrategyWrapper {
        if s.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { CStr::from_ptr(s) }.to_string_lossy();
        match SortStrategy::from_str(&s) {
            Some(strategy) => Box::into_raw(Box::new(SortStrategyWrapper(strategy))),
            None => ptr::null_mut(),
        }
    }

    /// Returns the strategy name. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn sort_strategy_name(ptr: *const SortStrategyWrapper) -> *mut c_char {
        if ptr.is_null() {
            return ptr::null_mut();
        }
        let name = unsafe { &*ptr }.0.name();
        CString::new(name).unwrap_or_default().into_raw()
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_free(ptr: *mut SortStrategyWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // IoLayoutBuilder FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn io_layout_builder_new() -> *mut IoLayoutBuilderWrapper {
        Box::into_raw(Box::new(IoLayoutBuilderWrapper(Some(
            IoLayoutBuilder::new(),
        ))))
    }

    /// Adds an input to the builder.
    /// `positions` is a flat array of [x,y,z,x,y,z,...] with `count` positions (array length = count*3).
    /// Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || layout.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_input(name_str, io_t, lay, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output to the builder.
    /// `positions` is a flat array of [x,y,z,...] with `count` positions.
    /// Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || layout.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_output(name_str, io_t, lay, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_input_auto(name_str, io_t, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_output_auto(name_str, io_t, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input from a DefinitionRegion.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input_from_region(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_input_from_region(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input from a DefinitionRegion with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input_from_region_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_input_from_region_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output from a DefinitionRegion.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output_from_region(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_output_from_region(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output from a DefinitionRegion with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output_from_region_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_output_from_region_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Builds the IoLayout from the builder. Consumes the builder.
    /// Returns null on error. Caller must free with `io_layout_free`.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_build(
        builder: *mut IoLayoutBuilderWrapper,
    ) -> *mut IoLayoutWrapper {
        if builder.is_null() {
            return ptr::null_mut();
        }
        let b = unsafe { &mut *builder };
        match b.0.take() {
            Some(inner) => {
                let layout = inner.build();
                Box::into_raw(Box::new(IoLayoutWrapper(layout)))
            }
            None => {
                set_last_error("Builder already consumed".into());
                ptr::null_mut()
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn io_layout_builder_free(ptr: *mut IoLayoutBuilderWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // IoLayout FFI Bindings
    // =========================================================================

    /// Returns input names as a JSON array string. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn io_layout_input_names(layout: *const IoLayoutWrapper) -> *mut c_char {
        if layout.is_null() {
            return ptr::null_mut();
        }
        let names: Vec<&str> = unsafe { &*layout }.0.input_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns output names as a JSON array string. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn io_layout_output_names(layout: *const IoLayoutWrapper) -> *mut c_char {
        if layout.is_null() {
            return ptr::null_mut();
        }
        let names: Vec<&str> = unsafe { &*layout }.0.output_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    #[no_mangle]
    pub extern "C" fn io_layout_free(ptr: *mut IoLayoutWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // CircuitBuilder FFI Bindings
    // =========================================================================

    /// Creates a new CircuitBuilder from a schematic.
    #[no_mangle]
    pub extern "C" fn circuit_builder_new(
        schematic: *const SchematicWrapper,
    ) -> *mut CircuitBuilderWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        Box::into_raw(Box::new(CircuitBuilderWrapper(Some(CircuitBuilder::new(
            s.clone(),
        )))))
    }

    /// Creates a CircuitBuilder pre-populated from Insign annotations.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_from_insign(
        schematic: *const SchematicWrapper,
    ) -> *mut CircuitBuilderWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        match CircuitBuilder::from_insign(s.clone()) {
            Ok(builder) => Box::into_raw(Box::new(CircuitBuilderWrapper(Some(builder)))),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Adds an input with full control. Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_input(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with full control and custom sort strategy.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_input_sorted(name_str, io_t, lay, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input_auto(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_input_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with automatic layout inference and custom sort.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input_auto_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_input_auto_sorted(name_str, io_t, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with full control.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_output(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with full control and custom sort strategy.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_output_sorted(name_str, io_t, lay, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output_auto(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_output_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with automatic layout inference and custom sort.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output_auto_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_output_auto_sorted(name_str, io_t, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Sets simulation options on the builder.
    /// `optimize`: 0=false, non-zero=true. `io_only`: 0=false, non-zero=true.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_options(
        builder: *mut CircuitBuilderWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> c_int {
        if builder.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        b.0 = Some(inner.with_options(options));
        0
    }

    /// Sets the state mode. `mode` is one of "stateless", "stateful", "manual".
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_state_mode(
        builder: *mut CircuitBuilderWrapper,
        mode: *const c_char,
    ) -> c_int {
        if builder.is_null() || mode.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let mode_str = unsafe { CStr::from_ptr(mode) }.to_string_lossy();
        let state_mode = match parse_state_mode(&mode_str) {
            Some(m) => m,
            None => {
                set_last_error(format!("Unknown state mode: {}", mode_str));
                return -1;
            }
        };
        b.0 = Some(inner.with_state_mode(state_mode));
        0
    }

    /// Validates the circuit builder configuration.
    /// Returns 0 if valid, -1 on validation error (check `schematic_last_error`).
    #[no_mangle]
    pub extern "C" fn circuit_builder_validate(builder: *const CircuitBuilderWrapper) -> c_int {
        if builder.is_null() {
            return -1;
        }
        let b = unsafe { &*builder };
        match &b.0 {
            Some(inner) => match inner.validate() {
                Ok(_) => 0,
                Err(e) => {
                    set_last_error(e.to_string());
                    -1
                }
            },
            None => {
                set_last_error("Builder already consumed".into());
                -1
            }
        }
    }

    /// Builds the TypedCircuitExecutor. Consumes the builder.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_build(
        builder: *mut CircuitBuilderWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if builder.is_null() {
            return ptr::null_mut();
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return ptr::null_mut();
            }
        };
        match inner.build() {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Builds the TypedCircuitExecutor with validation. Consumes the builder.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_build_validated(
        builder: *mut CircuitBuilderWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if builder.is_null() {
            return ptr::null_mut();
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return ptr::null_mut();
            }
        };
        match inner.build_validated() {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn circuit_builder_input_count(builder: *const CircuitBuilderWrapper) -> usize {
        if builder.is_null() {
            return 0;
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => inner.input_count(),
            None => 0,
        }
    }

    #[no_mangle]
    pub extern "C" fn circuit_builder_output_count(builder: *const CircuitBuilderWrapper) -> usize {
        if builder.is_null() {
            return 0;
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => inner.output_count(),
            None => 0,
        }
    }

    /// Returns input names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn circuit_builder_input_names(
        builder: *const CircuitBuilderWrapper,
    ) -> *mut c_char {
        if builder.is_null() {
            return ptr::null_mut();
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => {
                let names: Vec<&str> = inner.input_names();
                let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
                CString::new(json).unwrap_or_default().into_raw()
            }
            None => ptr::null_mut(),
        }
    }

    /// Returns output names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn circuit_builder_output_names(
        builder: *const CircuitBuilderWrapper,
    ) -> *mut c_char {
        if builder.is_null() {
            return ptr::null_mut();
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => {
                let names: Vec<&str> = inner.output_names();
                let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
                CString::new(json).unwrap_or_default().into_raw()
            }
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn circuit_builder_free(ptr: *mut CircuitBuilderWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // TypedCircuitExecutor FFI Bindings
    // =========================================================================

    /// Creates a TypedCircuitExecutor from a MchprsWorld and IoLayout.
    /// Takes ownership of world's data (clones internally).
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_layout(
        world: *const MchprsWorldWrapper,
        layout: *const IoLayoutWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if world.is_null() || layout.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let l = unsafe { &*layout };
        // We need to create a new MchprsWorld from the schematic
        let schematic = w.get_schematic().clone();
        match MchprsWorld::new(schematic) {
            Ok(new_world) => {
                let executor = TypedCircuitExecutor::from_layout(new_world, l.0.clone());
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Creates a TypedCircuitExecutor from a MchprsWorld and IoLayout with simulation options.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_layout_with_options(
        world: *const MchprsWorldWrapper,
        layout: *const IoLayoutWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> *mut TypedCircuitExecutorWrapper {
        if world.is_null() || layout.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let l = unsafe { &*layout };
        let schematic = w.get_schematic().clone();
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        match MchprsWorld::with_options(schematic, options.clone()) {
            Ok(new_world) => {
                let executor =
                    TypedCircuitExecutor::from_layout_with_options(new_world, l.0.clone(), options);
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Creates a TypedCircuitExecutor from Insign annotations in a schematic.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_insign(
        schematic: *const SchematicWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        match crate::simulation::circuit_builder::create_circuit_from_insign(s) {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Creates a TypedCircuitExecutor from Insign annotations with options.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_insign_with_options(
        schematic: *const SchematicWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> *mut TypedCircuitExecutorWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        match crate::simulation::circuit_builder::create_circuit_from_insign_with_options(
            s, options,
        ) {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Sets the state mode. `mode` is "stateless", "stateful", or "manual".
    #[no_mangle]
    pub extern "C" fn typed_executor_set_state_mode(
        executor: *mut TypedCircuitExecutorWrapper,
        mode: *const c_char,
    ) -> c_int {
        if executor.is_null() || mode.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        let mode_str = unsafe { CStr::from_ptr(mode) }.to_string_lossy();
        match parse_state_mode(&mode_str) {
            Some(m) => {
                e.set_state_mode(m);
                0
            }
            None => {
                set_last_error(format!("Unknown state mode: {}", mode_str));
                -1
            }
        }
    }

    /// Resets the executor to its initial state.
    #[no_mangle]
    pub extern "C" fn typed_executor_reset(executor: *mut TypedCircuitExecutorWrapper) -> c_int {
        if executor.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        match e.reset() {
            Ok(()) => 0,
            Err(err) => {
                set_last_error(err);
                -1
            }
        }
    }

    /// Advances the simulation by the specified number of ticks.
    #[no_mangle]
    pub extern "C" fn typed_executor_tick(
        executor: *mut TypedCircuitExecutorWrapper,
        ticks: u32,
    ) -> c_int {
        if executor.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        e.tick(ticks);
        0
    }

    /// Flushes pending changes.
    #[no_mangle]
    pub extern "C" fn typed_executor_flush(executor: *mut TypedCircuitExecutorWrapper) -> c_int {
        if executor.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        e.flush();
        0
    }

    /// Sets a single input value. Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn typed_executor_set_input(
        executor: *mut TypedCircuitExecutorWrapper,
        name: *const c_char,
        value: *const ValueWrapper,
    ) -> c_int {
        if executor.is_null() || name.is_null() || value.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy();
        let v = &unsafe { &*value }.0;
        match e.set_input(&name_str, v) {
            Ok(()) => 0,
            Err(err) => {
                set_last_error(err);
                -1
            }
        }
    }

    /// Reads a single output value. Returns null on error.
    /// Caller must free with `value_free`.
    #[no_mangle]
    pub extern "C" fn typed_executor_read_output(
        executor: *mut TypedCircuitExecutorWrapper,
        name: *const c_char,
    ) -> *mut ValueWrapper {
        if executor.is_null() || name.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &mut *(*executor).0 };
        let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy();
        match e.read_output(&name_str) {
            Ok(value) => Box::into_raw(Box::new(ValueWrapper(value))),
            Err(err) => {
                set_last_error(err);
                ptr::null_mut()
            }
        }
    }

    /// Executes the circuit with given inputs and execution mode.
    /// `inputs_json` is a JSON object like `{"input_name": {"type": "u32", "value": 42}}`.
    /// Returns a JSON string with the execution result, or null on error.
    /// Caller must free the returned string with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_execute(
        executor: *mut TypedCircuitExecutorWrapper,
        inputs_json: *const c_char,
        mode: *const ExecutionModeWrapper,
    ) -> *mut c_char {
        if executor.is_null() || inputs_json.is_null() || mode.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &mut *(*executor).0 };
        let json_str = unsafe { CStr::from_ptr(inputs_json) }.to_string_lossy();
        let exec_mode = unsafe { &*mode }.0.clone();

        // Parse inputs JSON
        let inputs = match parse_inputs_json(&json_str) {
            Ok(inputs) => inputs,
            Err(err) => {
                set_last_error(err);
                return ptr::null_mut();
            }
        };

        // Execute
        match e.execute(inputs, exec_mode) {
            Ok(result) => {
                // Serialize result to JSON
                let json = serialize_execution_result(&result);
                CString::new(json).unwrap_or_default().into_raw()
            }
            Err(err) => {
                set_last_error(err);
                ptr::null_mut()
            }
        }
    }

    /// Returns input names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_input_names(
        executor: *const TypedCircuitExecutorWrapper,
    ) -> *mut c_char {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &*(*executor).0 };
        let names: Vec<&str> = e.input_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns output names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_output_names(
        executor: *const TypedCircuitExecutorWrapper,
    ) -> *mut c_char {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &*(*executor).0 };
        let names: Vec<&str> = e.output_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns layout info as JSON. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_get_layout_info(
        executor: *const TypedCircuitExecutorWrapper,
    ) -> *mut c_char {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &*(*executor).0 };
        let info = e.get_layout_info();
        let json = serialize_layout_info(&info);
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Syncs the simulation state to a new schematic and returns it.
    /// Caller must free with `schematic_free`.
    #[no_mangle]
    pub extern "C" fn typed_executor_sync_to_schematic(
        executor: *mut TypedCircuitExecutorWrapper,
    ) -> *mut SchematicWrapper {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &mut *(*executor).0 };
        let schematic = e.sync_and_get_schematic().clone();
        let boxed = Box::into_raw(Box::new(schematic));
        Box::into_raw(Box::new(SchematicWrapper(boxed)))
    }

    #[no_mangle]
    pub extern "C" fn typed_executor_free(ptr: *mut TypedCircuitExecutorWrapper) {
        if !ptr.is_null() {
            unsafe {
                let wrapper = Box::from_raw(ptr);
                let _ = Box::from_raw(wrapper.0);
            }
        }
    }

    // =========================================================================
    // Additional MchprsWorld Methods
    // =========================================================================

    /// Gets the redstone power level at a position.
    /// Returns 0-15, or 0 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_redstone_power(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> u8 {
        if world.is_null() {
            return 0;
        }
        let w = unsafe { &*(*world).0 };
        w.get_redstone_power(BlockPos::new(x, y, z))
    }

    /// Checks for custom IO changes since last check.
    /// Must be called before `poll_custom_io_changes` to detect changes.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_check_custom_io_changes(
        world: *mut MchprsWorldWrapper,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.check_custom_io_changes();
        0
    }

    /// Returns queued custom IO changes as a JSON array and clears the queue.
    /// JSON format: `[{"x":0,"y":0,"z":0,"old_power":0,"new_power":15}, ...]`
    /// Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_poll_custom_io_changes(
        world: *mut MchprsWorldWrapper,
    ) -> *mut c_char {
        if world.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &mut *(*world).0 };
        let changes = w.poll_custom_io_changes();
        let json = serialize_custom_io_changes(&changes);
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns queued custom IO changes as JSON without clearing the queue.
    /// Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_peek_custom_io_changes(
        world: *const MchprsWorldWrapper,
    ) -> *mut c_char {
        if world.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let changes = w.peek_custom_io_changes();
        let json = serialize_custom_io_changes(changes);
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Clears all queued custom IO changes.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_clear_custom_io_changes(
        world: *mut MchprsWorldWrapper,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.clear_custom_io_changes();
        0
    }

    // =========================================================================
    // Helper Functions
    // =========================================================================

    /// Parse a flat [x,y,z,x,y,z,...] array into Vec<(i32,i32,i32)>.
    fn parse_positions(positions: *const c_int, count: usize) -> Vec<(i32, i32, i32)> {
        if positions.is_null() || count == 0 {
            return Vec::new();
        }
        let coords = unsafe { std::slice::from_raw_parts(positions, count * 3) };
        coords.chunks_exact(3).map(|c| (c[0], c[1], c[2])).collect()
    }

    /// Parse a state mode string.
    fn parse_state_mode(s: &str) -> Option<StateMode> {
        match s.to_lowercase().as_str() {
            "stateless" => Some(StateMode::Stateless),
            "stateful" => Some(StateMode::Stateful),
            "manual" => Some(StateMode::Manual),
            _ => None,
        }
    }

    /// Parse inputs JSON to HashMap<String, Value>.
    /// Format: `{"name": {"type": "u32", "value": 42}, ...}`
    fn parse_inputs_json(json: &str) -> Result<HashMap<String, Value>, String> {
        let parsed: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

        let obj = parsed
            .as_object()
            .ok_or_else(|| "Inputs must be a JSON object".to_string())?;

        let mut inputs = HashMap::new();
        for (name, val) in obj {
            let value = parse_json_value(val)?;
            inputs.insert(name.clone(), value);
        }
        Ok(inputs)
    }

    /// Parse a single JSON value to a Value.
    /// Supports: `{"type": "u32", "value": 42}` or shorthand: just `42` (inferred as u32/i32/f32/bool/string).
    fn parse_json_value(v: &serde_json::Value) -> Result<Value, String> {
        // Try typed format first: {"type": "...", "value": ...}
        if let Some(obj) = v.as_object() {
            if let (Some(type_val), Some(value_val)) = (obj.get("type"), obj.get("value")) {
                if let Some(type_str) = type_val.as_str() {
                    return match type_str {
                        "u32" => {
                            let n = value_val
                                .as_u64()
                                .ok_or("Expected unsigned integer for u32")?;
                            Ok(Value::U32(n as u32))
                        }
                        "i32" => {
                            let n = value_val.as_i64().ok_or("Expected integer for i32")?;
                            Ok(Value::I32(n as i32))
                        }
                        "f32" => {
                            let n = value_val.as_f64().ok_or("Expected number for f32")?;
                            Ok(Value::F32(n as f32))
                        }
                        "bool" => {
                            let b = value_val.as_bool().ok_or("Expected boolean for bool")?;
                            Ok(Value::Bool(b))
                        }
                        "string" => {
                            let s = value_val.as_str().ok_or("Expected string for string")?;
                            Ok(Value::String(s.to_string()))
                        }
                        _ => Err(format!("Unknown value type: {}", type_str)),
                    };
                }
            }
        }

        // Shorthand: infer type from JSON value
        match v {
            serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= 0 {
                        Ok(Value::U32(i as u32))
                    } else {
                        Ok(Value::I32(i as i32))
                    }
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::F32(f as f32))
                } else {
                    Err("Cannot parse number".to_string())
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            _ => Err(format!("Cannot convert JSON value to Value: {}", v)),
        }
    }

    /// Serialize a Value to a serde_json::Value.
    fn value_to_json(v: &Value) -> serde_json::Value {
        match v {
            Value::U32(n) => serde_json::json!({"type": "u32", "value": n}),
            Value::U64(n) => serde_json::json!({"type": "u64", "value": n}),
            Value::I32(n) => serde_json::json!({"type": "i32", "value": n}),
            Value::I64(n) => serde_json::json!({"type": "i64", "value": n}),
            Value::F32(n) => serde_json::json!({"type": "f32", "value": n}),
            Value::Bool(b) => serde_json::json!({"type": "bool", "value": b}),
            Value::String(s) => serde_json::json!({"type": "string", "value": s}),
            Value::BitArray(bits) => serde_json::json!({"type": "bit_array", "value": bits}),
            Value::Bytes(bytes) => serde_json::json!({"type": "bytes", "value": bytes}),
            Value::Array(arr) => {
                let vals: Vec<serde_json::Value> = arr.iter().map(value_to_json).collect();
                serde_json::json!({"type": "array", "value": vals})
            }
            Value::Struct(fields) => {
                let obj: serde_json::Map<String, serde_json::Value> = fields
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_json(v)))
                    .collect();
                serde_json::json!({"type": "struct", "value": obj})
            }
        }
    }

    /// Serialize an ExecutionResult to JSON string.
    fn serialize_execution_result(result: &ExecutionResult) -> String {
        let mut outputs = serde_json::Map::new();
        for (name, value) in &result.outputs {
            outputs.insert(name.clone(), value_to_json(value));
        }
        let json = serde_json::json!({
            "outputs": outputs,
            "ticks_elapsed": result.ticks_elapsed,
            "condition_met": result.condition_met,
        });
        serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
    }

    /// Serialize LayoutInfo to JSON string.
    fn serialize_layout_info(info: &LayoutInfo) -> String {
        let mut inputs = serde_json::Map::new();
        for (name, li) in &info.inputs {
            let positions: Vec<Vec<i32>> = li
                .positions
                .iter()
                .map(|&(x, y, z)| vec![x, y, z])
                .collect();
            inputs.insert(
                name.clone(),
                serde_json::json!({
                    "io_type": li.io_type,
                    "positions": positions,
                    "bit_count": li.bit_count,
                }),
            );
        }
        let mut outputs = serde_json::Map::new();
        for (name, li) in &info.outputs {
            let positions: Vec<Vec<i32>> = li
                .positions
                .iter()
                .map(|&(x, y, z)| vec![x, y, z])
                .collect();
            outputs.insert(
                name.clone(),
                serde_json::json!({
                    "io_type": li.io_type,
                    "positions": positions,
                    "bit_count": li.bit_count,
                }),
            );
        }
        let json = serde_json::json!({
            "inputs": inputs,
            "outputs": outputs,
        });
        serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
    }

    /// Serialize custom IO changes to JSON array string.
    fn serialize_custom_io_changes(changes: &[CustomIoChange]) -> String {
        let arr: Vec<serde_json::Value> = changes
            .iter()
            .map(|c| {
                serde_json::json!({
                    "x": c.x,
                    "y": c.y,
                    "z": c.z,
                    "old_power": c.old_power,
                    "new_power": c.new_power,
                })
            })
            .collect();
        serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
    }
}

// =============================================================================
// Meshing FFI Bindings (feature-gated)
// =============================================================================

#[cfg(feature = "meshing")]
#[allow(unused_imports)]
pub mod meshing_ffi {
    use super::*;
    use crate::meshing::{
        ChunkMeshResult, MeshConfig, MeshPhase, MeshProgress, MeshResult, MultiMeshResult,
        RawMeshExport, ResourcePackSource,
    };
    use schematic_mesher::TextureAtlas;

    // --- Wrapper Structs ---

    pub struct FFIResourcePack(ResourcePackSource);
    pub struct FFIMeshConfig(MeshConfig);
    pub struct FFIMeshResult(MeshResult);
    pub struct FFIMultiMeshResult(MultiMeshResult);
    pub struct FFIChunkMeshResult(ChunkMeshResult);
    pub struct FFIRawMeshExport(RawMeshExport);
    pub struct FFITextureAtlas(TextureAtlas);

    // --- ResourcePack Lifecycle ---

    #[no_mangle]
    pub extern "C" fn resourcepack_from_bytes(
        data: *const c_uchar,
        data_len: usize,
    ) -> *mut FFIResourcePack {
        if data.is_null() || data_len == 0 {
            return ptr::null_mut();
        }
        let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
        match ResourcePackSource::from_bytes(data_slice) {
            Ok(pack) => Box::into_raw(Box::new(FFIResourcePack(pack))),
            Err(_) => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_free(ptr: *mut FFIResourcePack) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_blockstate_count(ptr: *const FFIResourcePack) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let pack = unsafe { &(*ptr).0 };
        pack.stats().blockstate_count
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_model_count(ptr: *const FFIResourcePack) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let pack = unsafe { &(*ptr).0 };
        pack.stats().model_count
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_texture_count(ptr: *const FFIResourcePack) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let pack = unsafe { &(*ptr).0 };
        pack.stats().texture_count
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_namespaces(ptr: *const FFIResourcePack) -> StringArray {
        if ptr.is_null() {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let pack = unsafe { &(*ptr).0 };
        vec_string_to_string_array(pack.stats().namespaces)
    }

    // --- ResourcePack Query/List/Mutate ---

    #[no_mangle]
    pub extern "C" fn resourcepack_list_blockstates(ptr: *const FFIResourcePack) -> StringArray {
        if ptr.is_null() {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let pack = unsafe { &(*ptr).0 };
        vec_string_to_string_array(pack.list_blockstates())
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_list_models(ptr: *const FFIResourcePack) -> StringArray {
        if ptr.is_null() {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let pack = unsafe { &(*ptr).0 };
        vec_string_to_string_array(pack.list_models())
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_list_textures(ptr: *const FFIResourcePack) -> StringArray {
        if ptr.is_null() {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let pack = unsafe { &(*ptr).0 };
        vec_string_to_string_array(pack.list_textures())
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_get_blockstate_json(
        ptr: *const FFIResourcePack,
        name: *const c_char,
    ) -> *mut c_char {
        if ptr.is_null() || name.is_null() {
            return ptr::null_mut();
        }
        let pack = unsafe { &(*ptr).0 };
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
        match pack.get_blockstate_json(&name_str) {
            Some(json) => CString::new(json).unwrap().into_raw(),
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_get_model_json(
        ptr: *const FFIResourcePack,
        name: *const c_char,
    ) -> *mut c_char {
        if ptr.is_null() || name.is_null() {
            return ptr::null_mut();
        }
        let pack = unsafe { &(*ptr).0 };
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
        match pack.get_model_json(&name_str) {
            Some(json) => CString::new(json).unwrap().into_raw(),
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_get_texture_info(
        ptr: *const FFIResourcePack,
        name: *const c_char,
    ) -> IntArray {
        if ptr.is_null() || name.is_null() {
            return IntArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let pack = unsafe { &(*ptr).0 };
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
        match pack.get_texture_info(&name_str) {
            Some((w, h, animated, frames)) => {
                let vals = vec![
                    w as c_int,
                    h as c_int,
                    if animated { 1 } else { 0 },
                    frames as c_int,
                ];
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
    pub extern "C" fn resourcepack_get_texture_pixels(
        ptr: *const FFIResourcePack,
        name: *const c_char,
    ) -> ByteArray {
        if ptr.is_null() || name.is_null() {
            return ByteArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let pack = unsafe { &(*ptr).0 };
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
        match pack.get_texture_pixels(&name_str) {
            Some(pixels) => {
                let mut data = pixels.to_vec();
                let p = data.as_mut_ptr();
                let len = data.len();
                std::mem::forget(data);
                ByteArray { data: p, len }
            }
            None => ByteArray {
                data: ptr::null_mut(),
                len: 0,
            },
        }
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_add_blockstate_json(
        ptr: *mut FFIResourcePack,
        name: *const c_char,
        json: *const c_char,
    ) -> c_int {
        if ptr.is_null() || name.is_null() || json.is_null() {
            return -1;
        }
        let pack = unsafe { &mut (*ptr).0 };
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
        let json_str = unsafe { CStr::from_ptr(json).to_string_lossy() };
        match pack.add_blockstate_json(&name_str, &json_str) {
            Ok(_) => 0,
            Err(_) => -2,
        }
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_add_model_json(
        ptr: *mut FFIResourcePack,
        name: *const c_char,
        json: *const c_char,
    ) -> c_int {
        if ptr.is_null() || name.is_null() || json.is_null() {
            return -1;
        }
        let pack = unsafe { &mut (*ptr).0 };
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
        let json_str = unsafe { CStr::from_ptr(json).to_string_lossy() };
        match pack.add_model_json(&name_str, &json_str) {
            Ok(_) => 0,
            Err(_) => -2,
        }
    }

    #[no_mangle]
    pub extern "C" fn resourcepack_add_texture(
        ptr: *mut FFIResourcePack,
        name: *const c_char,
        width: u32,
        height: u32,
        pixels: *const c_uchar,
        pixels_len: usize,
    ) -> c_int {
        if ptr.is_null() || name.is_null() || pixels.is_null() {
            return -1;
        }
        let pack = unsafe { &mut (*ptr).0 };
        let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
        let pixels_slice = unsafe { std::slice::from_raw_parts(pixels, pixels_len) };
        match pack.add_texture(&name_str, width, height, pixels_slice.to_vec()) {
            Ok(_) => 0,
            Err(_) => -2,
        }
    }

    // --- MeshConfig FFI ---

    #[no_mangle]
    pub extern "C" fn meshconfig_new() -> *mut FFIMeshConfig {
        Box::into_raw(Box::new(FFIMeshConfig(MeshConfig::default())))
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_free(ptr: *mut FFIMeshConfig) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_set_cull_hidden_faces(ptr: *mut FFIMeshConfig, val: c_int) {
        if !ptr.is_null() {
            let config = unsafe { &mut (*ptr).0 };
            config.cull_hidden_faces = val != 0;
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_set_ambient_occlusion(ptr: *mut FFIMeshConfig, val: c_int) {
        if !ptr.is_null() {
            let config = unsafe { &mut (*ptr).0 };
            config.ambient_occlusion = val != 0;
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_set_ao_intensity(ptr: *mut FFIMeshConfig, val: c_float) {
        if !ptr.is_null() {
            let config = unsafe { &mut (*ptr).0 };
            config.ao_intensity = val;
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_set_biome(ptr: *mut FFIMeshConfig, biome: *const c_char) {
        if !ptr.is_null() {
            let config = unsafe { &mut (*ptr).0 };
            if biome.is_null() {
                config.biome = None;
            } else {
                let biome_str = unsafe { CStr::from_ptr(biome).to_string_lossy().into_owned() };
                config.biome = Some(biome_str);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_set_atlas_max_size(ptr: *mut FFIMeshConfig, size: u32) {
        if !ptr.is_null() {
            let config = unsafe { &mut (*ptr).0 };
            config.atlas_max_size = size;
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_get_cull_hidden_faces(ptr: *const FFIMeshConfig) -> c_int {
        if ptr.is_null() {
            return 0;
        }
        let config = unsafe { &(*ptr).0 };
        if config.cull_hidden_faces {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_get_ambient_occlusion(ptr: *const FFIMeshConfig) -> c_int {
        if ptr.is_null() {
            return 0;
        }
        let config = unsafe { &(*ptr).0 };
        if config.ambient_occlusion {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_get_ao_intensity(ptr: *const FFIMeshConfig) -> c_float {
        if ptr.is_null() {
            return 0.0;
        }
        let config = unsafe { &(*ptr).0 };
        config.ao_intensity
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_get_biome(ptr: *const FFIMeshConfig) -> *mut c_char {
        if ptr.is_null() {
            return ptr::null_mut();
        }
        let config = unsafe { &(*ptr).0 };
        match &config.biome {
            Some(biome) => CString::new(biome.clone()).unwrap().into_raw(),
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_get_atlas_max_size(ptr: *const FFIMeshConfig) -> u32 {
        if ptr.is_null() {
            return 0;
        }
        let config = unsafe { &(*ptr).0 };
        config.atlas_max_size
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_set_cull_occluded_blocks(ptr: *mut FFIMeshConfig, val: c_int) {
        if ptr.is_null() {
            return;
        }
        let config = unsafe { &mut (*ptr).0 };
        config.cull_occluded_blocks = val != 0;
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_get_cull_occluded_blocks(ptr: *const FFIMeshConfig) -> c_int {
        if ptr.is_null() {
            return 1; // default: true
        }
        let config = unsafe { &(*ptr).0 };
        config.cull_occluded_blocks as c_int
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_set_greedy_meshing(ptr: *mut FFIMeshConfig, val: c_int) {
        if ptr.is_null() {
            return;
        }
        let config = unsafe { &mut (*ptr).0 };
        config.greedy_meshing = val != 0;
    }

    #[no_mangle]
    pub extern "C" fn meshconfig_get_greedy_meshing(ptr: *const FFIMeshConfig) -> c_int {
        if ptr.is_null() {
            return 0; // default: false
        }
        let config = unsafe { &(*ptr).0 };
        config.greedy_meshing as c_int
    }

    // --- MeshResult FFI ---

    #[no_mangle]
    pub extern "C" fn meshresult_free(ptr: *mut FFIMeshResult) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn meshresult_glb_data(ptr: *const FFIMeshResult) -> ByteArray {
        if ptr.is_null() {
            return ByteArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let result = unsafe { &(*ptr).0 };
        let mut data = match result.to_glb() {
            Ok(d) => d,
            Err(_) => {
                return ByteArray {
                    data: ptr::null_mut(),
                    len: 0,
                }
            }
        };
        let p = data.as_mut_ptr();
        let len = data.len();
        std::mem::forget(data);
        ByteArray { data: p, len }
    }

    #[no_mangle]
    pub extern "C" fn meshresult_nucm_data(ptr: *const FFIMeshResult) -> ByteArray {
        if ptr.is_null() {
            return ByteArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let result = unsafe { &(*ptr).0 };
        let mut data = crate::meshing::cache::serialize_meshes(&[result.clone()]);
        let p = data.as_mut_ptr();
        let len = data.len();
        std::mem::forget(data);
        ByteArray { data: p, len }
    }

    #[no_mangle]
    pub extern "C" fn meshresult_vertex_count(ptr: *const FFIMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.total_vertices()
    }

    #[no_mangle]
    pub extern "C" fn meshresult_triangle_count(ptr: *const FFIMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.total_triangles()
    }

    #[no_mangle]
    pub extern "C" fn meshresult_has_transparency(ptr: *const FFIMeshResult) -> c_int {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        if result.has_transparency() {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub extern "C" fn meshresult_bounds(ptr: *const FFIMeshResult) -> CFloatArray {
        if ptr.is_null() {
            return CFloatArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let result = unsafe { &(*ptr).0 };
        let mut vals = Vec::with_capacity(6);
        vals.extend_from_slice(&result.bounds.min);
        vals.extend_from_slice(&result.bounds.max);
        let mut boxed = vals.into_boxed_slice();
        let p = boxed.as_mut_ptr();
        let len = boxed.len();
        std::mem::forget(boxed);
        CFloatArray { data: p, len }
    }

    // --- MultiMeshResult FFI ---

    #[no_mangle]
    pub extern "C" fn multimeshresult_free(ptr: *mut FFIMultiMeshResult) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn multimeshresult_region_names(ptr: *const FFIMultiMeshResult) -> StringArray {
        if ptr.is_null() {
            return StringArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let result = unsafe { &(*ptr).0 };
        let names: Vec<String> = result.keys().cloned().collect();
        vec_string_to_string_array(names)
    }

    #[no_mangle]
    pub extern "C" fn multimeshresult_get_mesh(
        ptr: *const FFIMultiMeshResult,
        region_name: *const c_char,
    ) -> *mut FFIMeshResult {
        if ptr.is_null() || region_name.is_null() {
            return ptr::null_mut();
        }
        let result = unsafe { &(*ptr).0 };
        let name = unsafe { CStr::from_ptr(region_name).to_string_lossy() };
        match result.get(name.as_ref()) {
            Some(mesh) => Box::into_raw(Box::new(FFIMeshResult(mesh.clone()))),
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn multimeshresult_total_vertex_count(ptr: *const FFIMultiMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.values().map(|m| m.total_vertices()).sum()
    }

    #[no_mangle]
    pub extern "C" fn multimeshresult_total_triangle_count(
        ptr: *const FFIMultiMeshResult,
    ) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.values().map(|m| m.total_triangles()).sum()
    }

    #[no_mangle]
    pub extern "C" fn multimeshresult_mesh_count(ptr: *const FFIMultiMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.len()
    }

    // --- ChunkMeshResult FFI ---

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_free(ptr: *mut FFIChunkMeshResult) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_chunk_coordinates(
        ptr: *const FFIChunkMeshResult,
    ) -> IntArray {
        if ptr.is_null() {
            return IntArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let result = unsafe { &(*ptr).0 };
        let mut flat: Vec<c_int> = Vec::with_capacity(result.meshes.len() * 3);
        for (cx, cy, cz) in result.meshes.keys() {
            flat.push(*cx);
            flat.push(*cy);
            flat.push(*cz);
        }
        let mut boxed = flat.into_boxed_slice();
        let p = boxed.as_mut_ptr();
        let len = boxed.len();
        std::mem::forget(boxed);
        IntArray { data: p, len }
    }

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_get_mesh(
        ptr: *const FFIChunkMeshResult,
        cx: c_int,
        cy: c_int,
        cz: c_int,
    ) -> *mut FFIMeshResult {
        if ptr.is_null() {
            return ptr::null_mut();
        }
        let result = unsafe { &(*ptr).0 };
        match result.meshes.get(&(cx, cy, cz)) {
            Some(mesh) => Box::into_raw(Box::new(FFIMeshResult(mesh.clone()))),
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_total_vertex_count(ptr: *const FFIChunkMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.total_vertex_count
    }

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_total_triangle_count(
        ptr: *const FFIChunkMeshResult,
    ) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.total_triangle_count
    }

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_nucm_data(ptr: *const FFIChunkMeshResult) -> ByteArray {
        if ptr.is_null() {
            return ByteArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let result = unsafe { &(*ptr).0 };
        let meshes: Vec<crate::meshing::MeshOutput> = result.meshes.values().cloned().collect();
        let mut data = crate::meshing::cache::serialize_meshes(&meshes);
        let p = data.as_mut_ptr();
        let len = data.len();
        std::mem::forget(data);
        ByteArray { data: p, len }
    }

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_chunk_count(ptr: *const FFIChunkMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.meshes.len()
    }

    // --- Schematic Meshing Methods ---

    #[no_mangle]
    pub extern "C" fn schematic_to_mesh(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
    ) -> *mut FFIMeshResult {
        if schematic.is_null() || pack.is_null() || config.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        match s.to_mesh(p, c) {
            Ok(result) => Box::into_raw(Box::new(FFIMeshResult(result))),
            Err(_) => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn schematic_mesh_by_region(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
    ) -> *mut FFIMultiMeshResult {
        if schematic.is_null() || pack.is_null() || config.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        match s.mesh_by_region(p, c) {
            Ok(result) => Box::into_raw(Box::new(FFIMultiMeshResult(result))),
            Err(_) => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn schematic_mesh_by_chunk(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
    ) -> *mut FFIChunkMeshResult {
        if schematic.is_null() || pack.is_null() || config.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        match s.mesh_by_chunk(p, c) {
            Ok(result) => Box::into_raw(Box::new(FFIChunkMeshResult(result))),
            Err(_) => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn schematic_mesh_by_chunk_size(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
        chunk_size: c_int,
    ) -> *mut FFIChunkMeshResult {
        if schematic.is_null() || pack.is_null() || config.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        match s.mesh_by_chunk_size(p, c, chunk_size) {
            Ok(result) => Box::into_raw(Box::new(FFIChunkMeshResult(result))),
            Err(_) => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn schematic_to_usdz(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
    ) -> *mut FFIMeshResult {
        if schematic.is_null() || pack.is_null() || config.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        match s.to_usdz(p, c) {
            Ok(result) => Box::into_raw(Box::new(FFIMeshResult(result))),
            Err(_) => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn schematic_to_raw_mesh(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
    ) -> *mut FFIRawMeshExport {
        if schematic.is_null() || pack.is_null() || config.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        match s.to_raw_mesh(p, c) {
            Ok(result) => Box::into_raw(Box::new(FFIRawMeshExport(result))),
            Err(_) => ptr::null_mut(),
        }
    }

    // --- RawMeshExport FFI ---

    #[no_mangle]
    pub extern "C" fn rawmeshexport_free(ptr: *mut FFIRawMeshExport) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_vertex_count(ptr: *const FFIRawMeshExport) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let raw = unsafe { &(*ptr).0 };
        raw.vertex_count()
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_triangle_count(ptr: *const FFIRawMeshExport) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let raw = unsafe { &(*ptr).0 };
        raw.triangle_count()
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_positions(ptr: *const FFIRawMeshExport) -> CFloatArray {
        if ptr.is_null() {
            return CFloatArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let raw = unsafe { &(*ptr).0 };
        let floats = raw.positions_flat();
        let len = floats.len();
        let boxed = floats.into_boxed_slice();
        let data = Box::into_raw(boxed) as *mut c_float;
        CFloatArray { data, len }
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_normals(ptr: *const FFIRawMeshExport) -> CFloatArray {
        if ptr.is_null() {
            return CFloatArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let raw = unsafe { &(*ptr).0 };
        let floats = raw.normals_flat();
        let len = floats.len();
        let boxed = floats.into_boxed_slice();
        let data = Box::into_raw(boxed) as *mut c_float;
        CFloatArray { data, len }
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_uvs(ptr: *const FFIRawMeshExport) -> CFloatArray {
        if ptr.is_null() {
            return CFloatArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let raw = unsafe { &(*ptr).0 };
        let floats = raw.uvs_flat();
        let len = floats.len();
        let boxed = floats.into_boxed_slice();
        let data = Box::into_raw(boxed) as *mut c_float;
        CFloatArray { data, len }
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_colors(ptr: *const FFIRawMeshExport) -> CFloatArray {
        if ptr.is_null() {
            return CFloatArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let raw = unsafe { &(*ptr).0 };
        let floats = raw.colors_flat();
        let len = floats.len();
        let boxed = floats.into_boxed_slice();
        let data = Box::into_raw(boxed) as *mut c_float;
        CFloatArray { data, len }
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_indices(ptr: *const FFIRawMeshExport) -> IntArray {
        if ptr.is_null() {
            return IntArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let raw = unsafe { &(*ptr).0 };
        let indices: Vec<c_int> = raw.indices().iter().map(|&i| i as c_int).collect();
        let len = indices.len();
        let boxed = indices.into_boxed_slice();
        let data = Box::into_raw(boxed) as *mut c_int;
        IntArray { data, len }
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_texture_rgba(ptr: *const FFIRawMeshExport) -> ByteArray {
        if ptr.is_null() {
            return ByteArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let raw = unsafe { &(*ptr).0 };
        let pixels = raw.texture_rgba().to_vec();
        let len = pixels.len();
        let boxed = pixels.into_boxed_slice();
        let data = Box::into_raw(boxed) as *mut c_uchar;
        ByteArray { data, len }
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_texture_width(ptr: *const FFIRawMeshExport) -> u32 {
        if ptr.is_null() {
            return 0;
        }
        let raw = unsafe { &(*ptr).0 };
        raw.texture_width()
    }

    #[no_mangle]
    pub extern "C" fn rawmeshexport_texture_height(ptr: *const FFIRawMeshExport) -> u32 {
        if ptr.is_null() {
            return 0;
        }
        let raw = unsafe { &(*ptr).0 };
        raw.texture_height()
    }

    /// Register a MeshExporter with the FormatManager so save_as("mesh", ...) works.
    /// Returns 0 on success, -1 on failure.
    #[no_mangle]
    pub extern "C" fn schematic_register_mesh_exporter(pack: *const FFIResourcePack) -> c_int {
        if pack.is_null() {
            return -1;
        }
        let p = unsafe { &(*pack).0 };
        let mesh_exporter = crate::meshing::MeshExporter::new(
            ResourcePackSource::from_resource_pack(p.pack().clone()),
        );

        let manager = get_manager();
        let mut manager = match manager.lock() {
            Ok(m) => m,
            Err(_) => return -1,
        };
        manager.register_exporter(mesh_exporter);
        0
    }

    // --- TextureAtlas Lifecycle ---

    #[no_mangle]
    pub extern "C" fn textureatlas_free(ptr: *mut FFITextureAtlas) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn textureatlas_width(ptr: *const FFITextureAtlas) -> u32 {
        if ptr.is_null() {
            return 0;
        }
        unsafe { (*ptr).0.width }
    }

    #[no_mangle]
    pub extern "C" fn textureatlas_height(ptr: *const FFITextureAtlas) -> u32 {
        if ptr.is_null() {
            return 0;
        }
        unsafe { (*ptr).0.height }
    }

    #[no_mangle]
    pub extern "C" fn textureatlas_rgba_data(ptr: *const FFITextureAtlas) -> ByteArray {
        if ptr.is_null() {
            return ByteArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let atlas = unsafe { &(*ptr).0 };
        let mut data = atlas.pixels.clone();
        let p = data.as_mut_ptr();
        let len = data.len();
        std::mem::forget(data);
        ByteArray { data: p, len }
    }

    // --- Global Atlas Builder ---

    #[no_mangle]
    pub extern "C" fn schematic_build_global_atlas(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
    ) -> *mut FFITextureAtlas {
        if schematic.is_null() || pack.is_null() || config.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        match crate::meshing::build_global_atlas(s, p, c) {
            Ok(atlas) => Box::into_raw(Box::new(FFITextureAtlas(atlas))),
            Err(_) => ptr::null_mut(),
        }
    }

    // --- Chunk Meshing with Shared Atlas ---

    #[no_mangle]
    pub extern "C" fn schematic_mesh_chunks_with_atlas(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
        chunk_size: c_int,
        atlas: *const FFITextureAtlas,
    ) -> *mut FFIChunkMeshResult {
        if schematic.is_null() || pack.is_null() || config.is_null() || atlas.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        let a = unsafe { &(*atlas).0 };

        let iter = s.mesh_chunks_with_atlas(p, c, chunk_size, a.clone());
        let mut meshes = std::collections::HashMap::new();
        let mut total_vertex_count = 0;
        let mut total_triangle_count = 0;

        for result in iter {
            match result {
                Ok(mesh) => {
                    total_vertex_count += mesh.total_vertices();
                    total_triangle_count += mesh.total_triangles();
                    if let Some(coord) = mesh.chunk_coord {
                        meshes.insert(coord, mesh);
                    }
                }
                Err(_) => return ptr::null_mut(),
            }
        }

        Box::into_raw(Box::new(FFIChunkMeshResult(ChunkMeshResult {
            meshes,
            total_vertex_count,
            total_triangle_count,
        })))
    }

    // --- NUCM v2 with Shared Atlas ---

    #[no_mangle]
    pub extern "C" fn chunkmeshresult_nucm_data_with_atlas(
        ptr: *const FFIChunkMeshResult,
        atlas: *const FFITextureAtlas,
    ) -> ByteArray {
        if ptr.is_null() || atlas.is_null() {
            return ByteArray {
                data: ptr::null_mut(),
                len: 0,
            };
        }
        let result = unsafe { &(*ptr).0 };
        let atlas = unsafe { &(*atlas).0 };
        let meshes: Vec<crate::meshing::MeshOutput> = result.meshes.values().cloned().collect();
        let mut data = crate::meshing::cache::serialize_meshes_with_atlas(&meshes, atlas);
        let p = data.as_mut_ptr();
        let len = data.len();
        std::mem::forget(data);
        ByteArray { data: p, len }
    }

    // --- Progress Callback Support ---

    /// C-compatible progress callback signature.
    /// phase: 0=BuildingAtlas, 1=MeshingChunks, 2=Complete
    pub type MeshProgressCallback = extern "C" fn(
        phase: c_int,
        chunks_done: u32,
        chunks_total: u32,
        vertices_so_far: u64,
        triangles_so_far: u64,
        user_data: *mut std::ffi::c_void,
    );

    #[no_mangle]
    pub extern "C" fn schematic_mesh_chunks_with_atlas_progress(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIMeshConfig,
        chunk_size: c_int,
        atlas: *const FFITextureAtlas,
        callback: MeshProgressCallback,
        user_data: *mut std::ffi::c_void,
    ) -> *mut FFIChunkMeshResult {
        if schematic.is_null() || pack.is_null() || config.is_null() || atlas.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let p = unsafe { &(*pack).0 };
        let c = unsafe { &(*config).0 };
        let a = unsafe { &(*atlas).0 };

        let mut iter = s.mesh_chunks_with_atlas(p, c, chunk_size, a.clone());

        // Wrap C callback into Rust closure.
        // Cast user_data to usize so the closure is Send-safe.
        let ud = user_data as usize;
        iter.set_progress_callback(Box::new(move |progress: MeshProgress| {
            let phase = match progress.phase {
                MeshPhase::BuildingAtlas => 0,
                MeshPhase::MeshingChunks => 1,
                MeshPhase::Complete => 2,
            };
            callback(
                phase,
                progress.chunks_done,
                progress.chunks_total,
                progress.vertices_so_far,
                progress.triangles_so_far,
                ud as *mut std::ffi::c_void,
            );
        }));

        let mut meshes = std::collections::HashMap::new();
        let mut total_vertex_count = 0;
        let mut total_triangle_count = 0;

        for result in iter {
            match result {
                Ok(mesh) => {
                    total_vertex_count += mesh.total_vertices();
                    total_triangle_count += mesh.total_triangles();
                    if let Some(coord) = mesh.chunk_coord {
                        meshes.insert(coord, mesh);
                    }
                }
                Err(_) => return ptr::null_mut(),
            }
        }

        Box::into_raw(Box::new(FFIChunkMeshResult(ChunkMeshResult {
            meshes,
            total_vertex_count,
            total_triangle_count,
        })))
    }
}

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
