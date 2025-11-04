// src/ffi.rs
#![cfg(feature = "ffi")]
use crate::{
    block_position::BlockPosition,
    bounding_box::BoundingBox,
    formats::{litematic, schematic},
    print_utils::{format_json_schematic, format_schematic},
    universal_schematic::ChunkLoadingStrategy,
    BlockState, UniversalSchematic,
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
pub extern "C" fn free_block_array(mut array: CBlockArray) {
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
pub extern "C" fn free_block_entity_array(mut array: CBlockEntityArray) {
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
pub extern "C" fn free_chunk_array(mut array: CChunkArray) {
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
    s.set_block(x, y, z, block_state);
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
    s.set_block(x, y, z, block_state);
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
