//! Deterministic segmentation of a voxel world into discrete builds.
//!
//! Layer 1 is pure: no I/O, no clock, no RNG, no order dependence. Identity is
//! derived from content, never from counters, so results are identical however
//! the work is ordered or sharded.

pub mod ids;
pub mod tile;
pub mod classify;
pub mod profile;
pub mod grid;
pub mod partition;
pub mod segment;

pub use ids::{ClusterId, ContentId, TileId};
pub use tile::{TileBounds, VoxelTile};
pub use classify::{classify, BlockClass};
pub use profile::WorldProfile;
pub use grid::{ComponentLabels, OccupancyGrid};
pub use partition::{PartitionHint, PartitionIndex, PartitionPolicy};
pub use segment::{segment_tile, Cluster, MarginCell, SegConfig, TileSegments};
