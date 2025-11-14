//! Redstone simulation support using MCHPRS
//!
//! This module is only available when the `simulation` feature is enabled.
//! It provides integration with the MCHPRS redstone simulator for:
//! - Simulating redstone circuits
//! - Generating truth tables
//! - Analyzing circuit behavior
//!
//! # Example
//!
//! ```ignore
//! use nucleation::simulation::{MchprsWorld, generate_truth_table};
//!
//! let schematic = /* load your schematic */;
//! let mut world = MchprsWorld::new(schematic)?;
//!
//! // Interact with levers
//! world.on_use_block(BlockPos::new(0, 1, 0));
//! world.tick(20);
//! world.flush();
//!
//! Check lamp state
//! let is_lit = world.is_lit(BlockPos::new(15, 1, 0));
//! ```

mod mchprs_world;
#[cfg(test)]
mod tests;
mod truth_table;
pub mod typed_executor;

pub use mchprs_world::{MchprsWorld, MchprsWorldError, SimulationOptions};
pub use truth_table::generate_truth_table;

// Re-export commonly used MCHPRS types for convenience
pub use mchprs_blocks::BlockPos;
