//! Deterministic segmentation of a voxel world into discrete builds.
//!
//! Layer 1 is pure: no I/O, no clock, no RNG, no order dependence. Identity is
//! derived from content, never from counters, so results are identical however
//! the work is ordered or sharded.

pub mod classify;
pub mod grid;
pub mod identity;
pub mod ids;
pub mod materialize;
pub mod partition;
pub mod profile;
pub mod provenance;
pub mod runner;
pub mod score;
pub mod segment;
pub mod source;
pub mod stitch;
pub mod store_source;
pub mod targz_source;
pub mod tile;
pub mod world_source;

pub use classify::{classify, BlockClass};
pub use grid::{ComponentLabels, OccupancyGrid};
pub use identity::{bbox_iou, match_snapshots, Outcome, PriorBuild, SnapshotMatch};
pub use ids::{ClusterId, ContentId, TileId};
pub use materialize::{materialize, materialize_with_block_entities, MaterializeCtx};
pub use partition::{PartitionHint, PartitionIndex, PartitionPolicy};
pub use profile::{ProfileParams, WorldProfile};
pub use provenance::{Provenance, StableBuildId};
pub use runner::{MaterializedBuild, RunStats, SegmentJob, WorldSegmenter};
pub use score::{score, ScoreConfig, Scored, Signal, Tier};
pub use segment::{
    segment_tile, segment_tile_membership, Cluster, DisconnectedSplit, MarginCell, SegConfig,
    TileSegments,
};
pub use source::{region_tile_bounds, Access, TileError, TileSource};
pub use stitch::{Build, GlobalCell, MarginEntry, StitchState};
pub use store_source::StoreRegionTiles;
pub use targz_source::{TarArchiveSource, TarGzSource, WorldRect};
pub use tile::{TileBounds, VoxelTile};
pub use world_source::WorldSourceTiles;
