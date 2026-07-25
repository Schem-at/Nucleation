//! World segmentation: deterministic voxel-world -> discrete-build extraction.
//! Bridges [`crate::world_segment::runner::WorldSegmenter`] and its supporting
//! types (job config, partition hints, world profile, run results).
//!
//! Every entry point that walks a world directory (`WsProfile::derive_from_dir`,
//! `WsRunResult::run_dir`) is `#[cfg(not(target_arch = "wasm32"))]`, matching
//! `WorldSource::open_dir` and the `world_stream` bridge module (no filesystem
//! on wasm32).
//!
//! # The panic path
//!
//! [`WorldSegmenter::run_streaming`](crate::world_segment::runner::WorldSegmenter::run_streaming)
//! calls `.expect("tile source failed")` internally — there is no fallible
//! variant in `src/world_segment/`. Letting that `expect` unwind across the FFI
//! boundary is undefined behavior, so `WsRunResult::run_dir` wraps the call in
//! `std::panic::catch_unwind` (the same pattern already used for redpiler
//! compilation in `src/simulation/graph.rs`) and maps a caught panic to
//! `NucleationError::Io`, since the only way `run_streaming` panics today is a
//! failing `TileSource`. This is an interim measure: the proper fix is a
//! `try_run_streaming` in `src/world_segment/runner.rs` that returns
//! `Result<RunStats, TileError>` instead of panicking, which would let this
//! wrapper go away.

#[diplomat::bridge]
pub mod ffi {
    use std::path::Path;

    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    use super::super::shared::ffi::{BlockPos, NucleationError};

    use crate::formats::manager::get_manager;
    use crate::formats::world_stream::WorldSource;
    use crate::world_segment::partition::{PartitionHint, PartitionIndex, PartitionPolicy};
    use crate::world_segment::profile::{ProfileParams, WorldProfile};
    use crate::world_segment::runner::{MaterializedBuild, RunStats, SegmentJob, WorldSegmenter};
    use crate::world_segment::score::{ScoreConfig, Tier};
    use crate::world_segment::segment::SegConfig;
    use crate::world_segment::source::TileError;
    use crate::world_segment::source::TileSource as _;
    use crate::world_segment::tile::VoxelTile;
    use crate::world_segment::world_source::WorldSourceTiles;

    fn utf8(bytes: &[u8]) -> Result<&str, NucleationError> {
        std::str::from_utf8(bytes).map_err(|_| NucleationError::InvalidArgument)
    }

    /// One segmentation run's parameters (the primitive knobs of
    /// [`SegmentJob`](crate::world_segment::runner::SegmentJob), plus a
    /// `hard_cut` flag selecting [`PartitionPolicy`]). Built once, passed by
    /// reference into [`WsRunResult::run_dir`].
    #[diplomat::opaque]
    pub struct WsSegmentJob(pub(crate) SegmentJob);

    impl WsSegmentJob {
        /// `algorithm_version` is pinned to
        /// [`SegConfig::default`](crate::world_segment::segment::SegConfig)'s
        /// value; `score_config` uses `ScoreConfig::default()`. Neither is
        /// exposed as a knob here — construct a `SegmentJob` directly in Rust
        /// if you need to override them.
        #[allow(clippy::too_many_arguments)]
        pub fn create(
            cell_size: u32,
            closing_radius: u32,
            min_cluster_blocks: u64,
            source_id: &DiplomatStr,
            snapshot_id: &DiplomatStr,
            min_y: i32,
            max_y: i32,
            extracted_at: i64,
            match_iou: f32,
            hard_cut: bool,
        ) -> Result<Box<WsSegmentJob>, NucleationError> {
            let source_id = utf8(source_id)?.to_string();
            let snapshot_id = utf8(snapshot_id)?.to_string();
            let config = SegConfig {
                cell_size,
                closing_radius,
                min_cluster_blocks,
                partition_policy: if hard_cut { PartitionPolicy::HardCut } else { PartitionPolicy::Off },
                algorithm_version: SegConfig::default().algorithm_version,
                partition_floor_share: SegConfig::default().partition_floor_share,
            };
            Ok(Box::new(WsSegmentJob(SegmentJob {
                config,
                score_config: ScoreConfig::default(),
                source_id,
                snapshot_id,
                min_y,
                max_y,
                extracted_at,
                match_iou,
            })))
        }
    }

    /// Caller-supplied partition hints (full-column boxes a cluster may never
    /// span under [`PartitionPolicy::HardCut`]). Order does not matter:
    /// [`PartitionIndex::new`](crate::world_segment::partition::PartitionIndex)
    /// sorts hints by full content at construction time.
    #[diplomat::opaque_mut]
    pub struct WsPartitionHints(pub(crate) Vec<PartitionHint>);

    impl WsPartitionHints {
        pub fn create() -> Box<WsPartitionHints> {
            Box::new(WsPartitionHints(Vec::new()))
        }

        /// Add a full-column hint (`y_range: None`) covering inclusive
        /// `x0..=x1, z0..=z1`.
        pub fn add(
            &mut self,
            id: &DiplomatStr,
            x0: i32,
            x1: i32,
            z0: i32,
            z1: i32,
        ) -> Result<(), NucleationError> {
            let id = utf8(id)?.to_string();
            self.0.push(PartitionHint { id, bbox_xz: (x0, x1, z0, z1), y_range: None });
            Ok(())
        }

        pub fn len(&self) -> u32 {
            self.0.len() as u32
        }
    }

    /// A pinned [`WorldProfile`](crate::world_segment::profile::WorldProfile):
    /// the substrate palette + Y band derived (or supplied) once per world and
    /// reused across every segmentation run against it.
    #[diplomat::opaque]
    pub struct WsProfile(pub(crate) WorldProfile);

    impl WsProfile {
        /// Derive a profile from up to `sample` tiles (regions) of a world
        /// directory, in ascending `(x, z)` region order. `coverage` is
        /// `ProfileParams::min_slab_coverage`; every other `ProfileParams`
        /// field uses its default (`sample_stride: 1`, `y_scan: (-64, 320)`).
        #[cfg(not(target_arch = "wasm32"))]
        pub fn derive_from_dir(
            world_dir: &DiplomatStr,
            min_y: i32,
            max_y: i32,
            sample: u32,
            coverage: f32,
        ) -> Result<Box<WsProfile>, NucleationError> {
            let dir = utf8(world_dir)?;
            let source =
                WorldSource::open_dir(Path::new(dir)).map_err(|_| NucleationError::Io)?;
            let tiles = WorldSourceTiles::new(source, min_y, max_y);

            let limit = sample.max(1) as usize;
            let mut samples: Vec<VoxelTile> = Vec::new();
            tiles
                .for_each_tile(&mut |tile| {
                    samples.push(tile);
                    if samples.len() >= limit {
                        Err(TileError::Stop)
                    } else {
                        Ok(())
                    }
                })
                .map_err(|_| NucleationError::Io)?;

            let params = ProfileParams { min_slab_coverage: coverage, ..ProfileParams::default() };
            Ok(Box::new(WsProfile(WorldProfile::derive(&samples, &params))))
        }

        /// The derived substrate Y band's lower bound (inclusive).
        pub fn band_min(&self) -> i32 {
            self.0.substrate_y_band.0
        }

        /// The derived substrate Y band's upper bound (inclusive).
        pub fn band_max(&self) -> i32 {
            self.0.substrate_y_band.1
        }

        /// Number of distinct block names in the substrate palette.
        pub fn palette_len(&self) -> u32 {
            self.0.substrate_palette.len() as u32
        }

        /// The substrate palette as a JSON array of block-name strings.
        pub fn write_palette_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let names: Vec<&String> = self.0.substrate_palette.iter().collect();
            let json = serde_json::to_string(&names).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }
    }

    /// The materialized output of one segmentation run: every build (in the
    /// pipeline's deterministic stable-id order) plus the aggregate
    /// [`RunStats`](crate::world_segment::runner::RunStats).
    #[diplomat::opaque]
    pub struct WsRunResult {
        builds: Vec<MaterializedBuild>,
        stats: RunStats,
    }

    impl WsRunResult {
        /// Run the full pipeline (source -> segment -> stitch -> score ->
        /// identity -> materialize) over a world directory. No prior-snapshot
        /// builds are supplied, so every build seeds a fresh stable id (see
        /// `StableBuildId::seed`).
        ///
        /// See the module docs for why this catches a panic instead of
        /// propagating it.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn run_dir(
            job: &WsSegmentJob,
            hints: &WsPartitionHints,
            profile: &WsProfile,
            world_dir: &DiplomatStr,
        ) -> Result<Box<WsRunResult>, NucleationError> {
            let dir = utf8(world_dir)?;
            let source =
                WorldSource::open_dir(Path::new(dir)).map_err(|_| NucleationError::Io)?;
            let tiles = WorldSourceTiles::new(source, job.0.min_y, job.0.max_y);
            let partitions = PartitionIndex::new(hints.0.clone());

            let job_ref = &job.0;
            let profile_ref = &profile.0;
            let tiles_ref = &tiles;
            let partitions_ref = &partitions;

            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut builds: Vec<MaterializedBuild> = Vec::new();
                let stats = WorldSegmenter::run_streaming(
                    tiles_ref,
                    profile_ref,
                    partitions_ref,
                    job_ref,
                    &[],
                    &mut |mb| builds.push(mb),
                );
                (builds, stats)
            }));

            match outcome {
                Ok((builds, stats)) => Ok(Box::new(WsRunResult { builds, stats })),
                // The only documented panic in `run_streaming` is the tile
                // source's `.expect("tile source failed")`; there is no
                // richer error to recover from a caught panic payload.
                Err(_) => Err(NucleationError::Io),
            }
        }

        /// Total builds materialized (same as `build_count`, from `RunStats`).
        pub fn builds(&self) -> u64 {
            self.stats.builds
        }

        pub fn tier_confident(&self) -> u64 {
            self.stats.tier_confident
        }

        pub fn tier_probable(&self) -> u64 {
            self.stats.tier_probable
        }

        pub fn tier_debris(&self) -> u64 {
            self.stats.tier_debris
        }

        pub fn cross_tile(&self) -> u64 {
            self.stats.cross_tile
        }

        pub fn largest_block_count(&self) -> u64 {
            self.stats.largest_block_count
        }

        /// Number of builds held in this result (indices `0..build_count()`
        /// are valid for every per-index accessor below).
        pub fn build_count(&self) -> u32 {
            self.builds.len() as u32
        }

        fn get(&self, index: u32) -> Result<&MaterializedBuild, NucleationError> {
            self.builds.get(index as usize).ok_or(NucleationError::NotFound)
        }

        /// The build's stable id (hex), stable across re-runs against the
        /// same source under the same config, absent a prior-snapshot match.
        pub fn stable_id_hex(
            &self,
            index: u32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let mb = self.get(index)?;
            let _ = write!(out, "{}", mb.provenance.stable_build_id);
            Ok(())
        }

        /// The build's content fingerprint, as 32 lowercase hex digits (u128,
        /// big-endian).
        pub fn fingerprint_hex(
            &self,
            index: u32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let mb = self.get(index)?;
            let _ = write!(out, "{:032x}", mb.provenance.fingerprint);
            Ok(())
        }

        /// `0` = Confident, `1` = Probable, `2` = Debris.
        pub fn tier_of(&self, index: u32) -> Result<u8, NucleationError> {
            let mb = self.get(index)?;
            Ok(match mb.provenance.tier {
                Tier::Confident => 0,
                Tier::Probable => 1,
                Tier::Debris => 2,
            })
        }

        pub fn block_count_of(&self, index: u32) -> Result<u64, NucleationError> {
            Ok(self.get(index)?.provenance.block_count)
        }

        /// The build's world-space bounding box minimum (inclusive).
        pub fn bbox_min_of(&self, index: u32) -> Result<BlockPos, NucleationError> {
            let (min, _max) = self.get(index)?.provenance.world_bbox;
            Ok(BlockPos { x: min.0, y: min.1, z: min.2 })
        }

        /// The build's world-space bounding box maximum (inclusive).
        pub fn bbox_max_of(&self, index: u32) -> Result<BlockPos, NucleationError> {
            let (_min, max) = self.get(index)?.provenance.world_bbox;
            Ok(BlockPos { x: max.0, y: max.1, z: max.2 })
        }

        /// Save the build's schematic to a file, picking the format from the
        /// file extension — same serializer as
        /// [`Schematic::save_to_file`](super::super::schematic::ffi::Schematic::save_to_file).
        /// Not available in JS: the WASM build has no filesystem.
        #[cfg(not(target_arch = "wasm32"))]
        #[diplomat::attr(js, disable)]
        pub fn write_schem_to(&self, index: u32, path: &DiplomatStr) -> Result<(), NucleationError> {
            let mb = self.get(index)?;
            let path = utf8(path)?;
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let bytes = manager
                .write_auto_with_settings(path, &mb.schematic, None, None)
                .map_err(|_| NucleationError::Serialize)?;
            std::fs::write(path, bytes).map_err(|_| NucleationError::Io)?;
            Ok(())
        }
    }
}
