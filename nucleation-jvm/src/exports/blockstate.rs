//! `BlockState` JNI exports — mirrors `PyBlockState`.

use crate::conv::{hashmap_to_jmap, jstring_to_string, string_to_jstring};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_ref, consume, to_handle};
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jlong, jobject, jstring};
use jni::{JNIEnv, NativeMethod};
use nucleation::BlockState;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nBlockStateCreate", "(Ljava/lang/String;)J", n_create as *mut _),
        nm("nBlockStateFree", "(J)V", n_free as *mut _),
        nm("nBlockStateGetName", "(J)Ljava/lang/String;", n_get_name as *mut _),
        nm("nBlockStateWithProperty",
           "(JLjava/lang/String;Ljava/lang/String;)J",
           n_with_property as *mut _),
        nm("nBlockStateGetProperties", "(J)Ljava/util/Map;", n_get_properties as *mut _),
        nm("nBlockStateToString", "(J)Ljava/lang/String;", n_to_string as *mut _),
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

unsafe extern "system" fn n_create<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    name: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let name = jstring_to_string(env, &name)?;
        Ok(to_handle(BlockState::new(name)))
    })
}

unsafe extern "system" fn n_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<BlockState>(handle);
    }
}

unsafe extern "system" fn n_get_name<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<BlockState>(handle);
        string_to_jstring(env, &s.name)
    })
}

unsafe extern "system" fn n_with_property<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    key: JString<'l>,
    value: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let s = as_ref::<BlockState>(handle);
        let k = jstring_to_string(env, &key)?;
        let v = jstring_to_string(env, &value)?;
        Ok(to_handle(s.clone().with_property(k, v)))
    })
}

unsafe extern "system" fn n_get_properties<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobject {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<BlockState>(handle);
        let map = s
            .properties
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let jmap = hashmap_to_jmap(env, &map)?;
        Ok(jmap.into_raw())
    })
}

unsafe extern "system" fn n_to_string<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<BlockState>(handle);
        let mut text = s.name.to_string();
        if !s.properties.is_empty() {
            let mut parts: Vec<String> = s
                .properties
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.sort();
            text.push('[');
            text.push_str(&parts.join(","));
            text.push(']');
        }
        string_to_jstring(env, &text)
    })
}

#[allow(dead_code)]
fn _silence(_: &JObject) {}
