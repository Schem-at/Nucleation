//! Conversions between JNI types and Rust types.

use crate::errors::JvmError;
use jni::objects::{JByteArray, JObject, JObjectArray, JString, JValue};
use jni::sys::{jbyteArray, jobjectArray, jstring};
use jni::JNIEnv;
use std::collections::HashMap;

pub fn jstring_to_string(env: &mut JNIEnv, s: &JString) -> Result<String, JvmError> {
    let java_str = env
        .get_string(s)
        .map_err(|e| JvmError::Generic(format!("JString conversion failed: {e}")))?;
    Ok(java_str.into())
}

pub fn jstring_opt_to_string(env: &mut JNIEnv, s: &JString) -> Result<Option<String>, JvmError> {
    if s.is_null() {
        Ok(None)
    } else {
        jstring_to_string(env, s).map(Some)
    }
}

pub fn string_to_jstring(env: &mut JNIEnv, s: &str) -> Result<jstring, JvmError> {
    env.new_string(s)
        .map(|js| js.into_raw())
        .map_err(|e| JvmError::Generic(format!("new_string failed: {e}")))
}

pub fn jbytearray_to_vec(env: &mut JNIEnv, a: &JByteArray) -> Result<Vec<u8>, JvmError> {
    let len = env
        .get_array_length(a)
        .map_err(|e| JvmError::Generic(format!("array length: {e}")))?;
    let mut buf = vec![0i8; len as usize];
    env.get_byte_array_region(a, 0, &mut buf)
        .map_err(|e| JvmError::Generic(format!("array copy: {e}")))?;
    Ok(buf.into_iter().map(|b| b as u8).collect())
}

pub fn vec_to_jbytearray(env: &mut JNIEnv, data: &[u8]) -> Result<jbyteArray, JvmError> {
    let arr = env
        .new_byte_array(data.len() as i32)
        .map_err(|e| JvmError::Generic(format!("new_byte_array: {e}")))?;
    let signed: Vec<i8> = data.iter().map(|&b| b as i8).collect();
    env.set_byte_array_region(&arr, 0, &signed)
        .map_err(|e| JvmError::Generic(format!("set_byte_array_region: {e}")))?;
    Ok(arr.into_raw())
}

pub fn string_vec_to_jarray(env: &mut JNIEnv, items: &[String]) -> Result<jobjectArray, JvmError> {
    let class = env
        .find_class("java/lang/String")
        .map_err(|e| JvmError::Generic(format!("find_class String: {e}")))?;
    let empty = env
        .new_string("")
        .map_err(|e| JvmError::Generic(format!("new_string: {e}")))?;
    let arr = env
        .new_object_array(items.len() as i32, &class, &empty)
        .map_err(|e| JvmError::Generic(format!("new_object_array: {e}")))?;
    for (i, item) in items.iter().enumerate() {
        let js = env
            .new_string(item)
            .map_err(|e| JvmError::Generic(format!("new_string item: {e}")))?;
        env.set_object_array_element(&arr, i as i32, js)
            .map_err(|e| JvmError::Generic(format!("set_object_array_element: {e}")))?;
    }
    Ok(arr.into_raw())
}

/// Convert a Java `Map<String, String>` (`JObject`) into a `HashMap`.
pub fn map_to_hashmap(
    env: &mut JNIEnv,
    map: &JObject,
) -> Result<HashMap<String, String>, JvmError> {
    let mut out = HashMap::new();
    if map.is_null() {
        return Ok(out);
    }
    let entry_set = env
        .call_method(map, "entrySet", "()Ljava/util/Set;", &[])
        .map_err(|e| JvmError::Generic(format!("Map.entrySet: {e}")))?
        .l()
        .map_err(|e| JvmError::Generic(format!("entrySet l(): {e}")))?;
    let iterator = env
        .call_method(&entry_set, "iterator", "()Ljava/util/Iterator;", &[])
        .map_err(|e| JvmError::Generic(format!("Set.iterator: {e}")))?
        .l()
        .map_err(|e| JvmError::Generic(format!("iterator l(): {e}")))?;
    loop {
        let has_next = env
            .call_method(&iterator, "hasNext", "()Z", &[])
            .map_err(|e| JvmError::Generic(format!("hasNext: {e}")))?
            .z()
            .map_err(|e| JvmError::Generic(format!("hasNext z(): {e}")))?;
        if !has_next {
            break;
        }
        let entry = env
            .call_method(&iterator, "next", "()Ljava/lang/Object;", &[])
            .map_err(|e| JvmError::Generic(format!("next: {e}")))?
            .l()
            .map_err(|e| JvmError::Generic(format!("next l(): {e}")))?;
        let k_obj = env
            .call_method(&entry, "getKey", "()Ljava/lang/Object;", &[])
            .map_err(|e| JvmError::Generic(format!("getKey: {e}")))?
            .l()
            .map_err(|e| JvmError::Generic(format!("getKey l(): {e}")))?;
        let v_obj = env
            .call_method(&entry, "getValue", "()Ljava/lang/Object;", &[])
            .map_err(|e| JvmError::Generic(format!("getValue: {e}")))?
            .l()
            .map_err(|e| JvmError::Generic(format!("getValue l(): {e}")))?;
        let k_js: JString = JString::from(k_obj);
        let v_js: JString = JString::from(v_obj);
        let k = jstring_to_string(env, &k_js)?;
        let v = jstring_to_string(env, &v_js)?;
        out.insert(k, v);
    }
    Ok(out)
}

/// Build a Java `HashMap<String, String>` from a Rust `HashMap`.
pub fn hashmap_to_jmap<'a>(
    env: &mut JNIEnv<'a>,
    map: &HashMap<String, String>,
) -> Result<JObject<'a>, JvmError> {
    let hashmap_class = env
        .find_class("java/util/HashMap")
        .map_err(|e| JvmError::Generic(format!("find_class HashMap: {e}")))?;
    let jmap = env
        .new_object(&hashmap_class, "()V", &[])
        .map_err(|e| JvmError::Generic(format!("new HashMap: {e}")))?;
    for (k, v) in map.iter() {
        let jk = env
            .new_string(k)
            .map_err(|e| JvmError::Generic(format!("new_string k: {e}")))?;
        let jv = env
            .new_string(v)
            .map_err(|e| JvmError::Generic(format!("new_string v: {e}")))?;
        env.call_method(
            &jmap,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
            &[JValue::Object(&jk.into()), JValue::Object(&jv.into())],
        )
        .map_err(|e| JvmError::Generic(format!("HashMap.put: {e}")))?;
    }
    Ok(jmap)
}

#[allow(dead_code)]
pub fn unused_jobjectarray_silencer(_a: &JObjectArray) {}
