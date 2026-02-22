//! WASM bindings for Nucleation
//!
//! This module provides JavaScript/TypeScript bindings via WebAssembly.
//! The API is organized into submodules by functionality:
//!
//! - `definition_region`: Region manipulation for circuit IO
//! - `schematic`: Core schematic operations
//! - `schematic_builder`: ASCII art schematic construction
//! - `meshing`: 3D mesh generation (feature-gated)
//! - `simulation`: MCHPRS simulation (feature-gated)
//! - `typed_executor`: Typed circuit execution (feature-gated)
//! - `circuit_builder`: Fluent executor builder (feature-gated)

mod building;
mod definition_region;
mod palettes;
mod schematic;
mod schematic_builder;

#[cfg(feature = "simulation")]
mod circuit_builder;
#[cfg(feature = "meshing")]
mod meshing;
#[cfg(feature = "rendering")]
mod rendering;
#[cfg(feature = "simulation")]
mod simulation;
#[cfg(feature = "simulation")]
mod typed_executor;

// Re-export all public types to maintain the same JS API
pub use building::{BrushWrapper, ShapeWrapper, WasmBuildingTool};
pub use definition_region::DefinitionRegionWrapper;
pub use palettes::PaletteManager;
pub use schematic::{
    debug_json_schematic, debug_schematic, BlockStateWrapper, LazyChunkIterator, SchematicWrapper,
};
pub use schematic_builder::SchematicBuilderWrapper;

#[cfg(feature = "simulation")]
pub use circuit_builder::{CircuitBuilderWrapper, SortStrategyWrapper, StateModeConstants};
#[cfg(feature = "meshing")]
pub use meshing::{
    ChunkMeshIteratorWrapper, ChunkMeshResultWrapper, MeshConfigWrapper, MeshResultWrapper,
    MultiMeshResultWrapper, ResourcePackWrapper, TextureAtlasWrapper,
};
#[cfg(feature = "rendering")]
pub use rendering::RenderConfigWrapper;
#[cfg(feature = "simulation")]
pub use simulation::{MchprsWorldWrapper, SimulationOptionsWrapper};
#[cfg(feature = "simulation")]
pub use typed_executor::{
    ExecutionModeWrapper, IoLayoutBuilderWrapper, IoLayoutWrapper, IoTypeWrapper,
    LayoutFunctionWrapper, OutputConditionWrapper, TypedCircuitExecutorWrapper, ValueWrapper,
};

// Re-export the start function
pub use schematic::start;
