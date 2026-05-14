//! Meshing JNI exports (feature-gated).
//!
//! Mirrors `PyResourcePack`, `PyMeshConfig`, `PyMeshResult`,
//! `PyMultiMeshResult`, and the `Schematic.mesh_*` family.
//!
//! Surface scope:
//! - `ResourcePack`: load from file path, from in-memory zip bytes, stats
//! - `MeshConfig`: full constructor + all getters/setters
//! - `MeshResult`: `glb_data` (GLB bytes), counts, bounds, atlas size
//! - `MultiMeshResult`: ordered list of (regionName, MeshResult)
//! - `Schematic.mesh_by_region` and `Schematic.mesh` (single-mesh helper)

#![cfg(feature = "meshing")]

use crate::conv::{jbytearray_to_vec, jstring_opt_to_string, jstring_to_string, string_to_jstring, vec_to_jbytearray};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_mut, as_ref, consume, to_handle};
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jbyteArray, jfloatArray, jint, jintArray, jlong, jlongArray, jobjectArray, jstring};
use jni::{JNIEnv, NativeMethod};
use nucleation::meshing::{MeshConfig, MeshOutput, MultiMeshResult, ResourcePackSource};
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        // ── ResourcePack ────────────────────────────────────────────────
        nm("nResourcePackFromFile", "(Ljava/lang/String;)J", n_pack_from_file as *mut _),
        nm("nResourcePackFromBytes", "([B)J", n_pack_from_bytes as *mut _),
        nm("nResourcePackFree", "(J)V", n_pack_free as *mut _),
        nm("nResourcePackBlockstateCount", "(J)I", n_pack_blockstate_count as *mut _),
        nm("nResourcePackModelCount", "(J)I", n_pack_model_count as *mut _),
        nm("nResourcePackTextureCount", "(J)I", n_pack_texture_count as *mut _),

        // ── MeshConfig ──────────────────────────────────────────────────
        nm("nMeshConfigCreate", "(ZZFLjava/lang/String;IZZ)J", n_config_create as *mut _),
        nm("nMeshConfigDefault", "()J", n_config_default as *mut _),
        nm("nMeshConfigFree", "(J)V", n_config_free as *mut _),
        nm("nMeshConfigGetCullHiddenFaces", "(J)Z", n_config_get_cull as *mut _),
        nm("nMeshConfigSetCullHiddenFaces", "(JZ)V", n_config_set_cull as *mut _),
        nm("nMeshConfigGetAmbientOcclusion", "(J)Z", n_config_get_ao as *mut _),
        nm("nMeshConfigSetAmbientOcclusion", "(JZ)V", n_config_set_ao as *mut _),
        nm("nMeshConfigGetAoIntensity", "(J)F", n_config_get_ao_intensity as *mut _),
        nm("nMeshConfigSetAoIntensity", "(JF)V", n_config_set_ao_intensity as *mut _),
        nm("nMeshConfigGetBiome", "(J)Ljava/lang/String;", n_config_get_biome as *mut _),
        nm("nMeshConfigSetBiome", "(JLjava/lang/String;)V", n_config_set_biome as *mut _),
        nm("nMeshConfigGetAtlasMaxSize", "(J)I", n_config_get_atlas_max_size as *mut _),
        nm("nMeshConfigSetAtlasMaxSize", "(JI)V", n_config_set_atlas_max_size as *mut _),
        nm("nMeshConfigGetCullOccludedBlocks", "(J)Z", n_config_get_cull_occluded as *mut _),
        nm("nMeshConfigSetCullOccludedBlocks", "(JZ)V", n_config_set_cull_occluded as *mut _),
        nm("nMeshConfigGetGreedyMeshing", "(J)Z", n_config_get_greedy as *mut _),
        nm("nMeshConfigSetGreedyMeshing", "(JZ)V", n_config_set_greedy as *mut _),

        // ── MeshResult ──────────────────────────────────────────────────
        nm("nMeshResultFree", "(J)V", n_result_free as *mut _),
        nm("nMeshResultGlbData", "(J)[B", n_result_glb_data as *mut _),
        nm("nMeshResultVertexCount", "(J)I", n_result_vertex_count as *mut _),
        nm("nMeshResultTriangleCount", "(J)I", n_result_triangle_count as *mut _),
        nm("nMeshResultIsEmpty", "(J)Z", n_result_is_empty as *mut _),
        nm("nMeshResultHasTransparency", "(J)Z", n_result_has_transparency as *mut _),
        nm("nMeshResultBounds", "(J)[F", n_result_bounds as *mut _),
        nm("nMeshResultAtlasWidth", "(J)I", n_result_atlas_width as *mut _),
        nm("nMeshResultAtlasHeight", "(J)I", n_result_atlas_height as *mut _),
        nm("nMeshResultAtlasRgba", "(J)[B", n_result_atlas_rgba as *mut _),
        nm("nMeshResultLodLevel", "(J)I", n_result_lod_level as *mut _),
        nm("nMeshResultChunkCoord", "(J)[I", n_result_chunk_coord as *mut _),

        // ── MultiMeshResult ─────────────────────────────────────────────
        nm("nMultiMeshFree", "(J)V", n_multi_free as *mut _),
        nm("nMultiMeshSize", "(J)I", n_multi_size as *mut _),
        nm("nMultiMeshRegionNames", "(J)[Ljava/lang/String;", n_multi_region_names as *mut _),
        nm("nMultiMeshGet", "(JLjava/lang/String;)J", n_multi_get as *mut _),
        nm("nMultiMeshAllHandles", "(J)[J", n_multi_all_handles as *mut _),

        // ── Schematic.mesh_* ────────────────────────────────────────────
        nm("nSchematicMeshByRegion", "(JJJ)J", n_mesh_by_region as *mut _),
        nm("nSchematicMeshSingle", "(JJJ)J", n_mesh_single as *mut _),
        nm(
            "nSchematicMeshAnimated",
            "(JJLjava/lang/String;)[B",
            n_mesh_animated as *mut _,
        ),
    ];
    env.register_native_methods(&class, methods)
}

fn nm(name: &str, sig: &str, ptr: *mut c_void) -> NativeMethod {
    NativeMethod {
        name: name.into(),
        sig: sig.into(),
        fn_ptr: ptr,
    }
}

// ============================================================================
// ResourcePack
// ============================================================================

unsafe extern "system" fn n_pack_from_file<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    path: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let p = jstring_to_string(env, &path)?;
        let pack = ResourcePackSource::from_file(&p)
            .map_err(|e| JvmError::Parse(format!("ResourcePack::from_file: {e}")))?;
        Ok(to_handle(pack))
    })
}

unsafe extern "system" fn n_pack_from_bytes<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    data: JByteArray<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let bytes = jbytearray_to_vec(env, &data)?;
        let pack = ResourcePackSource::from_bytes(&bytes)
            .map_err(|e| JvmError::Parse(format!("ResourcePack::from_bytes: {e}")))?;
        Ok(to_handle(pack))
    })
}

unsafe extern "system" fn n_pack_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<ResourcePackSource>(handle);
    }
}

unsafe extern "system" fn n_pack_blockstate_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        let pack = as_ref::<ResourcePackSource>(handle);
        Ok(pack.list_blockstates().len() as jint)
    })
}

unsafe extern "system" fn n_pack_model_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        let pack = as_ref::<ResourcePackSource>(handle);
        Ok(pack.list_models().len() as jint)
    })
}

unsafe extern "system" fn n_pack_texture_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        let pack = as_ref::<ResourcePackSource>(handle);
        Ok(pack.list_textures().len() as jint)
    })
}

// ============================================================================
// MeshConfig
// ============================================================================

unsafe extern "system" fn n_config_create<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    cull_hidden_faces: jboolean,
    ambient_occlusion: jboolean,
    ao_intensity: f32,
    biome: JString<'l>,
    atlas_max_size: jint,
    cull_occluded_blocks: jboolean,
    greedy_meshing: jboolean,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let biome = jstring_opt_to_string(env, &biome)?;
        Ok(to_handle(MeshConfig {
            cull_hidden_faces: cull_hidden_faces != 0,
            ambient_occlusion: ambient_occlusion != 0,
            ao_intensity,
            biome,
            atlas_max_size: atlas_max_size.max(1) as u32,
            cull_occluded_blocks: cull_occluded_blocks != 0,
            greedy_meshing: greedy_meshing != 0,
        }))
    })
}

unsafe extern "system" fn n_config_default<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| Ok(to_handle(MeshConfig::default())))
}

unsafe extern "system" fn n_config_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<MeshConfig>(handle);
    }
}

unsafe extern "system" fn n_config_get_cull<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |_env| {
        Ok(as_ref::<MeshConfig>(handle).cull_hidden_faces as u8)
    })
}

unsafe extern "system" fn n_config_set_cull<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    value: jboolean,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<MeshConfig>(handle).cull_hidden_faces = value != 0;
        Ok(())
    })
}

unsafe extern "system" fn n_config_get_ao<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |_env| {
        Ok(as_ref::<MeshConfig>(handle).ambient_occlusion as u8)
    })
}

unsafe extern "system" fn n_config_set_ao<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    value: jboolean,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<MeshConfig>(handle).ambient_occlusion = value != 0;
        Ok(())
    })
}

unsafe extern "system" fn n_config_get_ao_intensity<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> f32 {
    with_jni_context(&mut env, 0.0f32, |_env| {
        Ok(as_ref::<MeshConfig>(handle).ao_intensity)
    })
}

unsafe extern "system" fn n_config_set_ao_intensity<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    value: f32,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<MeshConfig>(handle).ao_intensity = value;
        Ok(())
    })
}

unsafe extern "system" fn n_config_get_biome<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        match &as_ref::<MeshConfig>(handle).biome {
            Some(b) => string_to_jstring(env, b),
            None => Ok(std::ptr::null_mut()),
        }
    })
}

unsafe extern "system" fn n_config_set_biome<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    value: JString<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        as_mut::<MeshConfig>(handle).biome = jstring_opt_to_string(env, &value)?;
        Ok(())
    })
}

unsafe extern "system" fn n_config_get_atlas_max_size<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<MeshConfig>(handle).atlas_max_size as jint)
    })
}

unsafe extern "system" fn n_config_set_atlas_max_size<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    value: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<MeshConfig>(handle).atlas_max_size = value.max(1) as u32;
        Ok(())
    })
}

unsafe extern "system" fn n_config_get_cull_occluded<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |_env| {
        Ok(as_ref::<MeshConfig>(handle).cull_occluded_blocks as u8)
    })
}

unsafe extern "system" fn n_config_set_cull_occluded<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    value: jboolean,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<MeshConfig>(handle).cull_occluded_blocks = value != 0;
        Ok(())
    })
}

unsafe extern "system" fn n_config_get_greedy<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |_env| {
        Ok(as_ref::<MeshConfig>(handle).greedy_meshing as u8)
    })
}

unsafe extern "system" fn n_config_set_greedy<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    value: jboolean,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<MeshConfig>(handle).greedy_meshing = value != 0;
        Ok(())
    })
}

// ============================================================================
// MeshResult (wraps MeshOutput)
// ============================================================================

unsafe extern "system" fn n_result_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<MeshOutput>(handle);
    }
}

unsafe extern "system" fn n_result_glb_data<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mesh = as_ref::<MeshOutput>(handle);
        let bytes = mesh
            .to_glb()
            .map_err(|e| JvmError::Generic(format!("to_glb: {e}")))?;
        vec_to_jbytearray(env, &bytes)
    })
}

unsafe extern "system" fn n_result_vertex_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<MeshOutput>(handle).total_vertices() as jint)
    })
}

unsafe extern "system" fn n_result_triangle_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<MeshOutput>(handle).total_triangles() as jint)
    })
}

unsafe extern "system" fn n_result_is_empty<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 1u8, |_env| {
        Ok(as_ref::<MeshOutput>(handle).is_empty() as u8)
    })
}

unsafe extern "system" fn n_result_has_transparency<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |_env| {
        Ok(as_ref::<MeshOutput>(handle).has_transparency() as u8)
    })
}

unsafe extern "system" fn n_result_bounds<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jfloatArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mesh = as_ref::<MeshOutput>(handle);
        let bounds = [
            mesh.bounds.min[0], mesh.bounds.min[1], mesh.bounds.min[2],
            mesh.bounds.max[0], mesh.bounds.max[1], mesh.bounds.max[2],
        ];
        let arr = env
            .new_float_array(6)
            .map_err(|e| JvmError::Generic(format!("new_float_array: {e}")))?;
        env.set_float_array_region(&arr, 0, &bounds)
            .map_err(|e| JvmError::Generic(format!("set_float_array_region: {e}")))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_result_atlas_width<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<MeshOutput>(handle).atlas.width as jint)
    })
}

unsafe extern "system" fn n_result_atlas_height<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<MeshOutput>(handle).atlas.height as jint)
    })
}

unsafe extern "system" fn n_result_atlas_rgba<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mesh = as_ref::<MeshOutput>(handle);
        vec_to_jbytearray(env, &mesh.atlas.pixels)
    })
}

unsafe extern "system" fn n_result_lod_level<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<MeshOutput>(handle).lod_level as jint)
    })
}

unsafe extern "system" fn n_result_chunk_coord<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jintArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mesh = as_ref::<MeshOutput>(handle);
        match mesh.chunk_coord {
            Some((cx, cy, cz)) => {
                let arr = env
                    .new_int_array(3)
                    .map_err(|e| JvmError::Generic(format!("new_int_array: {e}")))?;
                env.set_int_array_region(&arr, 0, &[cx, cy, cz])
                    .map_err(|e| JvmError::Generic(format!("set_int_array_region: {e}")))?;
                Ok(arr.into_raw())
            }
            None => Ok(std::ptr::null_mut()),
        }
    })
}

// ============================================================================
// MultiMeshResult (HashMap<String, MeshOutput>)
// ============================================================================

/// Newtype so we can `consume::<MultiMeshWrapper>` cleanly.
pub struct MultiMeshWrapper {
    pub regions: Vec<(String, MeshOutput)>,
}

unsafe extern "system" fn n_multi_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<MultiMeshWrapper>(handle);
    }
}

unsafe extern "system" fn n_multi_size<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<MultiMeshWrapper>(handle).regions.len() as jint)
    })
}

unsafe extern "system" fn n_multi_region_names<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let names: Vec<String> = as_ref::<MultiMeshWrapper>(handle)
            .regions
            .iter()
            .map(|(n, _)| n.clone())
            .collect();
        crate::conv::string_vec_to_jarray(env, &names)
    })
}

unsafe extern "system" fn n_multi_get<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    region_name: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let name = jstring_to_string(env, &region_name)?;
        let multi = as_ref::<MultiMeshWrapper>(handle);
        match multi.regions.iter().find(|(n, _)| n == &name) {
            Some((_, mesh)) => Ok(to_handle(mesh.clone())),
            None => Ok(0),
        }
    })
}

unsafe extern "system" fn n_multi_all_handles<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlongArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let multi = as_ref::<MultiMeshWrapper>(handle);
        let handles: Vec<jlong> = multi
            .regions
            .iter()
            .map(|(_, mesh)| to_handle(mesh.clone()))
            .collect();
        let arr = env
            .new_long_array(handles.len() as i32)
            .map_err(|e| JvmError::Generic(format!("new_long_array: {e}")))?;
        env.set_long_array_region(&arr, 0, &handles)
            .map_err(|e| JvmError::Generic(format!("set_long_array_region: {e}")))?;
        Ok(arr.into_raw())
    })
}

// ============================================================================
// Schematic mesh-build entry points
// ============================================================================

unsafe extern "system" fn n_mesh_by_region<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
    pack_handle: jlong,
    config_handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let schematic = as_ref::<UniversalSchematic>(schematic_handle);
        let pack = as_ref::<ResourcePackSource>(pack_handle);
        let default_cfg;
        let config = if config_handle == 0 {
            default_cfg = MeshConfig::default();
            &default_cfg
        } else {
            as_ref::<MeshConfig>(config_handle)
        };
        let map: MultiMeshResult = schematic
            .mesh_by_region(pack, config)
            .map_err(|e| JvmError::Generic(format!("mesh_by_region: {e}")))?;
        let mut regions: Vec<(String, MeshOutput)> = map.into_iter().collect();
        regions.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(to_handle(MultiMeshWrapper { regions }))
    })
}

/// Convenience: mesh the entire schematic by region and merge all regions
/// into a single MeshResult by combining their MeshLayers. Returns 0 if
/// the schematic produced no geometry.
unsafe extern "system" fn n_mesh_single<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
    pack_handle: jlong,
    config_handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let schematic = as_ref::<UniversalSchematic>(schematic_handle);
        let pack = as_ref::<ResourcePackSource>(pack_handle);
        let default_cfg;
        let config = if config_handle == 0 {
            default_cfg = MeshConfig::default();
            &default_cfg
        } else {
            as_ref::<MeshConfig>(config_handle)
        };
        let map: MultiMeshResult = schematic
            .mesh_by_region(pack, config)
            .map_err(|e| JvmError::Generic(format!("mesh: {e}")))?;
        // For now return the first region's mesh — sufficient for single-region
        // schematics (the common case). Multi-region consumers should call
        // meshByRegion() and combine on the Java side.
        let mut sorted: Vec<(String, MeshOutput)> = map.into_iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        match sorted.into_iter().next() {
            Some((_, mesh)) => Ok(to_handle(mesh)),
            None => Ok(0),
        }
    })
}

/// Build an animated GLB replaying a captured scenario. The schematic is the
/// initial state; `timeline_json` is the decoded MCAP event timeline. Returns
/// the GLB bytes directly (no handle — animated GLBs aren't reused).
unsafe extern "system" fn n_mesh_animated<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
    pack_handle: jlong,
    timeline_json: JString<'l>,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let schematic = as_ref::<UniversalSchematic>(schematic_handle);
        let pack = as_ref::<ResourcePackSource>(pack_handle);
        let json = jstring_to_string(env, &timeline_json)?;
        let glb = schematic
            .to_animated_glb(pack, &json)
            .map_err(|e| JvmError::Generic(format!("to_animated_glb: {e}")))?;
        vec_to_jbytearray(env, &glb)
    })
}
