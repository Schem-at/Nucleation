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
use crate::world_segment::score::{score, ScoreConfig};
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

        let mut out: Vec<MaterializedBuild> = Vec::with_capacity(builds.len());
        for build in &builds {
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
            out.push(MaterializedBuild { schematic, provenance });
        }

        out.sort_by_key(|b| b.provenance.stable_build_id);
        out
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
