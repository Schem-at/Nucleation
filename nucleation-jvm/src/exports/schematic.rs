//! `Schematic` JNI exports — mirrors `PySchematic`.
//!
//! Every method here is registered against `NucleationNative` so the Java
//! side calls them as static methods that take the handle as the first arg.
//! This keeps the Java-side public API class (`Schematic`) free of `native`
//! decls and lets `RegisterNatives` collapse all binding into one table.

use crate::conv::{
    hashmap_to_jmap, jbytearray_to_vec, jstring_opt_to_string, jstring_to_string,
    string_to_jstring, string_vec_to_jarray, vec_to_jbytearray,
};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_mut, as_ref, consume, to_handle};
use jni::objects::{JByteArray, JClass, JObject, JString};
use jni::sys::{jboolean, jbyteArray, jint, jintArray, jlong, jobjectArray, jstring};
use jni::{JNIEnv, NativeMethod};
use nucleation::formats::{manager, mcstructure, snapshot};
use nucleation::{
    format_json_schematic, format_schematic, litematic, schematic as schem, BlockState,
    UniversalSchematic,
};
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nSchematicCreate", "(Ljava/lang/String;)J", n_create as *mut _),
        nm("nSchematicOpen", "(Ljava/lang/String;)J", n_open as *mut _),
        nm("nSchematicSave", "(JLjava/lang/String;Ljava/lang/String;)V", n_save as *mut _),
        nm("nSchematicFree", "(J)V", n_free as *mut _),
        nm("nSchematicGetName", "(J)Ljava/lang/String;", n_get_name as *mut _),
        nm("nSchematicSetName", "(JLjava/lang/String;)V", n_set_name as *mut _),
        nm("nSchematicGetDimensions", "(J)[I", n_get_dimensions as *mut _),
        nm("nSchematicGetBlockCount", "(J)I", n_get_block_count as *mut _),
        nm("nSchematicGetVolume", "(J)I", n_get_volume as *mut _),
        nm("nSchematicGetRegionNames", "(J)[Ljava/lang/String;", n_get_region_names as *mut _),
        nm("nSchematicDebugInfo", "(J)Ljava/lang/String;", n_debug_info as *mut _),
        nm("nSchematicPrint", "(J)Ljava/lang/String;", n_print as *mut _),
        nm("nSchematicPrintJson", "(J)Ljava/lang/String;", n_print_json as *mut _),
        nm("nSchematicSetBlockSimple", "(JIIILjava/lang/String;)Z", n_set_block_simple as *mut _),
        nm("nSchematicSetBlockEntity", "(JIIILjava/lang/String;Ljava/lang/String;)Z", n_set_block_entity as *mut _),
        nm("nSchematicSetBlockState", "(JIIIJ)Z", n_set_block_state as *mut _),
        nm("nSchematicSetBlockWithProperties", "(JIIILjava/lang/String;Ljava/util/Map;)Z", n_set_block_with_properties as *mut _),
        nm("nSchematicGetBlock", "(JIII)J", n_get_block as *mut _),
        nm("nSchematicGetBlockName", "(JIII)Ljava/lang/String;", n_get_block_name as *mut _),
        nm("nSchematicFromData", "(J[B)I", n_from_data as *mut _),
        nm("nSchematicFromLitematic", "(J[B)I", n_from_litematic as *mut _),
        nm("nSchematicToLitematic", "(J)[B", n_to_litematic as *mut _),
        nm("nSchematicFromSchematic", "(J[B)I", n_from_schematic as *mut _),
        nm("nSchematicToSchematic", "(J)[B", n_to_schematic as *mut _),
        nm("nSchematicFromMcStructure", "(J[B)I", n_from_mcstructure as *mut _),
        nm("nSchematicToMcStructure", "(J)[B", n_to_mcstructure as *mut _),
        nm("nSchematicToSnapshot", "(J)[B", n_to_snapshot as *mut _),
        nm("nSchematicFromSnapshot", "(J[B)I", n_from_snapshot as *mut _),
        nm("nSchematicGetAllBlocksJson", "(J)Ljava/lang/String;", n_get_all_blocks_json as *mut _),
        nm("nSchematicGetSupportedImportFormats", "()[Ljava/lang/String;", n_get_supported_import_formats as *mut _),
        nm("nSchematicGetSupportedExportFormats", "()[Ljava/lang/String;", n_get_supported_export_formats as *mut _),
        nm("nSchematicCountBlockTypesJson", "(J)Ljava/lang/String;", n_count_block_types_json as *mut _),
        nm("nSchematicCopy", "(J)J", n_copy as *mut _),
        nm("nSchematicFillCuboid", "(JIIIIIILjava/lang/String;)V", n_fill_cuboid as *mut _),
        nm("nSchematicFillSphere", "(JIIIDLjava/lang/String;)V", n_fill_sphere as *mut _),
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
// Native implementations
// ============================================================================

unsafe extern "system" fn n_create<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    name: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let name = jstring_opt_to_string(env, &name)?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Untitled".to_string());
        Ok(to_handle(UniversalSchematic::new(name)))
    })
}

/// Open a schematic from a store URI — a local path, `file://…`, or (when the
/// cdylib is built with the `store-s3` feature) `s3://bucket/key.schem`. The
/// format is auto-detected from the bytes/extension by the core. Returns a new
/// handle, mirroring the static `n_create` factory.
unsafe extern "system" fn n_open<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    uri: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let uri = jstring_to_string(env, &uri)?;
        let schematic = UniversalSchematic::open(&uri)
            .map_err(|e| JvmError::Parse(format!("open '{uri}': {e}")))?;
        Ok(to_handle(schematic))
    })
}

/// Save this schematic to a store URI — a local path, `file://…`, or (with the
/// `store-s3` feature) `s3://bucket/key.schem`. The output format is chosen from
/// the URI's file extension. `version` may be null to use the default data
/// version for the target format.
unsafe extern "system" fn n_save<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    uri: JString<'l>,
    version: JString<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        let uri = jstring_to_string(env, &uri)?;
        let version = jstring_opt_to_string(env, &version)?;
        let s = as_ref::<UniversalSchematic>(handle);
        s.save(&uri, version.as_deref())
            .map_err(|e| JvmError::Generic(format!("save '{uri}': {e}")))?;
        Ok(())
    })
}

unsafe extern "system" fn n_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<UniversalSchematic>(handle);
    }
}

unsafe extern "system" fn n_get_name<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let name = s.metadata.name.clone().unwrap_or_default();
        string_to_jstring(env, &name)
    })
}

unsafe extern "system" fn n_set_name<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        let s = as_mut::<UniversalSchematic>(handle);
        s.metadata.name = Some(jstring_to_string(env, &name)?);
        Ok(())
    })
}

unsafe extern "system" fn n_get_dimensions<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jintArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let (x, y, z) = s.get_dimensions();
        let arr = env
            .new_int_array(3)
            .map_err(|e| JvmError::Generic(format!("new_int_array: {e}")))?;
        env.set_int_array_region(&arr, 0, &[x, y, z])
            .map_err(|e| JvmError::Generic(format!("set_int_array_region: {e}")))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_get_block_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        let s = as_ref::<UniversalSchematic>(handle);
        Ok(s.total_blocks())
    })
}

unsafe extern "system" fn n_get_volume<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        let s = as_ref::<UniversalSchematic>(handle);
        Ok(s.total_volume())
    })
}

unsafe extern "system" fn n_get_region_names<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let names = s.get_region_names();
        string_vec_to_jarray(env, &names)
    })
}

unsafe extern "system" fn n_debug_info<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let info = format!(
            "Schematic name: {}, Regions: {}",
            s.metadata.name.as_deref().unwrap_or("Unnamed"),
            s.other_regions.len() + 1
        );
        string_to_jstring(env, &info)
    })
}

unsafe extern "system" fn n_print<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        string_to_jstring(env, &format_schematic(s))
    })
}

unsafe extern "system" fn n_print_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        string_to_jstring(env, &format_json_schematic(s))
    })
}

unsafe extern "system" fn n_set_block_simple<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
    name: JString<'l>,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |env| {
        let s = as_mut::<UniversalSchematic>(handle);
        let name = jstring_to_string(env, &name)?;
        Ok(s.set_block_str(x, y, z, &name) as u8)
    })
}

/// Set (or replace) a block entity at a position from a typed SNBT string
/// (the block itself must be set separately via setBlock).
unsafe extern "system" fn n_set_block_entity<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
    id: JString<'l>,
    snbt: JString<'l>,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |env| {
        let s = as_mut::<UniversalSchematic>(handle);
        let id_str = jstring_to_string(env, &id)?;
        let snbt_str = jstring_to_string(env, &snbt)?;
        let compound = quartz_nbt::snbt::parse(&snbt_str)
            .map_err(|e| JvmError::Generic(format!("invalid SNBT: {e}")))?;
        let nbt = nucleation::nbt::NbtMap::from_quartz_nbt(&compound);
        let mut be = nucleation::block_entity::BlockEntity::new(id_str, (x, y, z));
        be.set_nbt(nbt);
        s.set_block_entity(nucleation::block_position::BlockPosition { x, y, z }, be);
        Ok(1u8)
    })
}

unsafe extern "system" fn n_set_block_state<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
    state_handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |_env| {
        let s = as_mut::<UniversalSchematic>(handle);
        let state = as_ref::<BlockState>(state_handle);
        Ok(s.set_block(x, y, z, state) as u8)
    })
}

unsafe extern "system" fn n_set_block_with_properties<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
    name: JString<'l>,
    properties: JObject<'l>,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |env| {
        let s = as_mut::<UniversalSchematic>(handle);
        let name = jstring_to_string(env, &name)?;
        let props = crate::conv::map_to_hashmap(env, &properties)?;
        let mut state = BlockState::new(name);
        for (k, v) in props {
            state = state.with_property(k, v);
        }
        Ok(s.set_block(x, y, z, &state) as u8)
    })
}

unsafe extern "system" fn n_get_block<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let s = as_ref::<UniversalSchematic>(handle);
        match s.get_block(x, y, z) {
            Some(state) => Ok(to_handle(state.clone())),
            None => Ok(0),
        }
    })
}

unsafe extern "system" fn n_get_block_name<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        match s.get_block(x, y, z) {
            Some(state) => string_to_jstring(env, &state.name),
            None => Ok(std::ptr::null_mut()),
        }
    })
}

unsafe extern "system" fn n_from_data<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    data: JByteArray<'l>,
) -> jint {
    with_jni_context(&mut env, -1, |env| {
        let bytes = jbytearray_to_vec(env, &data)?;
        let mgr_arc = manager::get_manager();
        let mgr = mgr_arc
            .lock()
            .map_err(|_| JvmError::Generic("format manager poisoned".into()))?;
        let new = mgr
            .read(&bytes)
            .map_err(|e| JvmError::Parse(format!("from_data: {e}")))?;
        let s = as_mut::<UniversalSchematic>(handle);
        *s = new;
        Ok(0)
    })
}

unsafe extern "system" fn n_from_litematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    data: JByteArray<'l>,
) -> jint {
    with_jni_context(&mut env, -1, |env| {
        let bytes = jbytearray_to_vec(env, &data)?;
        let new = litematic::from_litematic(&bytes)
            .map_err(|e| JvmError::Parse(format!("from_litematic: {e}")))?;
        let s = as_mut::<UniversalSchematic>(handle);
        *s = new;
        Ok(0)
    })
}

unsafe extern "system" fn n_to_litematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let bytes = litematic::to_litematic(s)
            .map_err(|e| JvmError::Generic(format!("to_litematic: {e}")))?;
        vec_to_jbytearray(env, &bytes)
    })
}

unsafe extern "system" fn n_from_schematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    data: JByteArray<'l>,
) -> jint {
    with_jni_context(&mut env, -1, |env| {
        let bytes = jbytearray_to_vec(env, &data)?;
        let new = schem::from_schematic(&bytes)
            .map_err(|e| JvmError::Parse(format!("from_schematic: {e}")))?;
        let s = as_mut::<UniversalSchematic>(handle);
        *s = new;
        Ok(0)
    })
}

unsafe extern "system" fn n_to_schematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let bytes = schem::to_schematic(s)
            .map_err(|e| JvmError::Generic(format!("to_schematic: {e}")))?;
        vec_to_jbytearray(env, &bytes)
    })
}

unsafe extern "system" fn n_from_mcstructure<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    data: JByteArray<'l>,
) -> jint {
    with_jni_context(&mut env, -1, |env| {
        let bytes = jbytearray_to_vec(env, &data)?;
        let new = mcstructure::from_mcstructure(&bytes)
            .map_err(|e| JvmError::Parse(format!("from_mcstructure: {e}")))?;
        let s = as_mut::<UniversalSchematic>(handle);
        *s = new;
        Ok(0)
    })
}

unsafe extern "system" fn n_to_mcstructure<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let bytes = mcstructure::to_mcstructure(s)
            .map_err(|e| JvmError::Generic(format!("to_mcstructure: {e}")))?;
        vec_to_jbytearray(env, &bytes)
    })
}

unsafe extern "system" fn n_to_snapshot<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let bytes = snapshot::to_snapshot(s)
            .map_err(|e| JvmError::Generic(format!("to_snapshot: {e}")))?;
        vec_to_jbytearray(env, &bytes)
    })
}

unsafe extern "system" fn n_from_snapshot<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    data: JByteArray<'l>,
) -> jint {
    with_jni_context(&mut env, -1, |env| {
        let bytes = jbytearray_to_vec(env, &data)?;
        let new = snapshot::from_snapshot(&bytes)
            .map_err(|e| JvmError::Parse(format!("from_snapshot: {e}")))?;
        let s = as_mut::<UniversalSchematic>(handle);
        *s = new;
        Ok(0)
    })
}

unsafe extern "system" fn n_get_all_blocks_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        // Hand-rolled output guarantees deterministic field order so the
        // Java-side regex parser does not need to handle every permutation.
        let mut out = String::from("[");
        let mut first = true;
        for (pos, block) in s.iter_blocks() {
            // Skip air-like blocks to match the Python `get_all_blocks` surface.
            if block.name.ends_with(":air")
                || block.name == "air"
                || block.name.ends_with("cave_air")
                || block.name.ends_with("void_air")
            {
                continue;
            }
            if !first {
                out.push(',');
            }
            first = false;
            let name = serde_json::to_string(block.name.as_str())
                .map_err(|e| JvmError::Generic(format!("json name: {e}")))?;
            let props: std::collections::BTreeMap<&str, &str> = block
                .properties
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let props_json = serde_json::to_string(&props)
                .map_err(|e| JvmError::Generic(format!("json props: {e}")))?;
            out.push_str(&format!(
                r#"{{"x":{},"y":{},"z":{},"name":{},"properties":{}}}"#,
                pos.x, pos.y, pos.z, name, props_json
            ));
        }
        out.push(']');
        string_to_jstring(env, &out)
    })
}

unsafe extern "system" fn n_get_supported_import_formats<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let formats: Vec<String> = vec![
            "litematic".into(),
            "schem".into(),
            "schematic".into(),
            "mcstructure".into(),
            "nucm".into(),
            "snapshot".into(),
        ];
        string_vec_to_jarray(env, &formats)
    })
}

unsafe extern "system" fn n_get_supported_export_formats<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let formats: Vec<String> = vec![
            "litematic".into(),
            "schem".into(),
            "schematic".into(),
            "mcstructure".into(),
            "nucm".into(),
            "snapshot".into(),
        ];
        string_vec_to_jarray(env, &formats)
    })
}

unsafe extern "system" fn n_count_block_types_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<UniversalSchematic>(handle);
        let counts = s.count_block_types();
        let mut out = std::collections::HashMap::new();
        for (state, count) in counts {
            *out.entry(state.name.to_string()).or_insert(0usize) += count;
        }
        let json = serde_json::to_string(&out)
            .map_err(|e| JvmError::Generic(format!("json: {e}")))?;
        string_to_jstring(env, &json)
    })
}

unsafe extern "system" fn n_copy<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let s = as_ref::<UniversalSchematic>(handle);
        Ok(to_handle(s.clone()))
    })
}

unsafe extern "system" fn n_fill_cuboid<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x1: jint,
    y1: jint,
    z1: jint,
    x2: jint,
    y2: jint,
    z2: jint,
    name: JString<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        let s = as_mut::<UniversalSchematic>(handle);
        let name = jstring_to_string(env, &name)?;
        let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        let (min_z, max_z) = if z1 <= z2 { (z1, z2) } else { (z2, z1) };
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    s.set_block_str(x, y, z, &name);
                }
            }
        }
        Ok(())
    })
}

unsafe extern "system" fn n_fill_sphere<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    cx: jint,
    cy: jint,
    cz: jint,
    radius: f64,
    name: JString<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        let s = as_mut::<UniversalSchematic>(handle);
        let name = jstring_to_string(env, &name)?;
        let r = radius.max(0.0);
        let r_ceil = r.ceil() as i32;
        let r_sq = r * r;
        for dx in -r_ceil..=r_ceil {
            for dy in -r_ceil..=r_ceil {
                for dz in -r_ceil..=r_ceil {
                    let d = (dx * dx + dy * dy + dz * dz) as f64;
                    if d <= r_sq {
                        s.set_block_str(cx + dx, cy + dy, cz + dz, &name);
                    }
                }
            }
        }
        Ok(())
    })
}

#[allow(dead_code)]
fn _silence(_: &JObject) {}

#[allow(dead_code)]
fn _silence_hashmap_helper<'a>(
    env: &mut JNIEnv<'a>,
    m: &std::collections::HashMap<String, String>,
) -> Result<JObject<'a>, JvmError> {
    hashmap_to_jmap(env, m)
}
