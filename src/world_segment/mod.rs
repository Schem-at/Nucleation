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
pub mod source;
pub mod world_source;
pub mod targz_source;

pub use ids::{ClusterId, ContentId, TileId};
pub use tile::{TileBounds, VoxelTile};
pub use classify::{classify, BlockClass};
pub use profile::WorldProfile;
pub use grid::{ComponentLabels, OccupancyGrid};
pub use partition::{PartitionHint, PartitionIndex, PartitionPolicy};
pub use segment::{segment_tile, Cluster, MarginCell, SegConfig, TileSegments};
pub use source::{region_tile_bounds, Access, TileError, TileSource};
pub use world_source::WorldSourceTiles;
pub use targz_source::TarGzSource;
