//! Opaque pointer ↔ `jlong` handle conversion.
//!
//! A handle is just `Box::into_raw(Box<T>) as jlong`. Java holds the integer,
//! Rust dereferences it back into a `&T` or `&mut T` on each call. Every
//! function in this module is unsafe at the boundary but exposes a safe
//! Rust-side API by panicking on `0` (which the Java side maps to "closed").

use jni::sys::jlong;

/// Allocate `value` on the heap and return its raw pointer as a `jlong`.
pub fn to_handle<T>(value: T) -> jlong {
    Box::into_raw(Box::new(value)) as usize as jlong
}

/// Convert a handle back to an owned `Box<T>`, dropping ownership on the Java
/// side. Panics if the handle is zero.
///
/// # Safety
/// The handle must have been produced by `to_handle::<T>` and not yet consumed.
pub unsafe fn consume<T>(handle: jlong) -> Box<T> {
    assert!(handle != 0, "consume called on closed handle");
    Box::from_raw(handle as usize as *mut T)
}

/// Borrow a `&T` from a handle. Panics if the handle is zero.
///
/// # Safety
/// The handle must point at a live `T` not currently mutably borrowed.
pub unsafe fn as_ref<'a, T>(handle: jlong) -> &'a T {
    assert!(handle != 0, "as_ref called on closed handle");
    &*(handle as usize as *const T)
}

/// Borrow a `&mut T` from a handle. Panics if the handle is zero.
///
/// # Safety
/// The handle must point at a live `T` and the caller must guarantee no other
/// reference is live for the duration of the borrow.
pub unsafe fn as_mut<'a, T>(handle: jlong) -> &'a mut T {
    assert!(handle != 0, "as_mut called on closed handle");
    &mut *(handle as usize as *mut T)
}
