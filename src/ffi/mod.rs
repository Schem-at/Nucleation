// src/ffi.rs
#![cfg(feature = "ffi")]
use crate::{
    block_position::BlockPosition,
    bounding_box::BoundingBox,
    building::{
        BezierCurve, BilinearGradientBrush, BrushEnum, BuildingTool, ColorBrush, Cone, Cuboid,
        CurveGradientBrush, Cylinder, Difference, Disk, Ellipsoid, Hollow, InterpolationSpace,
        Intersection, Line, LinearGradientBrush, Plane, PointGradientBrush, Pyramid, ShadedBrush,
        ShapeEnum, SolidBrush, Sphere, Torus, Triangle, Union,
    },
    definition_region::DefinitionRegion,
    formats::{litematic, manager::get_manager, mcstructure},
    print_utils::{format_json_schematic, format_schematic},
    universal_schematic::ChunkLoadingStrategy,
    BlockState, SchematicBuilder, UniversalSchematic,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_uchar};
use std::ptr;

// --- Last Error ---

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn set_last_error(msg: String) {
    eprintln!("[nucleation] {}", &msg);
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

/// Returns the last error message as a C string.
/// Caller must free the returned pointer with `schematic_free_string`.
/// Returns null if no error.
#[no_mangle]
pub extern "C" fn schematic_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| {
        let err = e.borrow();
        match err.as_ref() {
            Some(msg) => CString::new(msg.as_str()).unwrap_or_default().into_raw(),
            None => ptr::null_mut(),
        }
    })
}

// --- C-Compatible Data Structures ---

#[repr(C)]
pub struct ByteArray {
    data: *mut c_uchar,
    len: usize,
}

#[repr(C)]
pub struct StringArray {
    data: *mut *mut c_char,
    len: usize,
}

#[repr(C)]
pub struct IntArray {
    data: *mut c_int,
    len: usize,
}

#[repr(C)]
pub struct CProperty {
    key: *mut c_char,
    value: *mut c_char,
}

#[repr(C)]
pub struct CPropertyArray {
    data: *mut CProperty,
    len: usize,
}

#[repr(C)]
pub struct CBlock {
    x: c_int,
    y: c_int,
    z: c_int,
    name: *mut c_char,
    properties_json: *mut c_char,
}

#[repr(C)]
#[derive(Clone)]
pub struct CBlockArray {
    data: *mut CBlock,
    len: usize,
}

#[repr(C)]
pub struct CBlockEntity {
    id: *mut c_char,
    x: c_int,
    y: c_int,
    z: c_int,
    nbt_json: *mut c_char,
}

#[repr(C)]
pub struct CBlockEntityArray {
    data: *mut CBlockEntity,
    len: usize,
}

#[repr(C)]
pub struct CEntity {
    id: *mut c_char,
    x: f64,
    y: f64,
    z: f64,
    nbt_json: *mut c_char,
}

#[repr(C)]
pub struct CEntityArray {
    data: *mut CEntity,
    len: usize,
}

#[repr(C)]
pub struct CChunk {
    chunk_x: c_int,
    chunk_y: c_int,
    chunk_z: c_int,
    blocks: CBlockArray,
}

#[repr(C)]
pub struct CChunkArray {
    data: *mut CChunk,
    len: usize,
}

// --- Wrapper Structs with Opaque Pointers ---

pub struct SchematicWrapper(*mut UniversalSchematic);
pub struct BlockStateWrapper(*mut BlockState);

#[cfg(feature = "simulation")]
pub struct MchprsWorldWrapper(*mut crate::simulation::MchprsWorld);

// --- Memory Management ---

/// Frees a ByteArray returned by the library.
#[no_mangle]
pub extern "C" fn free_byte_array(array: ByteArray) {
    if !array.data.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(array.data, array.len, array.len);
        }
    }
}

/// Frees a StringArray returned by the library.
#[no_mangle]
pub extern "C" fn free_string_array(array: StringArray) {
    if !array.data.is_null() {
        unsafe {
            let strings = Vec::from_raw_parts(array.data, array.len, array.len);
            for s in strings {
                let _ = CString::from_raw(s);
            }
        }
    }
}

/// Frees an IntArray returned by the library.
#[no_mangle]
pub extern "C" fn free_int_array(array: IntArray) {
    if !array.data.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(array.data, array.len, array.len);
        }
    }
}

/// Frees a C string returned by the library.
#[no_mangle]
pub extern "C" fn free_string(string: *mut c_char) {
    if !string.is_null() {
        unsafe {
            let _ = CString::from_raw(string);
        }
    }
}

/// Frees a CPropertyArray returned by `blockstate_get_properties`.
#[no_mangle]
pub extern "C" fn free_property_array(array: CPropertyArray) {
    if !array.data.is_null() {
        unsafe {
            let props = Vec::from_raw_parts(array.data, array.len, array.len);
            for prop in props {
                free_string(prop.key);
                free_string(prop.value);
            }
        }
    }
}

/// Frees a single CBlock. Used as a helper by `free_block_array`.
fn free_single_block(block: &mut CBlock) {
    free_string(block.name);
    free_string(block.properties_json);
}

/// Frees a CBlockArray returned by functions like `schematic_get_all_blocks`.
#[no_mangle]
pub extern "C" fn free_block_array(array: CBlockArray) {
    if !array.data.is_null() {
        unsafe {
            let mut blocks = Vec::from_raw_parts(array.data, array.len, array.len);
            for block in &mut blocks {
                free_single_block(block);
            }
        }
    }
}

/// Frees a single CBlockEntity. Used as a helper by `free_block_entity_array`.
fn free_single_block_entity(entity: &mut CBlockEntity) {
    free_string(entity.id);
    free_string(entity.nbt_json);
}

/// Frees a CBlockEntityArray returned by `schematic_get_all_block_entities`.
#[no_mangle]
pub extern "C" fn free_block_entity_array(array: CBlockEntityArray) {
    if !array.data.is_null() {
        unsafe {
            let mut entities = Vec::from_raw_parts(array.data, array.len, array.len);
            for entity in &mut entities {
                free_single_block_entity(entity);
            }
        }
    }
}

/// Frees a single CChunk. Used as a helper by `free_chunk_array`.
fn free_single_chunk(chunk: &mut CChunk) {
    free_block_array(chunk.blocks.clone());
}

/// Frees a CChunkArray returned by `schematic_get_chunks`.
#[no_mangle]
pub extern "C" fn free_chunk_array(array: CChunkArray) {
    if !array.data.is_null() {
        unsafe {
            let mut chunks = Vec::from_raw_parts(array.data, array.len, array.len);
            for chunk in &mut chunks {
                free_single_chunk(chunk);
            }
        }
    }
}

// --- NBT helpers (chest / sign / text builders) ---
//
// These return owned C strings the caller must release with `free_string`.
// Output targets the modern (1.20+) Minecraft NBT schemas.

/// A single inventory item for `nbt_chest_build`.
/// `count <= 0` defaults to 1; `slot < 0` auto-assigns positionally.
#[repr(C)]
pub struct CItem {
    pub id: *const c_char,
    pub count: c_int,
    pub slot: c_int,
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn snbt_string(s: &str) -> String {
    format!("\"{}\"", json_escape(s))
}

fn cstr_opt(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
    }
}

fn build_text_json(s: &str, color: Option<&str>, bold: c_int, italic: c_int) -> String {
    let mut parts = vec![format!("\"text\":\"{}\"", json_escape(s))];
    if let Some(c) = color {
        parts.push(format!("\"color\":\"{}\"", json_escape(c)));
    }
    if bold >= 0 {
        parts.push(format!("\"bold\":{}", bold != 0));
    }
    if italic >= 0 {
        parts.push(format!("\"italic\":{}", italic != 0));
    }
    format!("{{{}}}", parts.join(","))
}

/// Build a Minecraft JSON text-component string.
///
/// `color` may be null. `bold` and `italic` use `-1` for unset, `0` for false,
/// nonzero for true. The caller must free the returned string with
/// `free_string`.
#[no_mangle]
pub extern "C" fn nbt_text_build(
    s: *const c_char,
    color: *const c_char,
    bold: c_int,
    italic: c_int,
) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    let text = unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned();
    let color_owned = cstr_opt(color);
    let json = build_text_json(&text, color_owned.as_deref(), bold, italic);
    CString::new(json)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Build a chest-NBT SNBT string for use as the `{...}` portion of a block
/// string. Returns null on error.
///
/// `items` is an array of `n` `CItem` entries. `name` is an optional plain
/// text custom name (null = no CustomName); it is wrapped in a JSON text
/// component automatically.
///
/// Caller frees with `free_string`.
#[no_mangle]
pub extern "C" fn nbt_chest_build(
    items: *const CItem,
    n: usize,
    name: *const c_char,
) -> *mut c_char {
    if items.is_null() && n > 0 {
        return ptr::null_mut();
    }
    let slice = if n > 0 {
        unsafe { std::slice::from_raw_parts(items, n) }
    } else {
        &[]
    };

    let mut entries = Vec::with_capacity(slice.len());
    for (i, it) in slice.iter().enumerate() {
        if it.id.is_null() {
            return ptr::null_mut();
        }
        let id = unsafe { CStr::from_ptr(it.id) }.to_string_lossy();
        let count = if it.count <= 0 { 1 } else { it.count };
        let slot: i32 = if it.slot < 0 { i as i32 } else { it.slot };
        entries.push(format!(
            "{{Slot:{}b,id:\"{}\",Count:{}b}}",
            slot,
            json_escape(&id),
            count
        ));
    }

    let mut parts = vec![format!("Items:[{}]", entries.join(","))];
    if let Some(n) = cstr_opt(name) {
        let already_json = n.starts_with('{');
        let inner = if already_json {
            n
        } else {
            build_text_json(&n, None, -1, -1)
        };
        // CustomName is stored as a string holding JSON.
        parts.push(format!("CustomName:{}", snbt_string(&inner)));
    }
    let snbt = format!("{{{}}}", parts.join(","));
    CString::new(snbt)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Build a modern (1.20+) sign-NBT SNBT string.
///
/// `front` and `back` are arrays of up to 4 C strings. Each line may be a
/// plain string (auto-wrapped via `nbt_text_build`) or an already-built JSON
/// component (starts with `{`). Either pointer may be null (treated as empty).
///
/// `color` is the dye color string (null defaults to `"black"`).
/// `glowing` and `waxed` are 0/non-zero booleans.
///
/// Caller frees with `free_string`.
#[no_mangle]
pub extern "C" fn nbt_sign_build(
    front: *const *const c_char,
    front_n: usize,
    back: *const *const c_char,
    back_n: usize,
    color: *const c_char,
    glowing: c_int,
    waxed: c_int,
) -> *mut c_char {
    fn collect(p: *const *const c_char, n: usize) -> Result<Vec<String>, ()> {
        if n > 4 {
            return Err(());
        }
        if p.is_null() && n > 0 {
            return Err(());
        }
        let slice = if n > 0 {
            unsafe { std::slice::from_raw_parts(p, n) }
        } else {
            &[]
        };
        let mut out = Vec::with_capacity(4);
        for sp in slice {
            if sp.is_null() {
                out.push(String::from("\"\""));
                continue;
            }
            let s = unsafe { CStr::from_ptr(*sp) }
                .to_string_lossy()
                .into_owned();
            if s.is_empty() {
                out.push(String::from("\"\""));
            } else if s.starts_with('{') {
                out.push(s);
            } else {
                out.push(build_text_json(&s, None, -1, -1));
            }
        }
        while out.len() < 4 {
            out.push(String::from("\"\""));
        }
        Ok(out)
    }

    let front_msgs = match collect(front, front_n) {
        Ok(v) => v,
        Err(_) => return ptr::null_mut(),
    };
    let back_msgs = match collect(back, back_n) {
        Ok(v) => v,
        Err(_) => return ptr::null_mut(),
    };

    let color_owned = cstr_opt(color).unwrap_or_else(|| "black".to_string());
    let g = if glowing != 0 { "1b" } else { "0b" };
    let w = if waxed != 0 { "1b" } else { "0b" };

    let messages = |msgs: &[String]| -> String {
        // Each message must be stored as a *string* containing JSON.
        let quoted: Vec<String> = msgs.iter().map(|m| snbt_string(m)).collect();
        format!("[{}]", quoted.join(","))
    };

    let snbt = format!(
        "{{front_text:{{messages:{},color:\"{}\",has_glowing_text:{}}},back_text:{{messages:{},color:\"{}\",has_glowing_text:{}}},is_waxed:{}}}",
        messages(&front_msgs),
        json_escape(&color_owned),
        g,
        messages(&back_msgs),
        json_escape(&color_owned),
        g,
        w,
    );
    CString::new(snbt)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

// --- Shared String Array Helper ---
//
// Used by multiple domain submodules (schematic, definition_region, meshing);
// kept here so it's reachable from all of them via `use super::*;`.

fn vec_string_to_string_array(strings: Vec<String>) -> StringArray {
    let c_strings: Vec<*mut c_char> = strings
        .into_iter()
        .map(|s| CString::new(s).unwrap().into_raw())
        .collect();
    let mut c_strings = c_strings;
    let ptr = c_strings.as_mut_ptr();
    let len = c_strings.len();
    std::mem::forget(c_strings);
    StringArray { data: ptr, len }
}

// --- Submodules ---

mod building;
mod definition_region;
mod diff;
mod schematic;
mod schematic_builder;
mod scripting;
mod sdf;
mod store_io;
mod world_stream;

#[cfg(feature = "autostack")]
mod autostack;
#[cfg(feature = "meshing")]
mod meshing;
#[cfg(feature = "rendering")]
mod rendering;
#[cfg(feature = "simulation")]
mod simulation;

// Re-export everything from the domain submodules so external callers keep
// using their original flat paths (e.g. `nucleation::ffi::schematic_new`),
// unaffected by which file the code now physically lives in. All items below
// are `#[no_mangle] pub extern "C" fn` (globally-unique link symbols) or
// FFI-only helper types, so these globs cannot introduce ambiguous names.
pub use building::*;
pub use definition_region::*;
pub use diff::*;
pub use schematic::*;
pub use schematic_builder::*;
pub use scripting::*;
pub use sdf::*;
pub use store_io::*;
pub use world_stream::*;

#[cfg(feature = "autostack")]
pub use autostack::*;

// The nested `simulation_ffi` / `rendering_ffi` namespaces are re-exported
// as-is (not flattened) because external code already refers to them as
// `nucleation::ffi::simulation_ffi::...` / `nucleation::ffi::rendering_ffi::...`.
#[cfg(feature = "rendering")]
pub use rendering::rendering_ffi;
#[cfg(feature = "simulation")]
pub use simulation::simulation_ffi;
