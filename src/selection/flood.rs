//! BFS flood selection over a [`Mask`].
//!
//! `flood` is the single-seed entry point — analogue of RedstoneTools'
//! `/that`. `connected_components` is the multi-seed version: it iterates
//! candidate seeds, skipping any already-flooded ones, so each connected
//! component is yielded exactly once. The shared visited set means
//! "extraction" (collecting a component's blocks) happens in the same pass
//! that removes those blocks from the seed pool.
//!
//! ## Determinism
//!
//! Component order follows the order of seeds in the `candidates` iterator.
//! Block order inside a component follows BFS expansion order from the seed
//! (after offset-table order). Both are stable for a given input.
//!
//! ## Memory
//!
//! Each [`Component`] owns a `Vec<BlockPosition>`. For a 1M-block component
//! that's ~12 MiB; the streaming form (`connected_components`) lets you
//! consume + drop components as they're produced.

use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::selection::connectivity::Connectivity;
use crate::selection::mask::Mask;
use crate::selection::visited::VisitedSet;
use std::collections::VecDeque;

/// Upper bounds on a single flood. `None` means "no limit".
#[derive(Debug, Clone, Copy, Default)]
pub struct Limits {
    /// Stop once a component reaches this many blocks. Useful as a safety
    /// valve against runaway selections through the natural terrain.
    pub max_blocks: Option<usize>,
    /// Stop once the component's bounding box exceeds this span on any axis.
    /// Mirrors `/that`'s `sizeLimit`.
    pub max_extent: Option<i32>,
}

impl Limits {
    pub fn unbounded() -> Self {
        Self::default()
    }

    pub fn with_max_blocks(mut self, n: usize) -> Self {
        self.max_blocks = Some(n);
        self
    }

    pub fn with_max_extent(mut self, n: i32) -> Self {
        self.max_extent = Some(n);
        self
    }
}

/// Why a flood stopped expanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// All reachable blocks were visited.
    Exhausted,
    /// `Limits::max_blocks` was hit before exhaustion.
    MaxBlocks,
    /// `Limits::max_extent` was hit before exhaustion.
    MaxExtent,
}

/// One connected component produced by a flood.
#[derive(Debug, Clone)]
pub struct Component {
    /// The seed that started this component.
    pub seed: BlockPosition,
    /// Tight axis-aligned bounding box of every block in [`Component::blocks`].
    pub bounds: BoundingBox,
    /// Every block reached by the flood, in BFS order from the seed.
    pub blocks: Vec<BlockPosition>,
    /// Why the flood stopped — see [`StopReason`].
    pub stop_reason: StopReason,
}

impl Component {
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

// ── Single-seed flood ──────────────────────────────────────────────────────

/// Flood-fill from a single seed. The analogue of `/that`.
///
/// If `mask.test(seed)` is false the returned [`Component`] contains zero
/// blocks (and a zero-volume bounds at the seed).
pub fn flood<M: Mask>(
    seed: BlockPosition,
    mask: &M,
    connectivity: Connectivity,
    limits: &Limits,
) -> Component {
    let mut visited = VisitedSet::new();
    flood_with_visited(seed, mask, connectivity, limits, &mut visited)
}

/// Like [`flood`], but reuses an external [`VisitedSet`]. Blocks already in
/// `visited` are treated as out-of-mask — they are never revisited and never
/// added to the returned component.
///
/// This is the building block for [`connected_components`]: by sharing one
/// `VisitedSet` across all seeds, every block in the world is touched at
/// most once across the whole pass.
pub fn flood_with_visited<M: Mask>(
    seed: BlockPosition,
    mask: &M,
    connectivity: Connectivity,
    limits: &Limits,
    visited: &mut VisitedSet,
) -> Component {
    let offsets = connectivity.offsets();
    let mut blocks: Vec<BlockPosition> = Vec::new();
    let mut queue: VecDeque<BlockPosition> = VecDeque::new();

    // The seed must pass both the visited check (idempotent re-runs) and the
    // mask (otherwise we'd return a phantom 1-block component centred on
    // empty space). Both failing yields an empty component.
    if !visited.contains(seed.x, seed.y, seed.z) && mask.test(seed.x, seed.y, seed.z) {
        visited.insert(seed.x, seed.y, seed.z);
        queue.push_back(seed);
    }

    let mut min = (seed.x, seed.y, seed.z);
    let mut max = (seed.x, seed.y, seed.z);
    let mut stop = StopReason::Exhausted;

    while let Some(pos) = queue.pop_front() {
        blocks.push(pos);

        // Expand min/max to include this block.
        min.0 = min.0.min(pos.x);
        min.1 = min.1.min(pos.y);
        min.2 = min.2.min(pos.z);
        max.0 = max.0.max(pos.x);
        max.1 = max.1.max(pos.y);
        max.2 = max.2.max(pos.z);

        // Check limits AFTER recording the block but BEFORE enqueuing
        // neighbours, so a `MaxBlocks` stop returns exactly `max_blocks`
        // blocks (not max+1) and a `MaxExtent` stop is reported on the block
        // that pushed us over the edge.
        if let Some(cap) = limits.max_blocks {
            if blocks.len() >= cap {
                stop = StopReason::MaxBlocks;
                break;
            }
        }
        if let Some(ext) = limits.max_extent {
            let dx = max.0 - min.0;
            let dy = max.1 - min.1;
            let dz = max.2 - min.2;
            if dx > ext || dy > ext || dz > ext {
                stop = StopReason::MaxExtent;
                break;
            }
        }

        for &(dx, dy, dz) in offsets {
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            let nz = pos.z + dz;
            if !visited.insert(nx, ny, nz) {
                continue; // already seen this round or in a prior flood
            }
            if mask.test(nx, ny, nz) {
                queue.push_back(BlockPosition::new(nx, ny, nz));
            }
        }
    }

    let bounds = if blocks.is_empty() {
        BoundingBox::new((seed.x, seed.y, seed.z), (seed.x, seed.y, seed.z))
    } else {
        BoundingBox::new(min, max)
    };

    Component {
        seed,
        bounds,
        blocks,
        stop_reason: stop,
    }
}

// ── Multi-seed flood: connected components ─────────────────────────────────

/// Whether the multi-seed driver should keep going after a callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continue {
    /// Continue scanning subsequent seeds.
    Yes,
    /// Stop the entire scan immediately.
    Stop,
}

/// Stream every connected component reachable from the given candidate seeds,
/// calling `on_component` for each. Components are yielded as soon as their
/// flood finishes, so the caller can drop / write / discard them between
/// calls. Returns the number of components produced.
///
/// Use this instead of [`connected_components_collect`] when components are
/// large enough or numerous enough that holding them all in memory is
/// undesirable (e.g. extracting builds from a world to disk).
pub fn connected_components<M, I, F>(
    candidates: I,
    mask: &M,
    connectivity: Connectivity,
    limits: &Limits,
    mut on_component: F,
) -> usize
where
    M: Mask,
    I: IntoIterator<Item = BlockPosition>,
    F: FnMut(Component) -> Continue,
{
    let mut visited = VisitedSet::new();
    let mut count = 0usize;

    for seed in candidates {
        if visited.contains(seed.x, seed.y, seed.z) {
            continue;
        }
        if !mask.test(seed.x, seed.y, seed.z) {
            // Still mark it visited so we don't re-evaluate the mask on every
            // future neighbour-check. This matches That.kt's behaviour where
            // failing mask blocks are added to `visited` to avoid re-tests.
            visited.insert(seed.x, seed.y, seed.z);
            continue;
        }

        let component = flood_with_visited(seed, mask, connectivity, limits, &mut visited);
        // `flood_with_visited` returns an empty component if the seed was
        // already visited or out-of-mask; we've ruled both out above, so the
        // component is non-empty.
        debug_assert!(!component.blocks.is_empty());

        count += 1;
        match on_component(component) {
            Continue::Yes => {}
            Continue::Stop => break,
        }
    }

    count
}

/// Convenience wrapper around [`connected_components`] that collects every
/// component into a `Vec`. Prefer the streaming form for large inputs.
pub fn connected_components_collect<M, I>(
    candidates: I,
    mask: &M,
    connectivity: Connectivity,
    limits: &Limits,
) -> Vec<Component>
where
    M: Mask,
    I: IntoIterator<Item = BlockPosition>,
{
    let mut out = Vec::new();
    connected_components(candidates, mask, connectivity, limits, |c| {
        out.push(c);
        Continue::Yes
    });
    out
}

/// Helper: yield every integer block position inside a [`BoundingBox`] in
/// y-major, z-mid, x-minor order. Useful as the candidate iterator for
/// `connected_components` when sweeping a known volume.
pub fn iter_bounds(bounds: &BoundingBox) -> impl Iterator<Item = BlockPosition> {
    let (mnx, mny, mnz) = bounds.min;
    let (mxx, mxy, mxz) = bounds.max;
    (mny..=mxy).flat_map(move |y| {
        (mnz..=mxz).flat_map(move |z| (mnx..=mxx).map(move |x| BlockPosition::new(x, y, z)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn bp(x: i32, y: i32, z: i32) -> BlockPosition {
        BlockPosition::new(x, y, z)
    }

    /// Mask backed by a HashSet of positions — easiest way to set up
    /// arbitrary shapes for unit tests.
    struct SetMask {
        on: HashSet<(i32, i32, i32)>,
    }
    impl SetMask {
        fn from_iter<I: IntoIterator<Item = (i32, i32, i32)>>(iter: I) -> Self {
            Self {
                on: iter.into_iter().collect(),
            }
        }
    }
    impl Mask for SetMask {
        fn test(&self, x: i32, y: i32, z: i32) -> bool {
            self.on.contains(&(x, y, z))
        }
    }

    // ── flood: single seed ────────────────────────────────────────────────

    #[test]
    fn flood_empty_when_mask_misses_seed() {
        let mask = SetMask::from_iter([]);
        let c = flood(bp(0, 0, 0), &mask, Connectivity::Face, &Limits::unbounded());
        assert!(c.blocks.is_empty());
        assert_eq!(c.stop_reason, StopReason::Exhausted);
    }

    #[test]
    fn flood_single_isolated_block() {
        let mask = SetMask::from_iter([(0, 0, 0)]);
        let c = flood(bp(0, 0, 0), &mask, Connectivity::Face, &Limits::unbounded());
        assert_eq!(c.blocks.len(), 1);
        assert_eq!(c.bounds.min, (0, 0, 0));
        assert_eq!(c.bounds.max, (0, 0, 0));
        assert_eq!(c.stop_reason, StopReason::Exhausted);
    }

    #[test]
    fn flood_3x3x3_solid_cube_face_connectivity() {
        let mut on = Vec::new();
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    on.push((x, y, z));
                }
            }
        }
        let mask = SetMask::from_iter(on);
        let c = flood(bp(1, 1, 1), &mask, Connectivity::Face, &Limits::unbounded());
        assert_eq!(c.blocks.len(), 27);
        assert_eq!(c.bounds.min, (0, 0, 0));
        assert_eq!(c.bounds.max, (2, 2, 2));
    }

    #[test]
    fn flood_diagonal_pair_face_vs_corner() {
        // Two blocks touching only at a corner: (0,0,0) and (1,1,1).
        let mask = SetMask::from_iter([(0, 0, 0), (1, 1, 1)]);

        let face = flood(bp(0, 0, 0), &mask, Connectivity::Face, &Limits::unbounded());
        assert_eq!(face.blocks.len(), 1, "Face: corner-touch is disconnected");

        let corner = flood(
            bp(0, 0, 0),
            &mask,
            Connectivity::Corner,
            &Limits::unbounded(),
        );
        assert_eq!(corner.blocks.len(), 2, "Corner: corner-touch is connected");
    }

    #[test]
    fn flood_edge_diagonal_with_edge_connectivity() {
        // Two blocks touching only at an edge (share one axis): (0,0,0) and (1,1,0).
        let mask = SetMask::from_iter([(0, 0, 0), (1, 1, 0)]);
        let face = flood(bp(0, 0, 0), &mask, Connectivity::Face, &Limits::unbounded());
        assert_eq!(face.blocks.len(), 1);
        let edge = flood(bp(0, 0, 0), &mask, Connectivity::Edge, &Limits::unbounded());
        assert_eq!(edge.blocks.len(), 2);
    }

    #[test]
    fn flood_hollow_shell_does_not_include_interior() {
        // 3x3x3 shell: all 27 minus the center (1,1,1).
        let mut on = Vec::new();
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    if !(x == 1 && y == 1 && z == 1) {
                        on.push((x, y, z));
                    }
                }
            }
        }
        let mask = SetMask::from_iter(on);
        let c = flood(bp(0, 0, 0), &mask, Connectivity::Face, &Limits::unbounded());
        assert_eq!(c.blocks.len(), 26);
        // The center (mask = false) must not appear in the component.
        assert!(!c.blocks.iter().any(|b| (b.x, b.y, b.z) == (1, 1, 1)));
    }

    // ── limits ────────────────────────────────────────────────────────────

    #[test]
    fn limit_max_blocks_stops_early() {
        // A long line of 100 blocks.
        let mask = SetMask::from_iter((0..100).map(|x| (x, 0, 0)));
        let limits = Limits::unbounded().with_max_blocks(10);
        let c = flood(bp(0, 0, 0), &mask, Connectivity::Face, &limits);
        assert_eq!(c.blocks.len(), 10);
        assert_eq!(c.stop_reason, StopReason::MaxBlocks);
    }

    #[test]
    fn limit_max_extent_stops_early() {
        let mask = SetMask::from_iter((0..100).map(|x| (x, 0, 0)));
        let limits = Limits::unbounded().with_max_extent(5);
        let c = flood(bp(0, 0, 0), &mask, Connectivity::Face, &limits);
        assert_eq!(c.stop_reason, StopReason::MaxExtent);
        let dx = c.bounds.max.0 - c.bounds.min.0;
        assert!(
            dx > 5,
            "extent that triggered the stop should exceed the cap (got dx={dx})"
        );
    }

    // ── connected_components ──────────────────────────────────────────────

    #[test]
    fn connected_components_finds_two_separated_cubes() {
        // Cube A at 0..2, cube B at 10..12. Face-connected.
        let mut on = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    on.push((x, y, z));
                    on.push((x + 10, y, z));
                }
            }
        }
        let mask = SetMask::from_iter(on);

        let candidates = iter_bounds(&BoundingBox::new((0, 0, 0), (11, 1, 1)));
        let comps = connected_components_collect(
            candidates,
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );
        assert_eq!(comps.len(), 2);
        let sizes: Vec<usize> = comps.iter().map(|c| c.blocks.len()).collect();
        assert_eq!(sizes, vec![8, 8]);
    }

    #[test]
    fn connected_components_visited_extracts_in_one_pass() {
        // Three components; verify total blocks == sum of component sizes
        // (i.e. nothing visited twice).
        let mut on = Vec::new();
        // Component 1: 5 blocks
        for x in 0..5 {
            on.push((x, 0, 0));
        }
        // Component 2: 3 blocks
        for y in 0..3 {
            on.push((100, y, 0));
        }
        // Component 3: 1 block
        on.push((0, 0, 100));

        let mask = SetMask::from_iter(on.clone());
        let candidates = on.iter().map(|&(x, y, z)| BlockPosition::new(x, y, z));
        let comps = connected_components_collect(
            candidates,
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );

        assert_eq!(comps.len(), 3);
        let total: usize = comps.iter().map(|c| c.blocks.len()).sum();
        assert_eq!(
            total,
            on.len(),
            "every mask-on block appears in exactly one component"
        );
    }

    #[test]
    fn connected_components_callback_can_stop_early() {
        // Six isolated blocks.
        let seeds: Vec<(i32, i32, i32)> = (0..6).map(|i| (i * 10, 0, 0)).collect();
        let mask = SetMask::from_iter(seeds.iter().copied());
        let candidates = seeds.iter().map(|&(x, y, z)| BlockPosition::new(x, y, z));

        let mut yielded = 0usize;
        let returned = connected_components(
            candidates,
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
            |_| {
                yielded += 1;
                if yielded == 2 {
                    Continue::Stop
                } else {
                    Continue::Yes
                }
            },
        );

        assert_eq!(yielded, 2);
        assert_eq!(returned, 2);
    }

    #[test]
    fn connectivity_changes_component_count() {
        // Diagonal chain: (0,0,0)-(1,1,0)-(2,2,0)
        let blocks = vec![(0, 0, 0), (1, 1, 0), (2, 2, 0)];
        let mask = SetMask::from_iter(blocks.clone());
        let candidates = || blocks.iter().map(|&(x, y, z)| BlockPosition::new(x, y, z));

        let face = connected_components_collect(
            candidates(),
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );
        assert_eq!(face.len(), 3, "Face: 3 isolated blocks");

        let edge = connected_components_collect(
            candidates(),
            &mask,
            Connectivity::Edge,
            &Limits::unbounded(),
        );
        assert_eq!(
            edge.len(),
            1,
            "Edge: all three connect via x/y edge diagonals"
        );
    }
}
