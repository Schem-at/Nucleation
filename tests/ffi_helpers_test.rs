// Tests for the FFI NBT helper builders (chest, sign, text).
//
// Run with: cargo test --features ffi --test ffi_helpers_test

#![cfg(feature = "ffi")]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use nucleation::ffi::{free_string, nbt_chest_build, nbt_sign_build, nbt_text_build, CItem};

fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn read_owned(p: *mut c_char) -> String {
    assert!(!p.is_null(), "builder returned null");
    let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
    free_string(p);
    s
}

#[test]
fn text_basic() {
    let s = cstr("Hello");
    let out = read_owned(nbt_text_build(s.as_ptr(), std::ptr::null(), -1, -1));
    assert_eq!(out, r#"{"text":"Hello"}"#);
}

#[test]
fn text_with_color_and_bold() {
    let s = cstr("Warn");
    let c = cstr("red");
    let out = read_owned(nbt_text_build(s.as_ptr(), c.as_ptr(), 1, -1));
    assert!(out.contains("\"color\":\"red\""), "got: {}", out);
    assert!(out.contains("\"bold\":true"), "got: {}", out);
}

#[test]
fn text_escapes_quotes() {
    let s = cstr(r#"He said "hi""#);
    let out = read_owned(nbt_text_build(s.as_ptr(), std::ptr::null(), -1, -1));
    assert!(out.contains(r#"\""#), "got: {}", out);
}

#[test]
fn chest_basic() {
    let id_diamond = cstr("minecraft:diamond");
    let id_elytra = cstr("minecraft:elytra");
    let items = [
        CItem { id: id_diamond.as_ptr(), count: 64, slot: -1 },
        CItem { id: id_elytra.as_ptr(),  count: 1,  slot: -1 },
    ];
    let out = read_owned(nbt_chest_build(items.as_ptr(), items.len(), std::ptr::null()));
    assert!(out.contains("Items:[{Slot:0b"), "got: {}", out);
    assert!(out.contains("minecraft:diamond"), "got: {}", out);
    assert!(out.contains("minecraft:elytra"), "got: {}", out);
    assert!(out.contains("Count:64b"), "got: {}", out);
    assert!(!out.contains("CustomName"), "no name expected: {}", out);
}

#[test]
fn chest_with_explicit_slots_and_name() {
    let id = cstr("minecraft:diamond");
    let items = [CItem { id: id.as_ptr(), count: 1, slot: 13 }];
    let name = cstr("Loot");
    let out = read_owned(nbt_chest_build(items.as_ptr(), items.len(), name.as_ptr()));
    assert!(out.contains("Slot:13b"), "got: {}", out);
    assert!(out.contains("CustomName:"), "got: {}", out);
    assert!(out.contains("Loot"), "got: {}", out);
}

#[test]
fn sign_basic() {
    let lines = [cstr("Welcome"), cstr("home")];
    let line_ptrs: [*const c_char; 2] = [lines[0].as_ptr(), lines[1].as_ptr()];
    let out = read_owned(nbt_sign_build(
        line_ptrs.as_ptr(),
        line_ptrs.len(),
        std::ptr::null(),
        0,
        std::ptr::null(),
        0,
        0,
    ));
    assert!(out.contains("front_text:"), "got: {}", out);
    assert!(out.contains("back_text:"), "got: {}", out);
    assert!(out.contains("is_waxed:0b"), "got: {}", out);
    // Inside SNBT, JSON quotes are backslash-escaped, so we look for the
    // escaped form of `{"text":"Welcome"}`.
    assert!(out.contains(r#"\"text\":\"Welcome\""#), "got: {}", out);
    assert!(out.contains(r#"\"text\":\"home\""#), "got: {}", out);
    assert_eq!(out.matches("messages:[").count(), 2, "wrong messages count: {}", out);
}

#[test]
fn sign_waxed_glowing_with_color() {
    let line = cstr("Hi");
    let line_ptrs: [*const c_char; 1] = [line.as_ptr()];
    let color = cstr("white");
    let out = read_owned(nbt_sign_build(
        line_ptrs.as_ptr(),
        1,
        std::ptr::null(),
        0,
        color.as_ptr(),
        1,
        1,
    ));
    assert!(out.contains("is_waxed:1b"), "got: {}", out);
    assert!(out.contains("has_glowing_text:1b"), "got: {}", out);
    assert!(out.contains("color:\"white\""), "got: {}", out);
}

#[test]
fn sign_rejects_more_than_four_lines() {
    let lines: Vec<CString> = (0..5).map(|i| cstr(&format!("L{}", i))).collect();
    let ptrs: Vec<*const c_char> = lines.iter().map(|c| c.as_ptr()).collect();
    let out = nbt_sign_build(
        ptrs.as_ptr(),
        ptrs.len(),
        std::ptr::null(),
        0,
        std::ptr::null(),
        0,
        0,
    );
    assert!(out.is_null(), "should reject >4 lines");
}

#[test]
fn batch_chest_via_set_blocks_complex() {
    use nucleation::ffi::{
        schematic_free, schematic_new, schematic_set_blocks,
    };

    let handle = schematic_new();
    assert!(!handle.is_null());

    let count = 30usize;
    let mut positions: Vec<i32> = Vec::with_capacity(count * 3);
    for i in 0..count {
        positions.extend_from_slice(&[i as i32, 0, 0]);
    }
    let chest_str = cstr(
        r#"minecraft:chest[facing=west]{Items:[{Slot:0b,id:"minecraft:diamond",Count:64b}]}"#,
    );
    let rc = schematic_set_blocks(
        handle,
        positions.as_ptr(),
        positions.len(),
        chest_str.as_ptr(),
    );
    assert_eq!(rc, count as c_int, "expected {}, got {}", count, rc);

    schematic_free(handle);
}

#[test]
fn end_to_end_chest_placement() {
    use nucleation::ffi::{schematic_free, schematic_new, schematic_set_block_from_string};

    let id = cstr("minecraft:diamond");
    let items = [CItem { id: id.as_ptr(), count: 64, slot: -1 }];
    let nbt = read_owned(nbt_chest_build(items.as_ptr(), items.len(), std::ptr::null()));

    let block_str = format!("minecraft:chest[facing=west]{}", nbt);
    let c = cstr(&block_str);

    let handle = schematic_new();
    assert!(!handle.is_null());
    let rc = schematic_set_block_from_string(handle, 0, 0, 0, c.as_ptr());
    assert_eq!(rc, 0, "set_block_from_string failed: rc={}", rc);

    schematic_free(handle);
}

#[test]
fn prepare_block_and_place_roundtrip() {
    use nucleation::ffi::{
        schematic_free, schematic_get_block_with_properties, schematic_new, schematic_place,
        schematic_prepare_block,
    };

    let handle = schematic_new();
    assert!(!handle.is_null());

    let stone = cstr("minecraft:stone");
    let dirt = cstr("minecraft:dirt");
    let stone_idx = schematic_prepare_block(handle, stone.as_ptr());
    let dirt_idx = schematic_prepare_block(handle, dirt.as_ptr());
    assert!(stone_idx >= 0 && dirt_idx >= 0);
    assert_ne!(stone_idx, dirt_idx, "different names → different indices");

    let rc1 = schematic_place(handle, 0, 0, 0, stone_idx);
    let rc2 = schematic_place(handle, 1, 0, 0, dirt_idx);
    assert_eq!(rc1, 0);
    assert_eq!(rc2, 0);

    let bs0 = schematic_get_block_with_properties(handle, 0, 0, 0);
    let bs1 = schematic_get_block_with_properties(handle, 1, 0, 0);
    assert!(!bs0.is_null());
    assert!(!bs1.is_null());

    unsafe { schematic_free(handle) };
}

#[test]
fn place_rejects_null_handle() {
    use nucleation::ffi::{schematic_place, schematic_prepare_block};
    let name = cstr("minecraft:stone");
    let rc1 = schematic_prepare_block(std::ptr::null_mut(), name.as_ptr());
    assert!(rc1 < 0);
    let rc2 = schematic_place(std::ptr::null_mut(), 0, 0, 0, 0);
    assert!(rc2 < 0);
}
