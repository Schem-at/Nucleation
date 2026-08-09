//! Port PROMOTION: turning a cell's executor-only IO into routable dust.
//!
//! Community redstone names *executor hardware* in its contract — inputs are
//! LEVERS or BUTTONS, outputs are LAMPS. Nothing in redstone drives a lever, so
//! `ADD007.sum -> BINTOBCD001.bin` is impossible however good the router is:
//! the sink has no dust to land on. That single fact is why the studio cannot
//! chain two library cells, and it is what this module removes.
//!
//! A port has two MODES (see [`crate::design::PortMode`]):
//!
//! - **Executor** — the shipped hardware. Drivable by `CellExecutor`, never
//!   routable.
//! - **Bus** — the hardware is replaced by a *driver stub* that ends in dust.
//!   Routable, no longer hand-drivable.
//!
//! The switch is a reversible per-instance PATCH, not an edit to the cell: the
//! original block states are saved so Bus -> Executor restores them
//! byte-exactly. Nothing here mutates the shared cell library.
//!
//! # The two input strategies (both verified in the tick engine)
//!
//! What a lever really does is STRONGLY power its attachment block; everything
//! downstream reads that block. Dust only ever powers a block WEAKLY, and weak
//! power does not reach dust — so replacing a lever with dust works only when
//! the attachment block's consumers are repeaters/comparators. That split is
//! measurable and it decides the strategy:
//!
//! - `face=floor` (attachment block DIRECTLY BELOW): put dust in the lever's
//!   own cell. It sits on the attachment block and powers it from above.
//!   Verified: `BINTOBCD001.bin`, 8/8 vectors identical to lever drive.
//! - `face=wall` (attachment block BESIDE): a repeater in the lever's cell
//!   pointing INTO the attachment block reproduces the lever's *strong* power
//!   exactly, and the connection dust goes one cell further out. Verified:
//!   `ADD007.a` 8/8 and `NUMDISPLAY001.bcd` 10/10 identical to lever drive.
//!   (Plain dust in the lever cell is NOT enough here — `ADD007.a` feeds bare
//!   dust and reads 0 forever, which is exactly the weak-power rule.)
//!
//! Outputs are easier: a lamp is already strongly powered by whatever drives
//! it, so dust placed on top of the lamp reads the signal without touching the
//! lamp at all — the port stays executor-READABLE and becomes routable.
//!
//! ## Why not use a repeater for EVERY face?
//!
//! Tempting, because a repeater takes its input from BEHIND, horizontally, so a
//! bus could approach in the port's own row orientation and the pivot below
//! would be unnecessary. It does not work for a FLOOR lever, and the reason is
//! geometric rather than a tuning matter: a repeater emits from its front FACE,
//! horizontally. It never powers the block BENEATH it, and a floor lever's
//! attachment block is precisely the block beneath it.
//!
//! Measured in the tick engine on 2026-08-09 (attachment block below the
//! driver, consumer = a repeater reading that block from behind):
//!
//! | driver in the lever's cell | consumer sees |
//! |----------------------------|---------------|
//! | floor LEVER (reference)    | 15 — DRIVES   |
//! | REPEATER                   | 0  — DEAD     |
//! | DUST (what we do)          | 15 — DRIVES   |
//!
//! So the per-face split above is not an accident of history; it is the only
//! assignment that works. The repeater strategy stays where the attachment
//! block is BESIDE the driver (`face=wall`), which is exactly where it is used.
//!
//! # Form: the PIVOT
//!
//! Promotion is only half the job. A bus realizes the verified vertical
//! 2y-pitch stack, and community IO is often a horizontal ROW (`BINTOBCD001`'s
//! `bin` levers march along x at pitch 2). Such a port is dust, routable in
//! principle, and still unusable — its step is `(2,0,0)`, not `(0,2,0)`.
//!
//! The pivot is a WORKAROUND for a realizer limitation, not a design goal, and
//! it is the reason promoted buses look more complicated than they should. The
//! form is not "forced vertical" by promotion — it is forced by
//! `design.rs::realize`, which hard-rejects any step other than `(0,2,0)`, and
//! by everything keyed to that stack: the corridor fabric's column test
//! (`y0-1 ..= y0+2*(width-1)`), the refresh-repeater stations, the crossing
//! rules and the DRC. Inferring the form from port geometry, as `DESIGN_SPEC`
//! describes, therefore means a SECOND realizer for the horizontal form rather
//! than relaxing a guard; until that exists the pivot is what makes a row port
//! routable at all.
//!
//! [`pivot_row_to_stack`] therefore grows a *form adapter*: bit `i` leaves the
//! row in its own private lane, climbs `2i` blocks on a dust staircase, runs
//! out to a common depth, then gathers back along the row axis so all bits
//! land in one vertical 2y-pitch column. Lanes are 2 apart on the row axis, so
//! no two bits are ever plan-adjacent; the gather column is a textbook bus
//! stack (dust, support, dust, ...). Refresh repeaters are inserted every 6
//! dust cells, with a flat landing around each one — dust cannot climb out of
//! a repeater, so the staircase pauses, repeats, and resumes.

use crate::routing::engine::blocks as rblocks;
use crate::UniversalSchematic;

/// Position triple (mirrors [`crate::design::P3`]).
pub type P3 = (i32, i32, i32);

/// Dust cells between refresh repeaters inside a promotion stub. Dust reaches
/// 15 cells; 6 leaves room for the corner cells a pivot leg adds.
const REFRESH_AT: usize = 6;

/// The block a promotion stub uses for supports.
const SUPPORT: &str = rblocks::STONE;

fn add(a: P3, b: P3) -> P3 {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

fn mul(a: P3, k: i32) -> P3 {
    (a.0 * k, a.1 * k, a.2 * k)
}

/// A reversible patch over a cell's blocks, in CELL-LOCAL coordinates.
///
/// `writes` is what Bus mode lays down (`None` = clear the cell); `saved` is
/// what was there before, so restoring is a byte-exact undo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PortPatch {
    /// Cells Bus mode writes: `Some(state)` places, `None` clears.
    pub writes: std::collections::BTreeMap<P3, Option<String>>,
    /// What was at every touched cell before: `None` = it was empty.
    pub saved: std::collections::BTreeMap<P3, Option<String>>,
    /// Bus-mode connection cells, bit order (the contract's Bus positions).
    pub wires: Vec<P3>,
    /// Executor-mode positions (the contract's shipped hardware).
    pub hardware: Vec<P3>,
    /// Step between consecutive `wires`.
    pub step: P3,
    /// Whether a form pivot was needed, for the report.
    pub pivoted: bool,
    /// One-sentence human summary for a UI toast.
    pub note: String,
}

impl PortPatch {
    /// `{"wires":[[x,y,z],..],"hardware":[..],"step":[x,y,z],"removed":n,
    ///   "added":n,"pivoted":bool,"note":".."}`
    pub fn to_json(&self) -> String {
        let pos = |ps: &[P3]| {
            let v: Vec<String> = ps.iter().map(|p| format!("[{},{},{}]", p.0, p.1, p.2)).collect();
            format!("[{}]", v.join(","))
        };
        let removed = self.writes.values().filter(|v| v.is_none()).count()
            + self.saved.iter().filter(|(p, v)| v.is_some() && self.writes.get(*p).is_some_and(|w| w.is_some())).count();
        format!(
            "{{\"wires\":{},\"hardware\":{},\"step\":[{},{},{}],\"removed\":{},\"added\":{},\
             \"pivoted\":{},\"note\":{:?}}}",
            pos(&self.wires),
            pos(&self.hardware),
            self.step.0,
            self.step.1,
            self.step.2,
            removed,
            self.writes.values().filter(|v| v.is_some()).count(),
            self.pivoted,
            self.note,
        )
    }
}

/// A cheap read-only view of the cell being patched.
struct Body<'a> {
    sch: &'a UniversalSchematic,
}

impl Body<'_> {
    fn at(&self, p: P3) -> Option<String> {
        self.sch
            .get_block(p.0, p.1, p.2)
            .map(|b| b.to_string())
            .filter(|s| !s.contains("minecraft:air"))
    }

    fn free(&self, p: P3) -> bool {
        self.at(p).is_none()
    }
}

/// Builder that records writes together with what they displaced.
struct Patcher<'a> {
    body: Body<'a>,
    patch: PortPatch,
}

impl<'a> Patcher<'a> {
    fn new(sch: &'a UniversalSchematic) -> Self {
        Patcher {
            body: Body { sch },
            patch: PortPatch::default(),
        }
    }

    /// Is `p` free once this patch's own writes are taken into account?
    fn free(&self, p: P3) -> bool {
        match self.patch.writes.get(&p) {
            Some(Some(_)) => false,
            Some(None) => true,
            None => self.body.free(p),
        }
    }

    fn write(&mut self, p: P3, block: Option<&str>) {
        self.patch
            .saved
            .entry(p)
            .or_insert_with(|| self.body.at(p));
        self.patch.writes.insert(p, block.map(|s| s.to_string()));
    }

    /// Place `block` at `p`, refusing to overwrite anything that is not air.
    fn place(&mut self, p: P3, block: &str, what: &str) -> Result<(), String> {
        if !self.free(p) {
            return Err(format!(
                "promotion needs {:?} for its {what}, but the cell has `{}` there",
                p,
                self.body.at(p).unwrap_or_else(|| "?".into())
            ));
        }
        self.write(p, Some(block));
        Ok(())
    }

    /// Make sure `p` can hold dust/a repeater by giving it a support block.
    fn support(&mut self, p: P3) -> Result<(), String> {
        let below = add(p, (0, -1, 0));
        match self.patch.writes.get(&below) {
            Some(Some(b)) if rblocks::is_sturdy_support(b) => Ok(()),
            Some(Some(b)) => Err(format!(
                "promotion wants a support under {below:?} but its own stub put `{b}` there"
            )),
            _ => match self.body.at(below) {
                Some(b) if rblocks::is_sturdy_support(&b) => Ok(()),
                Some(b) => Err(format!(
                    "promotion wants a support under {below:?}; the cell has non-sturdy `{b}` there"
                )),
                None => {
                    self.write(below, Some(SUPPORT));
                    Ok(())
                }
            },
        }
    }

    fn dust(&mut self, p: P3, what: &str) -> Result<(), String> {
        self.place(p, rblocks::DUST, what)?;
        self.support(p)
    }
}

/// The attachment block of a lever/button, and the direction it faces AWAY
/// from that block.
fn attachment(block: &str, at: P3) -> Result<(P3, P3), String> {
    let face = block
        .find("face=")
        .map(|i| {
            let rest = &block[i + 5..];
            let end = rest.find([',', ']']).unwrap_or(rest.len());
            &rest[..end]
        })
        .ok_or_else(|| format!("`{block}` at {at:?} has no `face=` property"))?;
    match face {
        // Floor: mounted on the block below, pointing up.
        "floor" => Ok((add(at, (0, -1, 0)), (0, 1, 0))),
        // Ceiling: mounted on the block above, pointing down.
        "ceiling" => Ok((add(at, (0, 1, 0)), (0, -1, 0))),
        "wall" => {
            let f = rblocks::facing_of(block)
                .and_then(rblocks::facing_vec)
                .ok_or_else(|| format!("wall lever `{block}` at {at:?} has no usable facing"))?;
            // A wall lever points AWAY from the block it hangs on.
            Ok((add(at, mul(f, -1)), f))
        }
        other => Err(format!("`{block}` at {at:?}: unsupported face `{other}`")),
    }
}

/// Plan Bus mode for an INPUT port whose `hardware` cells hold levers or
/// buttons. See the module docs for why the strategy depends on `face`.
pub fn plan_input(sch: &UniversalSchematic, hardware: &[P3]) -> Result<PortPatch, String> {
    if hardware.is_empty() {
        return Err("port declares no positions".to_string());
    }
    let mut p = Patcher::new(sch);
    let mut wires = Vec::new();
    let mut how = "";
    for (k, hp) in hardware.iter().enumerate() {
        let block = p
            .body
            .at(*hp)
            .ok_or_else(|| format!("bit {k}: nothing at {hp:?} to promote"))?;
        if rblocks::is_dust(&block) {
            // Already a dust port: nothing to do for this bit.
            wires.push(*hp);
            how = "already dust";
            continue;
        }
        if !(rblocks::is_lever(&block) || block.contains("button")) {
            return Err(format!(
                "bit {k}: {hp:?} holds `{block}`, which is not a lever or button — promotion \
                 replaces executor hardware, so declare the port over the real hardware first"
            ));
        }
        let (att, out) = attachment(&block, *hp)?;
        if out.1 != 0 {
            // Floor (or ceiling) lever: the attachment block is directly
            // below/above, and dust in the lever's own cell powers it.
            if out.1 < 0 {
                return Err(format!(
                    "bit {k}: {hp:?} is a CEILING lever (attachment block above at {att:?}); \
                     nothing may sit above a block to power it, so this port cannot be promoted \
                     — rebuild the cell with a floor or wall lever"
                ));
            }
            p.write(*hp, Some(rblocks::DUST));
            p.support(*hp)?;
            wires.push(*hp);
            how = "dust on the lever's own support (weak power into the attachment block)";
        } else {
            // Wall lever: a repeater pointing into the attachment block gives
            // the same STRONG power the lever gave.
            let facing = rblocks::facing_of(&block)
                .ok_or_else(|| format!("bit {k}: wall lever `{block}` has no facing"))?
                .to_string();
            p.write(*hp, Some(&rblocks::repeater(&facing, 1)));
            p.support(*hp)?;
            let wire = add(*hp, out);
            p.dust(wire, "connection cell")?;
            wires.push(wire);
            how = "repeater into the attachment block, connection dust one cell out";
        }
    }
    finish(p, hardware, wires, how, sch, false)
}

/// Plan Bus mode for an OUTPUT port whose `hardware` cells hold lamps: a dust
/// tap on top of each lamp. The lamp is untouched, so the port stays readable
/// through the typed executor as well as routable.
pub fn plan_output(sch: &UniversalSchematic, hardware: &[P3]) -> Result<PortPatch, String> {
    if hardware.is_empty() {
        return Err("port declares no positions".to_string());
    }
    let mut p = Patcher::new(sch);
    let mut wires = Vec::new();
    for (k, hp) in hardware.iter().enumerate() {
        let block = p
            .body
            .at(*hp)
            .ok_or_else(|| format!("bit {k}: nothing at {hp:?} to tap"))?;
        if rblocks::is_dust(&block) {
            wires.push(*hp);
            continue;
        }
        if !rblocks::is_sturdy_support(&block) {
            return Err(format!(
                "bit {k}: {hp:?} holds `{block}`, which cannot support a dust tap — an output \
                 port is promoted by putting dust on the lamp that already carries the signal"
            ));
        }
        let tap = add(*hp, (0, 1, 0));
        p.place(tap, rblocks::DUST, "output tap")?;
        wires.push(tap);
    }
    finish(
        p,
        hardware,
        wires,
        "dust tap on top of the output lamp (the lamp keeps working)",
        sch,
        true,
    )
}

/// Uniform step of `wires`, or an error naming why there is none.
fn uniform_step(wires: &[P3]) -> Result<P3, String> {
    if wires.len() == 1 {
        return Ok((0, 2, 0));
    }
    let s = (
        wires[1].0 - wires[0].0,
        wires[1].1 - wires[0].1,
        wires[1].2 - wires[0].2,
    );
    if wires
        .windows(2)
        .all(|w| (w[1].0 - w[0].0, w[1].1 - w[0].1, w[1].2 - w[0].2) == s)
    {
        Ok(s)
    } else {
        Err(format!("connection cells {wires:?} do not lie on a uniform step"))
    }
}

/// Close a patch. Promotion is MINIMAL and IN-PLACE: the port keeps its NATIVE
/// geometry (a horizontal row of 8 stays a horizontal row of 8 at its native
/// pitch) and the patch never reaches outside the cell's own footprint.
///
/// FORM ADAPTATION IS THE BUS'S JOB. Promotion used to grow the row->stack
/// pivot here, which put a staircase and a gather column — geometry that
/// exists only to serve one bus, and that extends well beyond the cell — into
/// a PER-INSTANCE patch. Ripping the bus then left it behind as orphaned
/// geometry the user could not remove. The adapter now belongs to the bus
/// fragment (see `Design::realize`), so it is created and ripped with the bus,
/// and the component stays untouched in its native form.
fn finish(
    mut p: Patcher<'_>,
    hardware: &[P3],
    wires: Vec<P3>,
    how: &str,
    _sch: &UniversalSchematic,
    _flow_out: bool,
) -> Result<PortPatch, String> {
    let step = uniform_step(&wires)?;
    let n = wires.len();
    if step.1 != 0 && step != (0, 2, 0) {
        return Err(format!(
            "promoted connection cells step {step:?}: a bus can adapt a horizontal ROW or the \
             canonical vertical 2y-pitch stack onto its form. This port's hardware is neither."
        ));
    }
    // DRAW THE WIRE. The dust above was authored in the default state, which
    // interns correctly (a bare `redstone_wire` sits inert) but is
    // geometrically a DOT. Derive the connection states from the neighbours the
    // way Minecraft does on placement, reading the cell body for anything the
    // patch does not itself write.
    let body_at = |q: P3| -> Option<String> { p.body.at(q) };
    let mut writes: std::collections::BTreeMap<P3, String> = p
        .patch
        .writes
        .iter()
        .filter_map(|(q, v)| v.clone().map(|b| (*q, b)))
        .collect();
    crate::routing::engine::wire::rewire(&mut writes, &body_at);
    for (q, b) in writes {
        p.patch.writes.insert(q, Some(b));
    }
    p.patch.step = if n == 1 { (0, 2, 0) } else { step };
    p.patch.wires = wires;
    p.patch.hardware = hardware.to_vec();
    p.patch.note = if p.patch.step == (0, 2, 0) {
        format!("{n} bit(s): {how}")
    } else {
        format!(
            "{n} bit(s): {how}; the port keeps its native {:?}-pitch form — the bus grows the \
             form adapter it needs, and rips it with itself",
            p.patch.step
        )
    };
    Ok(p.patch)
}

/// A form-adapter PLAN: the cells to place and the vertical 2y-pitch column
/// they gather the row into (bit order).
#[derive(Clone, Debug, Default)]
pub struct PivotPlan {
    /// Cells the adapter needs, block per position (caller coordinates).
    pub cells: std::collections::BTreeMap<P3, String>,
    /// The resulting 2y-pitch column, bit order — where a bus lands.
    pub column: Vec<P3>,
    /// One-sentence human summary.
    pub note: String,
}

/// A write target for [`plan_pivot`]: collects cells and answers "what is
/// there?" through the caller's own view of the world, so the same verified
/// geometry can be planned against a cell body or against a whole design's
/// occupancy index.
struct PivotSink<'f> {
    at: &'f dyn Fn(P3) -> Option<String>,
    cells: std::collections::BTreeMap<P3, String>,
}

impl PivotSink<'_> {
    fn look(&self, p: P3) -> Option<String> {
        self.cells.get(&p).cloned().or_else(|| (self.at)(p))
    }

    fn free(&self, p: P3) -> bool {
        self.look(p).is_none()
    }

    fn write(&mut self, p: P3, block: Option<&str>) {
        match block {
            Some(b) => {
                self.cells.insert(p, b.to_string());
            }
            None => {
                self.cells.remove(&p);
            }
        }
    }

    fn place(&mut self, p: P3, block: &str, what: &str) -> Result<(), String> {
        if let Some(b) = self.look(p) {
            return Err(format!(
                "the form adapter needs {p:?} for its {what}, but `{b}` is there"
            ));
        }
        self.write(p, Some(block));
        Ok(())
    }

    fn support(&mut self, p: P3) -> Result<(), String> {
        let below = add(p, (0, -1, 0));
        match self.look(below) {
            Some(b) if rblocks::is_sturdy_support(&b) => Ok(()),
            Some(b) => Err(format!(
                "the form adapter wants a support under {below:?}; `{b}` is not sturdy"
            )),
            None => {
                self.write(below, Some(SUPPORT));
                Ok(())
            }
        }
    }

    fn dust(&mut self, p: P3, what: &str) -> Result<(), String> {
        self.place(p, rblocks::DUST, what)?;
        self.support(p)
    }
}

/// Plan a form adapter turning a horizontal ROW of connection cells into a
/// vertical 2y-pitch COLUMN. PURE: nothing is written, the caller owns the
/// cells — for a bus that means they live in the bus's fragment and are ripped
/// with it.
///
/// Shape: bit `i` leaves the row in its own private lane, climbs `2i` blocks on
/// a dust staircase, runs out to a common depth, then gathers back along the
/// row axis so all bits land in one vertical 2y-pitch column. Lanes are `pitch`
/// apart on the row axis, so no two bits are ever plan-adjacent; the gather
/// column is a textbook bus stack. Refresh repeaters go in every
/// [`REFRESH_AT`] dust cells with a flat landing around each — dust cannot
/// climb out of a repeater, so the staircase pauses, repeats and resumes.
///
/// Both perpendicular directions are tried; the one whose whole volume is free
/// wins, and if both are, the one pointing AWAY from `prefer_away_from` (the
/// cell's centre) is preferred, so the adapter grows out of the component.
pub fn plan_pivot(
    wires: &[P3],
    step: P3,
    prefer_away_from: P3,
    flow_out: bool,
    at: &dyn Fn(P3) -> Option<String>,
    toward: Option<P3>,
) -> Result<PivotPlan, String> {
    if wires.is_empty() {
        return Err("form adapter needs at least one connection cell".to_string());
    }
    if step.1 != 0 {
        return Err(format!(
            "form adapter turns a HORIZONTAL row into the vertical stack; this port's step is \
             {step:?}"
        ));
    }
    let along = if step.0 != 0 { (1, 0, 0) } else { (0, 0, 1) };
    let mut cands: Vec<P3> = if step.0 != 0 {
        vec![(0, 0, -1), (0, 0, 1)]
    } else {
        vec![(-1, 0, 0), (1, 0, 0)]
    };
    // COST-DRIVEN, not structural. This used to take the first direction that
    // merely FIT, ordered by "grows away from the cell body" — which is how the
    // adapter ended up building its gather bar on the far side of the port and
    // making the bus double back to reach it. Plan BOTH sides and keep the one
    // that costs less, breaking ties toward the outside so the old behaviour is
    // the tie-break rather than the rule.
    //
    // Cost is the realized CELL COUNT plus how far the column ends up from the
    // partner the bus has to reach (`toward`, when the caller knows it): a
    // cheap adapter that lands the stack head on the wrong side of the
    // component just moves the cost into the transport leg.
    let outward = |d: &P3| {
        -((d.0 * (prefer_away_from.0 - wires[0].0)) + (d.2 * (prefer_away_from.2 - wires[0].2)))
    };
    cands.sort_by_key(|d| -outward(d));
    let mut errs = Vec::new();
    let mut best: Option<(i64, PivotPlan)> = None;
    for out in cands {
        let mut sink = PivotSink {
            at,
            cells: std::collections::BTreeMap::new(),
        };
        match lay_pivot(&mut sink, wires, step, along, out, flow_out) {
            Ok(column) => {
                let cells = sink.cells.len() as i64;
                // Distance from the column head to whatever the bus must reach.
                let reach = toward.map_or(0, |t| {
                    ((column[0].0 - t.0).abs() + (column[0].2 - t.2).abs()) as i64
                });
                // Tie-break toward the outside (the old, structural preference).
                let cost = cells + reach - i64::from(outward(&out) > 0);
                let plan = PivotPlan {
                    cells: sink.cells,
                    column,
                    note: format!(
                        "form adapter: pivoted the {:?}-pitch row onto a vertical 2y stack via a \
                         staircase growing {} block(s) toward {:?} ({cells} cells; the cheaper of \
                         the two sides)",
                        step,
                        2 * wires.len(),
                        out
                    ),
                };
                if best.as_ref().is_none_or(|(c, _)| cost < *c) {
                    best = Some((cost, plan));
                }
            }
            Err(e) => errs.push(format!("toward {out:?}: {e}")),
        }
    }
    if let Some((_, plan)) = best {
        return Ok(plan);
    }
    Err(format!(
        "the row needs a form adapter to reach the vertical 2y-pitch bus stack, but neither side \
         of the port face has room for one ({})",
        errs.join("; ")
    ))
}

/// One attempt at the adapter, in the direction `out`.
#[allow(clippy::too_many_arguments)]
fn lay_pivot(
    p: &mut PivotSink<'_>,
    wires: &[P3],
    step: P3,
    along: P3,
    out: P3,
    flow_out: bool,
) -> Result<Vec<P3>, String> {
    let n = wires.len() as i32;
    let pitch = step.0.abs().max(step.2.abs()); // 2 for a 2-pitch row
    if pitch < 2 {
        return Err(format!("row pitch {pitch} leaves no lane between bits"));
    }
    // The direction from bit i's lane back toward bit 0's lane.
    let back = mul(along, -(step.0 + step.2).signum());
    // Depth every bit runs out to, from the deepest climb any bit needs.
    let depth = 2 + 2 * (n - 1) + 2 * refresh_pauses(2 * (n - 1) as usize) as i32;
    let mut column = Vec::new();
    for i in 0..n {
        let w = wires[i as usize];
        let mut y = w.1;
        let mut t = 1i32;
        let mut since = 0usize;
        // A repeater right at the mouth, so every bit starts at full strength
        // whatever the bus delivered.
        let sgn = if flow_out { -1 } else { 1 };
        let rep_in = rblocks::facing_name(sgn * out.0, sgn * out.2)
            .ok_or("pivot direction is not axis-aligned")?;
        p.place(add(w, out), &rblocks::repeater(rep_in, 1), "stub repeater")?;
        p.support(add(w, out))?;
        t += 1;
        // Dust must follow a repeater before the staircase may climb.
        p.dust(add(w, mul(out, t)), "stub run")?;
        since = 1;
        t += 1;
        let mut climb = 2 * i;
        while climb > 0 {
            if since >= REFRESH_AT {
                let c = add(add(w, mul(out, t)), (0, y - w.1, 0));
                p.place(c, &rblocks::repeater(rep_in, 1), "refresh repeater")?;
                p.support(c)?;
                t += 1;
                let d = add(add(w, mul(out, t)), (0, y - w.1, 0));
                p.dust(d, "landing")?;
                since = 1;
                t += 1;
                continue;
            }
            // The climb step needs its own headroom.
            let here = add(add(w, mul(out, t - 1)), (0, y - w.1, 0));
            if !p.free(add(here, (0, 1, 0))) {
                return Err(format!(
                    "the staircase for bit {i} needs {:?} clear to climb",
                    add(here, (0, 1, 0))
                ));
            }
            y += 1;
            p.dust(add(add(w, mul(out, t)), (0, y - w.1, 0)), "staircase")?;
            since += 1;
            climb -= 1;
            t += 1;
        }
        // Level out to the shared depth.
        while t <= depth {
            let c = add(add(w, mul(out, t)), (0, y - w.1, 0));
            if since >= REFRESH_AT && t < depth {
                p.place(c, &rblocks::repeater(rep_in, 1), "refresh repeater")?;
                p.support(c)?;
                since = 0;
            } else {
                p.dust(c, "stub run")?;
                since += 1;
            }
            t += 1;
        }
        // Gather back along the row axis to bit 0's lane. `t` now points one
        // past the last cell written, which MUST be the shared depth — every
        // lane has to reach the same plane for the gather to exist.
        debug_assert_eq!(
            t - 1,
            depth,
            "bit {i}'s lane ended at depth {} but the gather plane is at {depth}",
            t - 1
        );
        if t - 1 != depth {
            return Err(format!(
                "internal: bit {i}'s staircase ended at depth {}, not the shared {depth}",
                t - 1
            ));
        }
        let corner = add(add(w, mul(out, depth)), (0, y - w.1, 0));
        // The gather leg carries signal from the corner toward the column on
        // an output, and the other way on an input.
        let gsgn = if flow_out { -1 } else { 1 };
        let gather_in = rblocks::facing_name(gsgn * back.0, gsgn * back.2).ok_or("gather axis")?;
        let hops = pitch * i;
        let mut since_g = 0usize;
        for k in 1..=hops {
            let c = add(corner, mul(back, k));
            if k == 1 || since_g >= REFRESH_AT {
                p.place(c, &rblocks::repeater(gather_in, 1), "gather repeater")?;
                p.support(c)?;
                since_g = 0;
            } else {
                p.dust(c, "gather run")?;
                since_g += 1;
            }
        }
        let last = add(corner, mul(back, hops));
        // The column cell itself must be plain dust for a bus to land on.
        if hops > 0 {
            p.write(last, Some(rblocks::DUST));
            p.support(last)?;
        }
        column.push(last);
    }
    Ok(column)
}

/// How many refresh pauses a climb of `h` steps needs (each costs 2 extra
/// cells of depth: the repeater plus its landing).
///
/// The staircase enters the climb with one dust cell already spent (the cell
/// that must follow the mouth repeater), so a pause lands after every
/// `REFRESH_AT - 1` climb steps — and never after the last one, because the
/// loop exits first. Getting this off by one silently truncates the deepest
/// bit's lane: its corner is then computed short of where its dust actually
/// ends, the gather starts in mid-air, and exactly the TOP bit of the port
/// goes dead. The `debug_assert` in `lay_pivot` is what catches that.
fn refresh_pauses(h: usize) -> usize {
    h.saturating_sub(1) / (REFRESH_AT - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slab(sx: i32, sy: i32, sz: i32) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("t".to_string());
        for x in 0..sx {
            for y in 0..sy {
                for z in 0..sz {
                    s.set_block_from_string(x, y, z, rblocks::STONE).unwrap();
                }
            }
        }
        s
    }

    #[test]
    fn a_wall_lever_is_promoted_through_a_repeater() {
        let mut s = slab(4, 8, 1);
        // Lever on the -x face of the slab, pointing west.
        s.set_block_from_string(-1, 3, 0, "minecraft:lever[face=wall,facing=west,powered=false]")
            .unwrap();
        let patch = plan_input(&s, &[(-1, 3, 0)]).unwrap();
        assert_eq!(patch.wires, vec![(-2, 3, 0)]);
        assert!(patch.writes[&(-1, 3, 0)].as_deref().unwrap().contains("repeater"));
        assert!(rblocks::is_dust(patch.writes[&(-2, 3, 0)].as_deref().unwrap()));
        // The saved state restores the lever byte-for-byte.
        assert_eq!(
            patch.saved[&(-1, 3, 0)].as_deref(),
            Some("minecraft:lever[face=wall,facing=west,powered=false]")
        );
    }

    #[test]
    fn a_floor_lever_is_promoted_in_place() {
        let mut s = slab(4, 4, 1);
        s.set_block_from_string(1, 4, 0, "minecraft:lever[face=floor,facing=north,powered=false]")
            .unwrap();
        let patch = plan_input(&s, &[(1, 4, 0)]).unwrap();
        assert_eq!(patch.wires, vec![(1, 4, 0)]);
        assert!(rblocks::is_dust(patch.writes[&(1, 4, 0)].as_deref().unwrap()));
    }

    #[test]
    fn a_horizontal_row_keeps_its_form_and_the_bus_adapts_it() {
        // Four floor levers marching along x at pitch 2 on top of a slab.
        let mut s = slab(8, 4, 4);
        for i in 0..4 {
            s.set_block_from_string(
                2 * i,
                4,
                0,
                "minecraft:lever[face=floor,facing=north,powered=false]",
            )
            .unwrap();
        }
        let hw: Vec<P3> = (0..4).map(|i| (2 * i, 4, 0)).collect();
        let patch = plan_input(&s, &hw).unwrap();

        // PROMOTION IS MINIMAL AND IN-PLACE: the row stays a row at its native
        // pitch, one write per bit, no form-conversion geometry anywhere.
        assert!(!patch.pivoted, "{}", patch.note);
        assert_eq!(patch.step, (2, 0, 0), "{}", patch.note);
        assert_eq!(patch.wires, hw, "the connection cells left the lever row");
        assert_eq!(
            patch.writes.len(),
            4,
            "promotion wrote {} cells for 4 bits: {:?}",
            patch.writes.len(),
            patch.writes
        );
        for w in &hw {
            assert!(
                patch.writes.get(w).and_then(|o| o.as_deref()).is_some_and(rblocks::is_dust),
                "bit at {w:?} is not dust in place"
            );
        }

        // FORM ADAPTATION IS THE BUS'S: the same verified pivot, planned as a
        // pure set of cells the caller (a bus) owns and rips with itself.
        let at = |q: P3| -> Option<String> {
            s.get_block(q.0, q.1, q.2)
                .map(|b| b.to_string())
                .filter(|b| !b.contains("minecraft:air"))
                .filter(|_| !hw.contains(&q))
        };
        let plan = plan_pivot(&patch.wires, patch.step, (4, 4, 0), false, &at)
            .expect("the row must be adaptable onto the stack");
        // A vertical 2y-pitch column, bit order preserved.
        for (k, w) in plan.column.iter().enumerate() {
            assert_eq!(w.0, plan.column[0].0, "bit {k} left the column");
            assert_eq!(w.2, plan.column[0].2, "bit {k} left the column");
            assert_eq!(w.1, plan.column[0].1 + 2 * k as i32, "bit {k} off pitch");
        }
        // Every dust cell the adapter plans has a support beneath it.
        for (q, b) in &plan.cells {
            if rblocks::is_dust(b) {
                let below = add(*q, (0, -1, 0));
                let has = plan
                    .cells
                    .get(&below)
                    .map(|b| rblocks::is_sturdy_support(b))
                    .unwrap_or(false)
                    || s.get_block(below.0, below.1, below.2)
                        .map(|b| rblocks::is_sturdy_support(&b.to_string()))
                        .unwrap_or(false);
                assert!(has, "adapter dust at {q:?} floats");
            }
        }
    }

    #[test]
    fn a_ceiling_lever_is_refused_with_a_reason() {
        let mut s = slab(4, 8, 1);
        s.set_block_from_string(1, 2, 0, "minecraft:lever[face=ceiling,facing=north,powered=false]")
            .unwrap();
        let e = plan_input(&s, &[(1, 2, 0)]).unwrap_err();
        assert!(e.contains("CEILING"), "{e}");
    }

    #[test]
    fn a_lamp_output_gets_a_dust_tap() {
        let mut s = slab(4, 4, 1);
        s.set_block_from_string(1, 4, 0, rblocks::LAMP).unwrap();
        let patch = plan_output(&s, &[(1, 4, 0)]).unwrap();
        assert_eq!(patch.wires, vec![(1, 5, 0)]);
        // The lamp itself is untouched.
        assert!(!patch.writes.contains_key(&(1, 4, 0)));
    }
}
