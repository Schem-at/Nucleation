//! SDF generation JNI exports (always available — the sdf module is core).

use crate::errors::{with_jni_context, JvmError};
use crate::handles::to_handle;
use jni::objects::{JClass, JIntArray, JString};
use jni::sys::{jfloat, jintArray, jlong};
use jni::{JNIEnv, NativeMethod};
use nucleation::sdf::{sample_to_schematic, MaterialRules, SampleBounds, SdfNode};
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm(
            "nSchematicFromSdf",
            "(Ljava/lang/String;Ljava/lang/String;[I)J",
            n_from_sdf as *mut _,
        ),
        nm("nSdfEval", "(Ljava/lang/String;FFF)F", n_sdf_eval as *mut _),
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

fn jstr(env: &mut JNIEnv, s: &JString) -> Result<String, JvmError> {
    crate::conv::jstring_to_string(env, s)
}

unsafe extern "system" fn n_from_sdf<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    sdf_json: JString<'l>,
    rules_json: JString<'l>,
    bounds: jintArray,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let sdf_str = jstr(env, &sdf_json)?;
        let rules_str = jstr(env, &rules_json)?;

        let node = SdfNode::from_json(&sdf_str).map_err(JvmError::Generic)?;
        let rules = MaterialRules::from_json(&rules_str).map_err(JvmError::Generic)?;

        let bounds = if bounds.is_null() {
            None
        } else {
            let arr = unsafe { JIntArray::from_raw(bounds) };
            let len = env
                .get_array_length(&arr)
                .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))?;
            if len != 6 {
                return Err(JvmError::Generic(format!(
                    "bounds must be [minX, minY, minZ, maxX, maxY, maxZ], got length {len}"
                )));
            }
            let mut buf = [0_i32; 6];
            env.get_int_array_region(&arr, 0, &mut buf)
                .map_err(|e| JvmError::Generic(format!("get_int_array_region: {e}")))?;
            Some(SampleBounds {
                min: [buf[0], buf[1], buf[2]],
                max: [buf[3], buf[4], buf[5]],
            })
        };

        let schematic =
            sample_to_schematic(&node, &rules, bounds, "sdf").map_err(JvmError::Generic)?;
        Ok(to_handle(schematic))
    })
}

unsafe extern "system" fn n_sdf_eval<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    sdf_json: JString<'l>,
    x: jfloat,
    y: jfloat,
    z: jfloat,
) -> jfloat {
    with_jni_context(&mut env, f32::NAN, |env| {
        let sdf_str = jstr(env, &sdf_json)?;
        let node = SdfNode::from_json(&sdf_str).map_err(JvmError::Generic)?;
        Ok(node.eval(x, y, z))
    })
}
