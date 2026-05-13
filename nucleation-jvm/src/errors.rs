//! Panic / `Result` → JVM exception bridge.
//!
//! Every JNI export should run inside `with_jni_context` so that:
//! - Rust panics are caught and turned into `NucleationException`s rather
//!   than unwinding into the JVM (UB).
//! - `Result::Err(String)` is mapped to the appropriate exception subclass.
//!
//! Exception classes:
//! - `com/github/schemat/nucleation/exceptions/NucleationException` (base)
//! - `com/github/schemat/nucleation/exceptions/SchematicParseException`
//! - `com/github/schemat/nucleation/exceptions/InvalidBlockStateException`
//! - `com/github/schemat/nucleation/exceptions/UnsupportedFeatureException`

use jni::JNIEnv;
use std::panic::{self, AssertUnwindSafe};

pub const EX_BASE: &str = "com/github/schemat/nucleation/exceptions/NucleationException";
pub const EX_PARSE: &str = "com/github/schemat/nucleation/exceptions/SchematicParseException";
pub const EX_INVALID: &str = "com/github/schemat/nucleation/exceptions/InvalidBlockStateException";
pub const EX_UNSUPPORTED: &str =
    "com/github/schemat/nucleation/exceptions/UnsupportedFeatureException";

#[derive(Debug)]
pub enum JvmError {
    Generic(String),
    Parse(String),
    InvalidBlockState(String),
    Unsupported(String),
}

impl JvmError {
    pub fn class(&self) -> &'static str {
        match self {
            JvmError::Generic(_) => EX_BASE,
            JvmError::Parse(_) => EX_PARSE,
            JvmError::InvalidBlockState(_) => EX_INVALID,
            JvmError::Unsupported(_) => EX_UNSUPPORTED,
        }
    }
    pub fn msg(&self) -> &str {
        match self {
            JvmError::Generic(s)
            | JvmError::Parse(s)
            | JvmError::InvalidBlockState(s)
            | JvmError::Unsupported(s) => s,
        }
    }
}

impl From<String> for JvmError {
    fn from(s: String) -> Self {
        JvmError::Generic(s)
    }
}

impl From<&str> for JvmError {
    fn from(s: &str) -> Self {
        JvmError::Generic(s.to_string())
    }
}

impl<E: std::fmt::Display> From<Box<E>> for JvmError {
    fn from(e: Box<E>) -> Self {
        JvmError::Generic(e.to_string())
    }
}

/// Throws the appropriate exception class with the given message. Best-effort —
/// if the JVM is in a state where it cannot throw (already has a pending
/// exception, OOM, etc.) we silently swallow.
pub fn throw(env: &mut JNIEnv, err: &JvmError) {
    let _ = env.throw_new(err.class(), err.msg());
}

/// Wrap a JNI export. Catches panics and converts `Err` into a JVM exception.
/// `default` is what to return when an exception is thrown — usually a zero
/// value of the JNI primitive return type, since the JVM ignores it.
pub fn with_jni_context<R: Default, F>(env: &mut JNIEnv, default: R, f: F) -> R
where
    F: FnOnce(&mut JNIEnv) -> Result<R, JvmError>,
{
    let result = panic::catch_unwind(AssertUnwindSafe(|| f(env)));
    match result {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            throw(env, &e);
            default
        }
        Err(p) => {
            let msg = if let Some(s) = p.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = p.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else {
                "Rust panic in JNI call".to_string()
            };
            throw(env, &JvmError::Generic(msg));
            default
        }
    }
}
