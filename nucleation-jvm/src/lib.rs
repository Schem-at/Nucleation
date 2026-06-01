//! JNI bindings for Nucleation.
//!
//! All native methods are bound via `RegisterNatives` from `JNI_OnLoad` so the
//! Java class name and package can be refactored without rebuilding the cdylib.
//!
//! Layering:
//! - `handles`: opaque pointer ↔ `jlong` with panic-safe deref
//! - `errors`: panic capture + JVM exception throwing
//! - `conv`: `JString` ↔ `String`, `jbyteArray` ↔ `Vec<u8>`, etc.
//! - `exports/*`: one file per Java class, each registers its own native table.

use jni::sys::{jint, JNI_VERSION_1_8};
use jni::{JNIEnv, JavaVM};
use std::ffi::c_void;

mod conv;
mod errors;
mod exports;
mod handles;

/// Entry point called by `System.load` exactly once per cdylib load.
#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    // Best-effort install of a panic hook that logs to stderr — anything that
    // panics inside a JNI call should already be caught by `with_jni_context`,
    // but this gives operators a breadcrumb if something slips through.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[nucleation-jvm] panic: {info}");
    }));

    let mut env = match vm.get_env() {
        Ok(env) => env,
        Err(_) => return JNI_VERSION_1_8,
    };

    // Register native methods for each Java class. Each module returns
    // `Result<(), jni::errors::Error>` — failures here will throw a
    // `LinkageError` on the JVM side when the corresponding method is called,
    // which is the expected JVM behaviour.
    let _ = exports::schematic::register(&mut env);
    let _ = exports::blockstate::register(&mut env);
    let _ = exports::shape::register(&mut env);
    let _ = exports::buildingtool::register(&mut env);
    let _ = exports::builder::register(&mut env);
    let _ = exports::nucleation::register(&mut env);
    let _ = exports::fingerprint::register(&mut env);
    let _ = exports::diff::register(&mut env);

    #[cfg(feature = "meshing")]
    let _ = exports::meshing::register(&mut env);

    #[cfg(feature = "simulation")]
    let _ = exports::simulation::register(&mut env);

    JNI_VERSION_1_8
}

/// Symbol the Java side calls to verify the cdylib loaded successfully.
/// Returns the crate version as a UTF-8 string.
#[no_mangle]
pub extern "system" fn Java_com_github_schemat_nucleation_NucleationNative_nVersion<'l>(
    env: JNIEnv<'l>,
    _class: jni::objects::JClass<'l>,
) -> jni::sys::jstring {
    let mut env = env;
    match env.new_string(env!("CARGO_PKG_VERSION")) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Reports whether the simulation feature was compiled in.
#[no_mangle]
pub extern "system" fn Java_com_github_schemat_nucleation_NucleationNative_nHasSimulation<'l>(
    _env: JNIEnv<'l>,
    _class: jni::objects::JClass<'l>,
) -> jni::sys::jboolean {
    #[cfg(feature = "simulation")]
    {
        jni::sys::JNI_TRUE
    }
    #[cfg(not(feature = "simulation"))]
    {
        jni::sys::JNI_FALSE
    }
}

/// Reports whether the meshing feature was compiled in.
#[no_mangle]
pub extern "system" fn Java_com_github_schemat_nucleation_NucleationNative_nHasMeshing<'l>(
    _env: JNIEnv<'l>,
    _class: jni::objects::JClass<'l>,
) -> jni::sys::jboolean {
    #[cfg(feature = "meshing")]
    {
        jni::sys::JNI_TRUE
    }
    #[cfg(not(feature = "meshing"))]
    {
        jni::sys::JNI_FALSE
    }
}
