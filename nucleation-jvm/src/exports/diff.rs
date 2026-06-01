//! Diff JNI exports — mirrors `nucleation::diff`.
//!
//! `Diff` is exposed as an opaque `jlong` handle, allocated with `to_handle`
//! and released with `nDiffFree` (mirroring the schematic handle scheme). The
//! `added/removed/changed/swapped/markers` accessors each materialize a fresh
//! `UniversalSchematic` and return it as its own handle, so the Java side frees
//! them with the existing `Schematic.close()` path.
//!
//! Surface scope:
//! - `diff(handleA, handleB, preset)` -> Diff handle
//! - `diffWithOverrides(handleA, handleB, preset, ...)` -> Diff handle
//! - `Diff` accessors: distance, support, toJson, summaryJson
//! - `fromJson(json)` -> Diff handle
//! - `added/removed/changed/swapped/markers` -> schematic handle
//! - meshing-gated `toOverlayGlb(diffHandle, afterGlb)` -> `byte[]`

use crate::conv::{jstring_to_string, string_to_jstring};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_ref, consume, to_handle};
use jni::objects::{JClass, JString};
use jni::sys::{jfloat, jint, jlong, jstring};
use jni::{JNIEnv, NativeMethod};
use nucleation::diff::{diff, Diff, DiffSpec, SpecOverrides};
use nucleation::fingerprint::symmetry::Symmetry;
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nDiff", "(JJLjava/lang/String;)J", n_diff as *mut _),
        nm(
            "nDiffWithOverrides",
            "(JJLjava/lang/String;IIIILjava/lang/String;)J",
            n_diff_with_overrides as *mut _,
        ),
        nm("nDiffFree", "(J)V", n_diff_free as *mut _),
        nm("nDiffDistance", "(J)I", n_diff_distance as *mut _),
        nm("nDiffSupport", "(J)F", n_diff_support as *mut _),
        nm("nDiffToJson", "(J)Ljava/lang/String;", n_diff_to_json as *mut _),
        nm("nDiffSummaryJson", "(J)Ljava/lang/String;", n_diff_summary_json as *mut _),
        nm("nDiffFromJson", "(Ljava/lang/String;)J", n_diff_from_json as *mut _),
        nm("nDiffAdded", "(J)J", n_diff_added as *mut _),
        nm("nDiffRemoved", "(J)J", n_diff_removed as *mut _),
        nm("nDiffChanged", "(J)J", n_diff_changed as *mut _),
        nm("nDiffSwapped", "(J)J", n_diff_swapped as *mut _),
        nm("nDiffMarkers", "(J)J", n_diff_markers as *mut _),
        #[cfg(feature = "meshing")]
        nm("nDiffToOverlayGlb", "(J[B)[B", n_diff_to_overlay_glb as *mut _),
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

/// Resolve a diff spec from preset + overrides, throwing on an unknown name.
fn resolve_spec(
    env: &mut JNIEnv,
    preset: &JString,
    ov: SpecOverrides,
) -> Result<DiffSpec, JvmError> {
    let name = jstring_to_string(env, preset)?;
    DiffSpec::resolve(&name, &ov)
        .ok_or_else(|| JvmError::Generic(format!("unknown diff preset '{name}'")))
}

// ============================================================================
// Diff construction
// ============================================================================

unsafe extern "system" fn n_diff<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle_a: jlong,
    handle_b: jlong,
    preset: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let spec = resolve_spec(env, &preset, SpecOverrides::default())?;
        let a = as_ref::<UniversalSchematic>(handle_a);
        let b = as_ref::<UniversalSchematic>(handle_b);
        Ok(to_handle(diff(a, b, &spec)))
    })
}

/// Diff with explicit cost / symmetry overrides. Each `cost_*` arg uses a
/// negative value to mean "leave the preset default"; non-negative values are
/// applied. `symmetry` may be null to keep the preset's symmetry group, else it
/// must name a valid group (none, yaw, yaw_mirror, octahedral, octahedral_full).
unsafe extern "system" fn n_diff_with_overrides<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle_a: jlong,
    handle_b: jlong,
    preset: JString<'l>,
    cost_add: jint,
    cost_delete: jint,
    cost_change: jint,
    cost_swap: jint,
    symmetry: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let opt_cost = |v: jint| -> Option<u32> {
            if v < 0 {
                None
            } else {
                Some(v as u32)
            }
        };
        let sym = if symmetry.is_null() {
            None
        } else {
            let name = jstring_to_string(env, &symmetry)?;
            Some(
                Symmetry::from_name(&name)
                    .ok_or_else(|| JvmError::Generic(format!("unknown symmetry '{name}'")))?,
            )
        };
        let ov = SpecOverrides {
            cost_add: opt_cost(cost_add),
            cost_delete: opt_cost(cost_delete),
            cost_change: opt_cost(cost_change),
            cost_swap: opt_cost(cost_swap),
            symmetry: sym,
        };
        let spec = resolve_spec(env, &preset, ov)?;
        let a = as_ref::<UniversalSchematic>(handle_a);
        let b = as_ref::<UniversalSchematic>(handle_b);
        Ok(to_handle(diff(a, b, &spec)))
    })
}

unsafe extern "system" fn n_diff_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<Diff>(handle);
    }
}

// ============================================================================
// Diff accessors
// ============================================================================

unsafe extern "system" fn n_diff_distance<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<Diff>(handle).distance as jint)
    })
}

unsafe extern "system" fn n_diff_support<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jfloat {
    with_jni_context(&mut env, 0.0f32, |_env| Ok(as_ref::<Diff>(handle).support))
}

unsafe extern "system" fn n_diff_to_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let json = as_ref::<Diff>(handle).to_json();
        string_to_jstring(env, &json)
    })
}

unsafe extern "system" fn n_diff_summary_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let json = as_ref::<Diff>(handle).summary_json();
        string_to_jstring(env, &json)
    })
}

unsafe extern "system" fn n_diff_from_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    json: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let s = jstring_to_string(env, &json)?;
        let d = Diff::from_json(&s).map_err(|e| JvmError::Parse(format!("Diff::from_json: {e}")))?;
        Ok(to_handle(d))
    })
}

unsafe extern "system" fn n_diff_added<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(as_ref::<Diff>(handle).added()))
    })
}

unsafe extern "system" fn n_diff_removed<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(as_ref::<Diff>(handle).removed()))
    })
}

unsafe extern "system" fn n_diff_changed<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(as_ref::<Diff>(handle).changed()))
    })
}

unsafe extern "system" fn n_diff_swapped<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(as_ref::<Diff>(handle).swapped()))
    })
}

unsafe extern "system" fn n_diff_markers<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(as_ref::<Diff>(handle).markers()))
    })
}

// ============================================================================
// Meshing-gated overlay
// ============================================================================

#[cfg(feature = "meshing")]
unsafe extern "system" fn n_diff_to_overlay_glb<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    after_glb: jni::objects::JByteArray<'l>,
) -> jni::sys::jbyteArray {
    use crate::conv::{jbytearray_to_vec, vec_to_jbytearray};
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let after = jbytearray_to_vec(env, &after_glb)?;
        let d = as_ref::<Diff>(handle);
        let opts = nucleation::diff::OverlayOptions::default();
        let glb = d
            .to_overlay_glb(&after, &opts)
            .map_err(|e| JvmError::Generic(format!("to_overlay_glb: {e}")))?;
        vec_to_jbytearray(env, &glb)
    })
}
