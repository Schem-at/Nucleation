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
pub mod schematic;
