//! Wire-crossing tiles: verified hardware for taking two independent lines
//! through the same volume.
//!
//! Cell listings, ports, delays and footprints are in
//! `redstone-eda/crosswire_tiles.md`; the geometry was extracted from
//! `redstone-eda/crosswire/CROSSWIRE001_instant_crosswire.schem` and is
//! verified in-sim by `crosswire/verify_crosswire.py` (881 checks, 0
//! crosstalk). The closed forms here are a port of `updown_lines()` in
//! `crosswire/test_crosswire_templates.py`, which asserts formula == schematic
//! for every complete tile in the file.
//!
//! # `xw_updown`, and why it is the bus-friendly one
//!
//! Both lines enter and leave at the **same y**. One dips a level with a ±1
//! lane jog, the other bumps a level with a ±1 lane jog, and the intersection
//! cell is left **AIR**. That is the property a bus planner wants: crossing
//! buses do not need a level shift, so the "two buses must occupy disjoint
//! y-bands or match widths" constraint — the level-adapter gap — goes away.
//!
//! | | |
//! |---|---|
//! | delay | **0 gt** on both axes |
//! | cost | +2 ss per line |
//! | envelope | 7 x 4 x 7 per tiling unit |
//! | carries | 2 X-lines + 2 Z-lines per unit ⇒ 1 y-level per line |
//! | crossing cell | air |
//!
//! Tiling period is 4 in y, with port levels at `3 + 4k` and `5 + 4k`. On
//! `3 + 4k` the X-line dips and the Z-line bumps; on `5 + 4k` they swap. Every
//! dust cell has a solid support directly beneath it.

use pnr_core::grid::Pos;

/// Plan-view span of an `xw_updown` tile on both axes: in-port to out-port
/// inclusive.
pub const XW_UPDOWN_SPAN: i32 = 7;

/// Signal strength each line spends crossing an `xw_updown` tile: one for the
/// level change and one for the lane jog.
pub const XW_UPDOWN_SS_COST: u32 = 2;

/// Game ticks an `xw_updown` crossing costs. It is pure dust — there is no
/// diode in it at all.
pub const XW_UPDOWN_DELAY_GT: u32 = 0;

/// Tiling period in y. Port levels recur every 4 levels, on two of them.
pub const XW_UPDOWN_PERIOD_Y: i32 = 4;

/// One stamped `xw_updown` crossing, in tile-local coordinates
/// (`x = 0..=6`, `z = 10..=16`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XwUpdown {
    /// The X-axis line's dust, in signal order from the `-x` port.
    pub x_line: Vec<Pos>,
    /// The Z-axis line's dust, in signal order from the `-z` port.
    pub z_line: Vec<Pos>,
    /// The solid cell under every dust cell. Deduplicated and sorted; one of
    /// these does quadruple duty as the bump's support, the dip's lid and the
    /// CUT cell that keeps the two lines apart.
    pub supports: Vec<Pos>,
    /// The cell where the two lines cross, which must be left AIR. This is the
    /// whole trick of the family.
    pub intersection: Pos,
}

impl XwUpdown {
    /// Both ports of the X line: `(in, out)`, at the tile's port level.
    pub fn x_ports(&self) -> (Pos, Pos) {
        (self.x_line[0], self.x_line[self.x_line.len() - 1])
    }

    /// Both ports of the Z line: `(in, out)`, at the tile's port level.
    pub fn z_ports(&self) -> (Pos, Pos) {
        (self.z_line[0], self.z_line[self.z_line.len() - 1])
    }
}

/// Whether the X line DIPS (and the Z line bumps) at port level `y`.
///
/// The two axes swap roles every other port level, which is what lets
/// consecutive tiles share their intermediate levels.
pub fn xw_updown_x_dips(y: i32) -> bool {
    (y - 3).rem_euclid(XW_UPDOWN_PERIOD_Y) == 0
}

/// The `xw_updown` tile whose two ports both sit at level `y`.
///
/// A port level is any `y` with `(y - 3) % 2 == 0`; `3 + 4k` dips the X line
/// and `5 + 4k` dips the Z line.
pub fn xw_updown(y: i32) -> XwUpdown {
    let p = |x: i32, y: i32, z: i32| Pos::new(x, y, z);
    let (x_line, z_line) = if xw_updown_x_dips(y) {
        let mut x_line = vec![p(0, y, 13), p(1, y, 13)];
        x_line.extend((1..=5).map(|x| p(x, y - 1, 12))); // DIP, -z jog
        x_line.extend([p(5, y, 13), p(6, y, 13)]);

        let mut z_line = vec![p(3, y, 10), p(3, y, 11), p(4, y, 11)];
        z_line.extend([12, 13, 14].map(|z| p(4, y + 1, z))); // BUMP, +x jog
        z_line.extend([p(4, y, 15), p(3, y, 15), p(3, y, 16)]);
        (x_line, z_line)
    } else {
        let mut z_line = vec![p(3, y, 10), p(3, y, 11)];
        z_line.extend((11..=15).map(|z| p(2, y - 1, z))); // DIP, -x jog
        z_line.extend([p(3, y, 15), p(3, y, 16)]);

        let mut x_line = vec![p(0, y, 13), p(1, y, 13), p(1, y, 14)];
        x_line.extend([2, 3, 4].map(|x| p(x, y + 1, 14))); // BUMP, +z jog
        x_line.extend([p(5, y, 14), p(5, y, 13), p(6, y, 13)]);
        (x_line, z_line)
    };

    let mut supports: Vec<Pos> = x_line
        .iter()
        .chain(z_line.iter())
        .map(|c| Pos::new(c.x, c.y - 1, c.z))
        .collect();
    supports.sort_by_key(|c| (c.x, c.y, c.z));
    supports.dedup();

    XwUpdown {
        x_line,
        z_line,
        supports,
        intersection: Pos::new(3, y, 13),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn set(cells: &[(i32, i32, i32)]) -> BTreeSet<(i32, i32, i32)> {
        cells.iter().copied().collect()
    }

    fn as_tuples(cells: &[Pos]) -> BTreeSet<(i32, i32, i32)> {
        cells.iter().map(|c| (c.x, c.y, c.z)).collect()
    }

    /// THE guard: the Rust closed form must reproduce, cell for cell, the
    /// Python formula that `test_crosswire_templates.py` asserts equal to the
    /// schematic. These literals are transcribed from `crosswire_tiles.md`, so
    /// a drift in either direction fails here.
    #[test]
    fn the_x_dipping_tile_matches_the_schematic_derived_cells() {
        let y = 3; // 3 + 4k: X dips, Z bumps
        let t = xw_updown(y);
        assert_eq!(
            as_tuples(&t.x_line),
            set(&[
                (0, 3, 13),
                (1, 3, 13),
                (1, 2, 12),
                (2, 2, 12),
                (3, 2, 12),
                (4, 2, 12),
                (5, 2, 12),
                (5, 3, 13),
                (6, 3, 13),
            ])
        );
        assert_eq!(
            as_tuples(&t.z_line),
            set(&[
                (3, 3, 10),
                (3, 3, 11),
                (4, 3, 11),
                (4, 4, 12),
                (4, 4, 13),
                (4, 4, 14),
                (4, 3, 15),
                (3, 3, 15),
                (3, 3, 16),
            ])
        );
    }

    #[test]
    fn the_z_dipping_tile_matches_the_schematic_derived_cells() {
        let y = 5; // 5 + 4k: the mirror — Z dips, X bumps
        let t = xw_updown(y);
        assert_eq!(
            as_tuples(&t.z_line),
            set(&[
                (3, 5, 10),
                (3, 5, 11),
                (2, 4, 11),
                (2, 4, 12),
                (2, 4, 13),
                (2, 4, 14),
                (2, 4, 15),
                (3, 5, 15),
                (3, 5, 16),
            ])
        );
        assert_eq!(
            as_tuples(&t.x_line),
            set(&[
                (0, 5, 13),
                (1, 5, 13),
                (1, 5, 14),
                (2, 6, 14),
                (3, 6, 14),
                (4, 6, 14),
                (5, 5, 14),
                (5, 5, 13),
                (6, 5, 13),
            ])
        );
    }

    #[test]
    fn both_axes_leave_on_the_port_level_at_span_seven() {
        // The property that kills the level adapter: a bus arrives at y and
        // leaves at y, on BOTH axes.
        for y in [3, 5, 7, 9, 11, 13] {
            let t = xw_updown(y);
            let (xi, xo) = t.x_ports();
            let (zi, zo) = t.z_ports();
            assert_eq!((xi.y, xo.y), (y, y), "X ports must stay on level {y}");
            assert_eq!((zi.y, zo.y), (y, y), "Z ports must stay on level {y}");
            assert_eq!(xo.x - xi.x, XW_UPDOWN_SPAN - 1);
            assert_eq!(zo.z - zi.z, XW_UPDOWN_SPAN - 1);
            // Each line spends exactly 9 dust getting across.
            assert_eq!(t.x_line.len(), 9);
            assert_eq!(t.z_line.len(), 9);
        }
    }

    #[test]
    fn the_intersection_cell_is_never_claimed() {
        // The defining property of the family: nothing occupies (3, y, 13).
        for y in [3, 5, 7, 9] {
            let t = xw_updown(y);
            let all = as_tuples(&t.x_line);
            let z = as_tuples(&t.z_line);
            let sup = as_tuples(&t.supports);
            let i = (t.intersection.x, t.intersection.y, t.intersection.z);
            assert!(!all.contains(&i), "X line claims the intersection at y={y}");
            assert!(!z.contains(&i), "Z line claims the intersection at y={y}");
            assert!(
                !sup.contains(&i),
                "a support claims the intersection at y={y}"
            );
        }
    }

    #[test]
    fn the_lines_are_isolated_only_because_of_the_cut_cells() {
        use crate::blocks;
        use crate::transport::{wire_connects, BlockView};
        use std::collections::BTreeMap;

        for y in [3, 5, 7, 9] {
            let t = xw_updown(y);
            let x = as_tuples(&t.x_line);
            let z = as_tuples(&t.z_line);
            assert!(x.is_disjoint(&z), "the lines share a cell at y={y}");

            // Bare tile: dust plus the support under each dust cell, and NO
            // lids over the dipping run.
            let mut g: BTreeMap<Pos, String> = BTreeMap::new();
            for c in t.supports.iter() {
                g.insert(*c, blocks::STONE.to_string());
            }
            for c in t.x_line.iter().chain(t.z_line.iter()) {
                g.insert(*c, blocks::DUST.to_string());
            }
            let view: &dyn BlockView = &g;
            let shorts = t
                .x_line
                .iter()
                .flat_map(|a| t.z_line.iter().map(move |b| (a, b)))
                .filter(|(a, b)| wire_connects(view, **a, **b))
                .count();
            assert!(
                shorts > 0,
                "at y={y} the bare tile must SHORT: the dipping run steps into \
                 the other line unless lidded. If this ever passes, either the \
                 dust closed form or the step law has drifted."
            );
        }
    }

    /// What is NOT ported, recorded so the gap is not mistaken for coverage.
    ///
    /// The dust closed forms above are exact and schematic-guarded. The tile's
    /// LID / CUT cells — the solid cells over the dipping run that keep the two
    /// lines apart — are **not** ported, because they are not regular in the
    /// source file: at port level `y = 3` the lid set is
    /// `(2,y,11..15) (3,y,12) (4,y,12..14)`, while the structurally identical
    /// `y = 7` unit additionally has `(3,y,14)`. Until that is resolved against
    /// the hardware, a stamper must copy the lids from the schematic rather
    /// than generate them, so `xw_updown` is template DATA here and is not yet
    /// stampable by the bus planner.
    #[test]
    fn the_lid_cells_are_deliberately_not_ported() {
        let t = xw_updown(3);
        assert!(
            t.supports
                .iter()
                .all(|c| c.y < 3 || c.z == 12 || c.z == 13 || c.z == 14),
            "`supports` must only ever be the cell under a dust cell, never a lid"
        );
    }

    #[test]
    fn every_dust_cell_stands_on_a_conductor() {
        use crate::blocks;
        use crate::transport::conducts;
        for y in [3, 5, 7, 9] {
            let t = xw_updown(y);
            let sup = as_tuples(&t.supports);
            for c in t.x_line.iter().chain(t.z_line.iter()) {
                let below = (c.x, c.y - 1, c.z);
                assert!(sup.contains(&below), "dust {c:?} has no support at y={y}");
            }
            // And the support material is a conductor, as the tile requires.
            assert!(conducts(Some(blocks::STONE)));
        }
    }

    #[test]
    fn a_support_is_never_also_a_dust_cell() {
        // If a support coincided with dust the tile would be self-shorting.
        for y in [3, 5, 7, 9] {
            let t = xw_updown(y);
            let dust: BTreeSet<(i32, i32, i32)> = as_tuples(&t.x_line)
                .union(&as_tuples(&t.z_line))
                .copied()
                .collect();
            let sup = as_tuples(&t.supports);
            assert!(dust.is_disjoint(&sup), "a support doubles as dust at y={y}");
        }
    }

    #[test]
    fn consecutive_port_levels_share_their_intermediate_level() {
        // The envelope of a crossing PAIR is 3 levels but the tiling pitch is
        // 2: `y + 1` of one pair is `y - 1` of the next. That is why the unit
        // carries 4 lines in 4 levels.
        let lo = xw_updown(3);
        let hi = xw_updown(5);
        let lo_cells: BTreeSet<(i32, i32, i32)> = as_tuples(&lo.x_line)
            .union(&as_tuples(&lo.z_line))
            .copied()
            .collect();
        let hi_cells: BTreeSet<(i32, i32, i32)> = as_tuples(&hi.x_line)
            .union(&as_tuples(&hi.z_line))
            .copied()
            .collect();
        assert!(
            lo_cells.is_disjoint(&hi_cells),
            "stacked tiles must not collide: {:?}",
            lo_cells.intersection(&hi_cells).collect::<Vec<_>>()
        );
        // The upper tile's dip level (4) is the lower tile's bump level (4).
        assert!(lo_cells.iter().any(|c| c.1 == 4));
        assert!(hi_cells.iter().any(|c| c.1 == 4));
    }

    #[test]
    fn the_tile_is_delay_free() {
        assert_eq!(XW_UPDOWN_DELAY_GT, 0);
        assert_eq!(XW_UPDOWN_SS_COST, 2);
    }
}
