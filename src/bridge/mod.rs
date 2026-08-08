//! Diplomat-generated bindings for nucleation — the successor to the hand-written
//! `ffi`/`wasm`/`python` layers. Built on a Diplomat fork
//! (`github.com/Nano112/diplomat`, adds a PHP backend); the codegen pipeline lives in
//! `/Users/harrison/code/stencil`. See `stencil/docs/nucleation-error.md` for the error
//! design and `stencil/docs/mesh-progress-api.md` for the polling mesh-progress design.
//!
//! Layout: one bridge module per domain, mirroring the old `ffi/*.rs` split. Shared
//! types (`NucleationError`, `Dimensions`, …) live in [`shared`]; every other module
//! references them (and each other's opaques) by path.

pub mod shared;

thread_local! {
    /// Why the last failing bridge call on this thread failed, in words.
    ///
    /// [`shared::ffi::NucleationError`] is a bare enum — the FFI cannot carry
    /// a message in the error itself — so modules that know the real story
    /// (mc_tick's constructors, principally) park it here on every failure
    /// path and clear it on success. Read back through
    /// `NucleationError::detail` or `TickSimulation::last_error_detail`.
    static LAST_ERROR_DETAIL: std::cell::RefCell<String> =
        const { std::cell::RefCell::new(String::new()) };
}

/// Record why the bridge call that is about to fail failed.
pub(crate) fn set_last_error_detail(detail: impl Into<String>) {
    LAST_ERROR_DETAIL.with(|e| *e.borrow_mut() = detail.into());
}

/// Forget the previous failure's story, so a stale detail is never read as
/// this call's.
pub(crate) fn clear_last_error_detail() {
    LAST_ERROR_DETAIL.with(|e| e.borrow_mut().clear());
}

/// The last recorded failure detail on this thread; empty when the last
/// detail-carrying call succeeded.
pub(crate) fn last_error_detail() -> String {
    LAST_ERROR_DETAIL.with(|e| e.borrow().clone())
}

pub mod animation;
pub mod autostack;
pub mod blocks;
pub mod building;
pub mod definition_region;
pub mod diff;
pub mod distance_field;
pub mod field;
pub mod geo;
#[cfg(feature = "mc-tick")]
pub mod mc_tick;
#[cfg(feature = "meshing")]
pub mod meshing;
pub mod nbt;
#[cfg(feature = "rendering")]
pub mod rendering;
pub mod schematic;
pub mod schematic_builder;
#[cfg(any(feature = "scripting-lua", feature = "scripting-js"))]
pub mod scripting;
pub mod sdf;
#[cfg(feature = "simulation")]
pub mod simulation;
pub mod store_io;
#[cfg(feature = "voxelize")]
pub mod voxelize;
pub mod world_generation;
#[cfg(all(feature = "bridge", feature = "world-segment"))]
pub mod world_segment;
pub mod world_stream;
