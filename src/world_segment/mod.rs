//! Deterministic segmentation of a voxel world into discrete builds.
//!
//! Layer 1 is pure: no I/O, no clock, no RNG, no order dependence. Identity is
//! derived from content, never from counters, so results are identical however
//! the work is ordered or sharded.

pub mod ids;

pub use ids::{ClusterId, ContentId, TileId};
