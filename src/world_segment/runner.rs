//! Task 5: `WorldSegmenter` — the single-process runner that ties the whole
//! pipeline together (source -> segment -> stitch -> score -> identity ->
//! materialize) into one deterministic, order-independent run.
//!
//! No clock reads (`extracted_at` is an input carried on [`SegmentJob`]), no
//! RNG, and no `HashMap`/`HashSet` whose iteration order could reach output —
//! only `BTreeMap`, matching the rest of the module. The result does not
//! depend on the order `TileSource::for_each_tile` visits tiles in, because
//! every downstream step (`StitchState::merge`, `Vec<Build>` sorted by id,
//! `match_snapshots` sorted by `build_id`) is itself order-independent.

use std::collections::BTreeMap;

use crate::block_state::BlockState;
use crate::universal_schematic::UniversalSchematic;
use crate::world_segment::identity::{match_snapshots, PriorBuild};
use crate::world_segment::ids::ClusterId;
use crate::world_segment::materialize::{materialize, MaterializeCtx};
use crate::world_segment::partition::PartitionIndex;
use crate::world_segment::profile::WorldProfile;
use crate::world_segment::provenance::Provenance;
use crate::world_segment::score::{score, ScoreConfig, Tier};
use crate::world_segment::segment::{segment_tile_membership, SegConfig};
use crate::world_segment::source::TileSource;
use crate::world_segment::stitch::StitchState;

/// Parameters for one segmentation run.
///
/// `extracted_at` is a caller-supplied unix-seconds timestamp — never
/// `SystemTime::now()` — so a run can be replayed byte-for-byte.
#[derive(Clone, Debug)]
pub struct SegmentJob {
    pub config: SegConfig,
    pub score_config: ScoreConfig,
    pub source_id: String,
    pub snapshot_id: String,
    pub min_y: i32,
    pub max_y: i32,
    pub extracted_at: i64,
    pub match_iou: f32,
}

/// One finished build: its schematic plus the provenance envelope describing
/// where it came from.
pub struct MaterializedBuild {
    pub schematic: UniversalSchematic,
    pub provenance: Provenance,
}

/// Aggregate counters produced by a run, without holding every materialized
/// build in memory at once. Populated identically by `run` and
/// `run_streaming` (the former simply also collects the builds).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunStats {
    pub builds: u64,
    pub tier_confident: u64,
    pub tier_probable: u64,
    pub tier_debris: u64,
    pub cross_tile: u64,
    pub largest_block_count: u64,
}

/// Single-process pipeline runner: streams every tile from `source` through
/// segmentation, stitches the results into whole builds, scores and
/// identity-matches them, and materializes each into a schematic.
pub struct WorldSegmenter;

impl WorldSegmenter {
    pub fn run(
        source: &dyn TileSource,
        profile: &WorldProfile,
        partitions: &PartitionIndex,
        job: &SegmentJob,
        prior: &[PriorBuild],
    ) -> Vec<MaterializedBuild> {
        let mut out = Vec::new();
        Self::run_streaming(source, profile, partitions, job, prior, &mut |mb| out.push(mb));
        out
    }

    /// Same pipeline as [`Self::run`], but emits each build to `emit` and
    /// drops it immediately after, instead of accumulating a `Vec` — so a
    /// whole-world run doesn't hold every output schematic in memory at
    /// once. Builds are still emitted in the same deterministic order `run`
    /// returns them in (sorted by stable build id).
    pub fn run_streaming(
        source: &dyn TileSource,
        profile: &WorldProfile,
        partitions: &PartitionIndex,
        job: &SegmentJob,
        prior: &[PriorBuild],
        emit: &mut dyn FnMut(MaterializedBuild),
    ) -> RunStats {
        let mut stitch = StitchState::empty();
        // Every surviving (non-substrate, non-dropped-cluster) block, grouped
        // by the per-tile ClusterId it belonged to before stitching. A build's
        // final block set is the union of its `cluster_ids`' entries here.
        let mut blocks_by_cluster: BTreeMap<ClusterId, BTreeMap<(i32, i32, i32), BlockState>> =
            BTreeMap::new();

        source
            .for_each_tile(&mut |tile| {
                let (segs, membership) =
                    segment_tile_membership(&tile, profile, &job.config, partitions);
                stitch = StitchState::merge(
                    std::mem::replace(&mut stitch, StitchState::empty()),
                    StitchState::from(&segs, job.config.cell_size, job.min_y),
                    job.config.closing_radius,
                );

                // Built once per tile, not once per membership entry.
                let tile_blocks: BTreeMap<(i32, i32, i32), BlockState> =
                    tile.blocks().map(|(p, b)| (p, b.clone())).collect();
                for (pos, cid) in membership {
                    if let Some(block) = tile_blocks.get(&pos) {
                        blocks_by_cluster.entry(cid).or_default().insert(pos, block.clone());
                    }
                }

                Ok(())
            })
            // Acceptable for this task: a failing source aborts the run rather
            // than partially materializing. See Task 5's report for the note.
            .expect("tile source failed");

        let builds = stitch.finish();

        let matches = match_snapshots(&builds, prior, &job.source_id, job.match_iou);
        let stable_by_build: BTreeMap<ClusterId, crate::world_segment::provenance::StableBuildId> =
            matches.into_iter().map(|m| (m.build_id, m.stable_id)).collect();

        let config_hash = job.config.config_hash(profile, partitions);
        let profile_hash = profile.profile_hash();

        // Sort builds by stable id up front so they're emitted in the same
        // deterministic order `run` used to return them in.
        let mut ordered_builds: Vec<&crate::world_segment::stitch::Build> = builds.iter().collect();
        ordered_builds.sort_by_key(|build| {
            *stable_by_build
                .get(&build.id)
                .expect("match_snapshots returns exactly one match per current build")
        });

        let mut stats = RunStats::default();
        for build in ordered_builds {
            // Union the blocks of every cluster this build absorbed.
            let mut blocks: BTreeMap<(i32, i32, i32), BlockState> = BTreeMap::new();
            for cid in &build.cluster_ids {
                if let Some(cluster_blocks) = blocks_by_cluster.get(cid) {
                    for (pos, b) in cluster_blocks {
                        blocks.insert(*pos, b.clone());
                    }
                }
            }

            let scored = score(build, &job.score_config);
            let stable_id = *stable_by_build
                .get(&build.id)
                .expect("match_snapshots returns exactly one match per current build");

            let ctx = MaterializeCtx {
                source_id: &job.source_id,
                snapshot_id: &job.snapshot_id,
                config_hash,
                profile_hash,
                extracted_at: job.extracted_at,
            };
            let (schematic, provenance) =
                materialize(build, &blocks, scored.tier, stable_id, &ctx);

            stats.builds += 1;
            match scored.tier {
                Tier::Confident => stats.tier_confident += 1,
                Tier::Probable => stats.tier_probable += 1,
                Tier::Debris => stats.tier_debris += 1,
            }
            if build.cluster_ids.len() > 1 {
                stats.cross_tile += 1;
            }
            stats.largest_block_count = stats.largest_block_count.max(build.block_count);

            emit(MaterializedBuild { schematic, provenance });
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::world_segment::ids::TileId;
    use crate::world_segment::partition::PartitionIndex;
    use crate::world_segment::profile::WorldProfile;
    use crate::world_segment::score::{ScoreConfig, Tier};
    use crate::world_segment::segment::SegConfig;
    use crate::world_segment::source::{Access, TileError, TileSource};
    use crate::world_segment::tile::{TileBounds, VoxelTile};
    use crate::BlockState;

    /// A source that yields exactly one pre-built tile.
    struct MemSource {
        id: TileId,
        bounds: TileBounds,
        blocks: Vec<((i32, i32, i32), BlockState)>,
    }

    impl TileSource for MemSource {
        fn access(&self) -> Access {
            Access::Forward
        }

        fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
            Err(TileError::NotRandomAccess)
        }

        fn tile(&self, _id: TileId) -> Result<Option<VoxelTile>, TileError> {
            Err(TileError::NotRandomAccess)
        }

        fn for_each_tile(
            &self,
            f: &mut dyn FnMut(VoxelTile) -> Result<(), TileError>,
        ) -> Result<(), TileError> {
            let tile = VoxelTile::from_blocks(self.id, self.bounds, self.blocks.iter().cloned());
            f(tile)
        }
    }

    fn profile() -> WorldProfile {
        WorldProfile::new(
            ["minecraft:stone"].iter().map(|s| s.to_string()).collect(),
            (-64, -50),
        )
    }

    #[test]
    fn run_streaming_emits_each_build_and_counts_stats() {
        let mut blocks: Vec<((i32, i32, i32), BlockState)> = Vec::new();
        // Flat stone substrate slab, 16x16, at y = -60 (inside the profile's band).
        for x in 0..16 {
            for z in 0..16 {
                blocks.push(((x, -60, z), BlockState::new("minecraft:stone")));
            }
        }
        // One small artificial build, standing on the slab: a redstone wire next
        // to a repeater, one block apart, forming a single cluster.
        blocks.push(((5, -59, 5), BlockState::new("minecraft:redstone_wire")));
        blocks.push(((6, -59, 5), BlockState::new("minecraft:repeater")));
        // A 1-block debris speck, far from the build, on the substrate.
        blocks.push(((14, -59, 14), BlockState::new("minecraft:redstone_wire")));

        let source = MemSource {
            id: TileId { x: 0, z: 0 },
            bounds: TileBounds { min: (0, -64, 0), max: (15, 63, 15) },
            blocks,
        };

        let profile = profile();
        let partitions = PartitionIndex::new(vec![]);
        let job = SegmentJob {
            config: SegConfig::default(),
            score_config: ScoreConfig::default(),
            source_id: "src".to_string(),
            snapshot_id: "snap1".to_string(),
            min_y: -64,
            max_y: 63,
            extracted_at: 1_700_000_000,
            match_iou: 0.5,
        };

        let mut emitted: Vec<MaterializedBuild> = Vec::new();
        let stats = WorldSegmenter::run_streaming(
            &source,
            &profile,
            &partitions,
            &job,
            &[],
            &mut |mb| emitted.push(mb),
        );

        assert_eq!(stats.builds, emitted.len() as u64);
        assert!(stats.tier_debris >= 1, "the speck should be scored as debris");

        let expected = WorldSegmenter::run(&source, &profile, &partitions, &job, &[]);
        let mut expected_provenance: Vec<Provenance> =
            expected.into_iter().map(|mb| mb.provenance).collect();
        let mut emitted_provenance: Vec<Provenance> =
            emitted.into_iter().map(|mb| mb.provenance).collect();
        expected_provenance.sort_by_key(|p| p.stable_build_id);
        emitted_provenance.sort_by_key(|p| p.stable_build_id);
        assert_eq!(emitted_provenance, expected_provenance);
    }

    #[test]
    fn single_tile_run_materializes_one_build() {
        let mut blocks: Vec<((i32, i32, i32), BlockState)> = Vec::new();
        // Flat stone substrate slab, 16x16, at y = -60 (inside the profile's band).
        for x in 0..16 {
            for z in 0..16 {
                blocks.push(((x, -60, z), BlockState::new("minecraft:stone")));
            }
        }
        // One small artificial build, standing on the slab: a redstone wire next
        // to a repeater, one block apart, forming a single cluster.
        blocks.push(((5, -59, 5), BlockState::new("minecraft:redstone_wire")));
        blocks.push(((6, -59, 5), BlockState::new("minecraft:repeater")));

        let source = MemSource {
            id: TileId { x: 0, z: 0 },
            bounds: TileBounds { min: (0, -64, 0), max: (15, 63, 15) },
            blocks,
        };

        let profile = profile();
        let partitions = PartitionIndex::new(vec![]);
        let job = SegmentJob {
            config: SegConfig::default(),
            score_config: ScoreConfig::default(),
            source_id: "src".to_string(),
            snapshot_id: "snap1".to_string(),
            min_y: -64,
            max_y: 63,
            extracted_at: 1_700_000_000,
            match_iou: 0.5,
        };

        let out = WorldSegmenter::run(&source, &profile, &partitions, &job, &[]);

        assert_eq!(out.len(), 1, "exactly one build should be materialized");
        let mb = &out[0];

        assert_eq!(mb.provenance.world_bbox, ((5, -59, 5), (6, -59, 5)));
        assert_eq!(mb.provenance.origin_offset, (5, -59, 5));
        assert_eq!(mb.provenance.block_count, 2);

        // 2 blocks is <= ScoreConfig::default().debris_max_blocks (100), so the
        // build is scored as Debris.
        assert_eq!(mb.provenance.tier, Tier::Debris);

        // The schematic is local-origin normalized: world (5,-59,5) -> (0,0,0).
        assert_eq!(
            mb.schematic.get_block(0, 0, 0).map(|b| b.get_name().to_string()),
            Some("minecraft:redstone_wire".to_string())
        );
        assert_eq!(
            mb.schematic.get_block(1, 0, 0).map(|b| b.get_name().to_string()),
            Some("minecraft:repeater".to_string())
        );
    }
}
