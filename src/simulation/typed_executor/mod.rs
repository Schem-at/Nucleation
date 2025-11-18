//! Typed circuit executor for high-level redstone circuit interaction
//!
//! This module provides a type-safe interface for interacting with redstone circuits,
//! allowing you to define inputs/outputs with semantic types (integers, floats, arrays, etc.)
//! and automatically handle the conversion to/from physical redstone signals.
//!
//! # Architecture
//!
//! The system works in two stages:
//! 1. **Type → Binary**: Convert high-level types (u32, ascii, etc.) to bit arrays
//! 2. **Binary → Physical**: Spread bits across physical redstone positions (nibbles 0-15)
//!
//! # Features
//!
//! ## Type System
//! - **Scalars**: `UnsignedInt`, `SignedInt`, `Float32`, `Boolean`, `Ascii`
//! - **Collections**: `Array`, `Matrix`, `PixelBuffer`, `Struct`
//! - **Extensible**: Easy to add new types
//!
//! ## Layout Functions
//! - **OneToOne**: 1 bit per nibble (traditional redstone)
//! - **Packed4**: 4 bits per nibble (efficient)
//! - **Custom**: Arbitrary bit-to-position mappings
//! - **2D Layouts**: RowMajor, ColumnMajor, Scanline, Tiled
//! - **Chunked**: For streaming data
//!
//! ## Execution Modes
//! - **FixedTicks**: Run for a specific number of ticks
//! - **UntilCondition**: Run until an output meets a condition
//! - **UntilStable**: Run until outputs stabilize
//! - **UntilChange**: Run until any output changes
//!
//! ## State Management
//! - **Stateless**: Reset before each execution (default)
//! - **Stateful**: Preserve state between executions
//! - **Manual**: User-controlled reset
//!
//! # Quick Start
//!
//! ```ignore
//! use nucleation::simulation::typed_executor::*;
//! use std::collections::HashMap;
//!
//! // Build IO layout
//! let layout = IoLayoutBuilder::new()
//!     .add_input(
//!         "a",
//!         IoType::UnsignedInt { bits: 32 },
//!         LayoutFunction::Packed4,  // 8 nibbles instead of 32!
//!         positions_a
//!     )?
//!     .add_output(
//!         "result",
//!         IoType::UnsignedInt { bits: 32 },
//!         LayoutFunction::Packed4,
//!         positions_result
//!     )?
//!     .build();
//!
//! // Create executor from world and layout
//! let mut executor = TypedCircuitExecutor::from_layout(world, layout);
//!
//! // Execute with typed inputs
//! let mut inputs = HashMap::new();
//! inputs.insert("a".to_string(), Value::U32(42));
//!
//! let result = executor.execute(
//!     inputs,
//!     ExecutionMode::FixedTicks { ticks: 100 }
//! )?;
//!
//! // Get typed output
//! let output = result.outputs.get("result").unwrap();
//! assert_eq!(*output, Value::U32(42));
//! ```
//!
//! # Advanced Usage
//!
//! ## Conditional Execution
//!
//! ```ignore
//! // Run until "done" flag is true
//! let result = executor.execute(
//!     inputs,
//!     ExecutionMode::UntilCondition {
//!         output_name: "done".to_string(),
//!         condition: OutputCondition::Equals(Value::Bool(true)),
//!         max_ticks: 1000,
//!         check_interval: 10,
//!     }
//! )?;
//!
//! if result.condition_met {
//!     println!("Completed in {} ticks", result.ticks_elapsed);
//! }
//! ```
//!
//! ## Stateful Circuits
//!
//! ```ignore
//! // Preserve state between executions (e.g., for counters, memory)
//! executor.set_state_mode(StateMode::Stateful);
//!
//! executor.execute(inputs1, mode)?;  // State preserved
//! executor.execute(inputs2, mode)?;  // Continues from previous state
//! ```
//!
//! ## Auto Layout Inference
//!
//! ```ignore
//! // Automatically infer OneToOne or Packed4 based on position count
//! let layout = IoLayoutBuilder::new()
//!     .add_input_auto("a", IoType::UnsignedInt { bits: 32 }, positions)?
//!     .build();
//! ```

mod executor;
pub mod insign_io;
mod io_layout_builder;
mod io_mapping;
mod io_type;
mod layout_function;
mod value;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

// Public API
pub use executor::{
    ExecutionMode, ExecutionResult, OutputCondition, StateMode, TypedCircuitExecutor,
};
pub use insign_io::{
    create_executor_from_insign, create_executor_from_insign_with_options,
    parse_io_layout_from_insign, InsignIoError, SortStrategy,
};
pub use io_layout_builder::{IoLayout, IoLayoutBuilder};
pub use io_mapping::IoMapping;
pub use io_type::IoType;
pub use layout_function::LayoutFunction;
pub use value::Value;
