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
    formats::{litematic, manager::get_manager, schematic},
    print_utils::{format_json_schematic, format_schematic},
    universal_schematic::ChunkLoadingStrategy,
    BlockState, SchematicBuilder, UniversalSchematic,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_uchar};
use std::ptr;

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
/// Returns 0 on success, negative on error.
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

    if litematic::is_litematic(data_slice) {
        match litematic::from_litematic(data_slice) {
            Ok(res) => {
                *s = res;
                0
            }
            Err(_) => -2,
        }
    } else if schematic::is_schematic(data_slice) {
        match schematic::from_schematic(data_slice) {
            Ok(res) => {
                *s = res;
                0
            }
            Err(_) => -2,
        }
    } else {
        -3 // Unknown format
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

    let mut props = HashMap::new();
    if !properties.is_null() {
        let props_slice = unsafe { std::slice::from_raw_parts(properties, properties_len) };
        for prop in props_slice {
            let key = unsafe { CStr::from_ptr(prop.key).to_string_lossy().into_owned() };
            let value = unsafe { CStr::from_ptr(prop.value).to_string_lossy().into_owned() };
            props.insert(key, value);
        }
    }

    let block_state = BlockState {
        name: block_name_str,
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
        CString::new(block_state.name.clone()).unwrap().into_raw()
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
                name: CString::new(block.name.clone()).unwrap().into_raw(),
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
                name: CString::new(block.name.clone()).unwrap().into_raw(),
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
                        name: CString::new(block.name.clone()).unwrap().into_raw(),
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
    CString::new(state.name.clone()).unwrap().into_raw()
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
            key: CString::new(k.clone()).unwrap().into_raw(),
            value: CString::new(v.clone()).unwrap().into_raw(),
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
    cx: c_int,
    cy: c_int,
    cz: c_int,
    radius: c_float,
    block_name: *const c_char,
) -> c_int {
    if schematic.is_null() || block_name.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let block = BlockState::new(name);
    let shape = ShapeEnum::Sphere(Sphere::new((cx, cy, cz), radius as f64));
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

    let manager = get_manager();
    let manager = match manager.lock() {
        Ok(m) => m,
        Err(_) => return empty,
    };
    match manager.write(&fmt, s, ver.as_deref()) {
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
pub extern "C" fn schematic_get_bounding_box(schematic: *const SchematicWrapper) -> CBoundingBox {
    let empty = CBoundingBox {
        min_x: 0,
        min_y: 0,
        min_z: 0,
        max_x: 0,
        max_y: 0,
        max_z: 0,
    };
    if schematic.is_null() {
        return empty;
    }
    let s = unsafe { &*(*schematic).0 };
    let bbox = s.get_bounding_box();
    CBoundingBox {
        min_x: bbox.min.0,
        min_y: bbox.min.1,
        min_z: bbox.min.2,
        max_x: bbox.max.0,
        max_y: bbox.max.1,
        max_z: bbox.max.2,
    }
}

#[no_mangle]
pub extern "C" fn schematic_get_region_bounding_box(
    schematic: *const SchematicWrapper,
    region_name: *const c_char,
) -> CBoundingBox {
    let empty = CBoundingBox {
        min_x: 0,
        min_y: 0,
        min_z: 0,
        max_x: 0,
        max_y: 0,
        max_z: 0,
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
    CBoundingBox {
        min_x: bbox.min.0,
        min_y: bbox.min.1,
        min_z: bbox.min.2,
        max_x: bbox.max.0,
        max_y: bbox.max.1,
        max_z: bbox.max.2,
    }
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
    let names: Vec<String> = merged.palette.iter().map(|bs| bs.name.clone()).collect();
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
pub extern "C" fn definitionregion_get_bounds(ptr: *const DefinitionRegionWrapper) -> CBoundingBox {
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
    match r.get_bounds() {
        Some(bbox) => CBoundingBox {
            min_x: bbox.min.0,
            min_y: bbox.min.1,
            min_z: bbox.min.2,
            max_x: bbox.max.0,
            max_y: bbox.max.1,
            max_z: bbox.max.2,
        },
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
        .map(|bs| bs.name.clone())
        .collect();
    palettes.insert("default".to_string(), default_blocks);
    for (name, region) in &s.other_regions {
        let blocks: Vec<String> = region.palette.iter().map(|bs| bs.name.clone()).collect();
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
        .map(|bs| bs.name.clone())
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
    let names: Vec<String> = region.palette.iter().map(|bs| bs.name.clone()).collect();
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
) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*ptr).0 };
    let json = serde_json::to_string(&r.metadata).unwrap_or_default();
    CString::new(json).unwrap().into_raw()
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
                    name: CString::new(block.name.clone()).unwrap().into_raw(),
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
    cx: c_int,
    cy: c_int,
    cz: c_int,
    radius: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Sphere(Sphere::new(
        (cx, cy, cz),
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
// Meshing FFI Bindings (feature-gated)
// =============================================================================

#[cfg(feature = "meshing")]
mod meshing_ffi {
    use super::*;
    use crate::meshing::{
        ChunkMeshResult, MeshConfig, MeshResult, MultiMeshResult, ResourcePackSource,
    };

    // --- Wrapper Structs ---

    pub struct FFIResourcePack(ResourcePackSource);
    pub struct FFIMeshConfig(MeshConfig);
    pub struct FFIMeshResult(MeshResult);
    pub struct FFIMultiMeshResult(MultiMeshResult);
    pub struct FFIChunkMeshResult(ChunkMeshResult);

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
        let mut data = result.glb_data.clone();
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
        result.vertex_count
    }

    #[no_mangle]
    pub extern "C" fn meshresult_triangle_count(ptr: *const FFIMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.triangle_count
    }

    #[no_mangle]
    pub extern "C" fn meshresult_has_transparency(ptr: *const FFIMeshResult) -> c_int {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        if result.has_transparency {
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
        let vals = result.bounds.to_vec();
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
        let names: Vec<String> = result.meshes.keys().cloned().collect();
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
        match result.meshes.get(name.as_ref()) {
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
        result.total_vertex_count
    }

    #[no_mangle]
    pub extern "C" fn multimeshresult_total_triangle_count(
        ptr: *const FFIMultiMeshResult,
    ) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.total_triangle_count
    }

    #[no_mangle]
    pub extern "C" fn multimeshresult_mesh_count(ptr: *const FFIMultiMeshResult) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let result = unsafe { &(*ptr).0 };
        result.meshes.len()
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
}
