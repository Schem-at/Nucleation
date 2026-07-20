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

pub mod autostack;
pub mod blocks;
pub mod building;
pub mod definition_region;
pub mod diff;
pub mod distance_field;
pub mod geo;
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
pub mod world_stream;
