//! Typed circuit executor JNI exports (feature-gated with `simulation`).
//!
//! Mirrors the Python `CircuitBuilder` / `TypedCircuitExecutor` surface:
//! named, typed circuit IO (Value / IoType / LayoutFunction / DefinitionRegion),
//! fluent circuit building (explicit layout, auto-inference, or insign signs),
//! and typed execution with fixed-tick / until-change / until-stable modes.

#![cfg(feature = "simulation")]

use crate::conv::{jstring_to_string, string_to_jstring, string_vec_to_jarray};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_mut, as_ref, consume, to_handle};
use jni::objects::{JBooleanArray, JByteArray, JClass, JIntArray, JLongArray, JObjectArray, JString};
use jni::sys::{
    jboolean, jbooleanArray, jbyteArray, jfloat, jint, jintArray, jlong, jlongArray, jobjectArray,
    jstring, JNI_TRUE,
};
use jni::{JNIEnv, NativeMethod};
use nucleation::definition_region::DefinitionRegion;
use nucleation::simulation::typed_executor::{
    ExecutionMode, ExecutionResult, IoType, LayoutFunction, StateMode, TypedCircuitExecutor, Value,
};
use nucleation::simulation::{CircuitBuilder, SimulationOptions};
use nucleation::UniversalSchematic;
use std::collections::HashMap;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        // ── Value ────────────────────────────────────────────────────────
        nm("nValueU32", "(I)J", n_value_u32 as *mut _),
        nm("nValueU64", "(J)J", n_value_u64 as *mut _),
        nm("nValueI32", "(I)J", n_value_i32 as *mut _),
        nm("nValueI64", "(J)J", n_value_i64 as *mut _),
        nm("nValueF32", "(F)J", n_value_f32 as *mut _),
        nm("nValueBool", "(Z)J", n_value_bool as *mut _),
        nm("nValueString", "(Ljava/lang/String;)J", n_value_string as *mut _),
        nm("nValueBits", "([Z)J", n_value_bits as *mut _),
        nm("nValueBytes", "([B)J", n_value_bytes as *mut _),
        nm("nValueTypeName", "(J)Ljava/lang/String;", n_value_type_name as *mut _),
        nm("nValueAsI64", "(J)J", n_value_as_i64 as *mut _),
        nm("nValueAsF32", "(J)F", n_value_as_f32 as *mut _),
        nm("nValueAsBool", "(J)Z", n_value_as_bool as *mut _),
        nm("nValueAsString", "(J)Ljava/lang/String;", n_value_as_string as *mut _),
        nm("nValueAsBits", "(J)[Z", n_value_as_bits as *mut _),
        nm("nValueAsBytes", "(J)[B", n_value_as_bytes as *mut _),
        nm("nValueDebug", "(J)Ljava/lang/String;", n_value_debug as *mut _),
        nm("nValueFree", "(J)V", n_value_free as *mut _),
        // ── IoType ───────────────────────────────────────────────────────
        nm("nIoTypeUnsignedInt", "(I)J", n_iotype_unsigned_int as *mut _),
        nm("nIoTypeSignedInt", "(I)J", n_iotype_signed_int as *mut _),
        nm("nIoTypeFloat32", "()J", n_iotype_float32 as *mut _),
        nm("nIoTypeBoolean", "()J", n_iotype_boolean as *mut _),
        nm("nIoTypeAscii", "(I)J", n_iotype_ascii as *mut _),
        nm("nIoTypeBitArray", "(I)J", n_iotype_bit_array as *mut _),
        nm("nIoTypeBitCount", "(J)I", n_iotype_bit_count as *mut _),
        nm("nIoTypeFree", "(J)V", n_iotype_free as *mut _),
        // ── LayoutFunction ───────────────────────────────────────────────
        nm("nLayoutOneToOne", "()J", n_layout_one_to_one as *mut _),
        nm("nLayoutPacked4", "()J", n_layout_packed4 as *mut _),
        nm("nLayoutCustom", "([I)J", n_layout_custom as *mut _),
        nm("nLayoutRowMajor", "(III)J", n_layout_row_major as *mut _),
        nm("nLayoutColumnMajor", "(III)J", n_layout_column_major as *mut _),
        nm("nLayoutScanline", "(III)J", n_layout_scanline as *mut _),
        nm("nLayoutFree", "(J)V", n_layout_free as *mut _),
        // ── DefinitionRegion ─────────────────────────────────────────────
        nm("nRegionFromPositions", "([I)J", n_region_from_positions as *mut _),
        nm("nRegionFromBounds", "(IIIIII)J", n_region_from_bounds as *mut _),
        nm("nRegionAddPoint", "(JIII)V", n_region_add_point as *mut _),
        nm("nRegionAddBounds", "(JIIIIII)V", n_region_add_bounds as *mut _),
        nm("nRegionVolume", "(J)J", n_region_volume as *mut _),
        nm("nRegionFree", "(J)V", n_region_free as *mut _),
        // ── CircuitBuilder ───────────────────────────────────────────────
        nm("nCircuitBuilderNew", "(J)J", n_builder_new as *mut _),
        nm("nCircuitBuilderFromInsign", "(J)J", n_builder_from_insign as *mut _),
        nm(
            "nCircuitBuilderWithInput",
            "(JLjava/lang/String;JJJ)V",
            n_builder_with_input as *mut _,
        ),
        nm(
            "nCircuitBuilderWithInputAuto",
            "(JLjava/lang/String;JJ)V",
            n_builder_with_input_auto as *mut _,
        ),
        nm(
            "nCircuitBuilderWithOutput",
            "(JLjava/lang/String;JJJ)V",
            n_builder_with_output as *mut _,
        ),
        nm(
            "nCircuitBuilderWithOutputAuto",
            "(JLjava/lang/String;JJ)V",
            n_builder_with_output_auto as *mut _,
        ),
        nm("nCircuitBuilderWithOptions", "(JZZ)V", n_builder_with_options as *mut _),
        nm("nCircuitBuilderWithStateMode", "(JI)V", n_builder_with_state_mode as *mut _),
        nm(
            "nCircuitBuilderValidate",
            "(J)Ljava/lang/String;",
            n_builder_validate as *mut _,
        ),
        nm("nCircuitBuilderBuild", "(J)J", n_builder_build as *mut _),
        nm("nCircuitBuilderInputCount", "(J)I", n_builder_input_count as *mut _),
        nm("nCircuitBuilderOutputCount", "(J)I", n_builder_output_count as *mut _),
        nm(
            "nCircuitBuilderInputNames",
            "(J)[Ljava/lang/String;",
            n_builder_input_names as *mut _,
        ),
        nm(
            "nCircuitBuilderOutputNames",
            "(J)[Ljava/lang/String;",
            n_builder_output_names as *mut _,
        ),
        nm("nCircuitBuilderFree", "(J)V", n_builder_free as *mut _),
        // ── TypedCircuitExecutor ─────────────────────────────────────────
        nm(
            "nExecutorSetInput",
            "(JLjava/lang/String;J)V",
            n_executor_set_input as *mut _,
        ),
        nm(
            "nExecutorReadOutput",
            "(JLjava/lang/String;)J",
            n_executor_read_output as *mut _,
        ),
        nm(
            "nExecutorExecute",
            "(J[Ljava/lang/String;[JIII)J",
            n_executor_execute as *mut _,
        ),
        nm("nExecutorTick", "(JI)V", n_executor_tick as *mut _),
        nm("nExecutorFlush", "(J)V", n_executor_flush as *mut _),
        nm("nExecutorReset", "(J)V", n_executor_reset as *mut _),
        nm(
            "nExecutorInputNames",
            "(J)[Ljava/lang/String;",
            n_executor_input_names as *mut _,
        ),
        nm(
            "nExecutorOutputNames",
            "(J)[Ljava/lang/String;",
            n_executor_output_names as *mut _,
        ),
        nm("nExecutorSetStateMode", "(JI)V", n_executor_set_state_mode as *mut _),
        nm(
            "nExecutorLayoutInfoJson",
            "(J)Ljava/lang/String;",
            n_executor_layout_info_json as *mut _,
        ),
        nm(
            "nExecutorSyncAndGetSchematic",
            "(J)J",
            n_executor_sync_and_get_schematic as *mut _,
        ),
        nm("nExecutorSerialize", "(J)[B", n_executor_serialize as *mut _),
        nm("nExecutorFromCompiled", "([B)J", n_executor_from_compiled as *mut _),
        nm("nExecutorFree", "(J)V", n_executor_free as *mut _),
        // ── ExecutionResult ──────────────────────────────────────────────
        nm("nExecResultTicks", "(J)I", n_result_ticks as *mut _),
        nm("nExecResultConditionMet", "(J)Z", n_result_condition_met as *mut _),
        nm(
            "nExecResultOutputNames",
            "(J)[Ljava/lang/String;",
            n_result_output_names as *mut _,
        ),
        nm(
            "nExecResultOutput",
            "(JLjava/lang/String;)J",
            n_result_output as *mut _,
        ),
        nm("nExecResultFree", "(J)V", n_result_free as *mut _),
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

// ════════════════════════════════════════════════════════════════════════
//  Value
// ════════════════════════════════════════════════════════════════════════

macro_rules! value_ctor {
    ($fn_name:ident, $jty:ty, $variant:expr) => {
        unsafe extern "system" fn $fn_name<'l>(
            mut env: JNIEnv<'l>,
            _class: JClass<'l>,
            v: $jty,
        ) -> jlong {
            with_jni_context(&mut env, 0, |_env| Ok(to_handle($variant(v))))
        }
    };
}

value_ctor!(n_value_u32, jint, |v: jint| Value::U32(v as u32));
value_ctor!(n_value_u64, jlong, |v: jlong| Value::U64(v as u64));
value_ctor!(n_value_i32, jint, |v: jint| Value::I32(v));
value_ctor!(n_value_i64, jlong, |v: jlong| Value::I64(v));
value_ctor!(n_value_f32, jfloat, |v: jfloat| Value::F32(v));
value_ctor!(n_value_bool, jboolean, |v: jboolean| Value::Bool(v == JNI_TRUE));

unsafe extern "system" fn n_value_string<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    s: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        Ok(to_handle(Value::String(jstring_to_string(env, &s)?)))
    })
}

unsafe extern "system" fn n_value_bits<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    bits: jbooleanArray,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let arr = unsafe { JBooleanArray::from_raw(bits) };
        let len = env
            .get_array_length(&arr)
            .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))? as usize;
        let mut buf = vec![0_u8; len];
        env.get_boolean_array_region(&arr, 0, &mut buf)
            .map_err(|e| JvmError::Generic(format!("get_boolean_array_region: {e}")))?;
        Ok(to_handle(Value::BitArray(
            buf.into_iter().map(|b| b != 0).collect(),
        )))
    })
}

unsafe extern "system" fn n_value_bytes<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    bytes: jbyteArray,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let arr = unsafe { JByteArray::from_raw(bytes) };
        let data = crate::conv::jbytearray_to_vec(env, &arr)?;
        Ok(to_handle(Value::Bytes(data)))
    })
}

unsafe extern "system" fn n_value_type_name<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let name = match as_ref::<Value>(handle) {
            Value::U32(_) => "U32",
            Value::U64(_) => "U64",
            Value::I32(_) => "I32",
            Value::I64(_) => "I64",
            Value::F32(_) => "F32",
            Value::Bool(_) => "Bool",
            Value::String(_) => "String",
            Value::BitArray(_) => "BitArray",
            Value::Bytes(_) => "Bytes",
            Value::Array(_) => "Array",
            Value::Struct(_) => "Struct",
        };
        string_to_jstring(env, name)
    })
}

unsafe extern "system" fn n_value_as_i64<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let v = as_ref::<Value>(handle);
        v.as_i64()
            .or_else(|_| v.as_u64().map(|u| u as i64))
            .map_err(JvmError::Generic)
    })
}

unsafe extern "system" fn n_value_as_f32<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jfloat {
    with_jni_context(&mut env, 0.0, |_env| {
        as_ref::<Value>(handle).as_f32().map_err(JvmError::Generic)
    })
}

unsafe extern "system" fn n_value_as_bool<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0, |_env| {
        as_ref::<Value>(handle)
            .as_bool()
            .map(|b| b as jboolean)
            .map_err(JvmError::Generic)
    })
}

unsafe extern "system" fn n_value_as_string<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<Value>(handle)
            .as_str()
            .map_err(JvmError::Generic)?
            .to_string();
        string_to_jstring(env, &s)
    })
}

unsafe extern "system" fn n_value_as_bits<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbooleanArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let bits = as_ref::<Value>(handle)
            .as_bit_array()
            .map_err(JvmError::Generic)?;
        let buf: Vec<jboolean> = bits.iter().map(|&b| b as jboolean).collect();
        let arr = env
            .new_boolean_array(buf.len() as i32)
            .map_err(|e| JvmError::Generic(format!("new_boolean_array: {e}")))?;
        env.set_boolean_array_region(&arr, 0, &buf)
            .map_err(|e| JvmError::Generic(format!("set_boolean_array_region: {e}")))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_value_as_bytes<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let bytes = as_ref::<Value>(handle)
            .as_bytes()
            .map_err(JvmError::Generic)?
            .clone();
        crate::conv::vec_to_jbytearray(env, &bytes)
    })
}

unsafe extern "system" fn n_value_debug<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        string_to_jstring(env, &format!("{:?}", as_ref::<Value>(handle)))
    })
}

unsafe extern "system" fn n_value_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<Value>(handle);
    }
}

// ════════════════════════════════════════════════════════════════════════
//  IoType
// ════════════════════════════════════════════════════════════════════════

unsafe extern "system" fn n_iotype_unsigned_int<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    bits: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(IoType::UnsignedInt {
            bits: bits.max(1) as usize,
        }))
    })
}

unsafe extern "system" fn n_iotype_signed_int<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    bits: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(IoType::SignedInt {
            bits: bits.max(1) as usize,
        }))
    })
}

unsafe extern "system" fn n_iotype_float32<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>) -> jlong {
    with_jni_context(&mut env, 0, |_env| Ok(to_handle(IoType::Float32)))
}

unsafe extern "system" fn n_iotype_boolean<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>) -> jlong {
    with_jni_context(&mut env, 0, |_env| Ok(to_handle(IoType::Boolean)))
}

unsafe extern "system" fn n_iotype_ascii<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    chars: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(IoType::Ascii {
            chars: chars.max(1) as usize,
        }))
    })
}

unsafe extern "system" fn n_iotype_bit_array<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    bits: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(IoType::BitArray {
            bits: bits.max(1) as usize,
        }))
    })
}

unsafe extern "system" fn n_iotype_bit_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<IoType>(handle).bit_count() as jint)
    })
}

unsafe extern "system" fn n_iotype_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<IoType>(handle);
    }
}

// ════════════════════════════════════════════════════════════════════════
//  LayoutFunction
// ════════════════════════════════════════════════════════════════════════

unsafe extern "system" fn n_layout_one_to_one<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>) -> jlong {
    with_jni_context(&mut env, 0, |_env| Ok(to_handle(LayoutFunction::OneToOne)))
}

unsafe extern "system" fn n_layout_packed4<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>) -> jlong {
    with_jni_context(&mut env, 0, |_env| Ok(to_handle(LayoutFunction::Packed4)))
}

unsafe extern "system" fn n_layout_custom<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    mapping: jintArray,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let arr = unsafe { JIntArray::from_raw(mapping) };
        let len = env
            .get_array_length(&arr)
            .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))? as usize;
        let mut buf = vec![0_i32; len];
        env.get_int_array_region(&arr, 0, &mut buf)
            .map_err(|e| JvmError::Generic(format!("get_int_array_region: {e}")))?;
        Ok(to_handle(LayoutFunction::Custom(
            buf.into_iter().map(|v| v.max(0) as usize).collect(),
        )))
    })
}

macro_rules! layout3 {
    ($fn_name:ident, $variant:ident, $a:ident, $b:ident, $c:ident) => {
        unsafe extern "system" fn $fn_name<'l>(
            mut env: JNIEnv<'l>,
            _class: JClass<'l>,
            $a: jint,
            $b: jint,
            $c: jint,
        ) -> jlong {
            with_jni_context(&mut env, 0, |_env| {
                Ok(to_handle(LayoutFunction::$variant {
                    $a: $a.max(0) as usize,
                    $b: $b.max(0) as usize,
                    $c: $c.max(0) as usize,
                }))
            })
        }
    };
}

layout3!(n_layout_row_major, RowMajor, rows, cols, bits_per_element);
layout3!(n_layout_column_major, ColumnMajor, rows, cols, bits_per_element);
layout3!(n_layout_scanline, Scanline, width, height, bits_per_pixel);

unsafe extern "system" fn n_layout_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<LayoutFunction>(handle);
    }
}

// ════════════════════════════════════════════════════════════════════════
//  DefinitionRegion
// ════════════════════════════════════════════════════════════════════════

unsafe extern "system" fn n_region_from_positions<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    flat: jintArray,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let arr = unsafe { JIntArray::from_raw(flat) };
        let len = env
            .get_array_length(&arr)
            .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))? as usize;
        if len % 3 != 0 {
            return Err(JvmError::Generic(format!(
                "positions length must be a multiple of 3 (x,y,z triples), got {len}"
            )));
        }
        let mut buf = vec![0_i32; len];
        env.get_int_array_region(&arr, 0, &mut buf)
            .map_err(|e| JvmError::Generic(format!("get_int_array_region: {e}")))?;
        let positions: Vec<(i32, i32, i32)> =
            buf.chunks_exact(3).map(|c| (c[0], c[1], c[2])).collect();
        Ok(to_handle(DefinitionRegion::from_positions(&positions)))
    })
}

unsafe extern "system" fn n_region_from_bounds<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    min_x: jint,
    min_y: jint,
    min_z: jint,
    max_x: jint,
    max_y: jint,
    max_z: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(DefinitionRegion::from_bounds(
            (min_x, min_y, min_z),
            (max_x, max_y, max_z),
        )))
    })
}

unsafe extern "system" fn n_region_add_point<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<DefinitionRegion>(handle).add_point(x, y, z);
        Ok(())
    })
}

unsafe extern "system" fn n_region_add_bounds<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    min_x: jint,
    min_y: jint,
    min_z: jint,
    max_x: jint,
    max_y: jint,
    max_z: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<DefinitionRegion>(handle).add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
        Ok(())
    })
}

unsafe extern "system" fn n_region_volume<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<DefinitionRegion>(handle).volume() as jlong)
    })
}

unsafe extern "system" fn n_region_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<DefinitionRegion>(handle);
    }
}

// ════════════════════════════════════════════════════════════════════════
//  CircuitBuilder — handle wraps Option<CircuitBuilder> (consumed by build)
// ════════════════════════════════════════════════════════════════════════

type BuilderSlot = Option<CircuitBuilder>;

fn take_builder(handle: jlong) -> Result<CircuitBuilder, JvmError> {
    unsafe { as_mut::<BuilderSlot>(handle) }.take().ok_or_else(|| {
        JvmError::Generic("CircuitBuilder has already been consumed (build() called?)".into())
    })
}

fn put_builder(handle: jlong, builder: CircuitBuilder) {
    *unsafe { as_mut::<BuilderSlot>(handle) } = Some(builder);
}

fn with_builder<R>(
    handle: jlong,
    f: impl FnOnce(&CircuitBuilder) -> R,
) -> Result<R, JvmError> {
    let slot = unsafe { as_ref::<BuilderSlot>(handle) };
    let b = slot.as_ref().ok_or_else(|| {
        JvmError::Generic("CircuitBuilder has already been consumed (build() called?)".into())
    })?;
    Ok(f(b))
}

unsafe extern "system" fn n_builder_new<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let s = as_ref::<UniversalSchematic>(schematic_handle).clone();
        Ok(to_handle::<BuilderSlot>(Some(CircuitBuilder::new(s))))
    })
}

unsafe extern "system" fn n_builder_from_insign<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let s = as_ref::<UniversalSchematic>(schematic_handle).clone();
        let builder = CircuitBuilder::from_insign(s).map_err(JvmError::Generic)?;
        Ok(to_handle::<BuilderSlot>(Some(builder)))
    })
}

unsafe extern "system" fn n_builder_with_input<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
    io_type: jlong,
    layout: jlong,
    region: jlong,
) {
    with_jni_context(&mut env, (), |env| {
        let name = jstring_to_string(env, &name)?;
        let builder = take_builder(handle)?;
        let result = builder.with_input(
            name,
            as_ref::<IoType>(io_type).clone(),
            as_ref::<LayoutFunction>(layout).clone(),
            as_ref::<DefinitionRegion>(region).clone(),
        );
        match result {
            Ok(b) => {
                put_builder(handle, b);
                Ok(())
            }
            Err(e) => Err(JvmError::Generic(e)),
        }
    })
}

unsafe extern "system" fn n_builder_with_input_auto<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
    io_type: jlong,
    region: jlong,
) {
    with_jni_context(&mut env, (), |env| {
        let name = jstring_to_string(env, &name)?;
        let builder = take_builder(handle)?;
        let result = builder.with_input_auto(
            name,
            as_ref::<IoType>(io_type).clone(),
            as_ref::<DefinitionRegion>(region).clone(),
        );
        match result {
            Ok(b) => {
                put_builder(handle, b);
                Ok(())
            }
            Err(e) => Err(JvmError::Generic(e)),
        }
    })
}

unsafe extern "system" fn n_builder_with_output<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
    io_type: jlong,
    layout: jlong,
    region: jlong,
) {
    with_jni_context(&mut env, (), |env| {
        let name = jstring_to_string(env, &name)?;
        let builder = take_builder(handle)?;
        let result = builder.with_output(
            name,
            as_ref::<IoType>(io_type).clone(),
            as_ref::<LayoutFunction>(layout).clone(),
            as_ref::<DefinitionRegion>(region).clone(),
        );
        match result {
            Ok(b) => {
                put_builder(handle, b);
                Ok(())
            }
            Err(e) => Err(JvmError::Generic(e)),
        }
    })
}

unsafe extern "system" fn n_builder_with_output_auto<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
    io_type: jlong,
    region: jlong,
) {
    with_jni_context(&mut env, (), |env| {
        let name = jstring_to_string(env, &name)?;
        let builder = take_builder(handle)?;
        let result = builder.with_output_auto(
            name,
            as_ref::<IoType>(io_type).clone(),
            as_ref::<DefinitionRegion>(region).clone(),
        );
        match result {
            Ok(b) => {
                put_builder(handle, b);
                Ok(())
            }
            Err(e) => Err(JvmError::Generic(e)),
        }
    })
}

unsafe extern "system" fn n_builder_with_options<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    optimize: jboolean,
    io_only: jboolean,
) {
    with_jni_context(&mut env, (), |_env| {
        let builder = take_builder(handle)?;
        let b = builder.with_options(SimulationOptions {
            optimize: optimize == JNI_TRUE,
            io_only: io_only == JNI_TRUE,
            custom_io: Vec::new(),
        });
        put_builder(handle, b);
        Ok(())
    })
}

fn state_mode_from_code(code: jint) -> Result<StateMode, JvmError> {
    match code {
        0 => Ok(StateMode::Stateless),
        1 => Ok(StateMode::Stateful),
        2 => Ok(StateMode::Manual),
        _ => Err(JvmError::Generic(format!("unknown StateMode code: {code}"))),
    }
}

unsafe extern "system" fn n_builder_with_state_mode<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    mode: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        let mode = state_mode_from_code(mode)?;
        let builder = take_builder(handle)?;
        put_builder(handle, builder.with_state_mode(mode));
        Ok(())
    })
}

/// Returns null when valid, otherwise the validation error message.
unsafe extern "system" fn n_builder_validate<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let message = with_builder(handle, |b| b.validate().err().map(|e| e.to_string()))?;
        match message {
            Some(msg) => string_to_jstring(env, &msg),
            None => Ok(std::ptr::null_mut()),
        }
    })
}

unsafe extern "system" fn n_builder_build<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let builder = take_builder(handle)?;
        let executor = builder.build().map_err(JvmError::Generic)?;
        Ok(to_handle(executor))
    })
}

unsafe extern "system" fn n_builder_input_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        with_builder(handle, |b| b.input_count() as jint)
    })
}

unsafe extern "system" fn n_builder_output_count<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        with_builder(handle, |b| b.output_count() as jint)
    })
}

unsafe extern "system" fn n_builder_input_names<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mut names: Vec<String> =
            with_builder(handle, |b| b.input_names().iter().map(|s| s.to_string()).collect())?;
        names.sort();
        string_vec_to_jarray(env, &names)
    })
}

unsafe extern "system" fn n_builder_output_names<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mut names: Vec<String> =
            with_builder(handle, |b| b.output_names().iter().map(|s| s.to_string()).collect())?;
        names.sort();
        string_vec_to_jarray(env, &names)
    })
}

unsafe extern "system" fn n_builder_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<BuilderSlot>(handle);
    }
}

// ════════════════════════════════════════════════════════════════════════
//  TypedCircuitExecutor
// ════════════════════════════════════════════════════════════════════════

unsafe extern "system" fn n_executor_set_input<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
    value_handle: jlong,
) {
    with_jni_context(&mut env, (), |env| {
        let name = jstring_to_string(env, &name)?;
        let value = as_ref::<Value>(value_handle).clone();
        as_mut::<TypedCircuitExecutor>(handle)
            .set_input(&name, &value)
            .map_err(JvmError::Generic)
    })
}

unsafe extern "system" fn n_executor_read_output<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let name = jstring_to_string(env, &name)?;
        let value = as_mut::<TypedCircuitExecutor>(handle)
            .read_output(&name)
            .map_err(JvmError::Generic)?;
        Ok(to_handle(value))
    })
}

/// mode: 0 = FixedTicks(p1), 1 = UntilChange(max=p1, interval=p2),
/// 2 = UntilStable(stable=p1, max=p2). Returns an ExecutionResult handle.
unsafe extern "system" fn n_executor_execute<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    names: jobjectArray,
    value_handles: jlongArray,
    mode: jint,
    p1: jint,
    p2: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let names_arr = unsafe { JObjectArray::from_raw(names) };
        let handles_arr = unsafe { JLongArray::from_raw(value_handles) };
        let n = env
            .get_array_length(&names_arr)
            .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))? as usize;
        let hn = env
            .get_array_length(&handles_arr)
            .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))? as usize;
        if n != hn {
            return Err(JvmError::Generic(format!(
                "input name/value count mismatch: {n} names, {hn} values"
            )));
        }
        let mut handle_buf = vec![0_i64; n];
        env.get_long_array_region(&handles_arr, 0, &mut handle_buf)
            .map_err(|e| JvmError::Generic(format!("get_long_array_region: {e}")))?;

        let mut inputs: HashMap<String, Value> = HashMap::with_capacity(n);
        for (i, &vh) in handle_buf.iter().enumerate() {
            let obj = env
                .get_object_array_element(&names_arr, i as i32)
                .map_err(|e| JvmError::Generic(format!("get_object_array_element: {e}")))?;
            let name = jstring_to_string(env, &JString::from(obj))?;
            inputs.insert(name, as_ref::<Value>(vh).clone());
        }

        let mode = match mode {
            0 => ExecutionMode::FixedTicks {
                ticks: p1.max(0) as u32,
            },
            1 => ExecutionMode::UntilChange {
                max_ticks: p1.max(0) as u32,
                check_interval: p2.max(1) as u32,
            },
            2 => ExecutionMode::UntilStable {
                stable_ticks: p1.max(1) as u32,
                max_ticks: p2.max(0) as u32,
            },
            other => return Err(JvmError::Generic(format!("unknown execution mode: {other}"))),
        };

        let result = as_mut::<TypedCircuitExecutor>(handle)
            .execute(inputs, mode)
            .map_err(JvmError::Generic)?;
        Ok(to_handle(result))
    })
}

unsafe extern "system" fn n_executor_tick<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    ticks: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<TypedCircuitExecutor>(handle).tick(ticks.max(0) as u32);
        Ok(())
    })
}

unsafe extern "system" fn n_executor_flush<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<TypedCircuitExecutor>(handle).flush();
        Ok(())
    })
}

unsafe extern "system" fn n_executor_reset<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    with_jni_context(&mut env, (), |_env| {
        as_mut::<TypedCircuitExecutor>(handle)
            .reset()
            .map_err(JvmError::Generic)
    })
}

unsafe extern "system" fn n_executor_input_names<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mut names: Vec<String> = as_ref::<TypedCircuitExecutor>(handle)
            .input_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        names.sort();
        string_vec_to_jarray(env, &names)
    })
}

unsafe extern "system" fn n_executor_output_names<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mut names: Vec<String> = as_ref::<TypedCircuitExecutor>(handle)
            .output_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        names.sort();
        string_vec_to_jarray(env, &names)
    })
}

unsafe extern "system" fn n_executor_set_state_mode<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    mode: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        let mode = state_mode_from_code(mode)?;
        as_mut::<TypedCircuitExecutor>(handle).set_state_mode(mode);
        Ok(())
    })
}

unsafe extern "system" fn n_executor_layout_info_json<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let info = as_ref::<TypedCircuitExecutor>(handle).get_layout_info();
        let side = |m: &HashMap<String, nucleation::simulation::typed_executor::IoLayoutInfo>| {
            let mut obj = serde_json::Map::new();
            for (name, io) in m {
                obj.insert(
                    name.clone(),
                    serde_json::json!({
                        "ioType": io.io_type,
                        "bitCount": io.bit_count,
                        "positions": io.positions.iter().map(|&(x, y, z)| vec![x, y, z]).collect::<Vec<_>>(),
                    }),
                );
            }
            serde_json::Value::Object(obj)
        };
        let json = serde_json::json!({
            "inputs": side(&info.inputs),
            "outputs": side(&info.outputs),
        });
        string_to_jstring(env, &json.to_string())
    })
}

unsafe extern "system" fn n_executor_sync_and_get_schematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let schematic = as_mut::<TypedCircuitExecutor>(handle)
            .sync_and_get_schematic()
            .clone();
        Ok(to_handle(schematic))
    })
}

unsafe extern "system" fn n_executor_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<TypedCircuitExecutor>(handle);
    }
}

/// Portable compiled-circuit container: one blob that restores an executor
/// without re-running insign extraction or layout building (the redpiler
/// compile still runs — see nucleation's typed_executor::compiled docs).
unsafe extern "system" fn n_executor_serialize<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jni::sys::jbyteArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let bytes = as_ref::<TypedCircuitExecutor>(handle)
            .to_compiled_bytes()
            .map_err(JvmError::Generic)?;
        let arr = env
            .byte_array_from_slice(&bytes)
            .map_err(|e| JvmError::Generic(format!("byte array: {}", e)))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_executor_from_compiled<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    data: jni::objects::JByteArray<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let bytes = env
            .convert_byte_array(&data)
            .map_err(|e| JvmError::Generic(format!("byte array: {}", e)))?;
        let executor =
            TypedCircuitExecutor::from_compiled_bytes(&bytes).map_err(JvmError::Generic)?;
        Ok(to_handle(executor))
    })
}

// ════════════════════════════════════════════════════════════════════════
//  ExecutionResult
// ════════════════════════════════════════════════════════════════════════

unsafe extern "system" fn n_result_ticks<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<ExecutionResult>(handle).ticks_elapsed as jint)
    })
}

unsafe extern "system" fn n_result_condition_met<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jboolean {
    with_jni_context(&mut env, 0, |_env| {
        Ok(as_ref::<ExecutionResult>(handle).condition_met as jboolean)
    })
}

unsafe extern "system" fn n_result_output_names<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jobjectArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let mut names: Vec<String> = as_ref::<ExecutionResult>(handle)
            .outputs
            .keys()
            .cloned()
            .collect();
        names.sort();
        string_vec_to_jarray(env, &names)
    })
}

unsafe extern "system" fn n_result_output<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let name = jstring_to_string(env, &name)?;
        let value = as_ref::<ExecutionResult>(handle)
            .outputs
            .get(&name)
            .cloned()
            .ok_or_else(|| JvmError::Generic(format!("Unknown output: {name}")))?;
        Ok(to_handle(value))
    })
}

unsafe extern "system" fn n_result_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<ExecutionResult>(handle);
    }
}
