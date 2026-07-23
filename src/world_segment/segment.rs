//! Phase 1: one tile in, clusters out.
//!
//! Substrate subtraction -> occupancy grid -> Chebyshev dilation -> connected
//! components -> assign original cells back to their component.
//!
//! # Merge threshold
//!
//! Write `R` for `closing_radius`. Two occupied cells at Chebyshev cell
//! distance `d`, each dilated into a cube of edge `2R+1`, span `[-R, R]` and
//! `[d-R, d+R]` on the separating axis. Those cubes *overlap* when `d <= 2R`.
//! At `d = 2R+1` they no longer overlap — but they are **face-adjacent**
//! (`max = R`, `min = R+1`), and components are labelled with 6-connectivity,
//! which fuses face-adjacent cells. So the merge threshold is `2R+1` cells,
//! i.e. up to about `(2R+1) * cell_size` blocks — not `2R`.
//!
//! Precisely: two cells merge when they are within `2R` on every axis, or
//! exactly `2R+1` apart on one axis and within `2R` on the other two. A pure
//! diagonal at `2R+1` on two or more axes does *not* merge, because
//! 6-connectivity requires the two cubes to share a face rather than an edge
//! or a corner. `2R+1` is therefore the maximum merge distance, reached along
//! an axis.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::world_segment::classify::{classify, BlockClass};
use crate::world_segment::grid::OccupancyGrid;
use crate::world_segment::ids::{ClusterId, ContentId, TileId};
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
    ///
    /// # This filter is per-tile and runs *before* stitching
    ///
    /// `segment_tile` sees one tile at a time. A build that straddles a tile
    /// edge arrives here as two independent fragments, and each is measured
    /// against this threshold on its own. A 30-block build split 18/12 across
    /// an edge is dropped entirely at `min_cluster_blocks = 20`, because
    /// neither half reaches 20 — the halves are never summed, since the sum
    /// only exists after cross-tile stitching, which is stage 2.
    ///
    /// The default of `1` drops nothing, so this only bites callers who raise
    /// it. If you want a "builds smaller than N blocks are noise" rule, apply
    /// it to stitched clusters downstream, not here. Use this field only for
    /// what it can honestly do: cheaply discarding per-tile specks before they
    /// are written out.
    pub min_cluster_blocks: u64,
    pub partition_policy: PartitionPolicy,
    pub algorithm_version: u32,
}

impl SegConfig {
    /// Stable hash of everything that can change what segmentation produces.
    ///
    /// Folded into every [`ClusterId`], so an id identifies a cluster *under a
    /// stated configuration* rather than merely a position. Two runs that
    /// disagree on `cell_size`, `closing_radius`, `min_cluster_blocks`,
    /// `partition_policy`, `algorithm_version` or the world profile can never
    /// mint the same id, which is what makes the ids safe to use as cache
    /// keys.
    ///
    /// Every field of `SegConfig` is covered. If a field is added, add it here
    /// too — an unhashed field is a silent id collision between runs that
    /// genuinely differ.
    pub fn config_hash(&self, profile: &WorldProfile) -> ContentId {
        // Explicit, pinned discriminants: derived ordering would silently
        // renumber if a variant were inserted, changing ids without any
        // behaviour change.
        let policy: u8 = match self.partition_policy {
            PartitionPolicy::HardCut => 0,
            PartitionPolicy::Prefer => 1,
            PartitionPolicy::Off => 2,
        };
        ContentId::of(&[
            b"segconfig.v1",
            &self.cell_size.to_le_bytes(),
            &self.closing_radius.to_le_bytes(),
            &self.min_cluster_blocks.to_le_bytes(),
            &[policy],
            &self.algorithm_version.to_le_bytes(),
            profile.profile_hash().as_bytes(),
        ])
    }
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

/// A labelled cell at cell-depth `0..=2R` from a tile face, for cross-tile
/// stitching. See `segment_tile` step 5 for why the band is `2R+1` cells wide.
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
    // Every ClusterId minted below is bound to this hash, so ids produced
    // under different settings or a different profile can never collide.
    let config_id = config.config_hash(profile);

    // Grid spans the tile bounds exactly.
    let dims = (
        span_cells(bounds.min.0, bounds.max.0, cell),
        span_cells(bounds.min.1, bounds.max.1, cell),
        span_cells(bounds.min.2, bounds.max.2, cell),
    );
    let origin = bounds.min;

    // Under HardCut, each partition is dilated and labelled in isolation, so a
    // component can never straddle a boundary.
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

    // 1. Substrate subtraction + voxelization, into one occupancy grid per
    // partition.
    //
    // The partition is decided **per block**, not per cell. Deciding per cell
    // — by, say, the partition containing the cell's low corner — assigns a
    // whole cell to one side, so a cell straddling a boundary drags the blocks
    // on the far side along with it and produces a cluster that genuinely
    // spans the boundary while claiming a single `partition_id`. At
    // `cell_size = 4` a boundary is cell-aligned only one time in four, so
    // that was the common case, not the corner case.
    //
    // Splitting the blocks first means a single cell may end up occupied in
    // two partitions' grids at once. That is correct and harmless: the grids
    // are separate, so the two occupancies never see each other, dilate
    // independently, and label independently. The only thing it costs is that
    // an anchor cell is no longer unique on its own — hence the partition is
    // folded into `ClusterId` alongside the anchor.
    //
    // Cost is unchanged in the usual case: grids are sparse `BTreeSet`s, one
    // per occupied partition, and a block is marked exactly once.
    let mut artificial: Vec<((i32, i32, i32), Option<u32>)> = Vec::new();
    let mut grids: BTreeMap<Option<u32>, OccupancyGrid> = BTreeMap::new();
    for (pos, state) in tile.blocks() {
        if classify(state, pos.1, profile) == BlockClass::Substrate {
            continue;
        }
        // Keyed by index, not by name, so grouping is independent of the order
        // the caller supplied the hints in.
        let pidx =
            if use_partitions { partitions.id_index_at(pos.0, pos.1, pos.2) } else { None };
        artificial.push((pos, pidx));
        grids
            .entry(pidx)
            .or_insert_with(|| OccupancyGrid::new(origin, dims, cell))
            .mark(pos.0, pos.1, pos.2);
    }
    if artificial.is_empty() {
        return TileSegments { tile_id: tile.id(), clusters: Vec::new(), margin: Vec::new() };
    }

    // An empty grid, used only for its `cell_of` coordinate transform. Sharing
    // the real transform keeps step 4 from re-deriving floor division and
    // drifting from `OccupancyGrid`.
    let geometry = OccupancyGrid::new(origin, dims, cell);

    // 2/3. Dilate and label, one partition at a time.
    //
    // Cells grouped into clusters. The dilated labelling only *groups*; cluster
    // identity is derived afterwards, from each group's original (undilated)
    // cells. See `assign_ids` for why that matters.
    let mut groups: BTreeMap<GroupKey, GroupAcc> = BTreeMap::new();
    for (pidx, part_grid) in &grids {
        let name = pidx.map(|i| partitions.id_of_index(i).to_string());
        group_into(part_grid, config.closing_radius, *pidx, &name, &mut groups);
    }

    let (cluster_of_cell, partition_of_cluster) = assign_ids(config_id, tile.id(), groups);

    // 4. Fold original blocks back into their cluster.
    //
    // Looked up by `(partition, cell)`: a cell shared by two partitions belongs
    // to a different cluster in each, and each block resolves through its own
    // partition.
    let mut acc: BTreeMap<ClusterId, ClusterAcc> = BTreeMap::new();
    for (pos, pidx) in artificial {
        let cell_coord = geometry.cell_of(pos.0, pos.1, pos.2);
        let Some(id) = cluster_of_cell.get(&(pidx, cell_coord)) else { continue };
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

    // 5. Margin band: cells at depth 0..=2R from any face, i.e. a band 2R+1
    // cells wide.
    //
    // Width, derived rather than guessed: a cell at depth `a` in this tile and
    // one at depth `b` in the abutting tile are Chebyshev `a + b + 1` cells
    // apart (the +1 crosses the face). The merge threshold is `2R+1`, so they
    // merge when `a + b + 1 <= 2R + 1`, i.e. `a + b <= 2R`. The worst case is
    // `b = 0`, giving `a <= 2R`. Depths `0..=2R` must therefore all be exported
    // — that is `2R + 1` layers. A band of `2R` omits depth `2R` and silently
    // loses joins that stage-2 stitching would otherwise make.
    let band = (config.closing_radius * 2 + 1) as i32;
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
/// Anchoring on original cells is injective *within a partition*: the groups
/// of one partition are the components of one grid, so their cell sets are
/// pairwise disjoint, and a set's minimum is a member of that set. It is also
/// independent of clipping and of `closing_radius`, which removes the whole
/// class of bug rather than the one instance.
///
/// Across partitions the anchor alone is *not* injective: blocks are
/// partitioned individually, so one cell can be occupied in two partitions'
/// grids and be the lexmin of a group in each. `(partition, anchor)` is
/// injective, which is why the partition is hashed into `ClusterId` too, and
/// why the returned cell map is keyed by `(partition, cell)` rather than by
/// cell alone.
fn assign_ids(
    config: ContentId,
    tile: TileId,
    groups: BTreeMap<GroupKey, GroupAcc>,
) -> (
    BTreeMap<(Option<u32>, (i32, i32, i32)), ClusterId>,
    BTreeMap<ClusterId, Option<String>>,
) {
    let mut cluster_of_cell: BTreeMap<(Option<u32>, (i32, i32, i32)), ClusterId> = BTreeMap::new();
    let mut partition_of_cluster: BTreeMap<ClusterId, Option<String>> = BTreeMap::new();

    for ((pidx, _label), group) in groups {
        let GroupAcc { cells, partition } = group;
        // `BTreeSet` iterates ascending, so the first cell is the lexmin.
        let Some(anchor) = cells.iter().next().copied() else { continue };
        let id = ClusterId::new(config, tile, partition.as_deref(), anchor);
        debug_assert!(
            !partition_of_cluster.contains_key(&id),
            "ClusterId collision on anchor {anchor:?} in partition {partition:?}: \
             (partition, anchor) must be unique across groups"
        );
        partition_of_cluster.insert(id, partition);
        for cell in cells {
            cluster_of_cell.insert((pidx, cell), id);
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
        // They are 5 = 2R+1 apart on BOTH x and z, which is a corner-diagonal,
        // not an axis step: dilated they span x,z in [0,4] and [5,9], and no
        // cell of one is 6-adjacent to a cell of the other (that would need the
        // two boxes to agree on two axes and differ by 1 on the third). So with
        // substrate subtracted they stay two clusters. Without subtraction the
        // 40x40 stone slab spans cells x,z in 0..=9 at cell y = 1 and swallows
        // both into one mass.
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
        // cell_size 4, closing_radius 2 -> merges cells up to 2R+1 = 5 apart
        // along an axis, i.e. roughly 20 blocks.
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
    fn hard_cut_holds_on_a_boundary_that_is_not_cell_aligned() {
        // The guarantee is "no cluster spans a partition boundary". Assigning a
        // whole cell to the partition of its low corner only approximates that:
        // a cell straddling the boundary lands entirely on one side, dragging
        // the blocks on the other side along with it.
        //
        // Boundary at x = 62, deliberately NOT a multiple of cell_size 4
        // (62 = 15*4 + 2), so one cell genuinely straddles it:
        //   cell x 15 covers world x 60..=63, and 62 falls strictly inside.
        //   (60,10,40) -> cell (60/4, (10+64)/4, 40/4) = (15,18,10)   -> "L"
        //   (63,10,40) -> cell (63/4, 18,       40/4) = (15,18,10)   -> "R"
        // Same cell, different partitions.
        //
        // Per-cell assignment gave that cell to "L" (its low corner, world
        // x = 60), producing ONE cluster with bbox ((60,10,40),(63,10,40))
        // labelled "L" while containing a block at x = 63, which is in "R".
        // Partitioning per block instead puts the two blocks in separate
        // grids, so the shared cell is occupied in each independently.
        let hints = PartitionIndex::new(vec![
            PartitionHint { id: "L".into(), bbox_xz: (0, 61, 0, 127), y_range: None },
            PartitionHint { id: "R".into(), bbox_xz: (62, 127, 0, 127), y_range: None },
        ]);
        let config = SegConfig { partition_policy: PartitionPolicy::HardCut, ..cfg() };
        let segs = segment_tile(
            &tile(vec![
                ((60, 10, 40), "minecraft:redstone_wire"),
                ((63, 10, 40), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &config,
            &hints,
        );

        for c in &segs.clusters {
            assert!(
                !(c.bbox.0 .0 <= 61 && c.bbox.1 .0 >= 62),
                "cluster {} spans the boundary at x=62: bbox {:?}",
                c.id,
                c.bbox
            );
        }
        assert_eq!(segs.clusters.len(), 2, "the boundary splits the straddling cell");

        let mut got: Vec<_> = segs
            .clusters
            .iter()
            .map(|c| (c.partition_id.clone().unwrap(), c.bbox))
            .collect();
        got.sort();
        assert_eq!(
            got,
            vec![
                ("L".to_string(), ((60, 10, 40), (60, 10, 40))),
                ("R".to_string(), ((63, 10, 40), (63, 10, 40))),
            ],
            "each block is attributed to the partition it actually sits in"
        );
    }

    #[test]
    fn a_cell_shared_by_two_partitions_yields_two_distinct_ids() {
        // Per-block partitioning lets one cell be occupied in two partitions'
        // grids, so two groups can share an anchor cell. They are different
        // clusters and must not collapse onto one ClusterId — which is why the
        // partition is folded into the id alongside the anchor.
        //
        // Same straddling cell (15,18,10) as above, reached from both sides.
        let hints = PartitionIndex::new(vec![
            PartitionHint { id: "L".into(), bbox_xz: (0, 61, 0, 127), y_range: None },
            PartitionHint { id: "R".into(), bbox_xz: (62, 127, 0, 127), y_range: None },
        ]);
        let config = SegConfig { partition_policy: PartitionPolicy::HardCut, ..cfg() };
        let segs = segment_tile(
            &tile(vec![
                ((60, 10, 40), "minecraft:redstone_wire"),
                ((63, 10, 40), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &config,
            &hints,
        );
        assert_eq!(segs.clusters.len(), 2);
        assert_ne!(
            segs.clusters[0].id, segs.clusters[1].id,
            "a shared anchor cell in two partitions must still give two ids"
        );
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

    /// Everything about a `TileSegments` except the `ClusterId`s: the clusters
    /// as `(bbox, block_count, cell_count, partition_id)` and the margin as
    /// bare cells, both in a canonical order.
    #[allow(clippy::type_complexity)]
    fn shape_of(
        segs: &TileSegments,
    ) -> (
        Vec<(((i32, i32, i32), (i32, i32, i32)), u64, u64, Option<String>)>,
        Vec<(i32, i32, i32)>,
    ) {
        let mut clusters: Vec<_> = segs
            .clusters
            .iter()
            .map(|c| (c.bbox, c.block_count, c.cell_count, c.partition_id.clone()))
            .collect();
        // Sort by content: `segs.clusters` is ordered by ClusterId, which
        // differs between the two runs being compared here.
        clusters.sort();
        let mut margin: Vec<_> = segs.margin.iter().map(|m| m.cell).collect();
        margin.sort();
        (clusters, margin)
    }

    #[test]
    fn prefer_policy_is_currently_inert_and_behaves_like_off() {
        // `Prefer` is documented as "crossings allowed but recorded".
        // Recording is NOT implemented yet, so `Prefer` deliberately takes the
        // unpartitioned path: it segments exactly as `Off` does, and records no
        // partition ids. This test pins the deferral so it cannot change
        // without notice — when crossing-recording lands, this test should be
        // replaced rather than silently deleted.
        //
        // The comparison is on segmentation *shape*, not on the whole value,
        // and deliberately so. `partition_policy` is part of `config_hash`, so
        // `Prefer` and `Off` are different configurations and MUST mint
        // different ClusterIds — that is the point of folding config into the
        // id. What "behaves like Off" means is that every cluster has the same
        // bbox, block count, cell count and partition attribution, and the
        // margin covers the same cells. Both facts are asserted below, so this
        // is strictly more specific than the whole-value equality it replaces.
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

        assert_eq!(prefer_segs.tile_id, off_segs.tile_id);
        assert_eq!(
            shape_of(&prefer_segs),
            shape_of(&off_segs),
            "Prefer must segment exactly as Off does"
        );
        assert_ne!(
            prefer_segs.clusters[0].id, off_segs.clusters[0].id,
            "different configs must still mint different ids"
        );
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
        // (2,10,2) -> cell (0,18,0): inside the 5-cell band (0..=4), but a lone
        // block, so `min_cluster_blocks = 2` drops it.
        // (64,10,64)/(65,10,64) -> cell (16,18,16): interior (5 <= 16 < 27),
        // survives.
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
    fn two_cells_exactly_2r_plus_1_apart_are_one_cluster() {
        // Pins the *merge* threshold directly, along a single axis.
        //
        // R = 2, cell_size = 4, origin (0,-64,0).
        //   (40,10,40) -> cell (40/4, (10+64)/4, 40/4) = (10,18,10)
        //   (60,10,40) -> cell (60/4, 18, 40/4)        = (15,18,10)
        // Chebyshev cell distance = 15 - 10 = 5 = 2R + 1.
        //
        // Dilating each by R=2 gives x-spans [8,12] and [13,17]: they do NOT
        // overlap (2R+1 > 2R), but cell (12,18,10) and cell (13,18,10) differ
        // by 1 on exactly one axis, so 6-connectivity fuses them.
        let segs = segment_tile(
            &tile(vec![
                ((40, 10, 40), "minecraft:redstone_wire"),
                ((60, 10, 40), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(
            segs.clusters.len(),
            1,
            "cell distance 2R+1 must merge: the dilated cubes are face-adjacent"
        );
    }

    #[test]
    fn two_cells_2r_plus_2_apart_are_two_clusters() {
        // The first non-merging distance.
        //   (40,10,40) -> cell (10,18,10)
        //   (64,10,40) -> cell (64/4, 18, 10) = (16,18,10)
        // Chebyshev cell distance = 6 = 2R + 2.
        //
        // Dilated x-spans [8,12] and [14,18]: cell x = 13 is empty on both
        // sides, so the two cubes are neither overlapping nor face-adjacent.
        let segs = segment_tile(
            &tile(vec![
                ((40, 10, 40), "minecraft:redstone_wire"),
                ((64, 10, 40), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &cfg(),
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 2, "cell distance 2R+2 leaves a one-cell gap");
    }

    #[test]
    fn margin_band_covers_cell_depths_0_through_2r() {
        // The band must cover every cell that could still merge with a cell in
        // the neighbouring tile. A cell at depth `a` here and depth `b` there
        // are Chebyshev a + b + 1 apart, and the merge threshold is 2R + 1, so
        // they merge when a + b + 1 <= 2R + 1, i.e. a + b <= 2R. Worst case
        // b = 0 gives a <= 2R, so depths 0..=2R must be in band — a width of
        // 2R + 1 cells, not 2R.
        //
        // Bounds (0,-64,0)..(127,63,127) at cell_size 4 give dims 32 on every
        // axis; band = 2 * closing_radius + 1 = 5 cells. So the near-face band
        // is cells 0..=4 and the far-face band starts at cell 32 - 5 = 27.
        //
        // Each case is segmented on its own tile so the depth under test is
        // isolated and cannot merge with another probe block.

        // Depth 0 — trivially in band. (2,10,2) -> (2/4, (10+64)/4, 2/4).
        assert_eq!(margin_cells((2, 10, 2)), vec![(0, 18, 0)]);

        // Depth 3 = 2R - 1, in band under both the old width 4 and the
        // correct width 5. Kept as a non-regression floor.
        // (13,10,13) -> (13/4, 18, 13/4) = (3,18,3).
        assert_eq!(margin_cells((13, 10, 13)), vec![(3, 18, 3)]);

        // Depth 4 = 2R: the LAST in-band layer, and the discriminating case.
        // A cell here is 4 + 0 + 1 = 5 = 2R+1 from a depth-0 cell in the
        // neighbouring tile, so it can still merge and MUST be stitched.
        // The old band width of 2R = 4 wrongly excluded it.
        // (17,10,17) -> (17/4, 18, 17/4) = (4,18,4).
        assert_eq!(
            margin_cells((17, 10, 17)),
            vec![(4, 18, 4)],
            "depth 2R must be IN band; this fails for any band width <= 4"
        );

        // Depth 5 = 2R + 1: the FIRST out-of-band layer, pinning the upper
        // edge. 5 + 0 + 1 = 6 = 2R+2 from the nearest neighbouring cell, which
        // cannot merge.
        // (21,10,21) -> (21/4, 18, 21/4) = (5,18,5).
        assert!(
            margin_cells((21, 10, 21)).is_empty(),
            "depth 2R+1 must be outside the band; this fails for any width >= 6"
        );

        // Same two-sided check against the far faces, which start at cell 27.
        // (108,10,108) -> (108/4, 18, 108/4) = (27,18,27), first in-band layer.
        assert_eq!(margin_cells((108, 10, 108)), vec![(27, 18, 27)]);
        // (107,10,107) -> (107/4, 18, 107/4) = (26,18,26), last interior layer.
        assert!(margin_cells((107, 10, 107)).is_empty(), "cell 26 is interior");
    }

    #[test]
    fn duplicate_positions_do_not_make_the_result_depend_on_input_order() {
        // The determinism guarantee, at its sharpest. Region and chunk readers
        // and overlapping tile margins all produce the same position twice.
        //
        // (10,-60,10) is inside the profile's substrate y-band (-64..=-50), so
        // "minecraft:stone" there classifies as Substrate and is discarded,
        // while "minecraft:redstone_wire" is artificial and yields a cluster.
        // Last-write-wins therefore gave 0 clusters one way and 1 the other.
        //
        // Canonical palette keys: "minecraft:redstone_wire[]" sorts before
        // "minecraft:stone[]", so redstone_wire wins in both orders.
        let forward = vec![
            ((10, -60, 10), "minecraft:stone"),
            ((10, -60, 10), "minecraft:redstone_wire"),
        ];
        let mut reverse = forward.clone();
        reverse.reverse();

        let a = segment_tile(&tile(forward), &profile(), &cfg(), &no_hints());
        let b = segment_tile(&tile(reverse), &profile(), &cfg(), &no_hints());

        assert_eq!(a, b, "a duplicated position must not let input order reach the output");
        assert_eq!(a.clusters.len(), 1, "redstone_wire wins the position, so one cluster");
    }

    /// The cluster ids a given input/config/profile combination produces.
    fn ids(config: &SegConfig, profile: &WorldProfile) -> Vec<ClusterId> {
        let segs = segment_tile(
            &tile(vec![
                ((40, 10, 40), "minecraft:redstone_wire"),
                ((41, 10, 40), "minecraft:repeater"),
            ]),
            profile,
            config,
            &no_hints(),
        );
        assert_eq!(segs.clusters.len(), 1, "the probe input must be a single cluster");
        segs.clusters.iter().map(|c| c.id).collect()
    }

    #[test]
    fn identical_config_and_profile_give_identical_cluster_ids() {
        assert_eq!(ids(&cfg(), &profile()), ids(&cfg(), &profile()));
    }

    #[test]
    fn cluster_ids_change_when_cell_size_changes() {
        // At cell_size 4 the two blocks land in cell (40/4,18,10) = (10,18,10);
        // at cell_size 8 they land in cell (40/8,(10+64)/8,40/8) = (5,9,5).
        // Distinct clusters either way, so the ids must not coincide.
        let coarse = SegConfig { cell_size: 8, ..cfg() };
        assert_ne!(ids(&cfg(), &profile()), ids(&coarse, &profile()));
    }

    #[test]
    fn cluster_ids_change_when_only_a_non_geometric_config_field_changes() {
        // The sharp case. `algorithm_version` cannot move a single cell, so
        // both runs produce the identical anchor cell (10,18,10) in the
        // identical tile. Hashing only (tile, anchor) therefore returned the
        // same ClusterId for output produced by a different algorithm — a
        // cache-poisoning hazard, since a consumer keyed on the id would serve
        // stale results after an algorithm change.
        let v2 = SegConfig { algorithm_version: 2, ..cfg() };
        assert_ne!(
            ids(&cfg(), &profile()),
            ids(&v2, &profile()),
            "a config change with no geometric effect must still change ids"
        );
    }

    #[test]
    fn cluster_ids_change_when_only_the_profile_changes() {
        // Neither profile classifies redstone_wire or repeater as substrate, so
        // the clusters are geometrically identical; only the pinned constants
        // that produced them differ, and the id must record that.
        let other = WorldProfile::new(
            ["minecraft:stone"].iter().map(|s| s.to_string()).collect(),
            (-64, -50),
        );
        assert_ne!(ids(&cfg(), &profile()), ids(&cfg(), &other));
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
