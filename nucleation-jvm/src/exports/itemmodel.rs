//! Item-model JNI exports (feature-gated with `meshing`).
//!
//! Mirrors the C-FFI `itemmodel_*` surface: `Schematic.to_item_model` plus
//! `ItemModelResult` accessors and resource-pack packaging. The config is
//! passed as discrete parameters (no native config handle needed).

#![cfg(feature = "meshing")]

use crate::conv::{jstring_to_string, string_to_jstring, vec_to_jbytearray};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_ref, consume, to_handle};
use jni::objects::{JClass, JLongArray, JString};
use jni::sys::{jboolean, jbyteArray, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, jstring, JNI_TRUE};
use jni::{JNIEnv, NativeMethod};
use nucleation::meshing::{
    build_resource_pack, ItemModelConfig, ItemModelResult, ItemModelScale, ResourcePackSource,
};
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm(
            "nSchematicToItemModel",
            "(JJLjava/lang/String;Ljava/lang/String;ZILjava/lang/String;Ljava/lang/String;IFFF)J",
            n_to_item_model as *mut _,
        ),
        nm("nItemModelResultFree", "(J)V", n_result_free as *mut _),
        nm(
            "nItemModelResultModelJson",
            "(J)Ljava/lang/String;",
            n_result_model_json as *mut _,
        ),
        nm("nItemModelResultElementCount", "(J)I", n_result_element_count as *mut _),
        nm("nItemModelResultTextureCount", "(J)I", n_result_texture_count as *mut _),
        nm("nItemModelResultPlaneCount", "(J)I", n_result_plane_count as *mut _),
        nm("nItemModelResultDimensions", "(J)[I", n_result_dimensions as *mut _),
        nm("nItemModelResultScale", "(J)[F", n_result_scale as *mut _),
        nm(
            "nItemModelResultToResourcePackZip",
            "(J)[B",
            n_result_to_resource_pack_zip as *mut _,
        ),
        nm(
            "nItemModelBuildResourcePack",
            "([J)[B",
            n_build_resource_pack as *mut _,
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

#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn n_to_item_model<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
    pack_handle: jlong,
    model_name: JString<'l>,
    namespace: JString<'l>,
    center: jboolean,
    texture_resolution: jint,
    item: JString<'l>,
    custom_model_data: JString<'l>,
    scale_mode: jint,
    sx: jfloat,
    sy: jfloat,
    sz: jfloat,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let schematic = as_ref::<UniversalSchematic>(schematic_handle);
        let pack = as_ref::<ResourcePackSource>(pack_handle);

        let scale = match scale_mode {
            1 => ItemModelScale::Uniform(sx),
            2 => ItemModelScale::NonUniform(sx, sy, sz),
            _ => ItemModelScale::Auto,
        };
        let config = ItemModelConfig::new(jstring_to_string(env, &model_name)?)
            .with_namespace(jstring_to_string(env, &namespace)?)
            .with_center(center == JNI_TRUE)
            .with_texture_resolution(texture_resolution.max(1) as u32)
            .with_item(jstring_to_string(env, &item)?)
            .with_custom_model_data(jstring_to_string(env, &custom_model_data)?)
            .with_scale(scale);

        let result = schematic
            .to_item_model(pack, &config)
            .map_err(|e| JvmError::Generic(e.to_string()))?;
        Ok(to_handle(result))
    })
}

unsafe extern "system" fn n_result_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<ItemModelResult>(handle);
    }
}

unsafe extern "system" fn n_result_model_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let r = as_ref::<ItemModelResult>(handle);
        string_to_jstring(env, &r.model_json)
    })
}

unsafe extern "system" fn n_result_element_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<ItemModelResult>(handle).stats.element_count as jint)
    })
}

unsafe extern "system" fn n_result_texture_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<ItemModelResult>(handle).stats.texture_count as jint)
    })
}

unsafe extern "system" fn n_result_plane_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<ItemModelResult>(handle).stats.plane_count as jint)
    })
}

unsafe extern "system" fn n_result_dimensions<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jintArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let (w, h, d) = as_ref::<ItemModelResult>(handle).stats.dimensions;
        let arr = env
            .new_int_array(3)
            .map_err(|e| JvmError::Generic(format!("new_int_array: {e}")))?;
        env.set_int_array_region(&arr, 0, &[w, h, d])
            .map_err(|e| JvmError::Generic(format!("set_int_array_region: {e}")))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_result_scale<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jfloatArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let (sx, sy, sz) = as_ref::<ItemModelResult>(handle).stats.scale;
        let arr = env
            .new_float_array(3)
            .map_err(|e| JvmError::Generic(format!("new_float_array: {e}")))?;
        env.set_float_array_region(&arr, 0, &[sx, sy, sz])
            .map_err(|e| JvmError::Generic(format!("set_float_array_region: {e}")))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_result_to_resource_pack_zip<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let r = as_ref::<ItemModelResult>(handle);
        let zip = r
            .to_resource_pack_zip()
            .map_err(|e| JvmError::Generic(e.to_string()))?;
        vec_to_jbytearray(env, &zip)
    })
}

unsafe extern "system" fn n_build_resource_pack<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handles: jlongArray,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let arr = unsafe { JLongArray::from_raw(handles) };
        let len = env
            .get_array_length(&arr)
            .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))? as usize;
        let mut buf = vec![0_i64; len];
        env.get_long_array_region(&arr, 0, &mut buf)
            .map_err(|e| JvmError::Generic(format!("get_long_array_region: {e}")))?;

        let results: Vec<&ItemModelResult> = buf
            .iter()
            .map(|&h| unsafe { as_ref::<ItemModelResult>(h) })
            .collect();
        let zip = build_resource_pack(&results).map_err(|e| JvmError::Generic(e.to_string()))?;
        vec_to_jbytearray(env, &zip)
    })
}
