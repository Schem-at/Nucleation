//! Python bindings for Nucleation
//!
//! This module provides Python bindings via PyO3.
//! The API is organized into submodules by functionality:
//!
//! - `schematic`: Core schematic operations
//! - `schematic_builder`: ASCII art schematic construction
//! - `definition_region`: Region manipulation for circuit IO
//! - `simulation`: MCHPRS simulation (feature-gated)
//! - `typed_executor`: Typed circuit execution (feature-gated)
//! - `circuit_builder`: Fluent executor builder (feature-gated)

#![cfg(feature = "python")]

mod definition_region;
mod schematic;
mod schematic_builder;

#[cfg(feature = "simulation")]
mod circuit_builder;
#[cfg(feature = "simulation")]
mod simulation;
#[cfg(feature = "simulation")]
mod typed_executor;

// Re-export all public types
pub use definition_region::PyDefinitionRegion;
pub use schematic::{PyBlockState, PySchematic};
pub use schematic_builder::PySchematicBuilder;

#[cfg(feature = "simulation")]
pub use circuit_builder::{PyCircuitBuilder, PySortStrategy};
#[cfg(feature = "simulation")]
pub use simulation::PyMchprsWorld;
#[cfg(feature = "simulation")]
pub use typed_executor::{
    PyExecutionMode, PyIoLayout, PyIoLayoutBuilder, PyIoType, PyLayoutFunction, PyOutputCondition,
    PyTypedCircuitExecutor, PyValue,
};

// Re-export module functions
pub use schematic::{debug_json_schematic, debug_schematic, load_schematic, save_schematic};

use pyo3::prelude::*;

#[pymodule]
pub fn nucleation(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySchematic>()?;
    m.add_class::<PyBlockState>()?;
    m.add_class::<PyDefinitionRegion>()?;
    m.add_class::<PySchematicBuilder>()?;
    m.add_function(wrap_pyfunction!(debug_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(debug_json_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(load_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(save_schematic, m)?)?;

    #[cfg(feature = "simulation")]
    {
        m.add_class::<PyMchprsWorld>()?;
        m.add_class::<PyValue>()?;
        m.add_class::<PyIoType>()?;
        m.add_class::<PyLayoutFunction>()?;
        m.add_class::<PyOutputCondition>()?;
        m.add_class::<PyExecutionMode>()?;
        m.add_class::<PyIoLayoutBuilder>()?;
        m.add_class::<PyIoLayout>()?;
        m.add_class::<PyTypedCircuitExecutor>()?;
        m.add_class::<PyCircuitBuilder>()?;
        m.add_class::<PySortStrategy>()?;
    }

    Ok(())
}
