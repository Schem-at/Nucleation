//! `SchematicBuilder` JNI exports — mirrors `PySchematicBuilder`.

use crate::conv::{jstring_to_string, string_to_jstring};
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_mut, as_ref, consume, to_handle};
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jchar, jint, jlong, jstring};
use jni::{JNIEnv, NativeMethod};
use nucleation::SchematicBuilder;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nBuilderCreate", "()J", n_create as *mut _),
        nm("nBuilderFree", "(J)V", n_free as *mut _),
        nm("nBuilderName", "(JLjava/lang/String;)V", n_name as *mut _),
        nm("nBuilderMap", "(JCLjava/lang/String;)V", n_map as *mut _),
        nm("nBuilderLayer", "(J[Ljava/lang/String;)V", n_layer as *mut _),
        nm("nBuilderOffset", "(JIII)V", n_offset as *mut _),
        nm("nBuilderUseStandardPalette", "(J)V", n_use_standard as *mut _),
        nm("nBuilderUseMinimalPalette", "(J)V", n_use_minimal as *mut _),
        nm("nBuilderUseCompactPalette", "(J)V", n_use_compact as *mut _),
        nm("nBuilderValidate", "(J)Ljava/lang/String;", n_validate as *mut _),
        nm("nBuilderBuild", "(J)J", n_build as *mut _),
        nm("nBuilderToTemplate", "(J)Ljava/lang/String;", n_to_template as *mut _),
        nm("nBuilderFromTemplate", "(Ljava/lang/String;)J", n_from_template as *mut _),
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
) -> jlong {
    with_jni_context(&mut env, 0, |_env| Ok(to_handle(SchematicBuilder::new())))
}

unsafe extern "system" fn n_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<SchematicBuilder>(handle);
    }
}

unsafe extern "system" fn n_name<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    name: JString<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        let name = jstring_to_string(env, &name)?;
        let b = as_mut::<SchematicBuilder>(handle);
        // SchematicBuilder uses move-style fluent API; we replace in place by
        // taking ownership, applying, and reinserting.
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        *b = owned.name(name);
        Ok(())
    })
}

unsafe extern "system" fn n_map<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    ch: jchar,
    block: JString<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        let block = jstring_to_string(env, &block)?;
        let c = char::from_u32(ch as u32).unwrap_or(' ');
        let b = as_mut::<SchematicBuilder>(handle);
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        *b = owned.map(c, block);
        Ok(())
    })
}

unsafe extern "system" fn n_layer<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    rows: JObjectArray<'l>,
) {
    with_jni_context(&mut env, (), |env| {
        let len = env
            .get_array_length(&rows)
            .map_err(|e| JvmError::Generic(format!("array length: {e}")))?;
        let mut strings: Vec<String> = Vec::with_capacity(len as usize);
        for i in 0..len {
            let row = env
                .get_object_array_element(&rows, i)
                .map_err(|e| JvmError::Generic(format!("get_object_array_element: {e}")))?;
            let js: JString = JString::from(row);
            strings.push(jstring_to_string(env, &js)?);
        }
        let b = as_mut::<SchematicBuilder>(handle);
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        let row_refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        *b = owned.layer(&row_refs);
        Ok(())
    })
}

unsafe extern "system" fn n_offset<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) {
    with_jni_context(&mut env, (), |_env| {
        let b = as_mut::<SchematicBuilder>(handle);
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        *b = owned.offset(x, y, z);
        Ok(())
    })
}

unsafe extern "system" fn n_use_standard<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) {
    with_jni_context(&mut env, (), |_env| {
        let b = as_mut::<SchematicBuilder>(handle);
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        *b = owned.use_standard_palette();
        Ok(())
    })
}

unsafe extern "system" fn n_use_minimal<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) {
    with_jni_context(&mut env, (), |_env| {
        let b = as_mut::<SchematicBuilder>(handle);
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        *b = owned.use_minimal_palette();
        Ok(())
    })
}

unsafe extern "system" fn n_use_compact<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) {
    with_jni_context(&mut env, (), |_env| {
        let b = as_mut::<SchematicBuilder>(handle);
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        *b = owned.use_compact_palette();
        Ok(())
    })
}

unsafe extern "system" fn n_validate<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let b = as_ref::<SchematicBuilder>(handle);
        match b.validate() {
            Ok(()) => string_to_jstring(env, ""),
            Err(msg) => string_to_jstring(env, &msg),
        }
    })
}

unsafe extern "system" fn n_build<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let b = as_mut::<SchematicBuilder>(handle);
        let owned = std::mem::replace(b, SchematicBuilder::empty());
        let schematic = owned.build().map_err(JvmError::Parse)?;
        Ok(to_handle(schematic))
    })
}

unsafe extern "system" fn n_to_template<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jstring {
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let b = as_ref::<SchematicBuilder>(handle);
        string_to_jstring(env, &b.to_template())
    })
}

unsafe extern "system" fn n_from_template<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    template: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let t = jstring_to_string(env, &template)?;
        let b = SchematicBuilder::from_template(&t).map_err(JvmError::Parse)?;
        Ok(to_handle(b))
    })
}
