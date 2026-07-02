//! `MchprsWorld` JNI exports (feature-gated).
//!
//! Mirrors `PyMchprsWorld` / the C FFI simulation surface: world creation
//! with options and custom IO nodes, ticking, signal injection/monitoring,
//! lever interaction, and custom-IO change polling.

#![cfg(feature = "simulation")]

use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_mut, as_ref, consume, to_handle};
use jni::objects::{JClass, JIntArray};
use jni::sys::{jboolean, jint, jintArray, jlong, JNI_TRUE};
use jni::{JNIEnv, NativeMethod};
use nucleation::simulation::{BlockPos, MchprsWorld, SimulationOptions};
use nucleation::UniversalSchematic;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nMchprsCreate", "(J)J", n_create as *mut _),
        nm(
            "nMchprsCreateWithOptions",
            "(JZZ[I)J",
            n_create_with_options as *mut _,
        ),
        nm("nMchprsFree", "(J)V", n_free as *mut _),
        nm("nMchprsTick", "(J)V", n_tick as *mut _),
        nm("nMchprsTickMany", "(JI)V", n_tick_many as *mut _),
        nm("nMchprsFlush", "(J)V", n_flush as *mut _),
        nm("nMchprsGetSchematic", "(J)J", n_get_schematic as *mut _),
        nm("nMchprsSyncToSchematic", "(J)V", n_sync_to_schematic as *mut _),
        nm(
            "nMchprsSetSignalStrength",
            "(JIIII)V",
            n_set_signal_strength as *mut _,
        ),
        nm(
            "nMchprsGetSignalStrength",
            "(JIII)I",
            n_get_signal_strength as *mut _,
        ),
        nm("nMchprsSetLeverPower", "(JIIIZ)V", n_set_lever_power as *mut _),
        nm("nMchprsGetLeverPower", "(JIII)Z", n_get_lever_power as *mut _),
        nm("nMchprsIsLit", "(JIII)Z", n_is_lit as *mut _),
        nm("nMchprsOnUseBlock", "(JIII)V", n_on_use_block as *mut _),
        nm(
            "nMchprsGetRedstonePower",
            "(JIII)I",
            n_get_redstone_power as *mut _,
        ),
        nm(
            "nMchprsCheckCustomIoChanges",
            "(J)V",
            n_check_custom_io_changes as *mut _,
        ),
        nm(
            "nMchprsPollCustomIoChanges",
            "(J)[I",
            n_poll_custom_io_changes as *mut _,
        ),
        nm(
            "nMchprsClearCustomIoChanges",
            "(J)V",
            n_clear_custom_io_changes as *mut _,
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

fn pos(x: jint, y: jint, z: jint) -> BlockPos {
    BlockPos::new(x, y, z)
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

unsafe extern "system" fn n_create_with_options<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    schematic_handle: jlong,
    optimize: jboolean,
    io_only: jboolean,
    custom_io: jintArray,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let s = as_ref::<UniversalSchematic>(schematic_handle).clone();

        let mut positions = Vec::new();
        if !custom_io.is_null() {
            let arr = unsafe { JIntArray::from_raw(custom_io) };
            let len = env
                .get_array_length(&arr)
                .map_err(|e| JvmError::Generic(format!("get_array_length: {e}")))?
                as usize;
            if len % 3 != 0 {
                return Err(JvmError::Generic(format!(
                    "customIo length must be a multiple of 3 (x,y,z triples), got {len}"
                )));
            }
            let mut buf = vec![0_i32; len];
            env.get_int_array_region(&arr, 0, &mut buf)
                .map_err(|e| JvmError::Generic(format!("get_int_array_region: {e}")))?;
            for c in buf.chunks_exact(3) {
                positions.push(BlockPos::new(c[0], c[1], c[2]));
            }
        }

        let options = SimulationOptions {
            optimize: optimize == JNI_TRUE,
            io_only: io_only == JNI_TRUE,
            custom_io: positions,
        };
        let world = MchprsWorld::with_options(s, options).map_err(JvmError::Generic)?;
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
        w.tick(1);
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
        w.tick(count.max(0) as u32);
        Ok(())
    })
}

unsafe extern "system" fn n_flush<'l>(mut env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.flush();
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

unsafe extern "system" fn n_sync_to_schematic<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.sync_to_schematic();
        Ok(())
    })
}

unsafe extern "system" fn n_set_signal_strength<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
    strength: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.set_signal_strength(pos(x, y, z), strength.clamp(0, 15) as u8);
        Ok(())
    })
}

unsafe extern "system" fn n_get_signal_strength<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        let w = as_ref::<MchprsWorld>(handle);
        Ok(w.get_signal_strength(pos(x, y, z)) as jint)
    })
}

unsafe extern "system" fn n_set_lever_power<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
    powered: jboolean,
) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.set_lever_power(pos(x, y, z), powered == JNI_TRUE);
        Ok(())
    })
}

unsafe extern "system" fn n_get_lever_power<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jboolean {
    with_jni_context(&mut env, 0, |_env| {
        let w = as_ref::<MchprsWorld>(handle);
        Ok(w.get_lever_power(pos(x, y, z)) as jboolean)
    })
}

unsafe extern "system" fn n_is_lit<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jboolean {
    with_jni_context(&mut env, 0, |_env| {
        let w = as_ref::<MchprsWorld>(handle);
        Ok(w.is_lit(pos(x, y, z)) as jboolean)
    })
}

unsafe extern "system" fn n_on_use_block<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.on_use_block(pos(x, y, z));
        Ok(())
    })
}

unsafe extern "system" fn n_get_redstone_power<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jint {
    with_jni_context(&mut env, 0, |_env| {
        let w = as_ref::<MchprsWorld>(handle);
        Ok(w.get_redstone_power(pos(x, y, z)) as jint)
    })
}

unsafe extern "system" fn n_check_custom_io_changes<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.check_custom_io_changes();
        Ok(())
    })
}

/// Returns pending custom-IO changes flattened as `[x, y, z, oldPower, newPower]*`
/// and clears the pending list (poll semantics).
unsafe extern "system" fn n_poll_custom_io_changes<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jintArray {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let changes = {
            let w = as_mut::<MchprsWorld>(handle);
            w.poll_custom_io_changes()
        };
        let mut flat = Vec::with_capacity(changes.len() * 5);
        for c in &changes {
            flat.extend_from_slice(&[c.x, c.y, c.z, c.old_power as i32, c.new_power as i32]);
        }
        let arr = env
            .new_int_array(flat.len() as i32)
            .map_err(|e| JvmError::Generic(format!("new_int_array: {e}")))?;
        env.set_int_array_region(&arr, 0, &flat)
            .map_err(|e| JvmError::Generic(format!("set_int_array_region: {e}")))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_clear_custom_io_changes<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) {
    with_jni_context(&mut env, (), |_env| {
        let w = as_mut::<MchprsWorld>(handle);
        w.clear_custom_io_changes();
        Ok(())
    })
}
