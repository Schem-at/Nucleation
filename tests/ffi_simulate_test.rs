// Test for the schematic_simulate_use_block FFI convenience.
//
// Calls the function via its Rust module path
// (`nucleation::ffi::simulation_ffi::schematic_simulate_use_block`).
// The function is `#[no_mangle] pub extern "C"`, so this exercises exactly
// the same code path that downstream C consumers will hit.
//
// Run with: cargo test --features ffi,simulation --test ffi_simulate_test

#![cfg(all(feature = "ffi", feature = "simulation"))]

use std::ffi::CString;
use std::os::raw::c_int;

use nucleation::ffi::{
    schematic_free, schematic_new, schematic_set_block,
    simulation_ffi::schematic_simulate_use_block,
};

fn put(handle: *mut nucleation::ffi::SchematicWrapper, x: i32, y: i32, z: i32, name: &str) {
    let c = CString::new(name).unwrap();
    let rc = unsafe { schematic_set_block(handle, x, y, z, c.as_ptr()) };
    assert_eq!(rc, 0, "set_block({}) failed: rc={}", name, rc);
}

#[test]
fn ticks_with_no_events_returns_zero() {
    let handle = unsafe { schematic_new() };
    assert!(!handle.is_null());
    put(handle, 0, 0, 0, "minecraft:stone");

    let rc = unsafe { schematic_simulate_use_block(handle, 1, std::ptr::null(), 0) };
    assert_eq!(rc, 0, "simulate failed: rc={}", rc);

    unsafe { schematic_free(handle) };
}

#[test]
fn rejects_null_handle() {
    let rc = unsafe { schematic_simulate_use_block(std::ptr::null_mut(), 1, std::ptr::null(), 0) };
    assert!(rc < 0);
}

#[test]
fn accepts_use_block_events() {
    let handle = unsafe { schematic_new() };
    assert!(!handle.is_null());
    put(handle, 0, 0, 0, "minecraft:stone");
    put(handle, 1, 0, 0, "minecraft:stone");

    // Fire one use-block event on the placed stone. The simulator treats
    // unsupported targets as no-ops; we're verifying the call path runs.
    let events: [c_int; 3] = [0, 0, 0];
    let rc = unsafe { schematic_simulate_use_block(handle, 2, events.as_ptr(), 1) };
    assert_eq!(rc, 0, "simulate failed: rc={}", rc);

    unsafe { schematic_free(handle) };
}
