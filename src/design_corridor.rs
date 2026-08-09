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
//! - a column is legal when the WHOLE vertical stack it would occupy
//!   (`y0 - 1 ..= y0 + 2*(width-1)`: a support and a dust per bit) is free of
//!   hard occupancy, outside every DECLARED keepout, and free of mechanism-level
//!   INTERFERENCE with foreign hardware — see [`BusFabric::column_free`];
//! - a column inside a cell's former blanket halo is legal but costs
//!   [`HUG_COST`], because that shell is the lane the cell's own ports escape
//!   through;
//! - turns cost real money (a corner needs a joint column and adds delay) and
//!   are illegal until the current leg is at least [`MIN_LEG`] cells long, so
//!   the search cannot emit a zigzag whose legs read into each other.
//!
//! The result is compressed to a waypoint chain and handed back to the
//! template planner, which realizes each leg with the same verified run /
//! joint-column vocabulary as before. No new redstone geometry is invented
//! here — only the order in which the existing tiles are laid down.

use crate::design::{OccupancyIndex, P3};
use crate::routing::engine::transport::{self, BlockView, Mechanism, Placement};
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
/// A third, far more determined rung (`turn_cost: 1, margin: 256, max_iter:
/// 1_500_000`) was measured on 2026-08-09 and recovered ZERO of the four
/// residual `design_routability` failures while adding ~25s to the suite. The
/// residuals are not search-budget-bound; see [`cross_level_probe`].
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
    /// Memo for [`BusFabric::column_hugs`], on the same hot path.
    hug_memo: RefCell<HashMap<(i32, i32), bool>>,
}

/// Extra cost per column that runs inside a cell's soft halo. Charged per
/// cell of travel, so a clean lane a few cells further out always wins, while
/// a hug still beats no route at all.
const HUG_COST: u32 = 4;

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
    /// Three conditions, and the third one is the whole reason a free-form
    /// corridor is harder than a template run:
    ///
    /// 1. the column's cells are unoccupied;
    /// 2. no cell of the column sits in a DECLARED keepout (designer intent);
    /// 3. no cell of the column INTERFERES with foreign hardware, in the
    ///    mechanism sense of
    ///    [`nucleation_routing::transport::interferes`]: one side's emission
    ///    lands where the other side reads, in a kind that side can read.
    ///
    /// Condition 3 replaced two blunter rules — a blanket one-cell shell
    /// dilated around every instance, and "no cell orthogonally adjacent to
    /// foreign redstone". Both over-forbade. An inert stone flank emits
    /// nothing and reads nothing, so a bus may lay dust flush against a cell
    /// body; a weakly powered block is invisible to dust; a repeater is blind
    /// to everything but its own back cell. The scalar halo forbade all of
    /// those, and it was the single largest exclusion bucket in
    /// `tests/design_routability.rs`. What condition 3 still forbids is every
    /// relation that is electrically real: foreign dust that would
    /// wire-connect to ours, a foreign repeater front or strong block driving
    /// our dust, and a support of ours that would sever a foreign net's 1-y
    /// step.
    ///
    /// The bus's own endpoint columns are exempt from all three: landing on
    /// the port and touching its dust is the entire point of a pin.
    pub fn column_free(&self, x: i32, z: i32) -> bool {
        if self.exempt.contains(&(x, z)) {
            return true;
        }
        if let Some(hit) = self.memo.borrow().get(&(x, z)) {
            return *hit;
        }
        let free = self
            .column_cells(x, z)
            .all(|p| self.cell_placeable(p, self.mech_at(p)));
        self.memo.borrow_mut().insert((x, z), free);
        free
    }

    /// Does a column here run inside some cell's soft halo?
    ///
    /// Electrically this is fine — that is the whole point of the transport
    /// predicate — but it is still *impolite*: the one-cell shell around a
    /// cell body is the lane that cell's own ports need in order to escape.
    /// The old blanket halo enforced that by forbidding the shell outright,
    /// which cost four legal routes; charging [`HUG_COST`] instead keeps
    /// corridors out of the shell whenever an open lane exists, and lets them
    /// in when the alternative is failing the bus.
    pub fn column_hugs(&self, x: i32, z: i32) -> bool {
        if self.exempt.contains(&(x, z)) {
            return false;
        }
        if let Some(hit) = self.hug_memo.borrow().get(&(x, z)) {
            return *hit;
        }
        let hugs = self
            .column_cells(x, z)
            .any(|p| self.occ.soft_halos.contains(&p));
        self.hug_memo.borrow_mut().insert((x, z), hugs);
        hugs
    }

    /// What the bus puts at this height. The stack alternates support / dust
    /// on a 2y pitch from `y0 - 1`, so the parity off the support level says
    /// which mechanism a cell carries — and the two answer to different rules.
    fn mech_at(&self, p: P3) -> Mechanism {
        if (p.1 - self.y0).rem_euclid(2) == 0 {
            Mechanism::Dust
        } else {
            Mechanism::SolidSupport
        }
    }

    /// Condition 1-3 for a single cell.
    fn cell_placeable(&self, p: P3, mech: Mechanism) -> bool {
        if self.occ.cells.contains_key(&p) {
            return false;
        }
        // A DECLARED keepout is absolute. A soft halo is only the old scalar
        // proxy for interference, so it defers to the real predicate below.
        if self.occ.halos.contains_key(&p) && !self.occ.soft_halos.contains(&p) {
            return false;
        }
        let view = OccView(self.occ);
        let ours = Placement {
            mech,
            cell: Pos::new(p.0, p.1, p.2),
            fwd: (1, 0, 0),
            net: OUR_NET,
        };
        for q in interference_scan(p) {
            if self.exempt.contains(&(q.0, q.2)) {
                continue; // our own port column
            }
            let Some((block, owner)) = self.occ.cells.get(&q) else {
                continue;
            };
            let theirs = Placement {
                mech: transport::mech_of(block),
                cell: Pos::new(q.0, q.1, q.2),
                fwd: transport::fwd_of(block),
                net: &owner_name(owner),
            };
            if transport::interferes(&theirs, &ours, &view).is_some() {
                return false;
            }
        }
        // A solid support of ours must not sever a foreign net's 1-y step:
        // the CUT cell is the one directly above the lower dust, and any
        // sturdy block there kills the step whether it conducts or not.
        if mech == Mechanism::SolidSupport && self.cuts_a_foreign_step(p) {
            return false;
        }
        true
    }

    /// Would a sturdy block at `p` sever a foreign dust step that currently
    /// conducts? `p` is the CUT cell of a step whose lower dust is directly
    /// beneath it and whose upper dust is one cell out, level with `p`.
    fn cuts_a_foreign_step(&self, p: P3) -> bool {
        let lower = (p.0, p.1 - 1, p.2);
        let Some((lb, lo)) = self.occ.cells.get(&lower) else {
            return false;
        };
        if transport::mech_of(lb) != Mechanism::Dust {
            return false;
        }
        let lower_owner = owner_name(lo);
        [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dz)| {
            let upper = (p.0 + dx, p.1, p.2 + dz);
            self.occ.cells.get(&upper).is_some_and(|(ub, uo)| {
                transport::mech_of(ub) == Mechanism::Dust && owner_name(uo) == lower_owner
            })
        })
    }
}

/// The cells whose contents can electrically reach `p`.
///
/// Every mechanism emits only into its own cell or its six faces, so an
/// emission relation never spans more than one cell. Dust's wire connection
/// reaches one cell horizontally with a 1-y step, which adds the eight
/// horizontal-plus-vertical offsets the six faces miss.
fn interference_scan(p: P3) -> impl Iterator<Item = P3> {
    let mut out = Vec::with_capacity(14);
    out.push((p.0, p.1 + 1, p.2));
    out.push((p.0, p.1 - 1, p.2));
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for dy in [-1, 0, 1] {
            out.push((p.0 + dx, p.1 + dy, p.2 + dz));
        }
    }
    out.into_iter()
}

/// A [`BlockView`] over the design's occupancy index, so the transport
/// predicates run against placed geometry without knowing what placed it.
struct OccView<'a>(&'a OccupancyIndex);

impl BlockView for OccView<'_> {
    fn block_at(&self, p: Pos) -> Option<&str> {
        self.0.cells.get(&(p.x, p.y, p.z)).map(|(b, _)| b.as_str())
    }
}

/// The net name for the bus being routed. Nothing already placed can own it,
/// so every hard cell in the index counts as foreign — which is right: the
/// bus's own surviving runs are ripped into `skip` before the search starts.
const OUR_NET: &str = "\0routing";

impl<'a> BusFabric<'a> {

    /// Who blocks a column here, if anybody — the location and the owner, for
    /// the user-facing failure reason.
    pub fn blocker(&self, x: i32, z: i32) -> Option<(P3, String)> {
        if self.exempt.contains(&(x, z)) {
            return None;
        }
        // Must agree with `column_free`, or the reason the studio shows the
        // user names a cause the search does not actually honour. A SOFT halo
        // is passable now, so it is never a blocker.
        for p in self.column_cells(x, z) {
            if let Some((block, owner)) = self.occ.cells.get(&p) {
                return Some((p, format!("{} `{block}`", owner_name(owner))));
            }
            if let Some(inst) = self.occ.halos.get(&p) {
                if !self.occ.soft_halos.contains(&p) {
                    return Some((p, format!("the declared keepout of instance `{inst}`")));
                }
            }
            if !self.cell_placeable(p, self.mech_at(p)) {
                return Some((
                    p,
                    "foreign redstone that would interfere with the bus here".to_string(),
                ));
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
                base_cost: 1
                    + if turning { self.turn_cost } else { 0 }
                    + if self.column_hugs(to.x, to.z) { HUG_COST } else { 0 },
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
        hug_memo: RefCell::new(HashMap::new()),
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

/// Would this bus route if it were allowed to change level?
///
/// Separates the two ways a corridor search can come back empty, which the user
/// has to fix in completely different ways:
///
/// - the bus's own LEVEL is congested, but a clear lane exists a few blocks up
///   or down. Nothing the user places can help; the bus form is a single-level
///   2y-pitch stack and cannot ramp (bit `k`'s dust at `y0 + 2k` is capped by
///   bit `k+1`'s opaque support at `y0 + 2k + 1`, so the stack cannot climb
///   without first spreading its pitch). This names the capability gap.
/// - no level is clear, so the workspace really is full and moving something is
///   the only fix.
///
/// Diagnosis-only: this runs once, on the failure path, never in the search.
fn cross_level_probe(occ: &OccupancyIndex, a: P3, b: P3, width: u8) -> String {
    let effort = LADDER[0];
    let mut clear = Vec::new();
    for dy in [-8, -6, -4, -2, 2, 4, 6, 8] {
        let (a2, b2) = ((a.0, a.1 + dy, a.2), (b.0, b.1 + dy, b.2));
        if search(occ, a2, b2, width, effort).is_some() {
            clear.push(a2.1);
        }
    }
    if clear.is_empty() {
        return " No level within 8 blocks up or down is clear either, so this is real congestion \
                 rather than a level-change limitation."
            .to_string();
    }
    format!(
        " A clear corridor DOES exist at y={}, and the router TRIED to hop there with a level \
         shift and could not fit one — a shift needs a straight run at each end, so the pair is \
         too short for the detour rather than blocked outright. Lengthen the run, or split it with \
         a gate at the clear level.",
        clear
            .iter()
            .map(|y| y.to_string())
            .collect::<Vec<_>>()
            .join(" or y=")
    )
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
    let level = cross_level_probe(occ, a, b, width);
    format!(
        "no corridor from {:?} to {:?} for a {width}-bit bus on level y={}: {line}. A bounded \
         detour search (margin {} cells, {} nodes) found no clear rectilinear corridor either — \
         the layers hemming the endpoints in are: {}.{level} Move one of them, give the bus a gate \
         to route through in two legs, or free a lane at least 1 cell clear of other redstone \
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
