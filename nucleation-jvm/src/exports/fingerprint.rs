//! Fingerprint JNI exports — mirrors `nucleation::fingerprint`.
//!
//! Every method here is registered against `NucleationNative` so the Java
//! side calls them as static methods that take the schematic handle(s) as the
//! leading args. All entry points run inside `with_jni_context` so an unknown
//! preset is surfaced as a `NucleationException` on the Java side.
//!
//! Surface scope:
//! - `fingerprint(handle, preset)` -> hex `String`
//! - `signature(handle, preset)` -> JSON `String`
//! - `footprintDistance(handleA, handleB, preset)` -> `float`
//! - `isDuplicateOf(handleA, handleB, preset)` -> `boolean`

use crate::conv::{jstring_to_string, string_to_jstring};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::as_ref;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jfloat, jlong, jstring};
use jni::{JNIEnv, NativeMethod};
use nucleation::fingerprint::{
    fingerprint, footprint_distance, is_duplicate, signature, FingerprintSpec,
};
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nFingerprint", "(JLjava/lang/String;)Ljava/lang/String;", n_fingerprint as *mut _),
        nm("nSignature", "(JLjava/lang/String;)Ljava/lang/String;", n_signature as *mut _),
        nm("nFootprintDistance", "(JJLjava/lang/String;)F", n_footprint_distance as *mut _),
        nm("nIsDuplicateOf", "(JJLjava/lang/String;)Z", n_is_duplicate as *mut _),
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

/// Resolve a fingerprint preset, throwing on an unknown name.
fn resolve_spec(env: &mut JNIEnv, preset: &JString) -> Result<FingerprintSpec, JvmError> {
    let name = jstring_to_string(env, preset)?;
    FingerprintSpec::from_preset(&name)
        .ok_or_else(|| JvmError::Generic(format!("unknown fingerprint preset '{name}'")))
}

// ============================================================================
// Native implementations
// ============================================================================

unsafe extern "system" fn n_fingerprint<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    preset: JString<'l>,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let spec = resolve_spec(env, &preset)?;
        let s = as_ref::<UniversalSchematic>(handle);
        let hex = fingerprint(s, &spec).to_hex();
        string_to_jstring(env, &hex)
    })
}

unsafe extern "system" fn n_signature<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    preset: JString<'l>,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let spec = resolve_spec(env, &preset)?;
        let s = as_ref::<UniversalSchematic>(handle);
        let json = signature(s, &spec).to_json();
        string_to_jstring(env, &json)
    })
}

unsafe extern "system" fn n_footprint_distance<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle_a: jlong,
    handle_b: jlong,
    preset: JString<'l>,
) -> jfloat {
    with_jni_context(&mut env, 0.0f32, |env| {
        let spec = resolve_spec(env, &preset)?;
        let a = as_ref::<UniversalSchematic>(handle_a);
        let b = as_ref::<UniversalSchematic>(handle_b);
        Ok(footprint_distance(a, b, &spec))
    })
}

unsafe extern "system" fn n_is_duplicate<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle_a: jlong,
    handle_b: jlong,
    preset: JString<'l>,
) -> jboolean {
    with_jni_context(&mut env, 0u8, |env| {
        let spec = resolve_spec(env, &preset)?;
        let a = as_ref::<UniversalSchematic>(handle_a);
        let b = as_ref::<UniversalSchematic>(handle_b);
        Ok(is_duplicate(a, b, &spec) as u8)
    })
}
