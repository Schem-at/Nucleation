//! WASM bindings for Nucleation
//!
//! This module provides JavaScript/TypeScript bindings via WebAssembly.
//! The API is organized into submodules by functionality:
//!
//! - `definition_region`: Region manipulation for circuit IO
//! - `schematic`: Core schematic operations
//! - `schematic_builder`: ASCII art schematic construction
//! - `simulation`: MCHPRS simulation (feature-gated)
//! - `typed_executor`: Typed circuit execution (feature-gated)
//! - `circuit_builder`: Fluent executor builder (feature-gated)

mod definition_region;
mod schematic;
mod schematic_builder;

#[cfg(feature = "simulation")]
mod circuit_builder;
#[cfg(feature = "simulation")]
mod simulation;
#[cfg(feature = "simulation")]
mod typed_executor;

// Re-export all public types to maintain the same JS API
pub use definition_region::DefinitionRegionWrapper;
pub use schematic::{
    debug_json_schematic, debug_schematic, BlockStateWrapper, LazyChunkIterator, SchematicWrapper,
};
pub use schematic_builder::SchematicBuilderWrapper;

#[cfg(feature = "simulation")]
pub use circuit_builder::{CircuitBuilderWrapper, SortStrategyWrapper, StateModeConstants};
#[cfg(feature = "simulation")]
pub use simulation::{MchprsWorldWrapper, SimulationOptionsWrapper};
#[cfg(feature = "simulation")]
pub use typed_executor::{
    ExecutionModeWrapper, IoLayoutBuilderWrapper, IoLayoutWrapper, IoTypeWrapper,
    LayoutFunctionWrapper, OutputConditionWrapper, TypedCircuitExecutorWrapper, ValueWrapper,
};

// Re-export the start function
pub use schematic::start;
