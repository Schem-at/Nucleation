// src/lib.rs

// Core modules
pub mod block_entity;
mod block_position;
mod block_state;
mod bounding_box;
mod chunk;
mod entity;
mod formats;
pub mod insign;
mod item;
mod metadata;
mod print_utils;
mod region;
pub mod schematic_builder;
mod transforms;
mod universal_schematic;
pub mod utils;

// Feature-specific modules
#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "php")]
mod php;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "simulation")]
pub mod simulation;
#[cfg(feature = "wasm")]
pub mod wasm;

// Public re-exports
pub use block_state::BlockState;
pub use formats::{litematic, schematic};
pub use print_utils::{format_json_schematic, format_schematic};
pub use region::Region;
pub use schematic_builder::{
    palettes, IoMarker, IoType as BuilderIoType, PaletteEntry, SchematicBuilder,
};
pub use universal_schematic::UniversalSchematic;

// Re-export WASM types when building with WASM feature
#[cfg(feature = "wasm")]
pub use wasm::*;

// Re-export PHP types when building with PHP feature
#[cfg(feature = "php")]
pub use php::*;
