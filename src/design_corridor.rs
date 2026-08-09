//! Bus corridor search: the detour a bus takes when the deterministic
//! template (a straight run, or one implicit L corner) is blocked.
//!
//! The template planner in [`crate::design`] knows exactly two shapes per
//! waypoint pair. That is enough in an empty field and hopeless in a real
//! design: the moment another instance's body or influence halo sits in the
//! corridor, both shapes collide and the bus lands in `FAILED`. This module
//! is the third shape — *any* rectilinear corridor — found with
//! [`pnr_core`]'s weighted A* over a [`BusFabric`] that models the bus form's
//! real footprint:
//!
//! - the search runs on the bit-0 dust plane `y = y0`, one node per column;
//! - a column is legal only when the WHOLE vertical stack it would occupy
//!   (`y0 - 1 ..= y0 + 2*(width-1)`: a support and a dust per bit) is free of
//!   hard occupancy and of every foreign instance halo;
//! - turns cost real money (a corner needs a joint column and adds delay) and
//!   are illegal until the current leg is at least [`MIN_LEG`] cells long, so
//!   the search cannot emit a zigzag whose legs read into each other.
//!
//! The result is compressed to a waypoint chain and handed back to the
//! template planner, which realizes each leg with the same verified run /
//! joint-column vocabulary as before. No new redstone geometry is invented
//! here — only the order in which the existing tiles are laid down.

use crate::design::{OccupancyIndex, P3};
use pnr_core::astar::{route, RouteRequest};
use pnr_core::fabric::{Budget, Candidate, Fabric, RouteCtx, State};
use pnr_core::grid::{Aabb, Pos};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

/// Minimum straight cells before a corner may turn again. Two corners closer
/// than this would put their joint columns diagonally adjacent, and dust reads
/// diagonally — the corridor would short itself.
pub const MIN_LEG: u8 = 3;

/// One rung of the retry ladder: how hard to try.
#[derive(Copy, Clone, Debug)]
pub struct Effort {
    /// Extra cost charged for a corner. Low = a wigglier but more determined
    /// search; high = prefers few, long legs.
    pub turn_cost: u32,
    /// Cells of slack added around the endpoints' bounding box.
    pub margin: i32,
    /// A* node budget.
    pub max_iter: usize,
}

/// The retry ladder, tried in order. Rung 1 wants a tidy corridor; rung 2
/// accepts a scrappier one over a much wider workspace before we give up.
///
/// The node budgets are deliberately modest: `route_bus` is an INTERACTIVE
/// call in the studio, and a bus that cannot be routed has to say so quickly.
/// A hopeless search explores its whole bound before failing, so the bound —
/// not the iteration cap — is what keeps the worst case bearable.
pub const LADDER: [Effort; 2] = [
    Effort {
        turn_cost: 12,
        margin: 24,
        max_iter: 80_000,
    },
    Effort {
        turn_cost: 4,
        margin: 96,
        max_iter: 400_000,
    },
];

/// Which way the search head last moved.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Heading {
    /// At the source; the first leg may go any way.
    Start,
    PlusX,
    MinusX,
    PlusZ,
    MinusZ,
}

impl Heading {
    fn delta(self) -> (i32, i32) {
        match self {
            Heading::Start => (0, 0),
            Heading::PlusX => (1, 0),
            Heading::MinusX => (-1, 0),
            Heading::PlusZ => (0, 1),
            Heading::MinusZ => (0, -1),
        }
    }

    fn opposite(self) -> Heading {
        match self {
            Heading::Start => Heading::Start,
            Heading::PlusX => Heading::MinusX,
            Heading::MinusX => Heading::PlusX,
            Heading::PlusZ => Heading::MinusZ,
            Heading::MinusZ => Heading::PlusZ,
        }
    }

    const ALL: [Heading; 4] = [
        Heading::PlusX,
        Heading::MinusX,
        Heading::PlusZ,
        Heading::MinusZ,
    ];
}

/// Search memory: the current heading plus how many cells the current leg has
/// run (saturating at [`MIN_LEG`], which is all the turn rule needs).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Leg {
    heading: Heading,
    run: u8,
}

/// The bus form as a [`Fabric`]: columns on the bit-0 dust plane, legal only
/// when the whole stack clears.
pub struct BusFabric<'a> {
    occ: &'a OccupancyIndex,
    /// Bit-0 canonical dust level.
    y0: i32,
    /// Bus width in bits.
    width: u8,
    /// Columns exempt from the occupancy test: the bus's own endpoint
    /// hardware, which is already dust on a support and is never re-stamped.
    exempt: BTreeSet<(i32, i32)>,
    bound: Aabb,
    turn_cost: u32,
    /// Column-legality memo. The search visits a column once per heading and
    /// once per incoming move, so without this the stack scan runs ~20x per
    /// column — the difference between an interactive reroute and a stall.
    memo: RefCell<HashMap<(i32, i32), bool>>,
}

impl<'a> BusFabric<'a> {
    /// Every cell a bus column at `(x, z)` would occupy: a support and a dust
    /// per bit, contiguous from `y0 - 1` to `y0 + 2*(width-1)`.
    fn column_cells(&self, x: i32, z: i32) -> impl Iterator<Item = P3> + '_ {
        let lo = self.y0 - 1;
        let hi = self.y0 + 2 * (self.width as i32 - 1);
        (lo..=hi).map(move |y| (x, y, z))
    }

    /// Can the bus stand a column here?
    ///
    /// Two conditions, and the second one is the whole reason a free-form
    /// corridor is harder than a template run:
    ///
    /// 1. the column's cells are unoccupied and outside every foreign halo;
    /// 2. no cell of the column is ORTHOGONALLY ADJACENT to foreign redstone
    ///    (another bus's dust or repeater, or a cell's exposed dust). Two dust
    ///    runs one cell apart short without ever sharing a cell — the template
    ///    planner never hit this because parallel runs collide outright, but a
    ///    corridor that is free to bend will happily hug a neighbour.
    ///
    /// The bus's own endpoint columns are exempt from both: landing on the port
    /// and touching its dust is the entire point of a pin.
    pub fn column_free(&self, x: i32, z: i32) -> bool {
        if self.exempt.contains(&(x, z)) {
            return true;
        }
        if let Some(hit) = self.memo.borrow().get(&(x, z)) {
            return *hit;
        }
        let free = self.column_cells(x, z).all(|p| {
            if self.occ.cells.contains_key(&p) || self.occ.halos.contains_key(&p) {
                return false;
            }
            // Electrical clearance against foreign redstone.
            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().all(|(dx, dz)| {
                let q = (p.0 + dx, p.1, p.2 + dz);
                if self.exempt.contains(&(q.0, q.2)) {
                    return true; // our own port column
                }
                !self
                    .occ
                    .cells
                    .get(&q)
                    .is_some_and(|(b, _)| is_live_redstone(b))
            })
        });
        self.memo.borrow_mut().insert((x, z), free);
        free
    }

    /// Who blocks a column here, if anybody — the location and the owner, for
    /// the user-facing failure reason.
    pub fn blocker(&self, x: i32, z: i32) -> Option<(P3, String)> {
        if self.exempt.contains(&(x, z)) {
            return None;
        }
        for p in self.column_cells(x, z) {
            if let Some((block, owner)) = self.occ.cells.get(&p) {
                return Some((p, format!("{} `{block}`", owner_name(owner))));
            }
            if let Some(inst) = self.occ.halos.get(&p) {
                return Some((p, format!("the influence halo of instance `{inst}`")));
            }
        }
        None
    }
}

/// A block that would electrically interact with dust laid one cell away.
fn is_live_redstone(block: &str) -> bool {
    use crate::routing::engine::blocks as rblocks;
    rblocks::is_dust(block) || rblocks::is_repeater(block)
}

fn owner_name(o: &crate::design::Occupant) -> String {
    match o {
        crate::design::Occupant::Loose => "loose block".to_string(),
        crate::design::Occupant::Instance(n) => format!("instance `{n}`"),
        crate::design::Occupant::Bus(n) => format!("bus `{n}`"),
    }
}

impl Fabric for BusFabric<'_> {
    type Memory = Leg;
    type Tag = Heading;

    fn start_memory(&self) -> Leg {
        Leg {
            heading: Heading::Start,
            run: 0,
        }
    }

    fn moves(&self, from: &State<Leg>, _ctx: &RouteCtx) -> Vec<Candidate<Leg, Heading>> {
        let mut out = Vec::with_capacity(4);
        for h in Heading::ALL {
            // No U-turns, and no corner until the current leg is long enough
            // for the two joint columns to stay non-adjacent.
            let turning = from.mem.heading != Heading::Start && from.mem.heading != h;
            if from.mem.heading != Heading::Start {
                if h == from.mem.heading.opposite() {
                    continue;
                }
                if turning && from.mem.run < MIN_LEG {
                    continue;
                }
            }
            let (dx, dz) = h.delta();
            let to = Pos::new(from.pos.x + dx, self.y0, from.pos.z + dz);
            let run = if turning || from.mem.heading == Heading::Start {
                1
            } else {
                from.mem.run.saturating_add(1).min(MIN_LEG)
            };
            out.push(Candidate {
                to: State {
                    pos: to,
                    mem: Leg { heading: h, run },
                },
                base_cost: 1 + if turning { self.turn_cost } else { 0 },
                tag: h,
                footprint: vec![to],
            });
        }
        out
    }

    fn legal(&self, _from: &State<Leg>, cand: &Candidate<Leg, Heading>, _ctx: &RouteCtx) -> bool {
        let p = cand.to.pos;
        self.bound.contains(p) && self.column_free(p.x, p.z)
    }

    fn budget(&self) -> Budget {
        Budget::default()
    }
}

/// Build the fabric for one corridor query.
fn fabric<'a>(
    occ: &'a OccupancyIndex,
    a: P3,
    b: P3,
    width: u8,
    effort: Effort,
) -> BusFabric<'a> {
    let mut exempt = BTreeSet::new();
    exempt.insert((a.0, a.2));
    exempt.insert((b.0, b.2));
    let m = effort.margin;
    let bound = Aabb::new(
        Pos::new(a.0.min(b.0) - m, a.1, a.2.min(b.2) - m),
        Pos::new(a.0.max(b.0) + m, a.1, a.2.max(b.2) + m),
    );
    BusFabric {
        occ,
        y0: a.1,
        width,
        exempt,
        bound,
        turn_cost: effort.turn_cost,
        memo: RefCell::new(HashMap::new()),
    }
}

/// Search for a corridor from `a` to `b` for a `width`-bit bus. Both anchors
/// must sit on the same level (the caller enforces the bus form). Returns the
/// compressed waypoint chain, `a` first and `b` last, every consecutive pair
/// axis-aligned and non-empty.
pub fn search(occ: &OccupancyIndex, a: P3, b: P3, width: u8, effort: Effort) -> Option<Vec<P3>> {
    if a == b || a.1 != b.1 {
        return None;
    }
    let f = fabric(occ, a, b, width, effort);
    let mut req = RouteRequest::new(Pos::new(a.0, a.1, a.2), Pos::new(b.0, b.1, b.2));
    req.max_iter = effort.max_iter;
    let path = route(&f, &req, &RouteCtx { net: 0 }, &|_| 0)?;
    let cells: Vec<P3> = path.iter().map(|s| (s.pos.x, s.pos.y, s.pos.z)).collect();
    let chain = compress(&cells);
    // A chain the template planner can actually realize: at least one leg,
    // every leg axis-aligned and non-degenerate.
    if chain.len() < 2 {
        return None;
    }
    for w in chain.windows(2) {
        let (p, q) = (w[0], w[1]);
        if p == q || (p.0 != q.0 && p.2 != q.2) {
            return None;
        }
    }
    if !self_clearance_ok(&chain) {
        return None;
    }
    Some(chain)
}

/// Reject a corridor that comes back within one cell of itself.
///
/// The search state is `(column, leg)`, so A* MAY legally revisit a column
/// with a different heading — and a corridor that touches itself closes a
/// ring through its own refresh repeaters. `Design::check` catches that
/// afterwards as `repeater_cycle`; catching it here means the next ladder rung
/// gets a chance instead of the bus failing.
///
/// Consecutive legs share a corner and are exempt. Everything else must keep
/// at least one empty cell of separation, which also rules out the diagonal
/// reads that would merge two legs into one dust net.
fn self_clearance_ok(chain: &[P3]) -> bool {
    let legs: Vec<(P3, P3)> = chain.windows(2).map(|w| (w[0], w[1])).collect();
    for i in 0..legs.len() {
        for j in (i + 2)..legs.len() {
            if leg_distance(legs[i], legs[j]) < 2 {
                return false;
            }
        }
    }
    true
}

/// Chebyshev distance between two axis-aligned legs in the xz plane. Each leg
/// IS its own bounding box, so a per-axis gap is exact.
fn leg_distance(a: (P3, P3), b: (P3, P3)) -> i32 {
    let gap = |alo: i32, ahi: i32, blo: i32, bhi: i32| (blo - ahi).max(alo - bhi).max(0);
    let gx = gap(
        a.0 .0.min(a.1 .0),
        a.0 .0.max(a.1 .0),
        b.0 .0.min(b.1 .0),
        b.0 .0.max(b.1 .0),
    );
    let gz = gap(
        a.0 .2.min(a.1 .2),
        a.0 .2.max(a.1 .2),
        b.0 .2.min(b.1 .2),
        b.0 .2.max(b.1 .2),
    );
    gx.max(gz)
}

/// Collapse a cell-by-cell path to its corners.
fn compress(cells: &[P3]) -> Vec<P3> {
    if cells.len() < 2 {
        return cells.to_vec();
    }
    let mut out = vec![cells[0]];
    for i in 1..cells.len() - 1 {
        let (prev, cur, next) = (cells[i - 1], cells[i], cells[i + 1]);
        let d0 = (cur.0 - prev.0, cur.2 - prev.2);
        let d1 = (next.0 - cur.0, next.2 - cur.2);
        if d0 != d1 {
            out.push(cur);
        }
    }
    out.push(cells[cells.len() - 1]);
    out
}

/// The user-facing reason a corridor could not be found. Names the cause AND
/// the location, because the studio shows this string to the user verbatim.
///
/// `tried` carries what the template shapes reported, so a reason never
/// degrades to a bare "no path".
pub fn diagnose(occ: &OccupancyIndex, a: P3, b: P3, width: u8, tried: &[String]) -> String {
    let effort = LADDER[LADDER.len() - 1];
    let f = fabric(occ, a, b, width, effort);

    // Endpoint escape: can the bus leave its own anchor at all?
    for (which, anchor) in [("driver", a), ("sink", b)] {
        let mut blocked = Vec::new();
        let mut open = false;
        for h in Heading::ALL {
            let (dx, dz) = h.delta();
            let (x, z) = (anchor.0 + dx, anchor.2 + dz);
            match f.blocker(x, z) {
                None => open = true,
                Some((p, owner)) => blocked.push(format!("{:?} blocked by {owner}", p)),
            }
        }
        if !open {
            return format!(
                "endpoint approach blocked: the {which} anchor {:?} is walled in — every \
                 neighbouring column of the {width}-bit stack (y {}..={}) is occupied: {}. Move \
                 the endpoint, shrink the neighbouring cell's keepout, or leave a clear lane \
                 beside the port",
                anchor,
                a.1 - 1,
                a.1 + 2 * (width as i32 - 1),
                blocked.join("; ")
            );
        }
    }

    // Otherwise: report what sits on the direct line, WHICH LAYERS are in the
    // way, and that a bounded detour search over the whole workspace still
    // found nothing. Naming the layers is what makes this fixable: "bus `x` is
    // in the way" tells the user to reroute or move something specific.
    let direct = first_blocker_on_line(&f, a, b);
    let line = match direct {
        Some((p, owner)) => format!("the direct line is blocked at {:?} by {owner}", p),
        None => "the direct line is clear but the template shapes were rejected".to_string(),
    };
    let culprits = blocking_layers(&f, a, b);
    format!(
        "no corridor from {:?} to {:?} for a {width}-bit bus on level y={}: {line}. A bounded \
         detour search (margin {} cells, {} nodes) found no clear rectilinear corridor either — \
         the layers hemming the endpoints in are: {}. Move one of them, give the bus a gate to \
         route through in two legs, or free a lane at least 1 cell clear of other redstone \
         (dust one cell apart shorts, so the corridor needs 2 cells of pitch). Template \
         attempts: {}",
        a,
        b,
        a.1,
        effort.margin,
        effort.max_iter,
        if culprits.is_empty() {
            "none found (the bound may be too tight)".to_string()
        } else {
            culprits.join(", ")
        },
        if tried.is_empty() {
            "none".to_string()
        } else {
            tried.join(" | ")
        }
    )
}

/// The distinct layers blocking the columns around both endpoints and along the
/// direct line — the things the user can actually move.
fn blocking_layers(f: &BusFabric<'_>, a: P3, b: P3) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for anchor in [a, b] {
        for dx in -2..=2i32 {
            for dz in -2..=2i32 {
                if let Some((_, owner)) = f.blocker(anchor.0 + dx, anchor.2 + dz) {
                    seen.insert(strip_block(&owner));
                }
            }
        }
    }
    if let Some((_, owner)) = first_blocker_on_line(f, a, b) {
        seen.insert(strip_block(&owner));
    }
    seen.into_iter().collect()
}

/// `instance \`u2\` \`minecraft:stone\`` -> `instance \`u2\`` — the owner is
/// what the user can move; which of its blocks was hit is noise in a list.
fn strip_block(owner: &str) -> String {
    match owner.find(" `minecraft:") {
        Some(i) => owner[..i].to_string(),
        None => owner.to_string(),
    }
}

/// Walk the L-shaped direct line and report the first blocked column.
fn first_blocker_on_line(f: &BusFabric<'_>, a: P3, b: P3) -> Option<(P3, String)> {
    let sx = (b.0 - a.0).signum();
    let sz = (b.2 - a.2).signum();
    let mut x = a.0;
    while x != b.0 {
        x += sx;
        if let Some(hit) = f.blocker(x, a.2) {
            return Some(hit);
        }
    }
    let mut z = a.2;
    while z != b.2 {
        z += sz;
        if let Some(hit) = f.blocker(b.0, z) {
            return Some(hit);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::Occupant;

    fn wall(occ: &mut OccupancyIndex, x: i32, zs: std::ops::RangeInclusive<i32>, ys: std::ops::RangeInclusive<i32>) {
        for z in zs {
            for y in ys.clone() {
                occ.cells
                    .insert((x, y, z), ("minecraft:stone".to_string(), Occupant::Loose));
            }
        }
    }

    #[test]
    fn a_wall_with_a_gap_is_routed_around() {
        let mut occ = OccupancyIndex::default();
        // Wall at x=20 with a gap at z in 14..=26.
        wall(&mut occ, 20, -40..=13, 0..=20);
        wall(&mut occ, 20, 27..=60, 0..=20);
        let chain = search(&occ, (1, 2, 8), (40, 2, 8), 8, LADDER[0]).expect("corridor exists");
        assert_eq!(chain[0], (1, 2, 8));
        assert_eq!(*chain.last().unwrap(), (40, 2, 8));
        // It must cross x=20 inside the gap.
        let f = fabric(&occ, (1, 2, 8), (40, 2, 8), 8, LADDER[0]);
        for w in chain.windows(2) {
            let (p, q) = (w[0], w[1]);
            assert!(p.0 == q.0 || p.2 == q.2, "leg not axis-aligned: {p:?}->{q:?}");
        }
        // Every interior corner column must be legal.
        for c in &chain[1..chain.len() - 1] {
            assert!(f.column_free(c.0, c.2), "corner {c:?} not free");
        }
    }

    #[test]
    fn a_sealed_wall_reports_an_actionable_reason() {
        let mut occ = OccupancyIndex::default();
        wall(&mut occ, 20, -400..=400, 0..=20);
        assert!(search(&occ, (1, 2, 8), (40, 2, 8), 8, LADDER[0]).is_none());
        let why = diagnose(&occ, (1, 2, 8), (40, 2, 8), 8, &[]);
        assert!(why.contains("no corridor"), "{why}");
        assert!(why.contains("(20,"), "names the blocker location: {why}");
        assert!(why.contains("loose block"), "names the owner: {why}");
    }

    #[test]
    fn a_walled_in_endpoint_says_so() {
        let mut occ = OccupancyIndex::default();
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for y in 0..=20 {
                occ.cells.insert(
                    (1 + dx, y, 8 + dz),
                    ("minecraft:stone".to_string(), Occupant::Loose),
                );
            }
        }
        let why = diagnose(&occ, (1, 2, 8), (40, 2, 8), 8, &[]);
        assert!(why.contains("endpoint approach blocked"), "{why}");
        assert!(why.contains("driver"), "{why}");
    }

    #[test]
    fn a_self_touching_corridor_is_rejected() {
        // A spiral that comes back alongside itself: legs 0 and 3 are one
        // cell apart, which would merge them into one dust net and close a
        // ring through the refresh repeaters.
        let spiral = [(0, 2, 0), (20, 2, 0), (20, 2, 10), (0, 2, 10), (0, 2, 1)];
        assert!(!self_clearance_ok(&spiral));
        // The same shape with real separation is fine.
        let roomy = [(0, 2, 0), (20, 2, 0), (20, 2, 10), (0, 2, 10), (0, 2, 6)];
        assert!(self_clearance_ok(&roomy));
        // A plain U-turn at MIN_LEG separation is legal.
        let u = [(0, 2, 0), (20, 2, 0), (20, 2, 3), (0, 2, 3)];
        assert!(self_clearance_ok(&u));
    }

    #[test]
    fn a_corridor_keeps_clearance_from_foreign_dust() {
        // A neighbouring bus's dust lane at z=9 must push the corridor away:
        // dust one cell apart shorts, so hugging it is illegal even though no
        // cell is shared.
        let mut occ = OccupancyIndex::default();
        for x in 0..60 {
            for k in 0..8i32 {
                occ.cells.insert(
                    (x, 2 + 2 * k, 9),
                    (
                        "minecraft:redstone_wire[power=0]".to_string(),
                        Occupant::Bus("other".into()),
                    ),
                );
            }
        }
        let f = fabric(&occ, (1, 2, 4), (40, 2, 4), 8, LADDER[0]);
        assert!(!f.column_free(20, 8), "z=8 hugs the foreign lane at z=9");
        assert!(!f.column_free(20, 10), "z=10 hugs it from the other side");
        assert!(f.column_free(20, 7), "z=7 has a clear cell of separation");
    }

    #[test]
    fn a_narrow_gap_still_admits_a_tall_bus() {
        // The gap must clear the WHOLE stack, not just bit 0: a wall with a
        // hole only at bit 0's level is not a corridor.
        let mut occ = OccupancyIndex::default();
        for z in -300i32..=300 {
            for y in 0i32..=40 {
                // A hole tall enough for bit 0 only (its support at y0-1=1 and
                // its dust at y0=2); bit 1's dust at y=4 still hits stone.
                if (z - 20).abs() <= 6 && (1..=3).contains(&y) {
                    continue;
                }
                occ.cells
                    .insert((20, y, z), ("minecraft:stone".to_string(), Occupant::Loose));
            }
        }
        assert!(
            search(&occ, (1, 2, 8), (40, 2, 8), 8, LADDER[1]).is_none(),
            "an 8-bit stack must not squeeze through a 3-high hole"
        );
        // A 1-bit bus fits the same hole.
        assert!(search(&occ, (1, 2, 8), (40, 2, 8), 1, LADDER[1]).is_some());
    }
}
