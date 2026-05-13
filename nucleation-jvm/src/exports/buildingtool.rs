//! `BuildingTool` JNI exports — mirrors `PyBuildingTool`.
//!
//! Because `BuildingTool<'a>` borrows the schematic, we cannot store one in a
//! heap handle (lifetime escapes). Instead, every call rents the schematic
//! from its handle, constructs a `BuildingTool` for the duration of the call,
//! performs the operation, and drops it.

use crate::errors::with_jni_context;
use crate::handles::{as_mut, as_ref};
use jni::objects::JClass;
use jni::sys::{jint, jlong};
use jni::{JNIEnv, NativeMethod};
use nucleation::building::{BrushEnum, BuildingTool, ShapeEnum};
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nBuildingFill", "(JJJ)V", n_fill as *mut _),
        nm("nBuildingRstack", "(JJJIIII)V", n_rstack as *mut _),
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

unsafe extern "system" fn n_fill<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
    shape_handle: jlong,
    brush_handle: jlong,
) {
    with_jni_context(&mut env, (), |_env| {
        let s = as_mut::<UniversalSchematic>(schematic_handle);
        let shape = as_ref::<ShapeEnum>(shape_handle);
        let brush = as_ref::<BrushEnum>(brush_handle);
        let mut tool = BuildingTool::new(s);
        tool.fill_enum(shape, brush);
        Ok(())
    })
}

unsafe extern "system" fn n_rstack<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
    shape_handle: jlong,
    brush_handle: jlong,
    count: jint,
    dx: jint,
    dy: jint,
    dz: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        let s = as_mut::<UniversalSchematic>(schematic_handle);
        let shape = as_ref::<ShapeEnum>(shape_handle);
        let brush = as_ref::<BrushEnum>(brush_handle);
        let mut tool = BuildingTool::new(s);
        tool.rstack(shape, brush, count.max(0) as usize, (dx, dy, dz));
        Ok(())
    })
}
