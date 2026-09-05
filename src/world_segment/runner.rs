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

use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::block_state::BlockState;
use crate::universal_schematic::UniversalSchematic;
use crate::world_segment::identity::{match_snapshots, PriorBuild};
use crate::world_segment::ids::ClusterId;
use crate::world_segment::materialize::{materialize_with_block_entities, MaterializeCtx};
use crate::world_segment::partition::PartitionIndex;
use crate::world_segment::profile::WorldProfile;
use crate::world_segment::provenance::Provenance;
use crate::world_segment::score::{score, ScoreConfig, Tier};
use crate::world_segment::segment::{segment_tile_membership, SegConfig};
use crate::world_segment::source::{TileError, TileSource};
use crate::world_segment::stitch::StitchState;
use crate::Connectivity;

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
    /// Local schematic positions added only by `preserve_support_blocks`.
    ///
    /// Consumers that further split a materialized build should use the
    /// support-aware helpers below. They split with these positions absent,
    /// then add each support to the component containing its block above, so
    /// a retained floor footprint can never reconnect independent builds.
    pub support_positions: Vec<BlockPosition>,
}

impl MaterializedBuild {
    fn split_support_aware(
        &self,
        split: impl FnOnce(&UniversalSchematic) -> Vec<UniversalSchematic>,
    ) -> Vec<UniversalSchematic> {
        if self.support_positions.is_empty() {
            return split(&self.schematic);
        }

        let mut basis = self.schematic.clone();
        let air = BlockState::new("minecraft:air");
        for pos in &self.support_positions {
            basis.set_block(pos.x, pos.y, pos.z, &air);
        }
        let mut pieces = split(&basis);
        if pieces.len() <= 1 {
            return vec![self.schematic.clone()];
        }

        for support in &self.support_positions {
            let Some(above_y) = support.y.checked_add(1) else {
                continue;
            };
            let Some(piece) = pieces.iter_mut().find(|piece| {
                piece
                    .get_block(support.x, above_y, support.z)
                    .is_some_and(|state| state.get_name() != "minecraft:air")
            }) else {
                continue;
            };
            if let Some(state) = self.schematic.get_block(support.x, support.y, support.z) {
                piece.set_block(support.x, support.y, support.z, &state.clone());
            }
            if let Some(entity) = self.schematic.get_block_entity_owned(*support) {
                piece.set_block_entity(*support, entity);
            }
        }
        for piece in &mut pieces {
            piece.default_region = piece.default_region.to_compact();
        }
        pieces
    }

    /// Exact connected-component split in which retained supports are added
    /// only after component identity has been decided.
    pub fn split_connected_support_aware(
        &self,
        connectivity: Connectivity,
    ) -> Vec<UniversalSchematic> {
        self.split_support_aware(|basis| basis.split_connected(connectivity))
    }

    /// Nearest-core split in which retained supports cannot merge components.
    pub fn split_connected_attach_support_aware(
        &self,
        connectivity: Connectivity,
        min_core_blocks: usize,
    ) -> Vec<UniversalSchematic> {
        self.split_support_aware(|basis| {
            basis.split_connected_attach(connectivity, min_core_blocks)
        })
    }

    /// Nearby-fixture split in which retained supports cannot merge
    /// components.
    pub fn split_connected_attach_nearby_support_aware(
        &self,
        connectivity: Connectivity,
        min_core_blocks: usize,
        max_gap: u32,
    ) -> Vec<UniversalSchematic> {
        self.split_support_aware(|basis| {
            basis.split_connected_attach_nearby(connectivity, min_core_blocks, max_gap)
        })
    }
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
        Self::run_streaming(source, profile, partitions, job, prior, &mut |mb| {
            out.push(mb)
        });
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
        Self::try_run_streaming(source, profile, partitions, job, prior, emit)
            .expect("tile source failed")
    }

    /// Fallible worker API. A source failure is returned before any build is
    /// emitted, so callers cannot mistake a partial read for deleted builds.
    pub fn try_run_streaming(
        source: &dyn TileSource,
        profile: &WorldProfile,
        partitions: &PartitionIndex,
        job: &SegmentJob,
        prior: &[PriorBuild],
        emit: &mut dyn FnMut(MaterializedBuild),
    ) -> Result<RunStats, TileError> {
        let mut stitch = StitchState::empty();
        // Every surviving (non-substrate, non-dropped-cluster) block, grouped
        // by the per-tile ClusterId it belonged to before stitching. A build's
        // final block set is the union of its `cluster_ids`' entries here.
        let mut blocks_by_cluster: BTreeMap<ClusterId, BTreeMap<(i32, i32, i32), BlockState>> =
            BTreeMap::new();
        let mut block_entities_by_cluster: BTreeMap<
            ClusterId,
            BTreeMap<(i32, i32, i32), BlockEntity>,
        > = BTreeMap::new();
        let mut supports_by_cluster: BTreeMap<ClusterId, BTreeMap<(i32, i32, i32), BlockState>> =
            BTreeMap::new();
        let mut support_entities_by_cluster: BTreeMap<
            ClusterId,
            BTreeMap<(i32, i32, i32), BlockEntity>,
        > = BTreeMap::new();

        source.for_each_tile(&mut |tile| {
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
            let membership_by_pos: BTreeMap<(i32, i32, i32), ClusterId> =
                membership.iter().map(|(pos, cid)| (*pos, *cid)).collect();
            let mut support_membership_by_pos = BTreeMap::new();
            for (&pos, &cid) in &membership_by_pos {
                if let Some(block) = tile_blocks.get(&pos) {
                    blocks_by_cluster
                        .entry(cid)
                        .or_default()
                        .insert(pos, block.clone());
                }
                if job.config.preserve_support_blocks {
                    let Some(support_y) = pos.1.checked_sub(1) else {
                        continue;
                    };
                    let support = (pos.0, support_y, pos.2);
                    // A block that already survived subtraction is normal
                    // build content, not support-only enrichment.
                    if membership_by_pos.contains_key(&support) {
                        continue;
                    }
                    if let Some(block) = tile_blocks.get(&support) {
                        supports_by_cluster
                            .entry(cid)
                            .or_default()
                            .insert(support, block.clone());
                        support_membership_by_pos.insert(support, cid);
                    }
                }
            }
            for block_entity in tile.block_entities() {
                if let Some(cid) = membership_by_pos.get(&block_entity.position) {
                    block_entities_by_cluster
                        .entry(*cid)
                        .or_default()
                        .insert(block_entity.position, block_entity.clone());
                } else if let Some(cid) = support_membership_by_pos.get(&block_entity.position) {
                    support_entities_by_cluster
                        .entry(*cid)
                        .or_default()
                        .insert(block_entity.position, block_entity.clone());
                }
            }

            Ok(())
        })?;

        let builds = stitch.finish();

        let matches = match_snapshots(&builds, prior, &job.source_id, job.match_iou);
        let stable_by_build: BTreeMap<ClusterId, crate::world_segment::provenance::StableBuildId> =
            matches
                .into_iter()
                .map(|m| (m.build_id, m.stable_id))
                .collect();

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
            let mut block_entities: BTreeMap<(i32, i32, i32), BlockEntity> = BTreeMap::new();
            let mut supports: BTreeMap<(i32, i32, i32), BlockState> = BTreeMap::new();
            let mut support_entities: BTreeMap<(i32, i32, i32), BlockEntity> = BTreeMap::new();
            for cid in &build.cluster_ids {
                if let Some(cluster_blocks) = blocks_by_cluster.get(cid) {
                    for (pos, b) in cluster_blocks {
                        blocks.insert(*pos, b.clone());
                    }
                }
                if let Some(cluster_entities) = block_entities_by_cluster.get(cid) {
                    for (pos, block_entity) in cluster_entities {
                        block_entities.insert(*pos, block_entity.clone());
                    }
                }
                if let Some(cluster_supports) = supports_by_cluster.get(cid) {
                    for (pos, block) in cluster_supports {
                        supports.insert(*pos, block.clone());
                    }
                }
                if let Some(cluster_entities) = support_entities_by_cluster.get(cid) {
                    for (pos, block_entity) in cluster_entities {
                        support_entities.insert(*pos, block_entity.clone());
                    }
                }
            }

            // A support is enrichment only if the final stitched build did
            // not already contain that position as ordinary build content.
            supports.retain(|pos, _| !blocks.contains_key(pos));
            support_entities.retain(|pos, _| supports.contains_key(pos));
            let mut output_blocks = blocks;
            output_blocks.extend(supports.iter().map(|(pos, state)| (*pos, state.clone())));
            let mut output_entities = block_entities;
            output_entities.extend(
                support_entities
                    .iter()
                    .map(|(pos, entity)| (*pos, entity.clone())),
            );

            let mut output_build = build.clone();
            for &(x, y, z) in supports.keys() {
                output_build.bbox.0 .0 = output_build.bbox.0 .0.min(x);
                output_build.bbox.0 .1 = output_build.bbox.0 .1.min(y);
                output_build.bbox.0 .2 = output_build.bbox.0 .2.min(z);
                output_build.bbox.1 .0 = output_build.bbox.1 .0.max(x);
                output_build.bbox.1 .1 = output_build.bbox.1 .1.max(y);
                output_build.bbox.1 .2 = output_build.bbox.1 .2.max(z);
            }
            output_build.block_count = output_blocks.len() as u64;

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
            let (mut schematic, provenance) = materialize_with_block_entities(
                &output_build,
                &output_blocks,
                &output_entities,
                scored.tier,
                stable_id,
                &ctx,
            );
            if let Some(embedded) = schematic.metadata.provenance.as_mut() {
                embedded.attributes.insert(
                    "nucleation:preserve_support_blocks".to_string(),
                    job.config.preserve_support_blocks.to_string(),
                );
                embedded.attributes.insert(
                    "nucleation:support_block_count".to_string(),
                    supports.len().to_string(),
                );
            }

            stats.builds += 1;
            match scored.tier {
                Tier::Confident => stats.tier_confident += 1,
                Tier::Probable => stats.tier_probable += 1,
                Tier::Debris => stats.tier_debris += 1,
            }
            if build.cluster_ids.len() > 1 {
                stats.cross_tile += 1;
            }
            stats.largest_block_count = stats.largest_block_count.max(output_build.block_count);

            let origin = output_build.bbox.0;
            let support_positions = supports
                .keys()
                .map(|&(x, y, z)| BlockPosition::new(x - origin.0, y - origin.1, z - origin.2))
                .collect();

            emit(MaterializedBuild {
                schematic,
                provenance,
                support_positions,
            });
        }

        Ok(stats)
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
            bounds: TileBounds {
                min: (0, -64, 0),
                max: (15, 63, 15),
            },
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
        let stats =
            WorldSegmenter::run_streaming(&source, &profile, &partitions, &job, &[], &mut |mb| {
                emitted.push(mb)
            });

        assert_eq!(stats.builds, emitted.len() as u64);
        assert!(
            stats.tier_debris >= 1,
            "the speck should be scored as debris"
        );

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
            bounds: TileBounds {
                min: (0, -64, 0),
                max: (15, 63, 15),
            },
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
            mb.schematic
                .get_block(0, 0, 0)
                .map(|b| b.get_name().to_string()),
            Some("minecraft:redstone_wire".to_string())
        );
        assert_eq!(
            mb.schematic
                .get_block(1, 0, 0)
                .map(|b| b.get_name().to_string()),
            Some("minecraft:repeater".to_string())
        );
    }

    #[test]
    fn support_policy_enriches_exactly_one_layer_after_segmentation() {
        let source = MemSource {
            id: TileId { x: 0, z: 0 },
            bounds: TileBounds {
                min: (0, -64, 0),
                max: (15, 63, 15),
            },
            blocks: vec![
                ((5, -61, 5), BlockState::new("minecraft:stone")),
                ((5, -60, 5), BlockState::new("minecraft:stone")),
                ((5, -59, 5), BlockState::new("minecraft:redstone_wire")),
            ],
        };
        let job = SegmentJob {
            config: SegConfig {
                preserve_support_blocks: true,
                ..SegConfig::default()
            },
            score_config: ScoreConfig::default(),
            source_id: "src".to_string(),
            snapshot_id: "snap1".to_string(),
            min_y: -64,
            max_y: 63,
            extracted_at: 1_700_000_000,
            match_iou: 0.5,
        };

        let out = WorldSegmenter::run(&source, &profile(), &PartitionIndex::new(vec![]), &job, &[]);
        assert_eq!(out.len(), 1);
        let mb = &out[0];
        assert_eq!(mb.provenance.block_count, 2);
        assert_eq!(mb.provenance.world_bbox, ((5, -60, 5), (5, -59, 5)));
        assert_eq!(mb.support_positions.len(), 1);
        assert_eq!(mb.support_positions[0].to_tuple(), (0, 0, 0));
        let embedded = mb.schematic.metadata.provenance.as_ref().unwrap();
        assert_eq!(
            embedded
                .attributes
                .get("nucleation:support_block_count")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            embedded
                .attributes
                .get("nucleation:preserve_support_blocks")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            mb.schematic.get_block(0, 0, 0).map(|b| b.get_name()),
            Some("minecraft:stone")
        );
        assert_eq!(
            mb.schematic.get_block(0, 1, 0).map(|b| b.get_name()),
            Some("minecraft:redstone_wire")
        );
        assert!(
            mb.schematic.get_block(0, -1, 0).is_none(),
            "the second substrate block is not retained recursively"
        );
    }

    #[test]
    fn support_aware_split_does_not_use_supports_as_connectivity() {
        let source = MemSource {
            id: TileId { x: 0, z: 0 },
            bounds: TileBounds {
                min: (0, 0, 0),
                max: (15, 15, 15),
            },
            blocks: vec![
                ((0, 0, 0), BlockState::new("minecraft:stone")),
                ((0, 1, 0), BlockState::new("minecraft:redstone_wire")),
                ((1, 2, 0), BlockState::new("minecraft:stone")),
                ((1, 3, 0), BlockState::new("minecraft:redstone_wire")),
            ],
        };
        let profile = WorldProfile::new(
            ["minecraft:stone"].iter().map(|s| s.to_string()).collect(),
            (0, 2),
        );
        let job = SegmentJob {
            config: SegConfig {
                preserve_support_blocks: true,
                ..SegConfig::default()
            },
            score_config: ScoreConfig::default(),
            source_id: "src".to_string(),
            snapshot_id: "snap1".to_string(),
            min_y: 0,
            max_y: 15,
            extracted_at: 1_700_000_000,
            match_iou: 0.5,
        };

        let out = WorldSegmenter::run(&source, &profile, &PartitionIndex::new(vec![]), &job, &[]);
        assert_eq!(out.len(), 1, "coarse segmentation groups the two seeds");
        let mb = &out[0];
        assert_eq!(mb.support_positions.len(), 2);
        assert_eq!(
            mb.schematic.split_connected(Connectivity::Corner).len(),
            1,
            "the enriched support footprint would reconnect the components"
        );
        let pieces = mb.split_connected_support_aware(Connectivity::Corner);
        assert_eq!(pieces.len(), 2);
        assert!(pieces.iter().all(|piece| piece.total_blocks() == 2));
    }
}
