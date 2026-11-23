//! Wall shape updates are observable signals, including during piston moves.
use crate::fluid::{FluidWorld, WaterKind};
use crate::pos::Dir;
use crate::{BlockBehaviour, Pos, StateId, TickCtx};
use std::sync::Arc;

pub(crate) const SIDES: [Dir; 4] = [Dir::North, Dir::East, Dir::South, Dir::West];
pub(crate) const PROPERTIES: [&str; 4] = ["north", "east", "south", "west"];
pub(crate) const VALUES: [&str; 3] = ["none", "low", "tall"];
pub(crate) type ShapeBox = ([f64; 3], [f64; 3]);

pub(crate) fn index(sides: [u8; 4], up: bool) -> usize {
    sides
        .into_iter()
        .fold(0, |i, side| i * 3 + usize::from(side))
        * 2
        + usize::from(up)
}

pub(crate) trait WallWorld: FluidWorld {
    fn wall_connects(&self, ctx: &TickCtx<'_>, pos: Pos, face: Dir) -> bool;
    /// Coverage of the four arm test strips and the central post, and a post override.
    fn wall_cover(&self, ctx: &TickCtx<'_>, above: Pos) -> ([bool; 5], bool);
}

pub(crate) struct Wall<R: WallWorld> {
    pub sides: [u8; 4],
    pub waterlogged: bool,
    pub family: Arc<[Option<StateId>; 162]>,
    pub rules: R,
}

impl<R: WallWorld> BlockBehaviour for Wall<R> {
    fn on_shape_update(&self, ctx: &mut TickCtx<'_>, pos: Pos, from: Dir) {
        if self.waterlogged {
            crate::fluid::schedule_fluid(ctx, pos);
        }
        if from == Dir::Down {
            return;
        }
        let mut connected = self.sides.map(|side| side != 0);
        if let Some(i) = SIDES.iter().position(|d| *d == from) {
            connected[i] = self
                .rules
                .wall_connects(ctx, pos.offset(from), from.opposite());
        }
        let (cover, override_post) = self.rules.wall_cover(ctx, pos.offset(Dir::Up));
        let sides = std::array::from_fn(|i| {
            if !connected[i] {
                0
            } else if cover[i] {
                2
            } else {
                1
            }
        });
        // An above wall's post is an unconditional override. Otherwise an
        // asymmetric junction needs a post; two opposite tall arms suppress it.
        let asymmetric = connected.iter().all(|c| !c)
            || connected[0] != connected[2]
            || connected[1] != connected[3];
        let opposite_tall = (sides[0] == 2 && sides[2] == 2) || (sides[1] == 2 && sides[3] == 2);
        // wall_cover separates the unconditional stacked-wall case
        // from the ordinary post-override tag / centre coverage.
        let up = override_post || asymmetric || (!opposite_tall && cover[4]);
        if let Some(state) = self.family[index(sides, up)] {
            ctx.set_shape_only(pos, state);
        }
    }

    fn on_fluid_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if self.waterlogged {
            crate::fluid::spread(&self.rules, ctx, pos, WaterKind::Source);
        }
    }

    fn name(&self) -> &'static str {
        "wall"
    }
}

/// Exact rectangular face coverage. Partition at every edge instead of sampling
/// points, so gaps and unions of a moving piston head/base remain observable.
pub(crate) fn covers(boxes: &[ShapeBox], face: Dir, lo: [f64; 2], hi: [f64; 2]) -> bool {
    let (axis, axes, positive) = match face {
        Dir::Down => (1, [0, 2], false),
        Dir::Up => (1, [0, 2], true),
        Dir::North => (2, [0, 1], false),
        Dir::South => (2, [0, 1], true),
        Dir::West => (0, [2, 1], false),
        Dir::East => (0, [2, 1], true),
    };
    let faces: Vec<_> = boxes
        .iter()
        .filter(|(a, b)| {
            if positive {
                a[axis] < 1.0 && b[axis] >= 1.0
            } else {
                a[axis] <= 0.0 && b[axis] > 0.0
            }
        })
        .map(|(a, b)| {
            (
                [a[axes[0]].max(lo[0]), a[axes[1]].max(lo[1])],
                [b[axes[0]].min(hi[0]), b[axes[1]].min(hi[1])],
            )
        })
        .filter(|(a, b)| a[0] < b[0] && a[1] < b[1])
        .collect();
    let mut edges = vec![lo[0], hi[0]];
    for (a, b) in &faces {
        edges.extend([a[0], b[0]]);
    }
    edges.sort_by(f64::total_cmp);
    edges.dedup();
    for strip in edges.windows(2) {
        let mut intervals: Vec<_> = faces
            .iter()
            .filter(|(a, b)| a[0] <= strip[0] && b[0] >= strip[1])
            .map(|(a, b)| (a[1], b[1]))
            .collect();
        intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut end = lo[1];
        for (a, b) in intervals {
            if a > end {
                return false;
            }
            end = end.max(b);
        }
        if end < hi[1] {
            return false;
        }
    }
    true
}

pub(crate) fn bottom_cover(boxes: &[ShapeBox]) -> [bool; 5] {
    // Vanilla's 2-pixel-wide strips overlap the centre by one pixel.
    [
        ([7., 0.], [9., 9.]),
        ([7., 7.], [16., 9.]),
        ([7., 7.], [9., 16.]),
        ([0., 7.], [9., 9.]),
        ([7., 7.], [9., 9.]),
    ]
    .map(|(lo, hi)| covers(boxes, Dir::Down, lo.map(|v| v / 16.), hi.map(|v| v / 16.)))
}

/// Static collision shapes needed by wall connections and overhead coverage.
/// Dynamic moving-piston shapes are supplied by the rules at query time.
pub(crate) fn boxes(d: &crate::vanilla::Descriptor, full_cube: bool) -> Vec<ShapeBox> {
    let n = d.name.as_str();
    if n == "minecraft:moving_piston" {
        return vec![];
    }
    if full_cube {
        return vec![([0.; 3], [1.; 3])];
    }
    if matches!(n, "minecraft:piston" | "minecraft:sticky_piston") {
        if let Some(facing) = d.facing() {
            return vec![crate::piston::retracting_base_box(
                Pos::new(0, 0, 0),
                facing.opposite(),
            )];
        }
    }
    if n == "minecraft:piston_head" {
        if let Some(facing) = d.facing() {
            return crate::piston::head_shape_boxes(facing, d.flag("short")).to_vec();
        }
    }
    if n.ends_with("_slab") {
        return match d.get("type") {
            Some("top") => vec![([0., 0.5, 0.], [1.; 3])],
            Some("double") => vec![([0.; 3], [1.; 3])],
            _ => vec![([0.; 3], [1., 0.5, 1.])],
        };
    }
    if n.ends_with("_trapdoor") {
        let mut lo = [0.; 3];
        let mut hi = [1.; 3];
        if d.flag("open") {
            match d.facing() {
                Some(Dir::North) => lo[2] = 13. / 16.,
                Some(Dir::South) => hi[2] = 3. / 16.,
                Some(Dir::West) => lo[0] = 13. / 16.,
                Some(Dir::East) => hi[0] = 3. / 16.,
                _ => (),
            }
        } else if d.get("half") == Some("top") {
            lo[1] = 13. / 16.;
        } else {
            hi[1] = 3. / 16.;
        }
        return vec![(lo, hi)];
    }
    if n.ends_with("_wall") {
        let mut boxes = Vec::new();
        if d.flag("up") {
            boxes.push(([0.25, 0., 0.25], [0.75, 1.5, 0.75]));
        }
        for (i, p) in PROPERTIES.iter().enumerate() {
            if d.get(p).is_some_and(|v| v != "none") {
                boxes.push(match i {
                    0 => ([5. / 16., 0., 0.], [11. / 16., 1.5, 0.5]),
                    1 => ([0.5, 0., 5. / 16.], [1., 1.5, 11. / 16.]),
                    2 => ([5. / 16., 0., 0.5], [11. / 16., 1.5, 1.]),
                    _ => ([0., 0., 5. / 16.], [0.5, 1.5, 11. / 16.]),
                });
            }
        }
        return boxes;
    }
    if n == "minecraft:soul_sand" {
        return vec![([0.; 3], [1., 14. / 16., 1.])];
    }
    if matches!(n, "minecraft:repeater" | "minecraft:comparator") {
        return vec![([0.; 3], [1., 2. / 16., 1.])];
    }
    // Other partial blocks retain the engine's existing collision limitation.
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn face_union_and_thin_gap_are_distinguished() {
        let halves = [([0.; 3], [0.5, 1., 1.]), ([0.5, 0., 0.], [1.; 3])];
        assert_eq!(bottom_cover(&halves), [true; 5]);
        let gap = [halves[0], ([0.500001, 0., 0.], [1.; 3])];
        assert_eq!(bottom_cover(&gap), [false; 5]);
        assert_eq!(bottom_cover(&[([0., 0.5, 0.], [1.; 3])]), [false; 5]);
    }
}
