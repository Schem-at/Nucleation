//! `MchprsWorld` JNI exports (feature-gated).
//!
//! Mirrors a subset of `PyMchprsWorld` for in-game redstone simulation.

#![cfg(feature = "simulation")]

use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_mut, as_ref, consume, to_handle};
use jni::objects::JClass;
use jni::sys::{jint, jlong};
use jni::{JNIEnv, NativeMethod};
use nucleation::simulation::MchprsWorld;
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nMchprsCreate", "(J)J", n_create as *mut _),
        nm("nMchprsFree", "(J)V", n_free as *mut _),
        nm("nMchprsTick", "(J)V", n_tick as *mut _),
        nm("nMchprsTickMany", "(JI)V", n_tick_many as *mut _),
        nm("nMchprsGetSchematic", "(J)J", n_get_schematic as *mut _),
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
    schematic_handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let s = as_ref::<UniversalSchematic>(schematic_handle).clone();
        let world = MchprsWorld::new(s).map_err(JvmError::Generic)?;
        Ok(to_handle(world))
    })
}

unsafe extern "system" fn n_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<MchprsWorld>(handle);
    }
}

unsafe extern "system" fn n_tick<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.tick();
        Ok(())
    })
}

unsafe extern "system" fn n_tick_many<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    count: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        for _ in 0..count.max(0) {
            w.tick();
        }
        Ok(())
    })
}

unsafe extern "system" fn n_get_schematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let w = as_ref::<MchprsWorld>(handle);
        let schematic = w.get_schematic().clone();
        Ok(to_handle(schematic))
    })
}
