//! Grouping and stagger: what animates as a unit, and when each unit starts.
//!
//! This module is deliberately free of any dependency on the building or
//! schematic layers — it works on plain positions and caller-supplied sort keys.
//! That keeps it fast to test, and lets callers order groups by anything
//! (a shape's parametric `t`, build order, a diff) without this module knowing
//! about those systems.
//!
//! Note the two *different* easings in play. [`Stagger::ease`] shapes **when**
//! each group starts; the easing inside a [`super::track::Clip`] shapes **how**
//! it moves once started. Conflating them is the usual mistake.

use serde::{Deserialize, Serialize};

use super::easing::Easing;

/// Identifies one animatable unit.
pub type GroupId = u32;

/// A block position in schematic space.
pub type Pos = (i32, i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn of(self, p: Pos) -> i32 {
        match self {
            Axis::X => p.0,
            Axis::Y => p.1,
            Axis::Z => p.2,
        }
    }
}

/// One animatable unit: a set of blocks that share a pose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub blocks: Vec<Pos>,
    /// Centroid in world space — the default pivot, so "scale in place" works
    /// without the caller doing arithmetic.
    pub centroid: [f32; 3],
}

impl Group {
    pub fn new(id: GroupId, blocks: Vec<Pos>) -> Self {
        let centroid = centroid_of(&blocks);
        Group {
            id,
            blocks,
            centroid,
        }
    }
}

/// Block centres, so a single block's centroid is its middle rather than its corner.
fn centroid_of(blocks: &[Pos]) -> [f32; 3] {
    if blocks.is_empty() {
        return [0.0; 3];
    }
    let n = blocks.len() as f32;
    let (mut x, mut y, mut z) = (0.0f32, 0.0f32, 0.0f32);
    for b in blocks {
        x += b.0 as f32 + 0.5;
        y += b.1 as f32 + 0.5;
        z += b.2 as f32 + 0.5;
    }
    [x / n, y / n, z / n]
}

/// How a set of positions becomes animatable units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Grouping {
    /// One group per block. Highest resolution, highest draw-call cost.
    PerBlock,
    /// One group per slice along an axis — the layer-printer effect.
    Layer(Axis),
    /// Cubes of `size` blocks.
    Chunk(u32),
    /// Caller-defined units.
    Custom(Vec<Vec<Pos>>),
}

impl Grouping {
    /// Build groups from a position list. Ordering of the result is stable and
    /// independent of the input order, so animations are reproducible.
    pub fn apply(&self, positions: &[Pos]) -> Vec<Group> {
        match self {
            Grouping::PerBlock => {
                let mut ps = positions.to_vec();
                ps.sort_unstable();
                ps.dedup();
                ps.into_iter()
                    .enumerate()
                    .map(|(i, p)| Group::new(i as GroupId, vec![p]))
                    .collect()
            }
            Grouping::Layer(axis) => {
                let mut by_layer: std::collections::BTreeMap<i32, Vec<Pos>> = Default::default();
                for &p in positions {
                    by_layer.entry(axis.of(p)).or_default().push(p);
                }
                by_layer
                    .into_values()
                    .enumerate()
                    .map(|(i, mut blocks)| {
                        blocks.sort_unstable();
                        Group::new(i as GroupId, blocks)
                    })
                    .collect()
            }
            Grouping::Chunk(size) => {
                let s = (*size).max(1) as i32;
                let key = |v: i32| v.div_euclid(s);
                let mut by_chunk: std::collections::BTreeMap<(i32, i32, i32), Vec<Pos>> =
                    Default::default();
                for &p in positions {
                    by_chunk
                        .entry((key(p.0), key(p.1), key(p.2)))
                        .or_default()
                        .push(p);
                }
                by_chunk
                    .into_values()
                    .enumerate()
                    .map(|(i, mut blocks)| {
                        blocks.sort_unstable();
                        Group::new(i as GroupId, blocks)
                    })
                    .collect()
            }
            Grouping::Custom(sets) => sets
                .iter()
                .enumerate()
                .map(|(i, b)| Group::new(i as GroupId, b.clone()))
                .collect(),
        }
    }
}

/// How groups are ranked before delays are assigned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Order {
    /// Group order as built.
    Index,
    /// By centroid along an axis; `ascending = false` reverses.
    Axis(Axis, bool),
    /// By distance of the centroid from a world-space point.
    DistanceFrom([f32; 3]),
    /// By a caller-supplied sort key, one per group.
    ///
    /// This is how geometric and historical orderings arrive: a shape's
    /// parametric `t` from `ShapeEnum::parameter_at`, or a build sequence
    /// number. Keeping it a plain key list means this module needs no knowledge
    /// of either system.
    Key(Vec<f64>),
    /// Explicit rank per group.
    Custom(Vec<usize>),
    /// Seeded shuffle. There is deliberately no unseeded variant — README media
    /// must regenerate byte-identically.
    Random(u64),
}

impl Order {
    /// Rank each group, `0..n-1`. Ties resolve by group index so the result is
    /// always a total order.
    pub fn ranks(&self, groups: &[Group]) -> Vec<usize> {
        let n = groups.len();
        if n == 0 {
            return Vec::new();
        }
        let mut idx: Vec<usize> = (0..n).collect();
        match self {
            Order::Index => {}
            Order::Axis(axis, ascending) => {
                let comp = |g: &Group| match axis {
                    Axis::X => g.centroid[0],
                    Axis::Y => g.centroid[1],
                    Axis::Z => g.centroid[2],
                };
                idx.sort_by(|&a, &b| {
                    let (ka, kb) = (comp(&groups[a]), comp(&groups[b]));
                    ka.partial_cmp(&kb)
                        .unwrap_or(core::cmp::Ordering::Equal)
                        .then(a.cmp(&b))
                });
                if !ascending {
                    idx.reverse();
                }
            }
            Order::DistanceFrom(o) => {
                let d = |g: &Group| {
                    let dx = g.centroid[0] - o[0];
                    let dy = g.centroid[1] - o[1];
                    let dz = g.centroid[2] - o[2];
                    dx * dx + dy * dy + dz * dz
                };
                idx.sort_by(|&a, &b| {
                    d(&groups[a])
                        .partial_cmp(&d(&groups[b]))
                        .unwrap_or(core::cmp::Ordering::Equal)
                        .then(a.cmp(&b))
                });
            }
            Order::Key(keys) => {
                idx.sort_by(|&a, &b| {
                    let ka = keys.get(a).copied().unwrap_or(f64::MAX);
                    let kb = keys.get(b).copied().unwrap_or(f64::MAX);
                    ka.partial_cmp(&kb)
                        .unwrap_or(core::cmp::Ordering::Equal)
                        .then(a.cmp(&b))
                });
            }
            Order::Custom(ranks) => {
                idx.sort_by_key(|&i| (ranks.get(i).copied().unwrap_or(usize::MAX), i));
            }
            Order::Random(seed) => {
                // SplitMix64: tiny, seeded, and identical on every platform.
                let mut state = *seed;
                let mut keyed: Vec<(u64, usize)> = idx
                    .iter()
                    .map(|&i| {
                        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
                        let mut z = state;
                        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                        (z ^ (z >> 31), i)
                    })
                    .collect();
                keyed.sort_unstable();
                idx = keyed.into_iter().map(|(_, i)| i).collect();
            }
        }
        // idx is "position -> group"; invert to "group -> rank".
        let mut ranks = vec![0usize; n];
        for (rank, &g) in idx.iter().enumerate() {
            ranks[g] = rank;
        }
        ranks
    }
}

/// Where the stagger wave originates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaggerFrom {
    First,
    Last,
    Center,
    Index(usize),
}

/// How the per-group delays are spread.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Spread {
    /// Fixed delay between consecutive ranks.
    EachMs(f32),
    /// Distribute so the first and last starts are exactly this far apart.
    TotalMs(f32),
}

/// Ordering plus delay distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stagger {
    pub order: Order,
    pub from: StaggerFrom,
    pub spread: Spread,
    /// Shapes **when** each group starts (an accelerating or decelerating wave).
    /// Distinct from the motion easing inside a clip.
    pub ease: Easing,
}

impl Stagger {
    /// A simple wave from the first group, `each_ms` apart.
    pub fn each(order: Order, each_ms: f32) -> Self {
        Stagger {
            order,
            from: StaggerFrom::First,
            spread: Spread::EachMs(each_ms),
            ease: Easing::Linear,
        }
    }

    /// Spread all groups across `total_ms`, however many there are.
    pub fn total(order: Order, total_ms: f32) -> Self {
        Stagger {
            order,
            from: StaggerFrom::First,
            spread: Spread::TotalMs(total_ms),
            ease: Easing::Linear,
        }
    }

    pub fn from(mut self, f: StaggerFrom) -> Self {
        self.from = f;
        self
    }

    pub fn eased(mut self, e: Easing) -> Self {
        self.ease = e;
        self
    }

    /// Delay in milliseconds for each group, indexed by group position.
    pub fn delays(&self, groups: &[Group]) -> Vec<f32> {
        let n = groups.len();
        if n == 0 {
            return Vec::new();
        }
        let ranks = self.order.ranks(groups);
        let last = (n - 1) as f32;

        // Distance in rank-space from the anchor.
        let dist: Vec<f32> = ranks
            .iter()
            .map(|&r| {
                let r = r as f32;
                match self.from {
                    StaggerFrom::First => r,
                    StaggerFrom::Last => last - r,
                    StaggerFrom::Center => (r - last / 2.0).abs(),
                    StaggerFrom::Index(i) => (r - i as f32).abs(),
                }
            })
            .collect();

        let max = dist.iter().cloned().fold(0.0f32, f32::max);
        if max <= 0.0 {
            return vec![0.0; n];
        }
        let span = match self.spread {
            Spread::EachMs(each) => each * max,
            Spread::TotalMs(total) => total,
        };
        dist.iter()
            .map(|&d| self.ease.eval(d / max) * span)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: i32) -> Vec<Pos> {
        (0..n).map(|i| (i, 0, 0)).collect()
    }

    #[test]
    fn per_block_makes_one_group_each_and_dedupes() {
        let g = Grouping::PerBlock.apply(&[(0, 0, 0), (1, 0, 0), (0, 0, 0)]);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].blocks, vec![(0, 0, 0)]);
    }

    #[test]
    fn per_block_centroid_is_the_block_centre() {
        let g = Grouping::PerBlock.apply(&[(2, 3, 4)]);
        assert_eq!(g[0].centroid, [2.5, 3.5, 4.5]);
    }

    #[test]
    fn layer_groups_by_slice_in_ascending_order() {
        let pos = vec![(0, 1, 0), (5, 0, 5), (2, 1, 3), (1, 2, 1)];
        let g = Grouping::Layer(Axis::Y).apply(&pos);
        assert_eq!(g.len(), 3);
        assert_eq!(g[0].blocks, vec![(5, 0, 5)], "y=0 layer first");
        assert_eq!(g[1].blocks.len(), 2, "y=1 layer has two blocks");
    }

    #[test]
    fn chunk_groups_by_cube_and_handles_negatives() {
        let pos = vec![(0, 0, 0), (3, 0, 0), (-1, 0, 0)];
        let g = Grouping::Chunk(4).apply(&pos);
        // -1 floors into the -1 chunk, not the 0 chunk.
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].blocks, vec![(-1, 0, 0)]);
        assert_eq!(g[1].blocks, vec![(0, 0, 0), (3, 0, 0)]);
    }

    #[test]
    fn chunk_size_zero_does_not_divide_by_zero() {
        let g = Grouping::Chunk(0).apply(&line(3));
        assert_eq!(g.len(), 3);
    }

    #[test]
    fn grouping_is_independent_of_input_order() {
        let a = Grouping::PerBlock.apply(&[(2, 0, 0), (0, 0, 0), (1, 0, 0)]);
        let b = Grouping::PerBlock.apply(&[(0, 0, 0), (1, 0, 0), (2, 0, 0)]);
        assert_eq!(a, b);
    }

    #[test]
    fn axis_order_ranks_along_the_axis() {
        let groups = Grouping::PerBlock.apply(&line(4));
        let asc = Order::Axis(Axis::X, true).ranks(&groups);
        assert_eq!(asc, vec![0, 1, 2, 3]);
        let desc = Order::Axis(Axis::X, false).ranks(&groups);
        assert_eq!(desc, vec![3, 2, 1, 0]);
    }

    #[test]
    fn key_order_drives_ranking() {
        let groups = Grouping::PerBlock.apply(&line(3));
        // Reverse the natural order via keys — this is how a shape parameter
        // or build sequence gets applied.
        let ranks = Order::Key(vec![2.0, 1.0, 0.0]).ranks(&groups);
        assert_eq!(ranks, vec![2, 1, 0]);
    }

    #[test]
    fn distance_order_starts_nearest() {
        let groups = Grouping::PerBlock.apply(&line(3));
        let ranks = Order::DistanceFrom([2.5, 0.5, 0.5]).ranks(&groups);
        assert_eq!(ranks, vec![2, 1, 0]);
    }

    #[test]
    fn ranks_are_always_a_permutation() {
        let groups = Grouping::PerBlock.apply(&line(6));
        for order in [
            Order::Index,
            Order::Axis(Axis::X, true),
            Order::DistanceFrom([0.0, 0.0, 0.0]),
            Order::Key(vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0]), // all ties
            Order::Random(42),
        ] {
            let mut r = order.ranks(&groups);
            r.sort_unstable();
            assert_eq!(r, vec![0, 1, 2, 3, 4, 5], "{order:?} is not a permutation");
        }
    }

    #[test]
    fn random_order_is_seeded_and_reproducible() {
        let groups = Grouping::PerBlock.apply(&line(20));
        let a = Order::Random(7).ranks(&groups);
        let b = Order::Random(7).ranks(&groups);
        let c = Order::Random(8).ranks(&groups);
        assert_eq!(a, b, "same seed must reproduce");
        assert_ne!(a, c, "different seed should differ");
    }

    #[test]
    fn each_ms_spaces_delays_evenly() {
        let groups = Grouping::PerBlock.apply(&line(4));
        let d = Stagger::each(Order::Axis(Axis::X, true), 100.0).delays(&groups);
        assert_eq!(d, vec![0.0, 100.0, 200.0, 300.0]);
    }

    #[test]
    fn total_ms_spans_exactly_the_requested_window() {
        let groups = Grouping::PerBlock.apply(&line(5));
        let d = Stagger::total(Order::Axis(Axis::X, true), 1000.0).delays(&groups);
        assert_eq!(d.first().copied(), Some(0.0));
        assert_eq!(d.last().copied(), Some(1000.0));
    }

    #[test]
    fn from_center_is_symmetric() {
        let groups = Grouping::PerBlock.apply(&line(5));
        let d = Stagger::each(Order::Axis(Axis::X, true), 100.0)
            .from(StaggerFrom::Center)
            .delays(&groups);
        assert_eq!(d[0], d[4], "ends should start together");
        assert_eq!(d[1], d[3]);
        assert_eq!(d[2], 0.0, "centre starts first");
    }

    #[test]
    fn from_last_reverses_the_wave() {
        let groups = Grouping::PerBlock.apply(&line(3));
        let d = Stagger::each(Order::Axis(Axis::X, true), 50.0)
            .from(StaggerFrom::Last)
            .delays(&groups);
        assert_eq!(d, vec![100.0, 50.0, 0.0]);
    }

    #[test]
    fn distribution_easing_bends_the_wave_but_keeps_the_span() {
        let groups = Grouping::PerBlock.apply(&line(5));
        let d = Stagger::total(Order::Axis(Axis::X, true), 1000.0)
            .eased(Easing::In(super::super::easing::Power::Quad))
            .delays(&groups);
        assert_eq!(d[0], 0.0);
        assert!((d[4] - 1000.0).abs() < 1e-3, "span preserved");
        assert!(d[2] < 500.0, "quadratic ease-in should bunch the start");
    }

    #[test]
    fn single_group_and_empty_are_safe() {
        let one = Grouping::PerBlock.apply(&[(0, 0, 0)]);
        assert_eq!(Stagger::each(Order::Index, 100.0).delays(&one), vec![0.0]);
        assert!(Stagger::each(Order::Index, 100.0).delays(&[]).is_empty());
        assert!(Order::Index.ranks(&[]).is_empty());
    }
}
