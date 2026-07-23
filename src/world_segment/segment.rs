//! Phase 1: one tile in, clusters out.
//!
//! Substrate subtraction -> occupancy grid -> Chebyshev dilation -> connected
//! components -> assign original cells back to their component. Two structures
//! land in the same cluster iff their occupied cells are within `2R` cells,
//! i.e. roughly `2 * closing_radius * cell_size` blocks.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::world_segment::classify::{classify, BlockClass};
use crate::world_segment::grid::OccupancyGrid;
use crate::world_segment::ids::{ClusterId, TileId};
use crate::world_segment::partition::{PartitionIndex, PartitionPolicy};
use crate::world_segment::profile::WorldProfile;
use crate::world_segment::tile::VoxelTile;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SegConfig {
    /// Occupancy cell edge, in blocks.
    pub cell_size: u32,
    /// Chebyshev dilation radius, in cells.
    pub closing_radius: u32,
    /// Clusters with fewer blocks than this are not recorded.
    pub min_cluster_blocks: u64,
    pub partition_policy: PartitionPolicy,
    pub algorithm_version: u32,
}

impl Default for SegConfig {
    fn default() -> Self {
        SegConfig {
            cell_size: 4,
            closing_radius: 2,
            min_cluster_blocks: 1,
            partition_policy: PartitionPolicy::Off,
            algorithm_version: 1,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Cluster {
    pub id: ClusterId,
    /// Inclusive world-space `(min, max)` of the cluster's original blocks.
    pub bbox: ((i32, i32, i32), (i32, i32, i32)),
    pub block_count: u64,
    pub cell_count: u64,
    /// The partition this cluster fell in, if any. Opaque to segmentation.
    pub partition_id: Option<String>,
}

/// A labelled cell within `2R` of a tile face, for cross-tile stitching.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MarginCell {
    pub cell: (i32, i32, i32),
    pub cluster: ClusterId,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TileSegments {
    pub tile_id: TileId,
    /// Ascending by `ClusterId`, so the output is order-independent.
    pub clusters: Vec<Cluster>,
    pub margin: Vec<MarginCell>,
}

pub fn segment_tile(
    tile: &VoxelTile,
    profile: &WorldProfile,
    config: &SegConfig,
    partitions: &PartitionIndex,
) -> TileSegments {
    let bounds = tile.bounds();
    let cell = config.cell_size.max(1);

    // Grid spans the tile bounds exactly.
    let dims = (
        span_cells(bounds.min.0, bounds.max.0, cell),
        span_cells(bounds.min.1, bounds.max.1, cell),
        span_cells(bounds.min.2, bounds.max.2, cell),
    );
    let origin = bounds.min;

    // 1. Substrate subtraction + voxelization.
    let mut grid = OccupancyGrid::new(origin, dims, cell);
    let mut artificial: Vec<(i32, i32, i32)> = Vec::new();
    for (pos, state) in tile.blocks() {
        if classify(state, pos.1, profile) == BlockClass::Substrate {
            continue;
        }
        artificial.push(pos);
        grid.mark(pos.0, pos.1, pos.2);
    }
    if artificial.is_empty() {
        return TileSegments { tile_id: tile.id(), clusters: Vec::new(), margin: Vec::new() };
    }

    // 2/3. Dilate and label. Under HardCut, each partition is dilated and
    // labelled in isolation, so a component can never straddle a boundary.
    let use_partitions =
        config.partition_policy == PartitionPolicy::HardCut && !partitions.is_empty();

    let mut cluster_of_cell: BTreeMap<(i32, i32, i32), ClusterId> = BTreeMap::new();
    let mut partition_of_cluster: BTreeMap<ClusterId, Option<String>> = BTreeMap::new();

    if use_partitions {
        // Group occupied cells by partition, keyed by index so grouping is
        // independent of caller-supplied hint order.
        let mut by_partition: BTreeMap<Option<u32>, Vec<(i32, i32, i32)>> = BTreeMap::new();
        for cell_coord in grid.occupied_cells() {
            let w = cell_world_min(origin, cell_coord, cell);
            by_partition.entry(partitions.id_index_at(w.0, w.1, w.2)).or_default().push(cell_coord);
        }
        for (pidx, cells) in by_partition {
            let mut sub = OccupancyGrid::new(origin, dims, cell);
            for c in &cells {
                sub.mark_cell(*c);
            }
            let name = pidx.map(|i| partitions.id_of_index(i).to_string());
            label_into(
                &sub,
                config.closing_radius,
                tile.id(),
                &name,
                &mut cluster_of_cell,
                &mut partition_of_cluster,
            );
        }
    } else {
        label_into(
            &grid,
            config.closing_radius,
            tile.id(),
            &None,
            &mut cluster_of_cell,
            &mut partition_of_cluster,
        );
    }

    // 4. Fold original blocks back into their cluster.
    let mut acc: BTreeMap<ClusterId, ClusterAcc> = BTreeMap::new();
    for pos in artificial {
        let cell_coord = grid.cell_of(pos.0, pos.1, pos.2);
        let Some(id) = cluster_of_cell.get(&cell_coord) else { continue };
        acc.entry(*id).or_insert_with(ClusterAcc::new).push(pos, cell_coord);
    }

    let mut clusters: Vec<Cluster> = Vec::new();
    for (id, a) in &acc {
        if a.block_count < config.min_cluster_blocks {
            continue;
        }
        clusters.push(Cluster {
            id: *id,
            bbox: (a.min, a.max),
            block_count: a.block_count,
            cell_count: a.cells.len() as u64,
            partition_id: partition_of_cluster.get(id).cloned().flatten(),
        });
    }
    clusters.sort_by_key(|c| c.id);

    // 5. Margin band: cells within 2R of any face.
    let band = (config.closing_radius * 2) as i32;
    let kept: std::collections::BTreeSet<ClusterId> = clusters.iter().map(|c| c.id).collect();
    let mut margin: Vec<MarginCell> = Vec::new();
    for (id, a) in &acc {
        if !kept.contains(id) {
            continue;
        }
        for cell_coord in &a.cells {
            if in_margin(*cell_coord, dims, band) {
                margin.push(MarginCell { cell: *cell_coord, cluster: *id });
            }
        }
    }
    margin.sort_by(|a, b| a.cell.cmp(&b.cell).then(a.cluster.cmp(&b.cluster)));

    TileSegments { tile_id: tile.id(), clusters, margin }
}

struct ClusterAcc {
    min: (i32, i32, i32),
    max: (i32, i32, i32),
    block_count: u64,
    cells: std::collections::BTreeSet<(i32, i32, i32)>,
}

impl ClusterAcc {
    fn new() -> Self {
        ClusterAcc {
            min: (i32::MAX, i32::MAX, i32::MAX),
            max: (i32::MIN, i32::MIN, i32::MIN),
            block_count: 0,
            cells: std::collections::BTreeSet::new(),
        }
    }

    fn push(&mut self, pos: (i32, i32, i32), cell: (i32, i32, i32)) {
        self.min = (self.min.0.min(pos.0), self.min.1.min(pos.1), self.min.2.min(pos.2));
        self.max = (self.max.0.max(pos.0), self.max.1.max(pos.1), self.max.2.max(pos.2));
        self.block_count += 1;
        self.cells.insert(cell);
    }
}

/// Dilate, label, and record each occupied cell's `ClusterId`.
fn label_into(
    grid: &OccupancyGrid,
    radius: u32,
    tile: TileId,
    partition: &Option<String>,
    cluster_of_cell: &mut BTreeMap<(i32, i32, i32), ClusterId>,
    partition_of_cluster: &mut BTreeMap<ClusterId, Option<String>>,
) {
    let labels = grid.dilated(radius).label_components();
    for cell in grid.occupied_cells() {
        let Some(label) = labels.label_of(cell) else { continue };
        // Identity comes from the component's anchor, never the label number.
        let id = ClusterId::new(tile, labels.anchor_of(label));
        cluster_of_cell.insert(cell, id);
        partition_of_cluster.insert(id, partition.clone());
    }
}

fn span_cells(lo: i32, hi: i32, cell: u32) -> usize {
    (((hi - lo) as i64 / cell as i64) + 1) as usize
}

/// Low corner of a cell, in world coordinates.
///
/// A whole cell is assigned to whichever partition contains its low corner, so
/// a cell straddling a boundary lands entirely on one side. At `cell_size = 4`
/// that is a sub-cell approximation of the true boundary; it is deterministic,
/// which is what matters here. If boundary precision ever matters more than
/// speed, classify per block instead of per cell.
fn cell_world_min(origin: (i32, i32, i32), cell: (i32, i32, i32), size: u32) -> (i32, i32, i32) {
    let s = size as i32;
    (origin.0 + cell.0 * s, origin.1 + cell.1 * s, origin.2 + cell.2 * s)
}

fn in_margin(cell: (i32, i32, i32), dims: (usize, usize, usize), band: i32) -> bool {
    cell.0 < band
        || cell.2 < band
        || cell.0 >= dims.0 as i32 - band
        || cell.2 >= dims.2 as i32 - band
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_state::BlockState;
    use crate::world_segment::ids::TileId;
    use crate::world_segment::partition::{PartitionHint, PartitionIndex};
    use crate::world_segment::tile::TileBounds;

    fn profile() -> WorldProfile {
        WorldProfile::new(
            ["minecraft:stone", "minecraft:bedrock"].iter().map(|s| s.to_string()).collect(),
            (-64, -50),
        )
    }

    fn cfg() -> SegConfig {
        SegConfig { cell_size: 4, closing_radius: 2, min_cluster_blocks: 1, ..SegConfig::default() }
    }

    fn bounds() -> TileBounds {
        TileBounds { min: (0, -64, 0), max: (127, 63, 127) }
    }

    /// Build a tile from `(pos, block_name)` pairs.
    fn tile(blocks: Vec<((i32, i32, i32), &str)>) -> VoxelTile {
        VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            blocks.into_iter().map(|(p, n)| (p, BlockState::new(n))),
        )
    }

    fn no_hints() -> PartitionIndex {
        PartitionIndex::new(vec![])
    }

    #[test]
    fn substrate_is_dropped_entirely() {
        let t = tile(vec![
            ((10, -60, 10), "minecraft:stone"),
            ((11, -60, 10), "minecraft:stone"),
        ]);
        let segs = segment_tile(&t, &profile(), &cfg(), &no_hints());
        assert!(segs.clusters.is_empty(), "a tile of pure substrate yields no clusters");
    }

    #[test]
    fn a_build_standing_on_substrate_does_not_merge_with_it() {
        // The headline failure mode: ground must not absorb the build.
        let mut blocks = vec![];
        for x in 0..40 {
            for z in 0..40 {
                blocks.push(((x, -60, z), "minecraft:stone")); // ground
            }
        }
        blocks.push(((10, 0, 10), "minecraft:redstone_wire")); // build A
        blocks.push(((30, 0, 30), "minecraft:redstone_wire")); // build B, far away
        let segs = segment_tile(&tile(blocks), &profile(), &cfg(), &no_hints());
        assert_eq!(segs.clusters.len(), 2, "two builds, ground removed");
    }

    #[test]
    fn a_detached_floating_component_does_not_split() {
        // The other failure mode: closing must bridge intra-build gaps.
        // cell_size 4 * closing_radius 2 -> merges up to ~16 blocks apart.
        let segs = segment_tile(
            &tile(vec![
                ((10, 10, 10), "minecraft:redstone_wire"),
                ((18, 10, 10), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 1, "an 8-block gap must bridge");
    }

    #[test]
    fn structures_beyond_the_closing_distance_stay_separate() {
        let segs = segment_tile(
            &tile(vec![
                ((10, 10, 10), "minecraft:redstone_wire"),
                ((90, 10, 10), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 2);
    }

    #[test]
    fn cluster_bbox_and_block_count_describe_the_original_blocks() {
        let segs = segment_tile(
            &tile(vec![
                ((10, 10, 10), "minecraft:redstone_wire"),
                ((12, 14, 11), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 1);
        let c = &segs.clusters[0];
        assert_eq!(c.block_count, 2);
        assert_eq!(c.bbox, ((10, 10, 10), (12, 14, 11)));
    }

    #[test]
    fn min_cluster_blocks_filters_small_clusters() {
        let config = SegConfig { min_cluster_blocks: 2, ..cfg() };
        let segs = segment_tile(
            &tile(vec![
                ((10, 10, 10), "minecraft:redstone_wire"),
                ((90, 10, 10), "minecraft:redstone_wire"),
                ((91, 10, 10), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &config,
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 1, "the single-block cluster is dropped");
        assert_eq!(segs.clusters[0].block_count, 2);
    }

    #[test]
    fn hard_cut_prevents_merging_across_a_partition_boundary() {
        // Two blocks 8 apart would normally bridge; a boundary between them
        // must stop that.
        let hints = PartitionIndex::new(vec![
            PartitionHint { id: "left".into(), bbox_xz: (0, 13, 0, 127), y_range: None },
            PartitionHint { id: "right".into(), bbox_xz: (14, 127, 0, 127), y_range: None },
        ]);
        let config = SegConfig { partition_policy: PartitionPolicy::HardCut, ..cfg() };
        let segs = segment_tile(
            &tile(vec![
                ((10, 10, 10), "minecraft:redstone_wire"),
                ((18, 10, 10), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &config,
            &hints,
        );
        assert_eq!(segs.clusters.len(), 2, "the boundary splits them");
        // Clusters are ordered by ClusterId (a hash), not by position, so
        // compare as a set rather than by index.
        let mut got: Vec<_> =
            segs.clusters.iter().map(|c| c.partition_id.clone().unwrap()).collect();
        got.sort();
        assert_eq!(got, vec!["left".to_string(), "right".to_string()]);
    }

    #[test]
    fn policy_off_reproduces_the_unpartitioned_result() {
        let hints = PartitionIndex::new(vec![
            PartitionHint { id: "left".into(), bbox_xz: (0, 13, 0, 127), y_range: None },
            PartitionHint { id: "right".into(), bbox_xz: (14, 127, 0, 127), y_range: None },
        ]);
        let config = SegConfig { partition_policy: PartitionPolicy::Off, ..cfg() };
        let blocks = vec![
            ((10, 10, 10), "minecraft:redstone_wire"),
            ((18, 10, 10), "minecraft:redstone_wire"),
        ];
        let with = segment_tile(&tile(blocks.clone()), &profile(), &config, &hints);
        let without = segment_tile(&tile(blocks), &profile(), &cfg(), &no_hints());
        assert_eq!(with.clusters.len(), without.clusters.len());
        assert_eq!(with.clusters[0].id, without.clusters[0].id);
    }

    #[test]
    fn margin_band_covers_cells_within_2r_of_a_face() {
        // closing_radius 2 -> band is 4 cells = 16 blocks from each face.
        let segs = segment_tile(
            &tile(vec![((2, 10, 2), "minecraft:redstone_wire")]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 1);
        assert!(!segs.margin.is_empty(), "a block 2 from the face is in the band");
    }

    #[test]
    fn interior_clusters_emit_no_margin_cells() {
        let segs = segment_tile(
            &tile(vec![((64, 10, 64), "minecraft:redstone_wire")]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 1);
        assert!(segs.margin.is_empty(), "a centre cluster is nowhere near a face");
    }
}
