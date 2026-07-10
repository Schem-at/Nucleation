#![allow(unused_imports)]

use super::definition_region::CFloatArray;
use super::*;

// =============================================================================
// Meshing FFI Bindings (feature-gated)
// =============================================================================

use crate::meshing::{
    ChunkMeshResult, MeshConfig, MeshPhase, MeshProgress, MeshResult, MultiMeshResult,
    RawMeshExport, ResourcePackSource,
};
use schematic_mesher::TextureAtlas;

// --- Wrapper Structs ---

pub struct FFIResourcePack(pub(crate) ResourcePackSource);
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

/// Load and merge multiple resource packs from in-memory byte buffers,
/// lowest priority first. `data_ptrs` and `data_lens` are parallel arrays
/// of length `count`. Later buffers overlay earlier ones on per-key
/// collision (matches Minecraft's own pack-ordering semantics).
///
/// Returns null on any load error or if inputs are invalid.
#[no_mangle]
pub extern "C" fn resourcepack_from_bytes_list(
    data_ptrs: *const *const c_uchar,
    data_lens: *const usize,
    count: usize,
) -> *mut FFIResourcePack {
    if count == 0 {
        return match ResourcePackSource::from_bytes_list(Vec::<Vec<u8>>::new()) {
            Ok(pack) => Box::into_raw(Box::new(FFIResourcePack(pack))),
            Err(_) => ptr::null_mut(),
        };
    }
    if data_ptrs.is_null() || data_lens.is_null() {
        return ptr::null_mut();
    }

    let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(count);
    for i in 0..count {
        let ptr = unsafe { *data_ptrs.add(i) };
        let len = unsafe { *data_lens.add(i) };
        if ptr.is_null() || len == 0 {
            return ptr::null_mut();
        }
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        buffers.push(slice.to_vec());
    }

    match ResourcePackSource::from_bytes_list(buffers) {
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
pub extern "C" fn multimeshresult_total_triangle_count(ptr: *const FFIMultiMeshResult) -> usize {
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
pub extern "C" fn chunkmeshresult_chunk_coordinates(ptr: *const FFIChunkMeshResult) -> IntArray {
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
pub extern "C" fn chunkmeshresult_total_triangle_count(ptr: *const FFIChunkMeshResult) -> usize {
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
    let mesh_exporter =
        crate::meshing::MeshExporter::new(ResourcePackSource::from_resource_pack(p.pack().clone()));

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

// =============================================================================
// Item Model FFI (feature-gated)
// =============================================================================

use super::*;
use crate::meshing::{ItemModelConfig, ItemModelResult};
use std::ffi::CStr;

pub struct FFIItemModelConfig(ItemModelConfig);
pub struct FFIItemModelResult(ItemModelResult);

#[no_mangle]
pub extern "C" fn itemmodel_config_new(model_name: *const c_char) -> *mut FFIItemModelConfig {
    if model_name.is_null() {
        return ptr::null_mut();
    }
    let name = unsafe { CStr::from_ptr(model_name) }
        .to_str()
        .unwrap_or("schematic");
    Box::into_raw(Box::new(FFIItemModelConfig(ItemModelConfig::new(name))))
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_namespace(
    config: *mut FFIItemModelConfig,
    namespace: *const c_char,
) {
    if config.is_null() || namespace.is_null() {
        return;
    }
    let ns = unsafe { CStr::from_ptr(namespace) }
        .to_str()
        .unwrap_or("nucleation");
    unsafe { (*config).0.namespace = ns.to_string() };
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_center(config: *mut FFIItemModelConfig, center: bool) {
    if config.is_null() {
        return;
    }
    unsafe { (*config).0.center = center };
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_texture_resolution(
    config: *mut FFIItemModelConfig,
    resolution: u32,
) {
    if config.is_null() {
        return;
    }
    unsafe { (*config).0.texture_resolution = resolution };
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_item(config: *mut FFIItemModelConfig, item: *const c_char) {
    if config.is_null() || item.is_null() {
        return;
    }
    let item = unsafe { CStr::from_ptr(item) }.to_str().unwrap_or("paper");
    unsafe { (*config).0.item = item.to_string() };
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_custom_model_data(
    config: *mut FFIItemModelConfig,
    cmd: *const c_char,
) {
    if config.is_null() || cmd.is_null() {
        return;
    }
    let cmd = unsafe { CStr::from_ptr(cmd) }.to_str().unwrap_or("1");
    unsafe { (*config).0.custom_model_data = cmd.to_string() };
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_scale(config: *mut FFIItemModelConfig, scale: f32) {
    if config.is_null() {
        return;
    }
    unsafe { (*config).0.scale = crate::meshing::ItemModelScale::Uniform(scale) };
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_scale_xyz(
    config: *mut FFIItemModelConfig,
    sx: f32,
    sy: f32,
    sz: f32,
) {
    if config.is_null() {
        return;
    }
    unsafe { (*config).0.scale = crate::meshing::ItemModelScale::NonUniform(sx, sy, sz) };
}

#[no_mangle]
pub extern "C" fn itemmodel_config_set_scale_auto(config: *mut FFIItemModelConfig) {
    if config.is_null() {
        return;
    }
    unsafe { (*config).0.scale = crate::meshing::ItemModelScale::Auto };
}

#[no_mangle]
pub extern "C" fn itemmodel_result_scale(
    result: *const FFIItemModelResult,
    out_sx: *mut f32,
    out_sy: *mut f32,
    out_sz: *mut f32,
) {
    if result.is_null() || out_sx.is_null() || out_sy.is_null() || out_sz.is_null() {
        return;
    }
    let (sx, sy, sz) = unsafe { &(*result) }.0.stats.scale;
    unsafe {
        *out_sx = sx;
        *out_sy = sy;
        *out_sz = sz;
    }
}

#[no_mangle]
pub extern "C" fn itemmodel_config_free(ptr: *mut FFIItemModelConfig) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn schematic_to_item_model(
    schematic: *const SchematicWrapper,
    pack: *const FFIResourcePack,
    config: *const FFIItemModelConfig,
) -> *mut FFIItemModelResult {
    if schematic.is_null() || pack.is_null() || config.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { &*(*schematic).0 };
    let p = unsafe { &(*pack).0 };
    let c = unsafe { &(*config).0 };
    match s.to_item_model(p, c) {
        Ok(result) => Box::into_raw(Box::new(FFIItemModelResult(result))),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn itemmodel_result_model_json(
    result: *const FFIItemModelResult,
    out_len: *mut usize,
) -> *const c_char {
    if result.is_null() || out_len.is_null() {
        return ptr::null();
    }
    let json = &unsafe { &(*result) }.0.model_json;
    unsafe { *out_len = json.len() };
    json.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn itemmodel_result_element_count(result: *const FFIItemModelResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { &(*result) }.0.stats.element_count
}

#[no_mangle]
pub extern "C" fn itemmodel_result_texture_count(result: *const FFIItemModelResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { &(*result) }.0.stats.texture_count
}

#[no_mangle]
pub extern "C" fn itemmodel_result_plane_count(result: *const FFIItemModelResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { &(*result) }.0.stats.plane_count
}

#[no_mangle]
pub extern "C" fn itemmodel_result_dimensions(
    result: *const FFIItemModelResult,
    out_width: *mut i32,
    out_height: *mut i32,
    out_depth: *mut i32,
) {
    if result.is_null() || out_width.is_null() || out_height.is_null() || out_depth.is_null() {
        return;
    }
    let (w, h, d) = unsafe { &(*result) }.0.stats.dimensions;
    unsafe {
        *out_width = w;
        *out_height = h;
        *out_depth = d;
    }
}

#[no_mangle]
pub extern "C" fn itemmodel_result_to_resource_pack_zip(
    result: *const FFIItemModelResult,
    out_len: *mut usize,
) -> *mut u8 {
    if result.is_null() || out_len.is_null() {
        return ptr::null_mut();
    }
    let r = unsafe { &(*result) };
    match r.0.to_resource_pack_zip() {
        Ok(data) => {
            unsafe { *out_len = data.len() };
            let boxed = data.into_boxed_slice();
            Box::into_raw(boxed) as *mut u8
        }
        Err(_) => {
            unsafe { *out_len = 0 };
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn itemmodel_result_free(ptr: *mut FFIItemModelResult) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn itemmodel_zip_data_free(data: *mut u8, len: usize) {
    if !data.is_null() && len > 0 {
        unsafe {
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(data, len)));
        }
    }
}

/// Build a resource pack ZIP from multiple item model results.
///
/// `results` is a pointer to an array of `*const FFIItemModelResult` pointers.
/// `count` is the number of elements in the array.
/// Returns a pointer to the ZIP data and writes the length to `out_len`.
/// Free the returned data with `itemmodel_zip_data_free`.
#[no_mangle]
pub extern "C" fn itemmodel_build_resource_pack(
    results: *const *const FFIItemModelResult,
    count: usize,
    out_len: *mut usize,
) -> *mut u8 {
    if results.is_null() || out_len.is_null() || count == 0 {
        if !out_len.is_null() {
            unsafe { *out_len = 0 };
        }
        return ptr::null_mut();
    }
    let result_ptrs = unsafe { std::slice::from_raw_parts(results, count) };
    let refs: Vec<&ItemModelResult> = result_ptrs
        .iter()
        .filter_map(|&p| {
            if p.is_null() {
                None
            } else {
                Some(&unsafe { &*p }.0)
            }
        })
        .collect();
    match crate::meshing::build_resource_pack(&refs) {
        Ok(data) => {
            unsafe { *out_len = data.len() };
            let boxed = data.into_boxed_slice();
            Box::into_raw(boxed) as *mut u8
        }
        Err(_) => {
            unsafe { *out_len = 0 };
            ptr::null_mut()
        }
    }
}
