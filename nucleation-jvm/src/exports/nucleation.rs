//! Free-function JNI exports (loaders, version helpers).

use crate::conv::string_to_jstring;
use crate::errors::with_jni_context;
use jni::objects::JClass;
use jni::sys::jstring;
use jni::{JNIEnv, NativeMethod};
use nucleation::{format_json_schematic, format_schematic};
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nDebugSchematic", "(J)Ljava/lang/String;", n_debug_schematic as *mut _),
        nm("nDebugJsonSchematic", "(J)Ljava/lang/String;", n_debug_json_schematic as *mut _),
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

unsafe extern "system" fn n_debug_schematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jni::sys::jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = crate::handles::as_ref::<nucleation::UniversalSchematic>(handle);
        string_to_jstring(env, &format_schematic(s))
    })
}

unsafe extern "system" fn n_debug_json_schematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jni::sys::jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = crate::handles::as_ref::<nucleation::UniversalSchematic>(handle);
        string_to_jstring(env, &format_json_schematic(s))
    })
}
