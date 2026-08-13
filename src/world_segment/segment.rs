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

// `Eq` is deliberately absent: `partition_floor_share` is an `Option<f32>`, and
// `f32` is only `PartialEq`, not `Eq`. No consumer needs `SegConfig: Eq`
// (`SegmentJob` derives only `Clone`/`Debug`, and the tests compare
// `TileSegments`, not configs), so `PartialEq` alone is sufficient.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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
    /// If set, names that account for at least this share of a partition's
    /// blocks within the substrate Y band are subtracted as that partition's
    /// floor, in addition to the global palette. `None` disables (default).
    /// Generic: floors are whatever material locally dominates a partition's
    /// band layer.
    ///
    /// # Scope
    ///
    /// This is a *per-partition* subtraction, so it only bites where a block
    /// actually falls inside some hint. It is therefore inert unless partitions
    /// are in force — i.e. [`PartitionPolicy::HardCut`] with non-empty hints.
    /// Under [`PartitionPolicy::Off`] and [`PartitionPolicy::Prefer`] no block
    /// has a partition (both take the unpartitioned path), so no floor material
    /// is ever detected and classification stays global-palette-only. A block
    /// that lies outside every hint under `HardCut` likewise keeps
    /// global-palette-only classification, since it belongs to no partition
    /// whose local floor could be defined.
    pub partition_floor_share: Option<f32>,
    /// If set, a partition/Y layer inside the substrate band whose occupied
    /// XZ coverage reaches this share is subtracted in full. Unlike
    /// `partition_floor_share`, this handles patterned or multi-material floors
    /// where no one block name dominates. `None` disables it (default).
    #[serde(default)]
    pub partition_dense_layer_coverage: Option<f32>,
    /// Preserve one existing block directly below every block that survives
    /// substrate/floor subtraction.
    ///
    /// This is deliberately unconditional and exactly one block deep: no
    /// Minecraft-physics or "is this support necessary?" inference is made.
    /// A floor block beneath retained build content is part of the emitted
    /// build even when that block would otherwise be classified as substrate
    /// or partition floor. Support enrichment happens after segmentation and
    /// support-aware component splitting ignores it, so support footprints do
    /// not reconnect independent builds. Newly retained supports do not
    /// recursively retain the blocks beneath them.
    ///
    /// `false` by default for backward compatibility.
    #[serde(default)]
    pub preserve_support_blocks: bool,
    /// When `Some`, undo the morphological closing where it has fused two
    /// genuinely disconnected builds.
    ///
    /// The closing (dilate by `closing_radius` then label) is *meant* to unify
    /// a single build's near-parts: two occupied cells within `2R` on every
    /// axis merge, bridging gaps up to about `(2R + 1) * cell_size` blocks.
    /// That is correct for one build's scattered pieces, but it also fuses two
    /// separate builds that merely happen to sit that close — the failure that
    /// produced a single "build" spanning two spatially-disconnected plots.
    ///
    /// With this set, each merged cluster's **original (undilated)** cells are
    /// re-examined: if they fall into two or more six-connected components that
    /// are each substantial (see [`DisconnectedSplit`]) and genuinely far
    /// apart, the cluster is split into one cluster per substantial component,
    /// with the small leftover fragments attached to their nearest substantial
    /// component. Clusters that do not meet the criteria are left exactly as
    /// the closing produced them, so a single build whose parts the closing was
    /// meant to unify is never fragmented.
    ///
    /// `None` (default) disables the split. Because it is folded into
    /// [`SegConfig::config_hash`] as a backward-compatible extension that
    /// appends nothing when `None`, every existing `ClusterId` is preserved
    /// byte-for-byte and the determinism goldens are unchanged. A future
    /// extraction opts in by setting this; already-extracted builds stay as
    /// they were until re-extracted.
    pub split_disconnected: Option<DisconnectedSplit>,
    /// Ignore blocks that do not fall inside any hard-cut partition hint.
    ///
    /// This is useful for regular plot/cell layouts: the hints describe the
    /// build-bearing interiors and roads or gutters between them never become
    /// schematics. It is only effective with [`PartitionPolicy::HardCut`] and
    /// non-empty hints. The default is `false` for backward compatibility.
    #[serde(default)]
    pub drop_unpartitioned: bool,
}

/// Thresholds governing [`SegConfig::split_disconnected`].
///
/// All three conditions must hold for a merged cluster to be split, which is
/// what keeps the split from fragmenting a legitimate single build:
///
/// * **`min_component_blocks`** — a component must hold at least this many
///   blocks to count as a *seed* (a build in its own right). A single build's
///   detached bits (a lamp, a wire run, a stray pillar) fall below this and are
///   attached to the nearest seed rather than split off.
/// * **`min_component_share`** — a seed must also account for at least this
///   fraction of the merged cluster's blocks. At the `0.40` default at most two
///   components can qualify, so a cluster only splits when it is dominated by
///   two comparably-sized masses — the signature of two builds accidentally
///   fused, not of one build with many small parts.
/// * **`min_gap_cells`** — two seeds are only split apart when the empty gap
///   between them is at least this many cells (Chebyshev, in occupancy cells).
///   A smaller gap is treated as internal to one build — precisely the
///   near-parts the closing is meant to unify.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DisconnectedSplit {
    /// Minimum blocks for a component to be treated as a standalone build.
    pub min_component_blocks: u64,
    /// Minimum share (0.0..=1.0) of the merged cluster's blocks a seed must hold.
    pub min_component_share: f32,
    /// Minimum Chebyshev cell gap between two seeds for them to be split apart.
    pub min_gap_cells: u32,
}

impl Default for DisconnectedSplit {
    fn default() -> Self {
        // Defaults chosen to catch the demonstrated failure (two ~180k-block
        // plot builds fused across a 7-block / 2-cell gap) while leaving
        // legitimate multi-part single builds untouched:
        //   - a real standalone build is far larger than 4_096 blocks;
        //   - two comparable halves each >= 40% is the two-builds signature and
        //     mathematically admits at most two seeds;
        //   - a >= 2-cell gap means a genuine empty cell layer separates them,
        //     not merely a diagonal touch.
        DisconnectedSplit {
            min_component_blocks: 4_096,
            min_component_share: 0.40,
            min_gap_cells: 2,
        }
    }
}

impl SegConfig {
    /// Stable hash of the inputs, other than the tile's own blocks, that can
    /// change what segmentation produces.
    ///
    /// Folded into every [`ClusterId`], so an id identifies a cluster *under a
    /// stated configuration* rather than merely a position. Two runs that
    /// disagree on `cell_size`, `closing_radius`, `min_cluster_blocks`,
    /// `partition_policy`, `algorithm_version`, either partition-floor rule,
    /// the disconnected split/drop rules, the world profile, or — under
    /// [`PartitionPolicy::HardCut`] — the partition hints, can never mint the
    /// same id, which is what makes the ids safe to use as cache keys.
    ///
    /// # What is covered
    ///
    /// Every field of `SegConfig`, the [`WorldProfile`]'s own hash, and the
    /// full geometry of the hints via [`PartitionIndex::hints_hash`]. If a
    /// `SegConfig` field is added, add it here too — an unhashed field is a
    /// silent id collision between runs that genuinely differ.
    ///
    /// The hints are hashed **only under `HardCut`**, the one policy that reads
    /// them. Under `Off` and `Prefer` the hints cannot affect a single cluster,
    /// so folding them in would make ids differ between runs whose output is
    /// byte-identical; a fixed, domain-separated constant stands in instead.
    /// `policy_off_reproduces_the_unpartitioned_result` pins that passing hints
    /// under `Off` is indistinguishable from passing none.
    ///
    /// # What is NOT covered
    ///
    /// The tile's blocks, its bounds and its [`TileId`]. Those are not config:
    /// the tile id is hashed into [`ClusterId`] separately, and the blocks are
    /// what the id is *about*. An id is therefore a claim about a cluster of a
    /// given tile under a given configuration, and says nothing about whether
    /// the tile's contents have since changed — a consumer that caches across
    /// world edits must version the tile itself.
    pub fn config_hash(&self, profile: &WorldProfile, partitions: &PartitionIndex) -> ContentId {
        // Explicit, pinned discriminants: derived ordering would silently
        // renumber if a variant were inserted, changing ids without any
        // behaviour change.
        let policy: u8 = match self.partition_policy {
            PartitionPolicy::HardCut => 0,
            PartitionPolicy::Prefer => 1,
            PartitionPolicy::Off => 2,
        };
        let hints = match self.partition_policy {
            PartitionPolicy::HardCut => partitions.hints_hash(),
            // Domain-separated, and distinct from any real `hints_hash` value:
            // that one's first framed part is `b"parthints.v1"`.
            PartitionPolicy::Prefer | PartitionPolicy::Off => {
                ContentId::of(&[b"parthints.ignored"])
            }
        };
        // `partition_floor_share` is folded in as a **backward-compatible
        // extension**, not a format bump: a `None` config appends nothing and
        // therefore digests byte-for-byte identically to the pre-feature `v2`
        // layout, so every existing `ClusterId` — golden-pinned in the
        // determinism suite — is unchanged, honouring "None preserves behavior
        // byte-for-byte". A `Some` config appends one extra `[1u8, f32 LE]`
        // part; since `ContentId::of` length-frames each part *and* the running
        // digest distinguishes an 8-part input from a 9-part one, a `Some`
        // config can never collide with any `None` config, and two `Some`s
        // differ by their float bits. `Some(0.0)` is thus distinct from `None`.
        let floor_share: Option<[u8; 5]> = self.partition_floor_share.map(|s| {
            let mut v = [1u8; 5];
            v[1..].copy_from_slice(&s.to_le_bytes());
            v
        });
        // `split_disconnected` is folded in as a second backward-compatible
        // extension, on exactly the same terms as `partition_floor_share`: a
        // `None` config appends nothing and therefore digests byte-for-byte
        // identically to the pre-feature layout, so every golden-pinned
        // `ClusterId` is unchanged. A `Some` config appends one framed part
        // `[1u8, min_component_blocks LE(8), min_component_share LE(4),
        // min_gap_cells LE(4)]` (17 bytes); `ContentId::of` length-frames each
        // part and the running digest distinguishes part counts, so a `Some`
        // can never collide with any `None`, and two `Some`s differ by content.
        let split: Option<[u8; 17]> = self.split_disconnected.as_ref().map(|s| {
            let mut v = [0u8; 17];
            v[0] = 1;
            v[1..9].copy_from_slice(&s.min_component_blocks.to_le_bytes());
            v[9..13].copy_from_slice(&s.min_component_share.to_le_bytes());
            v[13..17].copy_from_slice(&s.min_gap_cells.to_le_bytes());
            v
        });
        // `drop_unpartitioned` is a third backward-compatible extension. It
        // appends nothing while false, preserving every pre-feature id.
        let drop_unpartitioned = self.drop_unpartitioned.then_some([1u8]);
        let dense_layer: Option<[u8; 5]> = self.partition_dense_layer_coverage.map(|coverage| {
            let mut value = [2u8; 5];
            value[1..].copy_from_slice(&coverage.to_le_bytes());
            value
        });
        // Another backward-compatible extension. False appends nothing; true
        // is domain-separated from the other one-byte extension.
        let preserve_support_blocks = self.preserve_support_blocks.then_some([3u8]);
        // Bound to locals so the `to_le_bytes` temporaries outlive the
        // `ContentId::of` call rather than being dropped at the end of a `let`.
        let cell = self.cell_size.to_le_bytes();
        let closing = self.closing_radius.to_le_bytes();
        let min = self.min_cluster_blocks.to_le_bytes();
        let algo = self.algorithm_version.to_le_bytes();
        let policy_bytes = [policy];
        let profile_hash = profile.profile_hash();
        let mut parts: Vec<&[u8]> = vec![
            // v2: the partition hints joined the input in this version.
            // `partition_floor_share` is a backward-compatible extension of v2
            // (appended only when `Some`), so the version stays v2.
            b"segconfig.v2",
            &cell,
            &closing,
            &min,
            &policy_bytes,
            &algo,
            profile_hash.as_bytes(),
            hints.as_bytes(),
        ];
        if let Some(bytes) = floor_share.as_ref() {
            parts.push(bytes);
        }
        // Ordering matters: `split_disconnected` is appended *after*
        // `partition_floor_share`. A config with only `split_disconnected` set
        // must not digest like one with only `partition_floor_share` set — the
        // two extension parts have different lengths (5 vs 17) and length
        // framing keeps them distinct, but appending in a fixed order also
        // keeps the two independent `Some`/`None` combinations unambiguous.
        if let Some(bytes) = split.as_ref() {
            parts.push(bytes);
        }
        if let Some(bytes) = drop_unpartitioned.as_ref() {
            parts.push(bytes);
        }
        if let Some(bytes) = dense_layer.as_ref() {
            parts.push(bytes);
        }
        if let Some(bytes) = preserve_support_blocks.as_ref() {
            parts.push(bytes);
        }
        ContentId::of(&parts)
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
            partition_floor_share: None,
            partition_dense_layer_coverage: None,
            preserve_support_blocks: false,
            split_disconnected: None,
            drop_unpartitioned: false,
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
///
/// # `cell` is not a key
///
/// Under [`PartitionPolicy::HardCut`] blocks are partitioned **per block**, so
/// a single cell can be occupied in two partitions' grids at once and belong to
/// a different cluster in each. Both clusters then emit a `MarginCell` with the
/// *same* `cell` and different `cluster`. That is correct output, not a bug —
/// see `segment_tile` step 1 and `assign_ids`.
///
/// A consumer that stitches clusters across tile faces by looking for margin
/// entries at coincident cells **must not union two entries whose `partition`
/// differs**. Doing so re-forms exactly the boundary-spanning cluster that
/// per-block partitioning exists to prevent. `partition` is carried here for
/// precisely that reason: `cell` alone cannot tell the two entries apart.
///
/// Entries in the same partition (including two `None`s) are the ones a
/// stitcher may legitimately consider joining.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MarginCell {
    pub cell: (i32, i32, i32),
    pub cluster: ClusterId,
    /// The partition the owning cluster fell in, mirroring
    /// [`Cluster::partition_id`]. `None` under `Off`/`Prefer`, or when the cell
    /// lies outside every hint.
    pub partition: Option<String>,
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
    segment_tile_inner(tile, profile, config, partitions, false).0
}

/// Sibling of [`segment_tile`] that also reports, for every surviving
/// (non-substrate) world block, which [`ClusterId`] it belongs to.
///
/// The returned `TileSegments` is identical to what `segment_tile` produces
/// for the same input — both are driven by the same inner labelling, so
/// there is a single source of truth and no risk of the two drifting apart.
///
/// The map only contains blocks whose cluster survives the
/// `min_cluster_blocks` filter; substrate blocks and blocks in dropped
/// clusters are absent. It is a `BTreeMap` keyed by world position so the
/// result is order-independent regardless of how the tile's blocks were
/// iterated.
pub fn segment_tile_membership(
    tile: &VoxelTile,
    profile: &WorldProfile,
    config: &SegConfig,
    partitions: &PartitionIndex,
) -> (TileSegments, BTreeMap<(i32, i32, i32), ClusterId>) {
    segment_tile_inner(tile, profile, config, partitions, true)
}

fn segment_tile_inner(
    tile: &VoxelTile,
    profile: &WorldProfile,
    config: &SegConfig,
    partitions: &PartitionIndex,
    want_membership: bool,
) -> (TileSegments, BTreeMap<(i32, i32, i32), ClusterId>) {
    let bounds = tile.bounds();
    let cell = config.cell_size.max(1);
    // Every ClusterId minted below is bound to this hash, so ids produced
    // under different settings or a different profile can never collide.
    let config_id = config.config_hash(profile, partitions);

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
    // Partition-scoped floor materials, keyed by the SAME partition index the
    // loop below resolves each block to. Empty (and skipped entirely) unless the
    // caller opted in with `partition_floor_share` AND partitions are in force,
    // so the `None` default is byte-for-byte inert: `floor_materials.get(..)` is
    // always `None` and the extra `continue` branch never fires.
    let (band_lo, band_hi) = profile.substrate_y_band;
    let floor_materials: BTreeMap<u32, std::collections::BTreeSet<String>> =
        match (use_partitions, config.partition_floor_share) {
            (true, Some(share)) => partition_floor_materials(tile, profile, partitions, share),
            _ => BTreeMap::new(),
        };
    let dense_floor_layers = match (use_partitions, config.partition_dense_layer_coverage) {
        (true, Some(coverage)) => partition_dense_floor_layers(tile, profile, partitions, coverage),
        _ => std::collections::BTreeSet::new(),
    };

    let mut artificial: Vec<((i32, i32, i32), Option<u32>)> = Vec::new();
    let mut grids: BTreeMap<Option<u32>, OccupancyGrid> = BTreeMap::new();
    for (pos, state) in tile.blocks() {
        if classify(state, pos.1, profile) == BlockClass::Substrate {
            continue;
        }
        // Keyed by index, not by name, so grouping is independent of the order
        // the caller supplied the hints in.
        let pidx = if use_partitions {
            partitions.id_index_at(pos.0, pos.1, pos.2)
        } else {
            None
        };
        if use_partitions && config.drop_unpartitioned && pidx.is_none() {
            continue;
        }
        if pidx.is_some_and(|partition| dense_floor_layers.contains(&(partition, pos.1))) {
            continue;
        }
        // Partition-scoped floor subtraction: a block in the substrate band
        // whose name locally dominates its own partition's band layer is that
        // partition's floor, and is dropped exactly like global substrate. This
        // is *in addition to* the global palette check above; a block outside
        // every hint has `pidx == None` and is never touched here.
        if let Some(p) = pidx {
            if pos.1 >= band_lo && pos.1 <= band_hi {
                if let Some(names) = floor_materials.get(&p) {
                    if names.contains(state.get_name()) {
                        continue;
                    }
                }
            }
        }
        artificial.push((pos, pidx));
        grids
            .entry(pidx)
            .or_insert_with(|| OccupancyGrid::new(origin, dims, cell))
            .mark(pos.0, pos.1, pos.2);
    }
    if artificial.is_empty() {
        return (
            TileSegments {
                tile_id: tile.id(),
                clusters: Vec::new(),
                margin: Vec::new(),
            },
            BTreeMap::new(),
        );
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

    let (cluster_of_cell, mut partition_of_cluster) = assign_ids(config_id, tile.id(), groups);

    // 3b. Undo the closing where it fused two disconnected builds.
    //
    // The closing at step 2/3 groups by the *dilated* occupancy, so two builds
    // that sit within `2R` cells of each other land in one group even though
    // their original cells never touch. When enabled, re-examine each group's
    // original cells and split it back apart if it is really two (or more)
    // substantial, well-separated builds. Disabled (`None`) by default, in
    // which case this is a no-op and `cluster_of_cell` is untouched.
    let cluster_of_cell = match &config.split_disconnected {
        Some(policy) => split_disconnected_clusters(
            policy,
            config_id,
            tile.id(),
            &artificial,
            &geometry,
            cluster_of_cell,
            &mut partition_of_cluster,
        ),
        None => cluster_of_cell,
    };

    // 4. Fold original blocks back into their cluster.
    //
    // Looked up by `(partition, cell)`: a cell shared by two partitions belongs
    // to a different cluster in each, and each block resolves through its own
    // partition.
    let mut acc: BTreeMap<ClusterId, ClusterAcc> = BTreeMap::new();
    // Pre-filter membership: every block's cluster, before the
    // `min_cluster_blocks` filter below decides which clusters survive. Kept
    // only when the caller actually wants it, so `segment_tile` pays nothing
    // extra.
    let mut pos_to_cluster: BTreeMap<(i32, i32, i32), ClusterId> = BTreeMap::new();
    for (pos, pidx) in artificial {
        let cell_coord = geometry.cell_of(pos.0, pos.1, pos.2);
        let Some(id) = cluster_of_cell.get(&(pidx, cell_coord)) else {
            continue;
        };
        acc.entry(*id)
            .or_insert_with(ClusterAcc::new)
            .push(pos, cell_coord);
        if want_membership {
            pos_to_cluster.insert(pos, *id);
        }
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
        // Cloned once per cluster, not once per cell.
        let partition = partition_of_cluster.get(id).cloned().flatten();
        for cell_coord in &a.cells {
            if in_margin(*cell_coord, dims, band) {
                margin.push(MarginCell {
                    cell: *cell_coord,
                    cluster: *id,
                    partition: partition.clone(),
                });
            }
        }
    }
    // `(cell, cluster)` is already unique — a cluster records each of its cells
    // once — so the `partition` term never actually decides an ordering. It is
    // appended anyway so that the sort key covers every field of `MarginCell`:
    // if a future change makes `(cell, cluster)` non-unique, the order stays
    // total and order-independent instead of silently becoming input-dependent.
    margin.sort_by(|a, b| {
        a.cell
            .cmp(&b.cell)
            .then_with(|| a.cluster.cmp(&b.cluster))
            .then_with(|| a.partition.cmp(&b.partition))
    });

    // Only blocks whose cluster is in `kept` are exported: a block whose
    // cluster was dropped by `min_cluster_blocks` is not part of any emitted
    // build, so it must not appear in the membership map either.
    let membership = if want_membership {
        pos_to_cluster
            .into_iter()
            .filter(|(_, id)| kept.contains(id))
            .collect()
    } else {
        BTreeMap::new()
    };

    (
        TileSegments {
            tile_id: tile.id(),
            clusters,
            margin,
        },
        membership,
    )
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
        self.min = (
            self.min.0.min(pos.0),
            self.min.1.min(pos.1),
            self.min.2.min(pos.2),
        );
        self.max = (
            self.max.0.max(pos.0),
            self.max.1.max(pos.1),
            self.max.2.max(pos.2),
        );
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
        let Some(label) = labels.label_of(cell) else {
            continue;
        };
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
        let Some(anchor) = cells.iter().next().copied() else {
            continue;
        };
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

/// Undo the closing where it fused two disconnected builds into one cluster.
///
/// See [`SegConfig::split_disconnected`] and [`DisconnectedSplit`]. Runs only
/// when the caller opted in, and rewrites the finished `cluster_of_cell`:
///
/// * the six-connected components of each `(partition, cluster)`'s **original**
///   cells are re-derived — the dilation is deliberately not consulted, since
///   that is exactly what over-merged them;
/// * a component is a *seed* if it clears both `min_component_blocks` and
///   `min_component_share` of the merged cluster;
/// * if two or more seeds exist and every seed pair is at least `min_gap_cells`
///   apart (Chebyshev, in cells), the cluster is split: one fresh `ClusterId`
///   per seed, anchored on that seed's own lexmin cell — which keeps the
///   anchor-injectivity invariant `assign_ids` relies on, and re-mints the
///   *same* id for the seed that still owns the original global lexmin — and
///   every non-seed fragment is attached to its nearest seed;
/// * clusters that are one component, or lack two well-separated seeds, are
///   returned byte-for-byte unchanged.
///
/// Determinism: cell groups come from a `BTreeMap` scan (ascending), components
/// are emitted in ascending-lexmin order, seed ids derive from content, and
/// fragment attachment breaks ties on the smaller seed anchor — so the output
/// depends only on content, never on iteration accidents.
fn split_disconnected_clusters(
    policy: &DisconnectedSplit,
    config: ContentId,
    tile: TileId,
    artificial: &[((i32, i32, i32), Option<u32>)],
    geometry: &OccupancyGrid,
    cluster_of_cell: BTreeMap<(Option<u32>, (i32, i32, i32)), ClusterId>,
    partition_of_cluster: &mut BTreeMap<ClusterId, Option<String>>,
) -> BTreeMap<(Option<u32>, (i32, i32, i32)), ClusterId> {
    // Blocks per (partition, cell): the substantiality thresholds are in
    // blocks, but the closing and this split both work in cells, so we need the
    // per-cell block tally step 4 would otherwise be the first to compute.
    let mut cell_blocks: BTreeMap<(Option<u32>, (i32, i32, i32)), u64> = BTreeMap::new();
    for (pos, pidx) in artificial {
        let cell = geometry.cell_of(pos.0, pos.1, pos.2);
        *cell_blocks.entry((*pidx, cell)).or_insert(0) += 1;
    }

    // The cells of each merged cluster, keyed by `(partition, cluster)` so the
    // per-partition `HardCut` case (one cell in two partitions' clusters) stays
    // separated exactly as `cluster_of_cell` already keeps it.
    let mut cells_of: BTreeMap<(Option<u32>, ClusterId), Vec<(i32, i32, i32)>> = BTreeMap::new();
    for ((pidx, cell), id) in &cluster_of_cell {
        cells_of.entry((*pidx, *id)).or_default().push(*cell);
    }

    let mut out: BTreeMap<(Option<u32>, (i32, i32, i32)), ClusterId> = BTreeMap::new();
    for ((pidx, old_id), cells) in cells_of {
        let comps = six_connected_components(&cells);
        if comps.len() < 2 {
            // One component: nothing the closing bridged. Leave it be.
            for c in &cells {
                out.insert((pidx, *c), old_id);
            }
            continue;
        }
        let partition = partition_of_cluster.get(&old_id).cloned().flatten();
        let comp_blocks: Vec<u64> = comps
            .iter()
            .map(|comp| {
                comp.iter()
                    .map(|c| cell_blocks.get(&(pidx, *c)).copied().unwrap_or(0))
                    .sum()
            })
            .collect();
        let total: u64 = comp_blocks.iter().sum();
        // A seed is substantial both absolutely and as a share of the whole.
        let share_floor = (f64::from(policy.min_component_share) * total as f64).ceil() as u64;
        let seeds: Vec<usize> = (0..comps.len())
            .filter(|&i| {
                comp_blocks[i] >= policy.min_component_blocks && comp_blocks[i] >= share_floor
            })
            .collect();
        // Two-plus seeds, and every seed pair clears the gap tolerance: only
        // then is the "two separate builds" reading safe. Otherwise the closing
        // was doing its job — leave the cluster whole.
        let split_ok = seeds.len() >= 2
            && seeds.iter().enumerate().all(|(a, &si)| {
                seeds[a + 1..]
                    .iter()
                    .all(|&sj| min_cell_gap_at_least(&comps[si], &comps[sj], policy.min_gap_cells))
            });
        if !split_ok {
            for c in &cells {
                out.insert((pidx, *c), old_id);
            }
            continue;
        }
        // Mint one id per seed, anchored on the seed's own lexmin original cell.
        let mut seeds_meta: Vec<(usize, ClusterId, (i32, i32, i32))> = Vec::new();
        for &si in &seeds {
            let anchor = *comps[si]
                .iter()
                .min()
                .expect("a seed component is non-empty");
            let id = ClusterId::new(config, tile, partition.as_deref(), anchor);
            partition_of_cluster
                .entry(id)
                .or_insert_with(|| partition.clone());
            for c in &comps[si] {
                out.insert((pidx, *c), id);
            }
            seeds_meta.push((si, id, anchor));
        }
        // Attach each non-seed fragment to its nearest seed, by Chebyshev
        // distance between component centroids, ties broken on the smaller seed
        // anchor. A fragment is small by definition, so the exact metric barely
        // matters; what matters is that it is deterministic and every block
        // keeps a home (no debris, no leftover of the old id).
        let seed_set: std::collections::BTreeSet<usize> = seeds.iter().copied().collect();
        for (ci, comp) in comps.iter().enumerate() {
            if seed_set.contains(&ci) {
                continue;
            }
            let cc = centroid(comp);
            let mut best: Option<(i64, (i32, i32, i32), ClusterId)> = None;
            for (si, id, anchor) in &seeds_meta {
                let d = cheb(cc, centroid(&comps[*si]));
                let cand = (d, *anchor, *id);
                let take = match &best {
                    None => true,
                    Some(b) => cand.0 < b.0 || (cand.0 == b.0 && cand.1 < b.1),
                };
                if take {
                    best = Some(cand);
                }
            }
            let id = best.expect("at least one seed exists").2;
            for c in comp {
                out.insert((pidx, *c), id);
            }
        }
    }
    out
}

/// Six-connected components of a set of cells, each returned sorted ascending
/// and the whole list in ascending-lexmin order, so callers get a deterministic
/// grouping independent of the input slice's order.
fn six_connected_components(cells: &[(i32, i32, i32)]) -> Vec<Vec<(i32, i32, i32)>> {
    let set: std::collections::BTreeSet<(i32, i32, i32)> = cells.iter().copied().collect();
    let mut seen: std::collections::BTreeSet<(i32, i32, i32)> = std::collections::BTreeSet::new();
    let mut comps: Vec<Vec<(i32, i32, i32)>> = Vec::new();
    for &start in &set {
        if seen.contains(&start) {
            continue;
        }
        seen.insert(start);
        let mut stack = vec![start];
        let mut comp: Vec<(i32, i32, i32)> = Vec::new();
        while let Some(c) = stack.pop() {
            comp.push(c);
            let nbrs = [
                (c.0 - 1, c.1, c.2),
                (c.0 + 1, c.1, c.2),
                (c.0, c.1 - 1, c.2),
                (c.0, c.1 + 1, c.2),
                (c.0, c.1, c.2 - 1),
                (c.0, c.1, c.2 + 1),
            ];
            for nb in nbrs {
                if set.contains(&nb) && seen.insert(nb) {
                    stack.push(nb);
                }
            }
        }
        comp.sort_unstable();
        comps.push(comp);
    }
    comps
}

/// True iff the minimum Chebyshev cell distance between `a` and `b` is at least
/// `g`. Tests, rather than computes, the distance: it scans the smaller set and
/// looks only in the `(2g-1)^3` box around each cell, so cost is
/// `O(min(|a|,|b|) * (2g-1)^3)` with `g` the small gap tolerance — never the
/// quadratic all-pairs distance.
fn min_cell_gap_at_least(a: &[(i32, i32, i32)], b: &[(i32, i32, i32)], g: u32) -> bool {
    if g == 0 {
        return true;
    }
    let (scan, other) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let other_set: std::collections::BTreeSet<(i32, i32, i32)> = other.iter().copied().collect();
    let r = (g - 1) as i32;
    for c in scan {
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if other_set.contains(&(c.0 + dx, c.1 + dy, c.2 + dz)) {
                        // A cell within Chebyshev `g-1` exists, so gap < g.
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Integer centroid (floored mean) of a non-empty cell set.
fn centroid(cells: &[(i32, i32, i32)]) -> (i64, i64, i64) {
    let n = cells.len() as i64;
    let (mut sx, mut sy, mut sz) = (0i64, 0i64, 0i64);
    for c in cells {
        sx += i64::from(c.0);
        sy += i64::from(c.1);
        sz += i64::from(c.2);
    }
    (sx / n, sy / n, sz / n)
}

/// Chebyshev distance between two points.
fn cheb(a: (i64, i64, i64), b: (i64, i64, i64)) -> i64 {
    (a.0 - b.0)
        .abs()
        .max((a.1 - b.1).abs())
        .max((a.2 - b.2).abs())
}

/// For each partition present in the tile, the set of block names that
/// locally dominate that partition's blocks *within the substrate Y band* —
/// i.e. names whose count reaches `share` of the partition's in-band block
/// total. These are subtracted as the partition's floor in addition to the
/// global palette.
///
/// The scope is deliberately per-partition: a plot's floor is an owner-chosen
/// material that is globally rare (so no global palette can catch it) yet
/// locally dominant within its own plot. A global threshold structurally
/// cannot express that; the partition is the right scope, and this is the one
/// place that knows each block's partition.
///
/// # Determinism
///
/// Counting is into a `BTreeMap` keyed by `(partition index, name)`, and the
/// per-partition total into a `BTreeMap` keyed by index, so neither the counts
/// nor which names cross the threshold depend on the order the tile's blocks
/// were iterated. Membership depends only on content.
///
/// Only blocks that actually fall inside some hint (`id_index_at` is `Some`)
/// are counted; a block outside every hint has no partition whose local floor
/// it could define.
fn partition_floor_materials(
    tile: &VoxelTile,
    profile: &WorldProfile,
    partitions: &PartitionIndex,
    share: f32,
) -> BTreeMap<u32, std::collections::BTreeSet<String>> {
    let (lo, hi) = profile.substrate_y_band;
    let mut counts: BTreeMap<(u32, String), u64> = BTreeMap::new();
    let mut totals: BTreeMap<u32, u64> = BTreeMap::new();
    for (pos, state) in tile.blocks() {
        if pos.1 < lo || pos.1 > hi {
            continue;
        }
        let Some(p) = partitions.id_index_at(pos.0, pos.1, pos.2) else {
            continue;
        };
        *counts.entry((p, state.get_name().to_string())).or_insert(0) += 1;
        *totals.entry(p).or_insert(0) += 1;
    }

    let share = share as f64;
    let mut floors: BTreeMap<u32, std::collections::BTreeSet<String>> = BTreeMap::new();
    for ((p, name), count) in counts {
        // `totals[&p]` exists and is >= count >= 1, so the division is safe.
        let total = totals[&p];
        if (count as f64) / (total as f64) >= share {
            floors.entry(p).or_default().insert(name);
        }
    }
    floors
}

/// Dense floor detection by partition and exact Y layer. Counting occupancy
/// rather than block names is what makes patterned/multi-material floors one
/// substrate surface instead of several individually-minority materials.
fn partition_dense_floor_layers(
    tile: &VoxelTile,
    profile: &WorldProfile,
    partitions: &PartitionIndex,
    coverage: f32,
) -> std::collections::BTreeSet<(u32, i32)> {
    let (lo, hi) = profile.substrate_y_band;
    let mut occupied: BTreeMap<(u32, i32), u64> = BTreeMap::new();
    for (pos, _state) in tile.blocks() {
        if pos.1 < lo || pos.1 > hi {
            continue;
        }
        if let Some(partition) = partitions.id_index_at(pos.0, pos.1, pos.2) {
            *occupied.entry((partition, pos.1)).or_default() += 1;
        }
    }

    let bounds = tile.bounds();
    let coverage = coverage.clamp(0.0, 1.0) as f64;
    occupied
        .into_iter()
        .filter_map(|((partition, y), count)| {
            let hint = partitions.hint_of_index(partition);
            if hint.y_range.is_some_and(|(y0, y1)| y < y0 || y > y1) {
                return None;
            }
            let (x0, x1, z0, z1) = hint.bbox_xz;
            let x0 = x0.max(bounds.min.0);
            let x1 = x1.min(bounds.max.0);
            let z0 = z0.max(bounds.min.2);
            let z1 = z1.min(bounds.max.2);
            if x0 > x1 || z0 > z1 {
                return None;
            }
            let area = (i64::from(x1) - i64::from(x0) + 1) * (i64::from(z1) - i64::from(z0) + 1);
            ((count as f64) / (area as f64) >= coverage).then_some((partition, y))
        })
        .collect()
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
            ["minecraft:stone", "minecraft:bedrock"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            (-64, -50),
        )
    }

    fn cfg() -> SegConfig {
        SegConfig {
            cell_size: 4,
            closing_radius: 2,
            min_cluster_blocks: 1,
            ..SegConfig::default()
        }
    }

    fn bounds() -> TileBounds {
        TileBounds {
            min: (0, -64, 0),
            max: (127, 63, 127),
        }
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
        assert!(
            segs.clusters.is_empty(),
            "a tile of pure substrate yields no clusters"
        );
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
        let config = SegConfig {
            min_cluster_blocks: 2,
            ..cfg()
        };
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
        assert_eq!(
            segs.clusters.len(),
            1,
            "the single-block cluster is dropped"
        );
        assert_eq!(segs.clusters[0].block_count, 2);
    }

    #[test]
    fn hard_cut_prevents_merging_across_a_partition_boundary() {
        // Two blocks 8 apart would normally bridge; a boundary between them
        // must stop that.
        let hints = PartitionIndex::new(vec![
            PartitionHint {
                id: "left".into(),
                bbox_xz: (0, 13, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "right".into(),
                bbox_xz: (14, 127, 0, 127),
                y_range: None,
            },
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
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
        let mut got: Vec<_> = segs
            .clusters
            .iter()
            .map(|c| c.partition_id.clone().unwrap())
            .collect();
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
            PartitionHint {
                id: "near".into(),
                bbox_xz: (0, 127, 0, 3),
                y_range: None,
            },
            PartitionHint {
                id: "far".into(),
                bbox_xz: (0, 127, 4, 127),
                y_range: None,
            },
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
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
        assert_ne!(
            segs.clusters[0].id, segs.clusters[1].id,
            "ids must stay distinct"
        );
        let mut got: Vec<_> = segs
            .clusters
            .iter()
            .map(|c| c.partition_id.clone().unwrap())
            .collect();
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
            PartitionHint {
                id: "L".into(),
                bbox_xz: (0, 61, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "R".into(),
                bbox_xz: (62, 127, 0, 127),
                y_range: None,
            },
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
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
        assert_eq!(
            segs.clusters.len(),
            2,
            "the boundary splits the straddling cell"
        );

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
            PartitionHint {
                id: "L".into(),
                bbox_xz: (0, 61, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "R".into(),
                bbox_xz: (62, 127, 0, 127),
                y_range: None,
            },
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
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
    fn margin_cells_at_a_shared_cell_carry_their_own_partitions() {
        // Per-block partitioning lets one cell be occupied in two partitions'
        // grids, so two *different* clusters can emit a margin entry for the
        // SAME cell. `MarginCell` used to record only (cell, cluster); a
        // stitching stage keying on `cell` would union the two clusters and
        // re-form precisely the boundary-spanning cluster that per-block
        // partitioning exists to prevent. The partition is carried so the
        // stitcher can refuse that union.
        //
        // Boundary at x = 2, deliberately NOT a multiple of cell_size 4, and
        // deliberately inside the margin band so the entries are exported:
        //   cell x 0 covers world x 0..=3, and 2 falls strictly inside it.
        //   (1,10,40) -> cell (1/4, (10+64)/4, 40/4) = (0,18,10)  -> "L"
        //   (2,10,40) -> cell (2/4, 18,       40/4) = (0,18,10)  -> "R"
        // cell x = 0 < band 5, so both are in the margin band.
        let hints = PartitionIndex::new(vec![
            PartitionHint {
                id: "L".into(),
                bbox_xz: (0, 1, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "R".into(),
                bbox_xz: (2, 127, 0, 127),
                y_range: None,
            },
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
        let segs = segment_tile(
            &tile(vec![
                ((1, 10, 40), "minecraft:redstone_wire"),
                ((2, 10, 40), "minecraft:redstone_wire"),
            ]),
            &profile(),
            &config,
            &hints,
        );

        assert_eq!(
            segs.clusters.len(),
            2,
            "the boundary splits the straddling cell"
        );

        let shared: Vec<&MarginCell> = segs
            .margin
            .iter()
            .filter(|m| m.cell == (0, 18, 10))
            .collect();
        assert_eq!(
            shared.len(),
            2,
            "the shared cell must appear once per partition, got {:?}",
            segs.margin
        );
        assert_ne!(
            shared[0].cluster, shared[1].cluster,
            "two distinct clusters occupy the shared cell"
        );
        assert_ne!(
            shared[0].partition, shared[1].partition,
            "margin entries at a shared cell must be distinguishable by partition; \
             without this a stitcher keying on `cell` unions across the boundary"
        );

        // And the partition recorded on each entry must be the one its own
        // cluster carries — not merely *some* differing value.
        for m in &shared {
            let owner = segs
                .clusters
                .iter()
                .find(|c| c.id == m.cluster)
                .expect("margin entry must reference a surviving cluster");
            assert_eq!(&m.partition, &owner.partition_id);
        }
        let mut names: Vec<Option<String>> = shared.iter().map(|m| m.partition.clone()).collect();
        names.sort();
        assert_eq!(names, vec![Some("L".to_string()), Some("R".to_string())]);
    }

    #[test]
    fn policy_off_reproduces_the_unpartitioned_result() {
        let hints = PartitionIndex::new(vec![
            PartitionHint {
                id: "left".into(),
                bbox_xz: (0, 13, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "right".into(),
                bbox_xz: (14, 127, 0, 127),
                y_range: None,
            },
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::Off,
            ..cfg()
        };
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
            PartitionHint {
                id: "left".into(),
                bbox_xz: (0, 13, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "right".into(),
                bbox_xz: (14, 127, 0, 127),
                y_range: None,
            },
        ]);
        let blocks = vec![
            ((10, 10, 10), "minecraft:redstone_wire"),
            ((18, 10, 10), "minecraft:redstone_wire"),
        ];

        let prefer = SegConfig {
            partition_policy: PartitionPolicy::Prefer,
            ..cfg()
        };
        let off = SegConfig {
            partition_policy: PartitionPolicy::Off,
            ..cfg()
        };
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
        assert_eq!(
            prefer_segs.clusters.len(),
            1,
            "the boundary is not enforced under Prefer"
        );
        assert!(
            prefer_segs
                .clusters
                .iter()
                .all(|c| c.partition_id.is_none()),
            "nothing is recorded: every partition_id is None"
        );

        // Contrast: the same input under HardCut *is* split, which shows the
        // hints and the boundary are real and Prefer is simply ignoring them.
        let hard = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
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
        let config = SegConfig {
            min_cluster_blocks: 2,
            ..cfg()
        };
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

        assert_eq!(
            segs.clusters.len(),
            1,
            "the near-face single-block cluster is dropped"
        );
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
        assert_eq!(
            segs.clusters.len(),
            1,
            "one block must give exactly one cluster"
        );
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
        assert_eq!(
            segs.clusters.len(),
            2,
            "cell distance 2R+2 leaves a one-cell gap"
        );
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
        assert!(
            margin_cells((107, 10, 107)).is_empty(),
            "cell 26 is interior"
        );
    }

    /// `margin_cells`, but on a caller-supplied tile so the grid need not be a
    /// cube.
    fn margin_cells_of(b: TileBounds, block: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
        let t = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            b,
            std::iter::once((block, BlockState::new("minecraft:redstone_wire"))),
        );
        let segs = segment_tile(&t, &profile(), &cfg(), &no_hints());
        assert_eq!(
            segs.clusters.len(),
            1,
            "one block must give exactly one cluster"
        );
        segs.margin.iter().map(|m| m.cell).collect()
    }

    #[test]
    fn margin_band_measures_each_axis_against_its_own_extent() {
        // `margin_band_covers_cell_depths_0_through_2r` probes at *equal* depth
        // on X and Z, on a tile whose dims are 32 on both axes. `in_margin` is
        // an OR of four terms, so on a square grid with symmetric probes every
        // permutation of X and Z gives the same answer: mirroring the whole
        // predicate, or testing one axis against the other's extent, is
        // completely invisible there.
        //
        // Fix both halves at once: a rectangular tile, and probes at different
        // depths on X and Z.
        //   bounds (0,-64,0)..(127,63,255) at cell_size 4 -> dims (32, 32, 64)
        //   band = 2R + 1 = 5
        //   X band: cells 0..=4 and 27..=31   (32 - 5 = 27)
        //   Z band: cells 0..=4 and 59..=63   (64 - 5 = 59)
        let b = TileBounds {
            min: (0, -64, 0),
            max: (127, 63, 255),
        };

        // Near X only, interior Z. (13,10,64) -> cell (13/4, (10+64)/4, 64/4)
        // = (3,18,16): depth 3 on X, depth 16 on Z.
        assert_eq!(margin_cells_of(b, (13, 10, 64)), vec![(3, 18, 16)]);

        // Near Z only, interior X. (64,10,13) -> (16,18,3).
        assert_eq!(margin_cells_of(b, (64, 10, 13)), vec![(16, 18, 3)]);

        // Far X only. (108,10,64) -> (27,18,16). In band because 27 >= 32 - 5.
        // A predicate that mirrors X and Z, or that tests X against the Z
        // extent, reads 27 >= 64 - 5 = 59 -> false, and drops this cell.
        assert_eq!(
            margin_cells_of(b, (108, 10, 64)),
            vec![(27, 18, 16)],
            "the far-X band starts at dims.0 - band, not dims.2 - band"
        );

        // Far Z only. (64,10,236) -> (16,18,59). In band because 59 >= 64 - 5.
        assert_eq!(
            margin_cells_of(b, (64, 10, 236)),
            vec![(16, 18, 59)],
            "the far-Z band starts at dims.2 - band"
        );

        // The negative that pins the same thing from the other side.
        // (64,10,108) -> (16,18,27). Cell z = 27 is deep interior on an axis
        // 64 cells long, but is exactly the far-band threshold for the 32-cell
        // X axis, so testing Z against dims.0 wrongly reports it in band.
        assert!(
            margin_cells_of(b, (64, 10, 108)).is_empty(),
            "cell z = 27 is interior on a 64-cell Z axis"
        );

        // And the last interior layer before the far-Z band, for the edge.
        // (64,10,232) -> (16,18,58).
        assert!(
            margin_cells_of(b, (64, 10, 232)).is_empty(),
            "cell z = 58 is interior"
        );
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

        assert_eq!(
            a, b,
            "a duplicated position must not let input order reach the output"
        );
        assert_eq!(
            a.clusters.len(),
            1,
            "redstone_wire wins the position, so one cluster"
        );
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
        assert_eq!(
            segs.clusters.len(),
            1,
            "the probe input must be a single cluster"
        );
        segs.clusters.iter().map(|c| c.id).collect()
    }

    #[test]
    fn identical_config_and_profile_give_identical_cluster_ids() {
        assert_eq!(ids(&cfg(), &profile()), ids(&cfg(), &profile()));
    }

    // `cluster_ids_change_when_cell_size_changes` used to live here. It was
    // deleted rather than kept: changing `cell_size` moves the anchor cell, so
    // the ids differ whether or not `cell_size` is hashed, and the test passed
    // with the property it claimed to check removed. The two tests below cover
    // the property properly, by changing something that cannot move a cell.

    #[test]
    fn cluster_ids_change_when_only_a_non_geometric_config_field_changes() {
        // The sharp case. `algorithm_version` cannot move a single cell, so
        // both runs produce the identical anchor cell (10,18,10) in the
        // identical tile. Hashing only (tile, anchor) therefore returned the
        // same ClusterId for output produced by a different algorithm — a
        // cache-poisoning hazard, since a consumer keyed on the id would serve
        // stale results after an algorithm change.
        let v2 = SegConfig {
            algorithm_version: 2,
            ..cfg()
        };
        assert_ne!(
            ids(&cfg(), &profile()),
            ids(&v2, &profile()),
            "a config change with no geometric effect must still change ids"
        );
    }

    #[test]
    fn cluster_ids_change_when_only_the_hint_geometry_changes() {
        // The cache-poisoning case `config_hash` exists to prevent, reached
        // through the hints rather than through `SegConfig`.
        //
        // Both runs use the identical tile, the identical `SegConfig` (HardCut)
        // and hints with the identical *ids*. Only the boxes differ: "L" ends at
        // x = 61 in one and at x = 40 in the other. The probe block sits at
        // x = 10, inside "L" either way, so both runs produce a cluster in the
        // same tile, in a partition of the same name, with the same anchor cell
        // (10/4, (10+64)/4, 40/4) = (2,18,10).
        //
        // Every input to `ClusterId` therefore matched, while the segmentation
        // these two configurations describe is genuinely different — a
        // downstream cache keyed on the id would serve one run's clusters for
        // the other's. The hint geometry must be part of the identity.
        let wide = PartitionIndex::new(vec![
            PartitionHint {
                id: "L".into(),
                bbox_xz: (0, 61, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "R".into(),
                bbox_xz: (62, 127, 0, 127),
                y_range: None,
            },
        ]);
        let narrow = PartitionIndex::new(vec![
            PartitionHint {
                id: "L".into(),
                bbox_xz: (0, 40, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "R".into(),
                bbox_xz: (41, 127, 0, 127),
                y_range: None,
            },
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
        let blocks = vec![((10, 10, 40), "minecraft:redstone_wire")];

        let a = segment_tile(&tile(blocks.clone()), &profile(), &config, &wide);
        let b = segment_tile(&tile(blocks), &profile(), &config, &narrow);

        // The precondition that makes this sharp: same partition name, same
        // anchor, same tile, same SegConfig.
        assert_eq!(a.clusters.len(), 1);
        assert_eq!(b.clusters.len(), 1);
        assert_eq!(a.clusters[0].partition_id.as_deref(), Some("L"));
        assert_eq!(b.clusters[0].partition_id.as_deref(), Some("L"));
        assert_eq!(a.clusters[0].bbox, b.clusters[0].bbox);

        assert_ne!(
            a.clusters[0].id, b.clusters[0].id,
            "hints sharing ids but differing in extent describe different \
             segmentations and must not mint the same ClusterId"
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
        assert!(
            segs.margin.is_empty(),
            "a centre cluster is nowhere near a face"
        );
    }

    #[test]
    fn membership_maps_every_emitted_block_to_its_cluster() {
        // Reuse the substrate-standing test's world: two builds on a slab.
        let profile = WorldProfile::new(
            ["minecraft:stone"].iter().map(|s| s.to_string()).collect(),
            (-64, -50),
        );
        let cfg = SegConfig {
            cell_size: 4,
            closing_radius: 2,
            min_cluster_blocks: 1,
            ..SegConfig::default()
        };
        let mut blocks = vec![];
        for x in 0..40 {
            for z in 0..40 {
                blocks.push(((x, -60, z), BlockState::new("minecraft:stone")));
            }
        }
        blocks.push(((10, -59, 10), BlockState::new("minecraft:redstone_wire")));
        let t = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            TileBounds {
                min: (0, -64, 0),
                max: (127, 63, 127),
            },
            blocks.into_iter(),
        );
        let (segs, membership) =
            segment_tile_membership(&t, &profile, &cfg, &PartitionIndex::new(vec![]));
        // The one build's block is mapped; substrate is not.
        assert_eq!(segs.clusters.len(), 1);
        let cluster = segs.clusters[0].id;
        assert_eq!(
            membership.get(&(10, -59, 10)),
            Some(&cluster),
            "build block maps to its cluster"
        );
        assert_eq!(
            membership.get(&(0, -60, 0)),
            None,
            "substrate is not in the membership map"
        );
        // Every mapped position belongs to an emitted cluster.
        for (_pos, cid) in &membership {
            assert!(segs.clusters.iter().any(|c| c.id == *cid));
        }
    }

    #[test]
    fn segment_tile_and_membership_agree_on_segments() {
        let profile = WorldProfile::new(
            ["minecraft:stone"].iter().map(|s| s.to_string()).collect(),
            (-64, -50),
        );
        let cfg = SegConfig::default();
        let t = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            TileBounds {
                min: (0, -64, 0),
                max: (127, 63, 127),
            },
            vec![((10, 10, 10), BlockState::new("minecraft:redstone_wire"))].into_iter(),
        );
        let a = segment_tile(&t, &profile, &cfg, &PartitionIndex::new(vec![]));
        let (b, _) = segment_tile_membership(&t, &profile, &cfg, &PartitionIndex::new(vec![]));
        assert_eq!(
            a, b,
            "membership variant must return identical TileSegments"
        );
    }

    /// Profile whose only *global* substrate is stone; the plot floors below
    /// (wool, concrete) are globally rare, so a global palette cannot catch
    /// them — only the per-partition floor pass can.
    fn floor_profile() -> WorldProfile {
        WorldProfile::new(
            ["minecraft:stone"].iter().map(|s| s.to_string()).collect(),
            (-64, -57),
        )
    }

    /// Two plots side by side, each a solid floor sheet with two redstone
    /// builds standing on it 24 blocks apart. Returns `(hints, blocks)`.
    ///
    /// # Hand-verified arithmetic (cell_size 4, closing_radius 2, origin.y -64)
    ///
    /// Merge threshold is `2R+1 = 5` cells along an axis.
    /// * L floor: blue_wool at y=-60, x 4..=35, z 4..=15 (32*12 = 384 blocks).
    ///   Cells x 1..=8, z 1..=3, y-cell (-60+64)/4 = 1 — a solid rectangle.
    /// * L builds: redstone_wire at (8,-59,10) and (32,-59,10). Cells
    ///   (2,1,2) and (8,1,2): both lie *inside* the sheet's cell footprint, so
    ///   with the floor present all of L is one component (1 cluster). Their
    ///   Chebyshev cell distance is 8-2 = 6 = 2R+2, so with the floor removed
    ///   they do NOT merge (2 clusters).
    /// * R is the mirror: white_concrete at y=-60, x 68..=99, z 4..=15; builds
    ///   at (72,-59,10) cell (18,1,2) and (96,-59,10) cell (24,1,2), distance 6.
    /// * Band (-64,-57) contains both y=-60 and y=-59. In L the band holds
    ///   384 wool + 2 redstone = 386 blocks: wool 384/386 = 99.5% >= 0.3 is
    ///   floor, redstone 2/386 = 0.5% < 0.3 is kept. R is identical with
    ///   concrete. Boundary x=63|64 is between the plots, cell-aligned and never
    ///   crossed by any block (L ends at x=35, R starts at x=68).
    fn two_plots() -> (PartitionIndex, Vec<((i32, i32, i32), &'static str)>) {
        let hints = PartitionIndex::new(vec![
            PartitionHint {
                id: "L".into(),
                bbox_xz: (0, 63, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "R".into(),
                bbox_xz: (64, 127, 0, 127),
                y_range: None,
            },
        ]);
        let mut blocks: Vec<((i32, i32, i32), &'static str)> = Vec::new();
        for x in 4..=35 {
            for z in 4..=15 {
                blocks.push(((x, -60, z), "minecraft:blue_wool"));
            }
        }
        for x in 68..=99 {
            for z in 4..=15 {
                blocks.push(((x, -60, z), "minecraft:white_concrete"));
            }
        }
        blocks.push(((8, -59, 10), "minecraft:redstone_wire"));
        blocks.push(((32, -59, 10), "minecraft:redstone_wire"));
        blocks.push(((72, -59, 10), "minecraft:redstone_wire"));
        blocks.push(((96, -59, 10), "minecraft:redstone_wire"));
        (hints, blocks)
    }

    #[test]
    fn partition_floor_is_subtracted_per_partition() {
        let (hints, blocks) = two_plots();

        // With the floor pass ON: each plot's dominant floor material is
        // subtracted, leaving only the two builds per plot — 24 blocks apart,
        // so they do not merge. Four small builds, no plot-sheet mega-cluster.
        let on = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            partition_floor_share: Some(0.3),
            ..cfg()
        };
        let segs = segment_tile(&tile(blocks.clone()), &floor_profile(), &on, &hints);
        assert_eq!(
            segs.clusters.len(),
            4,
            "floor subtracted: two separate builds per plot, four total; got {:?}",
            segs.clusters
                .iter()
                .map(|c| (c.bbox, c.block_count))
                .collect::<Vec<_>>()
        );
        for c in &segs.clusters {
            let x_extent = c.bbox.1 .0 - c.bbox.0 .0;
            assert!(
                x_extent < 15,
                "each build is small; a plot-sheet bbox would span the whole plot. bbox {:?}",
                c.bbox
            );
            assert_eq!(
                c.block_count, 1,
                "only the one-block build survives, not the 384-block floor sheet"
            );
        }

        // With the floor pass OFF (None): the floor sheet survives as an
        // artificial slab and morphological closing fuses both builds and the
        // whole sheet into one mega-cluster per plot — the exact real-data
        // failure. This is the discriminating case: None => merged blobs.
        let off = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
        assert!(off.partition_floor_share.is_none());
        let merged = segment_tile(&tile(blocks), &floor_profile(), &off, &hints);
        assert_eq!(
            merged.clusters.len(),
            2,
            "no floor subtraction: one whole-plot mega-cluster per partition"
        );
        for c in &merged.clusters {
            assert!(
                c.block_count > 300,
                "the merged cluster swallows the ~384-block floor sheet; got {}",
                c.block_count
            );
        }
    }

    #[test]
    fn dense_layer_subtracts_a_patterned_multi_material_floor() {
        let hints = PartitionIndex::new(vec![PartitionHint {
            id: "plot".into(),
            bbox_xz: (0, 31, 0, 31),
            y_range: None,
        }]);
        let mut blocks = Vec::new();
        for x in 0..32 {
            for z in 0..32 {
                let material = if (x + z) % 2 == 0 {
                    "minecraft:petrified_oak_slab"
                } else {
                    "minecraft:sandstone_slab"
                };
                blocks.push(((x, -60, z), material));
            }
        }
        blocks.push(((4, -59, 4), "minecraft:redstone_wire"));
        blocks.push(((28, -59, 4), "minecraft:redstone_wire"));

        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            partition_floor_share: None,
            partition_dense_layer_coverage: Some(0.80),
            ..cfg()
        };
        let result = segment_tile(&tile(blocks), &floor_profile(), &config, &hints);
        assert_eq!(result.clusters.len(), 2);
        assert!(result
            .clusters
            .iter()
            .all(|cluster| cluster.block_count == 1));
    }

    #[test]
    fn floor_share_none_is_behavior_preserving() {
        // The default disables the feature entirely.
        assert!(SegConfig::default().partition_floor_share.is_none());

        // An explicit `None` must be byte-for-byte identical to the pre-feature
        // path across every existing scenario shape: a plain build, a build on
        // global substrate, and a HardCut split. `cfg()` already carries
        // `partition_floor_share: None` via `..SegConfig::default()`, so an
        // explicit `None` cannot diverge from it — but pin it against a config
        // that never mentions the field, exercised on real inputs.
        let explicit_none = SegConfig {
            partition_floor_share: None,
            ..cfg()
        };

        // (a) plain build.
        let plain = vec![
            ((10, 10, 10), "minecraft:redstone_wire"),
            ((18, 10, 10), "minecraft:redstone_wire"),
        ];
        assert_eq!(
            segment_tile(&tile(plain.clone()), &profile(), &cfg(), &no_hints()),
            segment_tile(&tile(plain), &profile(), &explicit_none, &no_hints()),
        );

        // (b) build standing on global substrate.
        let mut slab = vec![];
        for x in 0..40 {
            for z in 0..40 {
                slab.push(((x, -60, z), "minecraft:stone"));
            }
        }
        slab.push(((10, -59, 10), "minecraft:redstone_wire"));
        slab.push(((30, -59, 30), "minecraft:redstone_wire"));
        assert_eq!(
            segment_tile(&tile(slab.clone()), &profile(), &cfg(), &no_hints()),
            segment_tile(&tile(slab), &profile(), &explicit_none, &no_hints()),
        );

        // (c) HardCut with hints, floor feature off: partitions still enforced,
        // floor pass inert.
        let hints = PartitionIndex::new(vec![
            PartitionHint {
                id: "left".into(),
                bbox_xz: (0, 13, 0, 127),
                y_range: None,
            },
            PartitionHint {
                id: "right".into(),
                bbox_xz: (14, 127, 0, 127),
                y_range: None,
            },
        ]);
        let hard = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
        let hard_none = SegConfig {
            partition_floor_share: None,
            ..hard.clone()
        };
        let split = vec![
            ((10, 10, 10), "minecraft:redstone_wire"),
            ((18, 10, 10), "minecraft:redstone_wire"),
        ];
        assert_eq!(
            segment_tile(&tile(split.clone()), &profile(), &hard, &hints),
            segment_tile(&tile(split), &profile(), &hard_none, &hints),
        );
    }

    #[test]
    fn config_hash_changes_with_floor_share() {
        // Two configs differing only in `partition_floor_share` must digest
        // differently, so a downstream cache keyed on `ClusterId` cannot serve
        // one run's clusters for the other's.
        let parts = PartitionIndex::new(vec![PartitionHint {
            id: "L".into(),
            bbox_xz: (0, 63, 0, 127),
            y_range: None,
        }]);
        let base = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            ..cfg()
        };
        let with_floor = SegConfig {
            partition_floor_share: Some(0.3),
            ..base.clone()
        };
        assert_ne!(
            base.config_hash(&floor_profile(), &parts),
            with_floor.config_hash(&floor_profile(), &parts),
            "partition_floor_share must be folded into config_hash"
        );

        // And the difference reaches the minted ClusterIds. The single build is
        // outside the band (y=10), so the floor pass cannot change its geometry
        // — only the hash differs, isolating the id change to config_hash.
        let probe = vec![((10, 10, 40), "minecraft:redstone_wire")];
        let a = segment_tile(&tile(probe.clone()), &floor_profile(), &base, &parts);
        let b = segment_tile(&tile(probe), &floor_profile(), &with_floor, &parts);
        assert_eq!(a.clusters.len(), 1);
        assert_eq!(b.clusters.len(), 1);
        assert_eq!(a.clusters[0].bbox, b.clusters[0].bbox, "geometry identical");
        assert_ne!(
            a.clusters[0].id, b.clusters[0].id,
            "different partition_floor_share must mint different ids"
        );
    }

    // -----------------------------------------------------------------------
    // split_disconnected: undo the closing where it fused two separate builds
    // -----------------------------------------------------------------------

    /// A solid cuboid of one block name, `[x0,x1] x [y0,y1] x [z0,z1]`.
    fn cuboid(
        x0: i32,
        x1: i32,
        y0: i32,
        y1: i32,
        z0: i32,
        z1: i32,
        name: &str,
    ) -> Vec<((i32, i32, i32), &str)> {
        let mut v = Vec::new();
        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    v.push(((x, y, z), name));
                }
            }
        }
        v
    }

    /// Small-threshold policy so a tiny test grid can exercise the split.
    fn split_policy() -> DisconnectedSplit {
        DisconnectedSplit {
            min_component_blocks: 8,
            min_component_share: 0.40,
            min_gap_cells: 2,
        }
    }

    /// Two 4x4x4 builds (64 blocks each), 12 blocks / 3 cells apart on z. That
    /// is inside the closing's `2R = 4`-cell reach at `closing_radius = 2`, so
    /// the default (split off) fuses them into one cluster — the exact bug.
    fn two_disconnected_builds() -> Vec<((i32, i32, i32), &'static str)> {
        let mut b = cuboid(8, 11, 0, 3, 8, 11, "minecraft:oak_planks");
        b.extend(cuboid(8, 11, 0, 3, 20, 23, "minecraft:oak_planks"));
        b
    }

    #[test]
    fn closing_merges_two_disconnected_builds_by_default() {
        // Documents the bug: with `split_disconnected: None` the morphological
        // closing bridges the 12-block gap and reports ONE build.
        let t = tile(two_disconnected_builds());
        let segs = segment_tile(&t, &profile(), &cfg(), &no_hints());
        assert_eq!(
            segs.clusters.len(),
            1,
            "closing fuses two builds within 2R+1 cells into one cluster"
        );
        assert_eq!(
            segs.clusters[0].block_count, 128,
            "both builds land in the one cluster"
        );
    }

    #[test]
    fn split_disconnected_separates_two_substantial_builds() {
        // The fix: the same input, with the split enabled, comes back as two
        // clusters — one per build — and no block is lost.
        let t = tile(two_disconnected_builds());
        let on = SegConfig {
            split_disconnected: Some(split_policy()),
            ..cfg()
        };
        let segs = segment_tile(&t, &profile(), &on, &no_hints());
        assert_eq!(
            segs.clusters.len(),
            2,
            "two disconnected substantial builds must split"
        );
        let counts: Vec<u64> = segs.clusters.iter().map(|c| c.block_count).collect();
        assert_eq!(counts, vec![64, 64], "each build keeps its own 64 blocks");
        assert_ne!(
            segs.clusters[0].id, segs.clusters[1].id,
            "distinct clusters get distinct ids"
        );
    }

    #[test]
    fn split_does_not_fragment_a_single_build_with_a_minor_detached_part() {
        // A legitimate single build: one 4x4x4 mass (64 blocks) plus a small
        // detached fixture (a 2x2x2, 8 blocks) 12 blocks away — the kind of
        // near-part the closing is meant to unify. The fixture is below the
        // 40% share (8 / 72 = 11%), so it is not a second seed: the cluster
        // stays whole, the fragment attached to the mass.
        let mut blocks = cuboid(8, 11, 0, 3, 8, 11, "minecraft:oak_planks");
        blocks.extend(cuboid(8, 9, 0, 1, 20, 21, "minecraft:oak_planks"));
        let t = tile(blocks);
        let on = SegConfig {
            split_disconnected: Some(split_policy()),
            ..cfg()
        };
        let segs = segment_tile(&t, &profile(), &on, &no_hints());
        assert_eq!(
            segs.clusters.len(),
            1,
            "a minor detached part must not split the build"
        );
        assert_eq!(
            segs.clusters[0].block_count, 72,
            "every block stays in the one cluster"
        );
    }

    #[test]
    fn split_respects_the_gap_tolerance() {
        // Two 64-block seeds that are only ONE cell apart (a diagonal touch,
        // gap < min_gap_cells) read as internal to one build, not two. Placed
        // at cells (2,16,2) and (3,16,3): Chebyshev cell distance 1.
        let mut blocks = cuboid(8, 11, 0, 3, 8, 11, "minecraft:oak_planks");
        blocks.extend(cuboid(12, 15, 0, 3, 12, 15, "minecraft:oak_planks"));
        let t = tile(blocks);
        let on = SegConfig {
            split_disconnected: Some(split_policy()),
            ..cfg()
        };
        let segs = segment_tile(&t, &profile(), &on, &no_hints());
        assert_eq!(
            segs.clusters.len(),
            1,
            "seeds closer than min_gap_cells stay merged"
        );
    }

    #[test]
    fn split_disconnected_is_folded_into_config_hash() {
        // A backward-compatible extension, exactly like partition_floor_share:
        // `None` digests identically to the pre-feature layout, `Some` differs.
        let base = cfg();
        let with_split = SegConfig {
            split_disconnected: Some(split_policy()),
            ..cfg()
        };
        assert_eq!(
            base.config_hash(&profile(), &no_hints()),
            SegConfig {
                split_disconnected: None,
                ..cfg()
            }
            .config_hash(&profile(), &no_hints()),
            "None must be byte-for-byte inert in config_hash"
        );
        assert_ne!(
            base.config_hash(&profile(), &no_hints()),
            with_split.config_hash(&profile(), &no_hints()),
            "split_disconnected must be folded into config_hash"
        );
    }

    #[test]
    fn drop_unpartitioned_keeps_only_declared_hard_cut_interiors() {
        let hints = PartitionIndex::new(vec![PartitionHint {
            id: "interior".into(),
            bbox_xz: (0, 31, 0, 31),
            y_range: None,
        }]);
        let input = tile(vec![
            ((8, 10, 8), "minecraft:redstone_wire"),
            ((80, 10, 80), "minecraft:redstone_wire"),
        ]);
        let config = SegConfig {
            partition_policy: PartitionPolicy::HardCut,
            drop_unpartitioned: true,
            ..cfg()
        };
        let result = segment_tile(&input, &profile(), &config, &hints);
        assert_eq!(result.clusters.len(), 1);
        assert_eq!(result.clusters[0].block_count, 1);
        assert_eq!(result.clusters[0].partition_id.as_deref(), Some("interior"));
    }

    #[test]
    fn drop_unpartitioned_is_folded_into_config_hash() {
        let base = cfg();
        let enabled = SegConfig {
            drop_unpartitioned: true,
            ..cfg()
        };
        assert_ne!(
            base.config_hash(&profile(), &no_hints()),
            enabled.config_hash(&profile(), &no_hints())
        );
    }

    #[test]
    fn dense_layer_coverage_is_folded_into_config_hash() {
        let base = cfg();
        let enabled = SegConfig {
            partition_dense_layer_coverage: Some(0.80),
            ..cfg()
        };
        assert_ne!(
            base.config_hash(&profile(), &no_hints()),
            enabled.config_hash(&profile(), &no_hints())
        );
    }

    #[test]
    fn preserve_support_blocks_is_folded_into_config_hash() {
        let base = cfg();
        let enabled = SegConfig {
            preserve_support_blocks: true,
            ..cfg()
        };
        assert_ne!(
            base.config_hash(&profile(), &no_hints()),
            enabled.config_hash(&profile(), &no_hints())
        );
    }
}
