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
    //
    // `Prefer` is NOT handled here on purpose: its documented contract is that
    // crossings are allowed but *recorded*, and crossing-recording is not yet
    // implemented. Until it is, `Prefer` deliberately falls through to the
    // unpartitioned path and behaves exactly like `Off` — every cluster comes
    // back with `partition_id: None` and nothing is recorded. This is pinned by
    // `prefer_policy_is_currently_inert_and_behaves_like_off` so the deferral
    // cannot change silently.
    let use_partitions =
        config.partition_policy == PartitionPolicy::HardCut && !partitions.is_empty();

    // Cells grouped into clusters. The dilated labelling only *groups*; cluster
    // identity is derived afterwards, from each group's original (undilated)
    // cells. See `assign_ids` for why that matters.
    let mut groups: BTreeMap<GroupKey, GroupAcc> = BTreeMap::new();

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
            group_into(&sub, config.closing_radius, pidx, &name, &mut groups);
        }
    } else {
        group_into(&grid, config.closing_radius, None, &None, &mut groups);
    }

    let (cluster_of_cell, partition_of_cluster) = assign_ids(tile.id(), groups);

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

/// Identifies one group of cells *before* it has a `ClusterId`.
///
/// `(partition index, component label)`. The label number is positional — it
/// comes from a sorted scan within a single `label_components()` call — so it
/// is only ever used to *group*, never as identity. Under `HardCut` each
/// partition gets its own call, and labels restart at 0 in each, so the
/// partition index is required to keep groups from different partitions apart.
type GroupKey = (Option<u32>, u32);

/// A group's original (undilated) cells plus the partition it came from.
struct GroupAcc {
    cells: std::collections::BTreeSet<(i32, i32, i32)>,
    partition: Option<String>,
}

/// Dilate, label, and bucket each occupied cell into its component's group.
///
/// Only the *original* occupied cells are recorded — the dilated cells exist
/// solely to decide which originals are connected.
fn group_into(
    grid: &OccupancyGrid,
    radius: u32,
    pidx: Option<u32>,
    partition: &Option<String>,
    groups: &mut BTreeMap<GroupKey, GroupAcc>,
) {
    let labels = grid.dilated(radius).label_components();
    for cell in grid.occupied_cells() {
        let Some(label) = labels.label_of(cell) else { continue };
        groups
            .entry((pidx, label))
            // The partition name is cloned once per group, not once per cell.
            .or_insert_with(|| GroupAcc {
                cells: std::collections::BTreeSet::new(),
                partition: partition.clone(),
            })
            .cells
            .insert(cell);
    }
}

/// Turn grouped cells into `ClusterId`s anchored on their own contents.
///
/// The anchor is the lexicographic minimum of the group's **original**
/// (undilated) cells. It must not be taken from the dilated component:
/// `OccupancyGrid::mark_cell` drops out-of-bounds cells, so `dilated()` is
/// clipped at the grid's minimum faces, and clipping is not injective — at
/// `cell_size = 4, closing_radius = 2, dims = 32^3` both cell `(0,18,0)` and
/// cell `(0,18,1)` dilate to the clipped lexmin `(0,16,0)`. Under `HardCut`
/// each partition is labelled separately but shares one output map, so two
/// cells in *different* partitions could collide onto one `ClusterId` and
/// emerge as a single cluster straddling a boundary.
///
/// Anchoring on original cells is injective by construction: groups have
/// pairwise disjoint cell sets (each occupied cell belongs to exactly one
/// partition and one component within it), and a set's minimum is a member of
/// that set, so distinct groups always yield distinct anchors. It is also
/// independent of clipping and of `closing_radius`, which removes the whole
/// class of bug rather than the one instance.
fn assign_ids(
    tile: TileId,
    groups: BTreeMap<GroupKey, GroupAcc>,
) -> (BTreeMap<(i32, i32, i32), ClusterId>, BTreeMap<ClusterId, Option<String>>) {
    let mut cluster_of_cell: BTreeMap<(i32, i32, i32), ClusterId> = BTreeMap::new();
    let mut partition_of_cluster: BTreeMap<ClusterId, Option<String>> = BTreeMap::new();

    for (_key, group) in groups {
        // `BTreeSet` iterates ascending, so the first cell is the lexmin.
        let Some(anchor) = group.cells.iter().next().copied() else { continue };
        let id = ClusterId::new(tile, anchor);
        debug_assert!(
            !partition_of_cluster.contains_key(&id),
            "ClusterId collision on anchor {anchor:?}: anchors must be unique across groups"
        );
        partition_of_cluster.insert(id, group.partition);
        for cell in group.cells {
            cluster_of_cell.insert(cell, id);
        }
    }

    (cluster_of_cell, partition_of_cluster)
}

/// Number of cells spanning the inclusive range `[lo, hi]`.
fn span_cells(lo: i32, hi: i32, cell: u32) -> usize {
    debug_assert!(hi >= lo, "span_cells: hi ({hi}) must be >= lo ({lo})");
    debug_assert!(cell > 0, "span_cells: cell size must be positive");
    // Widen before subtracting: `hi - lo` overflows i32 for extreme bounds.
    let span = i64::from(hi) - i64::from(lo);
    if span < 0 {
        // Degenerate bounds: an empty grid is safer than a huge/negative cast.
        return 0;
    }
    ((span / i64::from(cell.max(1))) + 1) as usize
}

/// Low corner of a cell, in world coordinates.
///
/// A whole cell is assigned to whichever partition contains its low corner, so
/// a cell straddling a boundary lands entirely on one side. At `cell_size = 4`
/// that is a sub-cell approximation of the true boundary; it is deterministic,
/// which is what matters here. If boundary precision ever matters more than
/// speed, classify per block instead of per cell.
fn cell_world_min(origin: (i32, i32, i32), cell: (i32, i32, i32), size: u32) -> (i32, i32, i32) {
    let s = i64::from(size);
    // Widen, then saturate: `origin + cell * size` can overflow i32 for a cell
    // coordinate near the grid's far edge on an extreme origin. Saturating
    // keeps the partition lookup total instead of panicking or wrapping into a
    // wildly wrong partition.
    let axis = |o: i32, c: i32| -> i32 {
        let v = i64::from(o) + i64::from(c) * s;
        debug_assert!(
            v >= i64::from(i32::MIN) && v <= i64::from(i32::MAX),
            "cell_world_min overflowed i32: origin {o} + cell {c} * size {s}"
        );
        v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    };
    (axis(origin.0, cell.0), axis(origin.1, cell.1), axis(origin.2, cell.2))
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
        // The headline failure mode: ground must not absorb the builds.
        //
        // The builds must genuinely *stand on* the ground, otherwise the test
        // cannot detect absorption. With origin.y = -64 and cell_size 4:
        //   ground y = -60 -> cell y = (-60 + 64) / 4 = 1
        //   build  y = -59 -> cell y = (-59 + 64) / 4 = 1   (same cell layer)
        // So the builds sit inside the ground's own cell footprint.
        //
        // Build cells: (10/4, 1, 10/4) = (2,1,2) and (30/4, 1, 30/4) = (7,1,7).
        // Chebyshev distance 5 > 2R = 4, so with substrate subtracted they stay
        // two clusters. Without subtraction the 40x40 stone slab spans cells
        // x,z in 0..=9 at cell y = 1 and swallows both into one mass.
        let mut blocks = vec![];
        for x in 0..40 {
            for z in 0..40 {
                blocks.push(((x, -60, z), "minecraft:stone")); // ground
            }
        }
        blocks.push(((10, -59, 10), "minecraft:redstone_wire")); // build A, on the ground
        blocks.push(((30, -59, 30), "minecraft:redstone_wire")); // build B, on the ground
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
    fn hard_cut_clusters_near_a_grid_min_face_keep_distinct_identities() {
        // Regression: cluster identity used to come from the *dilated*
        // component's anchor. `OccupancyGrid::mark_cell` drops out-of-bounds
        // cells, so dilation is clipped at the grid's minimum faces, and
        // clipping destroys injectivity. At cell_size 4 / closing_radius 2 /
        // dims 32^3:
        //   cell (0,18,0) dilates to clipped lexmin (0,16,0)
        //   cell (0,18,1) dilates to clipped lexmin (0,16,0)   <- identical
        //
        // Under HardCut each partition is labelled separately but writes into a
        // shared cluster map, so both cells collapsed onto one ClusterId and
        // emerged as a single cluster spanning both partitions.
        //
        // Coordinates: (0,8,0) -> cell (0,18,0); (0,8,4) -> cell (0,18,1).
        // Both have x = 0 and z < closing_radius, i.e. inside the clipped zone.
        // Their cell low corners are world (0,8,0) and (0,8,4), which the
        // hints below place in "near" (z <= 3) and "far" (z >= 4).
        let hints = PartitionIndex::new(vec![
            PartitionHint { id: "near".into(), bbox_xz: (0, 127, 0, 3), y_range: None },
            PartitionHint { id: "far".into(), bbox_xz: (0, 127, 4, 127), y_range: None },
        ]);
        let config = SegConfig { partition_policy: PartitionPolicy::HardCut, ..cfg() };
        let segs = segment_tile(
            &tile(vec![
                ((0, 8, 0), "minecraft:redstone_wire"),
                ((0, 8, 4), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &config,
            &hints,
        );

        assert_eq!(
            segs.clusters.len(),
            2,
            "a clipped dilation anchor must not fuse two partitions' clusters"
        );
        assert_ne!(segs.clusters[0].id, segs.clusters[1].id, "ids must stay distinct");
        let mut got: Vec<_> =
            segs.clusters.iter().map(|c| c.partition_id.clone().unwrap()).collect();
        got.sort();
        assert_eq!(got, vec!["far".to_string(), "near".to_string()]);
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
        // Compare the whole value, not just len + first id: policy Off must be
        // byte-for-byte identical to having no hints at all, margin included.
        assert_eq!(with, without);
    }

    #[test]
    fn prefer_policy_is_currently_inert_and_behaves_like_off() {
        // `Prefer` is documented as "crossings allowed but recorded".
        // Recording is NOT implemented yet, so `Prefer` deliberately takes the
        // unpartitioned path: identical output to `Off`, and no partition ids.
        // This test pins the deferral so it cannot change without notice — when
        // crossing-recording lands, this test should be replaced rather than
        // silently deleted.
        let hints = PartitionIndex::new(vec![
            PartitionHint { id: "left".into(), bbox_xz: (0, 13, 0, 127), y_range: None },
            PartitionHint { id: "right".into(), bbox_xz: (14, 127, 0, 127), y_range: None },
        ]);
        let blocks = vec![
            ((10, 10, 10), "minecraft:redstone_wire"),
            ((18, 10, 10), "minecraft:redstone_wire"),
        ];

        let prefer = SegConfig { partition_policy: PartitionPolicy::Prefer, ..cfg() };
        let off = SegConfig { partition_policy: PartitionPolicy::Off, ..cfg() };
        let prefer_segs = segment_tile(&tile(blocks.clone()), &profile(), &prefer, &hints);
        let off_segs = segment_tile(&tile(blocks.clone()), &profile(), &off, &hints);

        assert_eq!(prefer_segs, off_segs, "Prefer is currently indistinguishable from Off");
        assert_eq!(prefer_segs.clusters.len(), 1, "the boundary is not enforced under Prefer");
        assert!(
            prefer_segs.clusters.iter().all(|c| c.partition_id.is_none()),
            "nothing is recorded: every partition_id is None"
        );

        // Contrast: the same input under HardCut *is* split, which shows the
        // hints and the boundary are real and Prefer is simply ignoring them.
        let hard = SegConfig { partition_policy: PartitionPolicy::HardCut, ..cfg() };
        let hard_segs = segment_tile(&tile(blocks), &profile(), &hard, &hints);
        assert_eq!(hard_segs.clusters.len(), 2);
    }

    #[test]
    fn a_cluster_dropped_by_min_cluster_blocks_leaves_no_margin_entry() {
        // The `kept` guard in step 5 exists so a filtered-out cluster cannot
        // leave a dangling `margin` entry pointing at a ClusterId that is not
        // in `clusters`. Nothing asserted that, so pin it.
        //
        // (2,10,2) -> cell (0,18,0): inside the 4-cell band, but a lone block,
        // so `min_cluster_blocks = 2` drops it.
        // (64,10,64)/(65,10,64) -> cell (16,18,16): interior, survives.
        let config = SegConfig { min_cluster_blocks: 2, ..cfg() };
        let segs = segment_tile(
            &tile(vec![
                ((2, 10, 2), "minecraft:redstone_wire"),
                ((64, 10, 64), "minecraft:redstone_wire"),
                ((65, 10, 64), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &config,
            &no_hints(),
        );

        assert_eq!(segs.clusters.len(), 1, "the near-face single-block cluster is dropped");
        assert_eq!(segs.clusters[0].block_count, 2);

        let kept: std::collections::BTreeSet<ClusterId> =
            segs.clusters.iter().map(|c| c.id).collect();
        assert!(
            segs.margin.iter().all(|m| kept.contains(&m.cluster)),
            "every margin entry must reference a surviving cluster"
        );
        assert!(
            segs.margin.is_empty(),
            "the dropped cluster was the only one in the band, so margin is empty"
        );
    }

    /// The single cell a one-block tile produces margin entries for.
    fn margin_cells(block: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
        let segs = segment_tile(
            &tile(vec![(block, "minecraft:redstone_wire")]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 1, "one block must give exactly one cluster");
        segs.margin.iter().map(|m| m.cell).collect()
    }

    #[test]
    fn margin_band_covers_cells_within_2r_of_a_face() {
        // Bounds (0,-64,0)..(127,63,127) at cell_size 4 give dims 32 on every
        // axis; band = 2 * closing_radius = 4 cells. So the near-face band is
        // cells 0..=3 and the far-face band starts at cell 32 - 4 = 28.
        //
        // Each case is segmented on its own tile: cells (0,18,0) and (3,18,3)
        // are only 3 apart, within 2R, so together they would merge into one
        // cluster and stop isolating the depth being tested.

        // Depth 0 — trivially in band. (2,10,2) -> (2/4, (10+64)/4, 2/4).
        assert_eq!(margin_cells((2, 10, 2)), vec![(0, 18, 0)]);

        // Depth 3 = 2R - 1: the LAST in-band layer. This is the discriminating
        // case — it is in-band for width 4 but out-of-band for any width <= 3,
        // so it is what actually pins the band width.
        // (13,10,13) -> (13/4, 18, 13/4) = (3,18,3).
        assert_eq!(
            margin_cells((13, 10, 13)),
            vec![(3, 18, 3)],
            "depth 2R-1 must be in band; this fails for any band width <= 3"
        );

        // Depth 4 = 2R: the FIRST out-of-band layer, pinning the upper edge.
        // (17,10,17) -> (17/4, 18, 17/4) = (4,18,4).
        assert!(
            margin_cells((17, 10, 17)).is_empty(),
            "depth 2R must be outside the band; this fails for any width >= 5"
        );

        // Same two-sided check against the far faces at cell 28.
        // (112,10,112) -> (28,18,28), the first in-band far layer.
        assert_eq!(margin_cells((112, 10, 112)), vec![(28, 18, 28)]);
        // (111,10,111) -> (27,18,27), the last interior layer.
        assert!(margin_cells((111, 10, 111)).is_empty(), "cell 27 is interior");
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
