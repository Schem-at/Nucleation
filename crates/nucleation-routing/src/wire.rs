//! GEOMETRIC redstone-wire connection states.
//!
//! A routed design authors its dust in the fully-spelled-out DEFAULT state
//! ([`crate::blocks::DUST`]: `east=none,north=none,south=none,west=none`).
//! That is a deliberate choice — authoring bare `minecraft:redstone_wire`
//! interns a property-less state that tick engines never normalise unless the
//! cell happens to catch a placement update, and such cells sit INERT (see
//! [`crate::blocks::DUST`]). But a wire with four `none` sides is, in the
//! renderer and in the model files, a DOT. So a correct route came out looking
//! like a trail of unconnected pips: right topology, wrong appearance, and
//! exported or rendered designs looked broken.
//!
//! The fix is not to simulate. Minecraft derives the connection state
//! GEOMETRICALLY when a wire is placed, from nothing but its neighbours, and
//! that derivation is cheap, deterministic and reproducible here. This module
//! is that derivation; the design realizer runs it over every dust cell it
//! authors, so the geometry is right the moment it is emitted and no bake is
//! required to look at it.
//!
//! # The rules (vanilla `RedStoneWireBlock::getConnectionState`)
//!
//! For each of the four horizontal directions, in order:
//!
//! 1. If the cell ABOVE this dust is not a full conductor, and the neighbour
//!    can hold dust on top, and there IS dust one up-and-over, the wire climbs:
//!    `up` when the neighbour is itself a solid block, else `side`.
//! 2. Otherwise, if the neighbour is something a wire connects to — dust, a
//!    repeater or comparator whose axis matches, a torch, a lever — `side`.
//! 3. Otherwise, if the neighbour is a full conductor, `none` (it blocks the
//!    view of anything beyond).
//! 4. Otherwise, if there is dust one down-and-over, `side` (the down
//!    diagonal).
//!
//! Then the axis extension: a wire with connections on ONE horizontal axis
//! only extends to the opposite side of the free axis, which is why a
//! dead-end renders as a line rather than a stub. This is the same fact the
//! POINTING LAW records from the other end (`probe_pivot.py`: "a single
//! connection extends to the opposite side"). A wire with NO connections at
//! all stays a dot, exactly as vanilla leaves it.
//!
//! # Two deliberate deviations, and why
//!
//! **Opacity defaults to TRUE.** Vanilla asks `isRedstoneConductor` ("full
//! collision box"). This crate has only block-state STRINGS, and
//! [`crate::blocks::is_solid_block`] recognises just the handful of blocks the
//! router itself places — ask it about `smooth_stone` or `quartz_block` and it
//! says no. Treating an unrecognised block as see-through invents connections
//! through it, so [`is_see_through`] carries a short allow-list and everything
//! else is opaque: that closes a side rather than fabricating a circuit.
//!
//! **Glass does not cap a climb.** Vanilla would (glass is a full cube), but
//! the probed material model has glass never cutting a diagonal, and a bus
//! dip's glass supports exist *precisely* so the diagonal beneath them
//! survives (`redstone-eda/notes-material-model.md`). Drawing that connection
//! closed would make the appearance contradict the topology the DRC, LVS and
//! mc-tick all compute. Laterally, glass is still opaque — it is a full cube.
//!
//! # What this must NEVER touch
//!
//! A placed library cell is a VERIFIED BLACK BOX. Its interior wire states
//! were authored by whoever built it, over blocks this module does not model,
//! and rewriting them breaks working redstone — measured: doing so turned the
//! ADD007 -> BINTOBCD001 chain's arithmetic wrong (1+1 read 0). Only cells the
//! DESIGN authors are ours to draw: bus fragments, adapters, and a promotion
//! patch's own writes.

use crate::blocks::{
    facing_of, facing_vec, is_comparator, is_dust, is_lever, is_repeater, is_solid_block,
    is_sturdy_support, is_torch,
};

/// Substrings marking a block a wire can SEE THROUGH — it neither closes a side
/// nor hides what lies beyond it.
///
/// The list is deliberately short, because the DEFAULT IS OPAQUE. This module
/// derives states over designs that contain arbitrary community redstone, and
/// [`is_solid_block`] only knows the handful of blocks the ROUTER places: asking
/// it about `smooth_stone` or `quartz_block` returns false, and treating an
/// unrecognised block as see-through invents connections straight through it.
/// "Unknown implies opaque" closes a side instead, which is the choice that
/// cannot fabricate a circuit that is not there.
const SEE_THROUGH: [&str; 14] = [
    "air",
    "redstone_wire",
    "repeater",
    "comparator",
    "redstone_torch",
    "redstone_wall_torch",
    "lever",
    "button",
    "pressure_plate",
    "tripwire",
    "rail",
    "fence",
    "pane",
    "torch",
];

/// Whether a wire can see through `block` (see [`SEE_THROUGH`]).
pub fn is_see_through(block: &str) -> bool {
    // Glass and top slabs are FULL CUBES, so a wire cannot see past them
    // laterally even though they never cut a diagonal from above.
    SEE_THROUGH.iter().any(|h| block.contains(h))
}

/// Whether `block`, sitting directly ABOVE a wire, stops it climbing.
///
/// Glass, stained glass and top-half slabs do NOT, because the probed material
/// model (`redstone-eda/notes-material-model.md`) has them never cutting a
/// diagonal — and a bus dip's glass supports exist precisely so the diagonal
/// beneath them survives. Drawing that connection closed would make the
/// appearance contradict the topology DRC, LVS and mc-tick all compute.
pub fn caps_climb(block: &str) -> bool {
    if is_see_through(block) {
        return false;
    }
    let transparent =
        block.contains("glass") || (block.contains("slab") && block.contains("type=top"));
    !transparent
}

/// A wire's state on one side, in vanilla's vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    /// No connection: this side is closed.
    None,
    /// Flat (or down-diagonal) connection.
    SideFlat,
    /// The wire climbs the neighbouring block's face.
    Up,
}

impl Side {
    /// The block-state property value.
    pub fn as_str(self) -> &'static str {
        match self {
            Side::None => "none",
            Side::SideFlat => "side",
            Side::Up => "up",
        }
    }

    fn connected(self) -> bool {
        self != Side::None
    }
}

/// The four horizontal directions with their unit offsets, in the order
/// vanilla's block state names them.
pub const DIRS: [(&str, i32, i32); 4] = [
    ("north", 0, -1),
    ("east", 1, 0),
    ("south", 0, 1),
    ("west", -1, 0),
];

/// Whether a wire connects sideways to `block` when looking along `(dx, dz)`.
///
/// Repeaters and comparators connect only on their own axis (a wire beside a
/// repeater's flank is not attached to it); torches and levers are signal
/// sources and connect on any side.
fn connects_to(block: &str, dx: i32, dz: i32) -> bool {
    if is_dust(block) {
        return true;
    }
    if is_repeater(block) || is_comparator(block) {
        // Vanilla technically connects a wire to a comparator on every side
        // (it is a signal source); treating it like a repeater — own axis
        // only — is what the diode actually means here, and it is what every
        // station this router builds wants drawn.
        return match facing_of(block).and_then(facing_vec) {
            Some((fx, _, fz)) => (fx != 0) == (dx != 0) && (fz != 0) == (dz != 0),
            None => false,
        };
    }
    is_torch(block) || is_lever(block)
}

/// Derive the connection state of dust at `pos`, preserving `power`.
///
/// `at` answers "what block is at this position?" over the WHOLE world the
/// wire will live in — the fragment being authored plus everything already
/// placed — so a bus drawn up against a cell's dust connects to it.
///
/// Returns the full block-state string, properties in the canonical
/// alphabetical order the rest of the crate uses.
pub fn derive(
    pos: (i32, i32, i32),
    power: u32,
    at: &dyn Fn((i32, i32, i32)) -> Option<String>,
) -> String {
    let (x, y, z) = pos;
    // A conductor overhead caps the wire: it cannot climb out.
    let open_above = !at((x, y + 1, z)).is_some_and(|b| caps_climb(&b));

    let mut sides = [Side::None; 4];
    for (i, (_, dx, dz)) in DIRS.iter().enumerate() {
        let n = (x + dx, y, z + dz);
        let nb = at(n);
        // 1. climb the neighbour's face
        if open_above {
            let can_hold = nb.as_deref().is_some_and(is_sturdy_support);
            let above_n = at((n.0, n.1 + 1, n.2));
            if can_hold && above_n.as_deref().is_some_and(is_dust) {
                sides[i] = if nb.as_deref().is_some_and(is_solid_block) {
                    Side::Up
                } else {
                    Side::SideFlat
                };
                continue;
            }
        }
        // 2. a neighbour a wire attaches to
        if nb.as_deref().is_some_and(|b| connects_to(b, *dx, *dz)) {
            sides[i] = Side::SideFlat;
            continue;
        }
        // 3. an opaque neighbour hides whatever is beyond it
        if nb.as_deref().is_some_and(|b| !is_see_through(b)) {
            continue;
        }
        // 4. the down diagonal
        if at((n.0, n.1 - 1, n.2)).as_deref().is_some_and(is_dust) {
            sides[i] = Side::SideFlat;
        }
    }

    // A wire with nothing to connect to stays a DOT, as vanilla leaves it.
    if sides.iter().all(|s| !s.connected()) {
        return state_string(&sides, power);
    }
    // Axis extension: connections on one axis only extend across the other, so
    // a dead-end reads as a line. (The POINTING LAW, from the other side.)
    let (n, e, s, w) = (sides[0], sides[1], sides[2], sides[3]);
    let free_x = !n.connected() && !s.connected();
    let free_z = !e.connected() && !w.connected();
    if free_x {
        if !w.connected() {
            sides[3] = Side::SideFlat;
        }
        if !e.connected() {
            sides[1] = Side::SideFlat;
        }
    }
    if free_z {
        if !n.connected() {
            sides[0] = Side::SideFlat;
        }
        if !s.connected() {
            sides[2] = Side::SideFlat;
        }
    }
    state_string(&sides, power)
}

/// `[east,north,power,south,west]` — alphabetical, matching
/// [`crate::blocks::DUST`] so authored and derived states compare cleanly.
fn state_string(sides: &[Side; 4], power: u32) -> String {
    format!(
        "minecraft:redstone_wire[east={},north={},power={},south={},west={}]",
        sides[1].as_str(),
        sides[0].as_str(),
        power,
        sides[2].as_str(),
        sides[3].as_str()
    )
}

/// The `power` a dust state carries (0 when unspecified).
pub fn power_of(block: &str) -> u32 {
    block
        .find("power=")
        .map(|i| &block[i + 6..])
        .and_then(|rest| {
            let end = rest.find([',', ']']).unwrap_or(rest.len());
            rest[..end].parse().ok()
        })
        .unwrap_or(0)
}

/// Rewrite every dust cell in `cells` with its geometrically derived state.
///
/// `outside` answers for positions NOT in `cells` (the rest of the design), so
/// a fragment connects correctly to the hardware it terminates on. Only dust
/// is touched; power is preserved.
pub fn rewire(
    cells: &mut std::collections::BTreeMap<(i32, i32, i32), String>,
    outside: &dyn Fn((i32, i32, i32)) -> Option<String>,
) {
    let snapshot = cells.clone();
    let at =
        |q: (i32, i32, i32)| -> Option<String> { snapshot.get(&q).cloned().or_else(|| outside(q)) };
    let dust: Vec<(i32, i32, i32)> = snapshot
        .iter()
        .filter(|(_, b)| is_dust(b))
        .map(|(p, _)| *p)
        .collect();
    for p in dust {
        let power = power_of(&snapshot[&p]);
        let derived = derive(p, power, &at);
        cells.insert(p, derived);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{repeater, DUST, STONE};
    use std::collections::BTreeMap;

    fn run(cells: &[((i32, i32, i32), &str)]) -> BTreeMap<(i32, i32, i32), String> {
        let mut m: BTreeMap<(i32, i32, i32), String> =
            cells.iter().map(|(p, b)| (*p, b.to_string())).collect();
        rewire(&mut m, &|_| None);
        m
    }

    /// The bug: a straight run's middle cell must read as a LINE, not a dot.
    #[test]
    fn a_straight_run_middle_cell_connects_east_and_west() {
        let mut cells = Vec::new();
        for x in 0..5 {
            cells.push(((x, 1, 0), STONE));
            cells.push(((x, 2, 0), DUST));
        }
        // fix the support rows
        let cells: Vec<_> = cells
            .into_iter()
            .map(|((x, y, z), b)| ((x, y, z), b))
            .collect();
        let m = run(&cells);
        let mid = &m[&(2, 2, 0)];
        assert!(mid.contains("east=side"), "{mid}");
        assert!(mid.contains("west=side"), "{mid}");
        assert!(mid.contains("north=none"), "{mid}");
        assert!(mid.contains("south=none"), "{mid}");
    }

    /// An L corner reads its TWO sides, and closes the other two.
    #[test]
    fn an_l_corner_connects_exactly_its_two_sides() {
        let mut cells = vec![];
        // west leg along -x into the corner at (0,2,0), then north leg along -z
        for x in -3..=0 {
            cells.push(((x, 1, 0), STONE));
            cells.push(((x, 2, 0), DUST));
        }
        for z in -3..0 {
            cells.push(((0, 1, z), STONE));
            cells.push(((0, 2, z), DUST));
        }
        let m = run(&cells);
        let corner = &m[&(0, 2, 0)];
        assert!(corner.contains("west=side"), "{corner}");
        assert!(corner.contains("north=side"), "{corner}");
        assert!(corner.contains("east=none"), "{corner}");
        assert!(corner.contains("south=none"), "{corner}");
    }

    /// A lone dust cell stays a DOT — vanilla does not invent connections.
    #[test]
    fn an_isolated_cell_stays_a_dot() {
        let m = run(&[((0, 1, 0), STONE), ((0, 2, 0), DUST)]);
        assert_eq!(m[&(0, 2, 0)], DUST, "an isolated wire must stay a dot");
    }

    /// A dead end extends across the free axis, so it draws as a line. This is
    /// the POINTING LAW seen from the render side.
    #[test]
    fn a_dead_end_extends_to_the_opposite_side() {
        let m = run(&[
            ((0, 1, 0), STONE),
            ((0, 2, 0), DUST),
            ((1, 1, 0), STONE),
            ((1, 2, 0), DUST),
        ]);
        let end = &m[&(0, 2, 0)];
        assert!(end.contains("east=side"), "{end}");
        assert!(
            end.contains("west=side"),
            "the dead end did not extend: {end}"
        );
    }

    /// A repeater is attached only on its own axis.
    #[test]
    fn a_repeater_connects_on_its_axis_only() {
        let m = run(&[
            ((0, 1, 0), STONE),
            ((0, 2, 0), DUST),
            ((1, 1, 0), STONE),
            ((1, 2, 0), &repeater("west", 1)),
            // a wire on the repeater's flank must NOT attach to it
            ((1, 1, 1), STONE),
            ((1, 2, 1), DUST),
        ]);
        assert!(m[&(0, 2, 0)].contains("east=side"), "{}", m[&(0, 2, 0)]);
        let flank = &m[&(1, 2, 1)];
        assert!(
            flank.contains("north=none"),
            "flank attached to a repeater: {flank}"
        );
    }

    /// A 1y step: the lower dust climbs, and the upper reads the diagonal back.
    /// Over a SOLID neighbour the climb is `up`.
    #[test]
    fn a_step_up_over_a_solid_reads_up() {
        let m = run(&[
            ((0, 1, 0), STONE),
            ((0, 2, 0), DUST),
            ((1, 2, 0), STONE), // the step block
            ((1, 3, 0), DUST),  // dust on top of it
        ]);
        let low = &m[&(0, 2, 0)];
        assert!(low.contains("east=up"), "the climb was not drawn: {low}");
        let high = &m[&(1, 3, 0)];
        assert!(
            high.contains("west=side"),
            "the down diagonal is missing: {high}"
        );
    }

    /// A conductor overhead caps the wire: it cannot climb out.
    #[test]
    fn a_lid_closes_the_climb() {
        let m = run(&[
            ((0, 1, 0), STONE),
            ((0, 2, 0), DUST),
            ((0, 3, 0), STONE), // the lid
            ((1, 2, 0), STONE),
            ((1, 3, 0), DUST),
        ]);
        let low = &m[&(0, 2, 0)];
        assert!(
            low.contains("east=none") || !low.contains("east=up"),
            "the lid did not close the climb: {low}"
        );
    }

    /// GLASS does not cap, because the verified model says glass never cuts a
    /// diagonal — the appearance must agree with the topology DRC/LVS/mc-tick
    /// compute (`notes-material-model.md`).
    #[test]
    fn glass_overhead_does_not_close_the_climb() {
        let m = run(&[
            ((0, 1, 0), STONE),
            ((0, 2, 0), DUST),
            ((0, 3, 0), "minecraft:glass"),
            ((1, 2, 0), STONE),
            ((1, 3, 0), DUST),
        ]);
        let low = &m[&(0, 2, 0)];
        assert!(
            low.contains("east=up"),
            "glass wrongly capped the climb: {low}"
        );
    }

    #[test]
    fn power_survives_the_rewrite() {
        let mut m: BTreeMap<(i32, i32, i32), String> = BTreeMap::new();
        m.insert((0, 1, 0), STONE.to_string());
        m.insert(
            (0, 2, 0),
            "minecraft:redstone_wire[east=none,north=none,power=11,south=none,west=none]"
                .to_string(),
        );
        m.insert((1, 1, 0), STONE.to_string());
        m.insert((1, 2, 0), DUST.to_string());
        rewire(&mut m, &|_| None);
        assert!(m[&(0, 2, 0)].contains("power=11"), "{}", m[&(0, 2, 0)]);
    }
}
