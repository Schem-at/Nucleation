//! The PLA geometry compiler: stages of AND/OR columns over lidded rails.
//!
//! Line-for-line port of `redstone-eda/build_ppa.py` at its current state
//! (station-pitch rail/route repeater rules included). The Python is the
//! spec: every constant and rule below was paid for by a real in-sim bug.
//!
//! One verified primitive, repeated: a lidded rail, a dust spur that
//! dead-ends into a block, a torch on top injecting NOT(rail) into a
//! collector, and a torch on the collector's dead-end inverting the OR —
//! so one column computes AND(its rails).
//!
//! Layout: X is the bit slice, Z is the stage band, Y is detail (rails
//! y=0..2, collector y=3..4, gate y=5..6, output lane y=7).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::HdlError;

// ---- slice geometry (offsets from a slice's x0) ---------------------------

/// Slice pitch along X.
pub const SLICE_W: i32 = 17;
/// Rail west end / wiring channel, one per outgoing route.
pub const CH: [i32; 3] = [0, 2, 4];
/// Inverter spur; stairs at 7,8,9; its output rail starts at 9.
pub const INV_X: i32 = 6;
/// Where an inverter's output rail starts.
pub const INV_RAIL_X: i32 = 9;
/// Group A columns (OR-ed onto one lane).
pub const COL_A: [i32; 2] = [11, 13];
/// Group B column.
pub const COL_B: [i32; 1] = [15];
/// Further gi=0 columns (the ALU's 4-term mux nodes). Empty for the HDL path.
pub const EXTRA_COLS: [i32; 0] = [];
/// Every rail must reach the last column.
pub const RAIL_E: i32 = 15;
/// Collector dust height.
pub const Y_COLL: i32 = 4;
/// Gate torch height.
pub const Y_GATE: i32 = 5;
/// Output lane height.
pub const Y_LANE: i32 = 7;
/// A rail loses a level per dust cell, so refresh at least this often. This
/// is NOT over-conservative: a rail is tapped from the side, the tap dust
/// reads rail-1, and a ss0 tap dust never flips its torch (probe_station G) —
/// so a rail needs ss>=2 everywhere. Worst gap is RAIL_REPEAT+1 (bad-column
/// skip), min rail ss = 17-(RAIL_REPEAT+1) = 4: the measured floor of 2 plus
/// 2 levels of margin.
pub const RAIL_REPEAT: i32 = 12;
/// Z padding between stage bands.
pub const STAGE_PAD: i32 = 1;
/// Routes are tap-free, so the pitch is bounded only by the repeater's own
/// input floor: ss1 fires it (probe_station I15), true max 15 dust between
/// refreshes. 14 leaves the last dust at ss3 — margin 2. (Was 6.)
pub const ROUTE_REPEAT: i32 = 14;

/// A slice's west X.
pub fn sx(i: i32) -> i32 {
    i * SLICE_W
}

// ---- block states (from redstone-eda/rs.py, verified) ---------------------

/// Fully spelled-out dust: a bare `minecraft:redstone_wire` interns a
/// property-less state the engine never normalises — always spell it out.
pub const DUST: &str =
    "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";
/// A lit standing torch.
pub const TORCH: &str = "minecraft:redstone_torch[lit=true]";
/// A floor lever, off.
pub const LEVER_OFF: &str = "minecraft:lever[face=floor,facing=north,powered=false]";

/// A repeater whose INPUT side is `input_from`; it outputs the opposite way.
/// Verified: `repeater[facing=west]` conducts toward +X.
pub fn repeater(input_from: &str) -> String {
    format!("minecraft:repeater[facing={input_from},delay=1,locked=false,powered=false]")
}

/// Colour-code structure by role. Concrete conducts exactly like stone, so
/// this is cosmetic — but it makes a 200k-block build readable.
pub fn palette(role: &str) -> &'static str {
    match role {
        "rail_floor" => "minecraft:gray_concrete",
        "lid" => "minecraft:light_blue_concrete",
        "tap" => "minecraft:red_concrete",
        "coll_floor" => "minecraft:lime_concrete",
        "gate" => "minecraft:orange_concrete",
        "lane" => "minecraft:yellow_concrete",
        "route" => "minecraft:magenta_concrete",
        "inv" => "minecraft:purple_concrete",
        _ => "minecraft:stone",
    }
}

// ---- netlist shape --------------------------------------------------------

/// The node groups of one slice: `(output signal, [product term, ...])`.
pub type Groups = Vec<(String, Vec<Vec<String>>)>;

/// One PLA stage, as handed to the compiler (the Python "stage dict").
#[derive(Debug, Clone)]
pub struct Stage {
    /// Stage name (`in`, `lv1`, ...).
    pub name: String,
    /// `(slice, groups)` node placements.
    pub nodes: Vec<(i32, Groups)>,
    /// Slice-local stage: every rail is slice-local, slots repeat per slice.
    pub local: bool,
    /// `(slice, src signal, dst signal)` inverter torches.
    pub inverters: Vec<(i32, String, String)>,
    /// `(slice, signal, channel)` lever-driven rails.
    pub levers: Vec<(i32, String, usize)>,
    /// Kogge-Stone prefix stride, if this is a prefix stage.
    pub stride: Option<i32>,
    /// `(slice, signal, channel)` externally driven input rails.
    pub ext: Vec<(i32, String, usize)>,
    /// Sequential cells (DFF bank) this stage carries — empty for
    /// combinational stages. See [`crate::seq`].
    pub seq: Vec<crate::seq::SeqCell>,
}

/// Per-node placed geometry.
#[derive(Debug, Clone)]
pub struct Geom {
    /// Bit slice.
    pub slice: i32,
    /// Group index within the slice (0 = COL_A side, 1 = COL_B).
    pub gi: usize,
    /// Product terms.
    pub terms: Vec<Vec<String>>,
    /// Z offset of the inverting gate within the stage band.
    pub gate: i32,
    /// Z offset of the output lane (gate + 1).
    pub lane: i32,
}

/// A stage with everything `plan`/`layout` computed attached.
#[derive(Debug, Clone)]
pub struct Planned {
    /// The input stage description.
    pub st: Stage,
    /// signal -> set of consumer slices.
    pub taps: BTreeMap<String, BTreeSet<i32>>,
    /// signal -> rail X span (west, east).
    pub spans: BTreeMap<String, (i32, i32)>,
    /// signal -> X of its drive point.
    pub drive: BTreeMap<String, i32>,
    /// signal -> rail Z slot.
    pub slot: BTreeMap<String, i32>,
    /// output signal -> placed node geometry.
    pub geom: BTreeMap<String, Geom>,
    /// Z base of this stage's band.
    pub z: i32,
}

/// One inter-stage wire.
#[derive(Debug, Clone)]
pub struct Route {
    /// The signal being carried.
    pub sig: String,
    /// Producing stage index.
    pub src: usize,
    /// Consuming stage index.
    pub dst: usize,
    /// Producing slice.
    pub slice: i32,
    /// Channel index into [`CH`].
    pub ch: usize,
    /// X offset of the riser column within the producing slice.
    pub riser: i32,
    /// Z offset (within the source band) of this route's exclusive corridor.
    pub t: i32,
}

/// Where to refresh a rail. The FIRST repeater sits right at the drive point:
/// a route arrives having decayed on the way, so without it the rail starts
/// at whatever trickle survived the journey and dies before the first column.
pub fn rail_repeater_xs(x0: i32, x1: i32, drive: i32) -> BTreeSet<i32> {
    // never on a tap or the inverter; CH[0] stays dust too — a DFF bank's
    // D spur taps its rail there (all bad offsets are isolated, so the
    // worst refresh gap stays RAIL_REPEAT+1 and the rail ss floor holds)
    let bad: Vec<i32> = [INV_X, CH[0]]
        .into_iter()
        .chain(COL_A)
        .chain(COL_B)
        .chain(EXTRA_COLS)
        .collect();
    let mut out = BTreeSet::new();
    let mut want = drive + 1;
    while want <= x1 {
        let mut x = want;
        while x <= x1 && bad.contains(&x.rem_euclid(SLICE_W)) {
            x += 1;
        }
        if x > x1 || x <= x0 {
            break;
        }
        out.insert(x);
        want = x + RAIL_REPEAT;
    }
    out
}

/// Which of a slice's channels this signal uses to reach `dst_stage`.
///
/// Channels are per-slice resources. Next-stage hops recycle channels 0/1
/// (their z-ranges are sequential and disjoint); far hops get an exclusive
/// channel per group index (2 for gi=0, 3 for gi=1), because far corridors
/// span many bands and would collide with everything.
pub fn route_channel(stages: &[Stage], sig: &str, dst_stage: usize) -> usize {
    let mut src: Option<(usize, usize)> = None;
    for (si, st) in stages.iter().enumerate() {
        for (_sl, groups) in &st.nodes {
            for (gi, (out, _t)) in groups.iter().enumerate() {
                if out == sig {
                    src = Some((si, gi));
                }
            }
        }
    }
    match src {
        None => 2,
        Some((si, gi)) => {
            if dst_stage == si + 1 {
                gi
            } else {
                2 + gi
            }
        }
    }
}

/// Left-edge channel assignment: rails whose X spans are 4+ apart share a slot.
fn interval_colour(spans: &BTreeMap<String, (i32, i32)>) -> BTreeMap<String, i32> {
    const GAP: i32 = 4;
    let mut slot: BTreeMap<String, i32> = BTreeMap::new();
    let mut ends: Vec<i32> = Vec::new();
    let mut order: Vec<&String> = spans.keys().collect();
    order.sort_by_key(|s| (spans[*s].0, (*s).clone()));
    for sig in order {
        let (x0, x1) = spans[sig];
        let mut placed = false;
        for (j, end) in ends.iter_mut().enumerate() {
            if x0 > *end + GAP {
                slot.insert(sig.clone(), j as i32);
                *end = x1;
                placed = true;
                break;
            }
        }
        if !placed {
            slot.insert(sig.clone(), ends.len() as i32);
            ends.push(x1);
        }
    }
    slot
}

/// Give every rail a Z slot, letting rails with disjoint X spans share one.
fn assign_slots(
    st: &Stage,
    spans: &BTreeMap<String, (i32, i32)>,
) -> Result<BTreeMap<String, i32>, HdlError> {
    if st.local {
        // stage 0 / sum: every rail is slice-local, so all slices reuse the
        // same slots. Order them X, ~X so an inverter's output rail always
        // sits exactly one slot above its input — what the stair expects.
        let mut slot: BTreeMap<String, i32> = BTreeMap::new();
        let mut per: BTreeMap<i32, Vec<(String, String)>> = BTreeMap::new();
        for (sl, src, dst) in &st.inverters {
            per.entry(*sl).or_default().push((src.clone(), dst.clone()));
        }
        for pairs in per.values() {
            for (j, (src, dst)) in pairs.iter().enumerate() {
                slot.insert(src.clone(), 2 * j as i32);
                slot.insert(dst.clone(), 2 * j as i32 + 1);
            }
        }
        // A "local" stage may still carry rails that are not inverter pairs
        // (an op rail, a lever-driven cin). Interval-colour those into slots
        // above the pinned pairs.
        let leftovers: BTreeMap<String, (i32, i32)> = spans
            .iter()
            .filter(|(s, _)| !slot.contains_key(*s))
            .map(|(s, sp)| (s.clone(), *sp))
            .collect();
        if !leftovers.is_empty() {
            let base = slot.values().max().map_or(0, |m| m + 1);
            for (s, j) in interval_colour(&leftovers) {
                slot.insert(s, base + j);
            }
        }
        return Ok(slot);
    }

    if let Some(s) = st.stride {
        // Assign slot by producer index mod (s+1): a node's two producers
        // land in ADJACENT slot groups, and producers s+1 apart share a
        // track with disjoint X spans by construction.
        let mut slot = BTreeMap::new();
        for sig in spans.keys() {
            let digits: String = sig
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let j: i32 = digits.parse().map_err(|_| {
                HdlError::Layout(format!("stride stage signal {sig} has no trailing index"))
            })?;
            let g = i32::from(!(sig.contains('g') || sig.contains('G')));
            slot.insert(sig.clone(), 2 * (j % (s + 1)) + g);
        }
        return Ok(slot);
    }

    Ok(interval_colour(spans))
}

/// Work out, for every signal: who makes it, who taps it, and its rail slot.
#[allow(clippy::type_complexity)]
pub fn plan(
    stages: Vec<Stage>,
) -> Result<(Vec<Planned>, HashMap<String, (usize, i32, usize)>), HdlError> {
    let mut produced: HashMap<String, (usize, i32, usize)> = HashMap::new();
    for (si, st) in stages.iter().enumerate() {
        for (sl, groups) in &st.nodes {
            for (gi, (out, _terms)) in groups.iter().enumerate() {
                produced.insert(out.clone(), (si, *sl, gi));
            }
        }
    }

    let mut planned: Vec<Planned> = Vec::new();
    for (si, st) in stages.iter().enumerate() {
        let mut taps: BTreeMap<String, BTreeSet<i32>> = BTreeMap::new();
        for (sl, groups) in &st.nodes {
            for (_out, terms) in groups {
                for term in terms {
                    for sig in term {
                        taps.entry(sig.clone()).or_default().insert(*sl);
                    }
                }
            }
        }
        for (sl, src, dst) in &st.inverters {
            taps.entry(src.clone()).or_default().insert(*sl);
            taps.entry(dst.clone()).or_default().insert(*sl);
        }
        // a DFF bank taps its D signals: each non-constant D gets a rail in
        // this band that the standard route machinery drives
        for sc in &st.seq {
            if let Some(d) = &sc.d {
                taps.entry(d.clone()).or_default().insert(sc.slice);
            }
        }

        // rail X span: from its drive point to the last slice that taps it
        let inv_dst: HashMap<&String, i32> =
            st.inverters.iter().map(|(sl, _s, d)| (d, *sl)).collect();
        let inv_src: BTreeSet<&String> = st.inverters.iter().map(|(_sl, s, _d)| s).collect();
        let mut spans: BTreeMap<String, (i32, i32)> = BTreeMap::new();
        let mut drive: BTreeMap<String, i32> = BTreeMap::new();
        for (sig, slices) in &taps {
            let hi = *slices.iter().max().expect("taps never empty");
            let d = if let Some(sl) = inv_dst.get(sig) {
                // made in-stage by a torch
                sx(*sl) + INV_RAIL_X
            } else if produced.get(sig).is_some_and(|(ps, _, _)| *ps < si) {
                let (_, psl, _) = produced[sig];
                let ch = route_channel(&stages, sig, si);
                let ch_x = *CH.get(ch).ok_or_else(|| {
                    HdlError::Layout(format!("signal {sig} needs far channel {ch} (no corridor)"))
                })?;
                sx(psl) + ch_x
            } else {
                // lever-driven, or "ext": an input rail a router will later
                // drive by landing a wire on its west-end dust cell
                let hit = st
                    .levers
                    .iter()
                    .chain(st.ext.iter())
                    .find(|(_sl, s, _c)| s == sig)
                    .ok_or_else(|| {
                        HdlError::Layout(format!("no driver for rail {sig} in stage {}", st.name))
                    })?;
                sx(hit.0) + CH[hit.2]
            };
            drive.insert(sig.clone(), d);
            let mut west = d;
            if inv_src.contains(sig) {
                // must reach the inverter spur
                west = west.min(sx(*slices.iter().min().unwrap()) + INV_X);
            }
            spans.insert(sig.clone(), (west, sx(hi) + RAIL_E));
        }
        let slot = assign_slots(st, &spans)?;
        planned.push(Planned {
            st: st.clone(),
            taps,
            spans,
            drive,
            slot,
            geom: BTreeMap::new(),
            z: 0,
        });
    }
    Ok((planned, produced))
}

/// Attach Z offsets to every node group, then place the stage bands.
#[allow(clippy::type_complexity)]
pub fn layout(
    stages: Vec<Stage>,
) -> Result<(Vec<Planned>, HashMap<String, (usize, i32, usize)>, Vec<Route>), HdlError> {
    let (mut planned, produced) = plan(stages)?;
    for p in &mut planned {
        let mut geom = BTreeMap::new();
        for (sl, groups) in &p.st.nodes {
            for (gi, (out, terms)) in groups.iter().enumerate() {
                let taps: Vec<i32> = terms
                    .iter()
                    .flatten()
                    .map(|s| 3 * p.slot[s] + 2)
                    .collect();
                let gate = taps.iter().max().copied().unwrap_or(0) + 2;
                geom.insert(
                    out.clone(),
                    Geom {
                        slice: *sl,
                        gi,
                        terms: terms.clone(),
                        gate,
                        lane: gate + 1,
                    },
                );
            }
        }
        p.geom = geom;
    }

    let mut routes: Vec<Route> = Vec::new();
    for (si, p) in planned.iter().enumerate() {
        for sig in p.taps.keys() {
            if let Some(&(ps, psl, _)) = produced.get(sig) {
                if ps < si {
                    let sts: Vec<Stage> = planned.iter().map(|q| q.st.clone()).collect();
                    routes.push(Route {
                        sig: sig.clone(),
                        src: ps,
                        dst: si,
                        slice: psl,
                        ch: route_channel(&sts, sig, si),
                        riser: 0,
                        t: 0,
                    });
                }
            }
        }
    }

    // Every route leaving a slice gets its OWN Z corridor at y=7. Corridors
    // are handed out west-to-east by riser column; the riser starts ON the
    // producing node's own lane (a gi=0 lane spans COL_A, a gi=1 lane only
    // COL_B — the riser column follows the group, not just the channel).
    // Same-signal routes to different stages share the same riser x
    // deliberately: the overlap becomes a common trunk and `run` treats
    // re-laying the same dust as a no-op.
    let all_cols: Vec<i32> = COL_A.iter().chain(COL_B.iter()).chain(EXTRA_COLS.iter()).copied().collect();
    let mut per_slice: BTreeMap<(usize, i32), Vec<usize>> = BTreeMap::new();
    for (ri, r) in routes.iter_mut().enumerate() {
        let gi = produced[&r.sig].2;
        let nterms = planned[r.src].geom[&r.sig].terms.len();
        // last actual column of the producing node == the east end of its lane
        r.riser = if gi != 0 { COL_B[0] } else { all_cols[nterms - 1] };
        per_slice.entry((r.src, r.slice)).or_default().push(ri);
    }
    for ((si, sl), ris) in &per_slice {
        let p = &planned[*si];
        let base = p
            .geom
            .values()
            .filter(|g| g.slice == *sl)
            .map(|g| g.lane)
            .max()
            .ok_or_else(|| HdlError::Layout(format!("route from empty slice {sl}")))?
            + 2;
        let mut order: Vec<usize> = ris.clone();
        order.sort_by_key(|&ri| routes[ri].riser); // stable: ties keep tap order
        for (idx, &ri) in order.iter().enumerate() {
            routes[ri].t = base + 4 * idx as i32;
        }
    }

    let mut z = vec![0i32];
    for k in 0..planned.len().saturating_sub(1) {
        let p = &planned[k];
        let ext = p
            .slot
            .values()
            .map(|s| 3 * s)
            .chain(p.geom.values().map(|g| g.lane))
            .chain(routes.iter().filter(|r| r.src == k).map(|r| r.t))
            .max()
            .unwrap_or(0);
        let need = routes
            .iter()
            .filter(|r| r.src == k && r.dst == k + 1)
            .map(|r| r.t + 8 - 3 * planned[r.dst].slot[&r.sig])
            .max()
            .unwrap_or(0);
        z.push(z[k] + ext.max(need) + STAGE_PAD);
    }
    for (p, zb) in planned.iter_mut().zip(&z) {
        p.z = *zb;
    }
    for r in &routes {
        // the plan must be physical
        let wz = planned[r.src].z + r.t;
        let dz = planned[r.dst].z + 3 * planned[r.dst].slot[&r.sig];
        if dz < wz + 8 {
            return Err(HdlError::Layout(format!(
                "no room to route {}: {wz} -> {dz}",
                r.sig
            )));
        }
    }
    Ok((planned, produced, routes))
}

// ---- geometry emission ----------------------------------------------------

/// The authored voxels: block-state string per cell, plus net labels.
#[derive(Debug, Clone, Default)]
pub struct Build {
    /// Cell -> block-state descriptor.
    pub cells: BTreeMap<(i32, i32, i32), String>,
    /// Cell -> net label (dust only).
    pub labels: HashMap<(i32, i32, i32), String>,
}

impl Build {
    fn put(&mut self, x: i32, y: i32, z: i32, block: &str) -> Result<(), HdlError> {
        if let Some(prev) = self.cells.get(&(x, y, z)) {
            if prev != block {
                return Err(HdlError::Layout(format!(
                    "cell collision at ({x},{y},{z}): {prev} vs {block}"
                )));
            }
        }
        self.cells.insert((x, y, z), block.to_string());
        Ok(())
    }

    /// Deliberately overwrite whatever is already in this cell.
    pub fn force(&mut self, x: i32, y: i32, z: i32, block: &str) {
        self.cells.insert((x, y, z), block.to_string());
    }

    fn stone(&mut self, x: i32, y: i32, z: i32, role: &str) -> Result<(), HdlError> {
        // Structural blocks legitimately coincide (a route's support IS the
        // rail floor beneath it). First role to claim a cell keeps its colour.
        if self.solid_at(x, y, z) {
            return Ok(());
        }
        self.put(x, y, z, palette(role))
    }

    fn solid_at(&self, x: i32, y: i32, z: i32) -> bool {
        self.cells.get(&(x, y, z)).is_some_and(|b| {
            b.contains("concrete") || b == "minecraft:stone" || b.contains("lamp")
        })
    }

    /// `(min, max)` corners of the authored cells.
    pub fn bounds(&self) -> ((i32, i32, i32), (i32, i32, i32)) {
        let mut lo = (i32::MAX, i32::MAX, i32::MAX);
        let mut hi = (i32::MIN, i32::MIN, i32::MIN);
        for &(x, y, z) in self.cells.keys() {
            lo = (lo.0.min(x), lo.1.min(y), lo.2.min(z));
            hi = (hi.0.max(x), hi.1.max(y), hi.2.max(z));
        }
        (lo, hi)
    }
}

fn facing_toward(frm: (i32, i32, i32), to: (i32, i32, i32)) -> Result<&'static str, HdlError> {
    match (to.0 - frm.0, to.2 - frm.2) {
        (-1, 0) => Ok("west"),
        (1, 0) => Ok("east"),
        (0, -1) => Ok("north"),
        (0, 1) => Ok("south"),
        (dx, dz) => Err(HdlError::Layout(format!(
            "repeater facing needs a unit step, got ({dx},{dz})"
        ))),
    }
}

/// The PLA compiler: plans a stage list and emits the full build.
pub struct Pla {
    /// The authored voxels.
    pub b: Build,
    /// Rail dust (x, z) columns, for lid placement.
    rail_dust: BTreeSet<(i32, i32)>,
    /// Planned stages.
    pub stages: Vec<Planned>,
    /// signal -> (stage, slice, group index).
    pub produced: HashMap<String, (usize, i32, usize)>,
    /// Inter-stage wires.
    pub routes: Vec<Route>,
    /// `(signal, lever cell)` in stage/lever-list order — the drive order.
    pub levers: Vec<(String, (i32, i32, i32))>,
    /// signal -> a dust cell that reads its settled value.
    pub probe: BTreeMap<String, (i32, i32, i32)>,
    /// How many rail lids were placed.
    pub lids: usize,
    /// Per DFF (in `Stage::seq` order): `(D port dust, Q port dust)` in
    /// build coordinates.
    pub seq_ports: Vec<((i32, i32, i32), (i32, i32, i32))>,
    /// The clock lever cell, when a DFF bank was built.
    pub clock_lever: Option<(i32, i32, i32)>,
}

impl Pla {
    /// Plan and lay out `stages`; call [`Pla::build`] to emit geometry.
    pub fn new(stages: Vec<Stage>) -> Result<Self, HdlError> {
        let (planned, produced, routes) = layout(stages)?;
        Ok(Pla {
            b: Build::default(),
            rail_dust: BTreeSet::new(),
            stages: planned,
            produced,
            routes,
            levers: Vec::new(),
            probe: BTreeMap::new(),
            lids: 0,
            seq_ports: Vec::new(),
            clock_lever: None,
        })
    }

    fn dust(&mut self, x: i32, y: i32, z: i32, label: &str) -> Result<(), HdlError> {
        self.b.put(x, y, z, DUST)?;
        self.b.labels.insert((x, y, z), label.to_string());
        Ok(())
    }

    fn rail_z(p: &Planned, sig: &str) -> i32 {
        p.z + 3 * p.slot[sig]
    }

    // -- rails --------------------------------------------------------------
    fn build_rails(&mut self) -> Result<(), HdlError> {
        for si in 0..self.stages.len() {
            let spans: Vec<(String, (i32, i32))> = self.stages[si]
                .spans
                .iter()
                .map(|(s, sp)| (s.clone(), *sp))
                .collect();
            for (sig, (x0, x1)) in spans {
                let z = Self::rail_z(&self.stages[si], &sig);
                let drive = self.stages[si].drive[&sig];
                let reps = rail_repeater_xs(x0, x1, drive);
                for x in x0..=x1 {
                    self.b.stone(x, 0, z, "rail_floor")?;
                    self.rail_dust.insert((x, z)); // lid decided later
                    if reps.contains(&x) {
                        self.b.put(x, 1, z, &repeater("west"))?;
                    } else {
                        self.dust(x, 1, z, &sig)?;
                    }
                }
                // read a DUST cell — a repeater has no power= property and
                // would silently read as 0 forever
                let px = (drive + 1..=x1)
                    .find(|x| !reps.contains(x))
                    .unwrap_or(drive);
                self.probe.insert(sig.clone(), (px, 1, z));
            }
            let levers: Vec<(i32, String, usize)> = self.stages[si].st.levers.clone();
            for (sl, sig, ch) in levers {
                let p = (sx(sl) + CH[ch], 1, Self::rail_z(&self.stages[si], &sig));
                self.b.force(p.0, p.1, p.2, LEVER_OFF);
                self.b.labels.remove(&p);
                self.levers.push((sig, p));
            }
        }
        Ok(())
    }

    // -- inverter: spur off a rail, torch, stair down onto the rail above ----
    fn build_inverters(&mut self) -> Result<(), HdlError> {
        for si in 0..self.stages.len() {
            let invs = self.stages[si].st.inverters.clone();
            for (sl, src, dst) in invs {
                let x = sx(sl) + INV_X;
                let zs = Self::rail_z(&self.stages[si], &src);
                if Self::rail_z(&self.stages[si], &dst) != zs + 3 {
                    return Err(HdlError::Layout(format!(
                        "inverter {src} -> {dst}: output rail not one slot above input"
                    )));
                }
                self.b.stone(x, 0, zs + 1, "inv")?;
                self.b.stone(x, 0, zs + 2, "inv")?;
                self.dust(x, 1, zs + 1, &src)?;
                self.b.stone(x, 1, zs + 2, "plain")?;
                self.b.put(x, 2, zs + 2, TORCH)?;
                self.b.stone(x, 3, zs + 2, "inv")?;
                self.dust(x, 4, zs + 2, &dst)?;
                for (k, y) in [(1, 3), (2, 2), (3, 1)] {
                    self.b.stone(x + k, y - 1, zs + 2, "inv")?;
                    self.dust(x + k, y, zs + 2, &dst)?;
                }
            }
        }
        Ok(())
    }

    // -- one product term: taps + collector + inverting gate -----------------
    fn build_column(
        &mut self,
        si: usize,
        x: i32,
        term: &[String],
        gate_off: i32,
        label: &str,
    ) -> Result<(), HdlError> {
        let zb = self.stages[si].z;
        for sig in term {
            let rz = Self::rail_z(&self.stages[si], sig);
            self.b.stone(x, 0, rz + 1, "tap")?;
            self.b.stone(x, 0, rz + 2, "tap")?;
            self.dust(x, 1, rz + 1, sig)?;
            self.b.stone(x, 1, rz + 2, "tap")?;
            self.b.put(x, 2, rz + 2, TORCH)?;
        }
        let lo = term
            .iter()
            .map(|s| 3 * self.stages[si].slot[s] + 2)
            .min()
            .expect("a term always has a literal");
        let mut since = 0;
        for off in lo..gate_off {
            self.b.stone(x, 3, zb + off, "coll_floor")?;
            since += 1;
            if since >= 10 && off % 3 == 0 && off > lo {
                self.b.put(x, Y_COLL, zb + off, &repeater("north"))?;
                since = 0;
            } else {
                self.dust(x, Y_COLL, zb + off, label)?;
            }
        }
        self.b.stone(x, Y_COLL, zb + gate_off, "gate")?;
        self.b.put(x, Y_GATE, zb + gate_off, TORCH)?;
        self.b.stone(x, 6, zb + gate_off, "gate")?;
        Ok(())
    }

    fn build_nodes(&mut self) -> Result<(), HdlError> {
        let all_cols: Vec<i32> = COL_A.iter().chain(COL_B.iter()).chain(EXTRA_COLS.iter()).copied().collect();
        for si in 0..self.stages.len() {
            let geoms: Vec<(String, Geom)> = self.stages[si]
                .geom
                .iter()
                .map(|(o, g)| (o.clone(), g.clone()))
                .collect();
            for (out, g) in geoms {
                // a gi=0 group may hold up to 3 terms by borrowing COL_B's
                // column — the slice then has one wide node
                let cols: Vec<i32> = if g.gi == 0 {
                    all_cols.iter().take(g.terms.len()).copied().collect()
                } else {
                    COL_B.iter().take(g.terms.len()).copied().collect()
                };
                if cols.len() != g.terms.len() {
                    return Err(HdlError::Layout(format!(
                        "node {out} needs {} columns, layout has {}",
                        g.terms.len(),
                        cols.len()
                    )));
                }
                let xs: Vec<i32> = cols.iter().map(|c| sx(g.slice) + c).collect();
                let zb = self.stages[si].z;
                for (x, term) in xs.iter().zip(&g.terms) {
                    self.build_column(si, *x, term, g.gate, &format!("{out}#{x}"))?;
                    self.dust(*x, Y_LANE, zb + g.gate, &out)?;
                }
                let (lo, hi) = (
                    *xs.iter().min().expect("node has columns"),
                    *xs.iter().max().expect("node has columns"),
                );
                for x in lo..=hi {
                    self.b.stone(x, 6, zb + g.lane, "lane")?;
                    self.dust(x, Y_LANE, zb + g.lane, &out)?;
                }
                self.probe.insert(out.clone(), (lo, Y_LANE, zb + g.lane));
            }
        }
        Ok(())
    }

    // -- routing --------------------------------------------------------------
    fn build_routes(&mut self) -> Result<(), HdlError> {
        let routes = self.routes.clone();
        for r in &routes {
            let src = &self.stages[r.src];
            let g = src.geom[&r.sig].clone();
            let cx = sx(g.slice) + CH[r.ch];
            let rx = sx(g.slice) + r.riser;
            let lane_z = src.z + g.lane;
            let wz = src.z + r.t;
            let dz = Self::rail_z(&self.stages[r.dst], &r.sig);
            let mut cells: Vec<(i32, i32, i32)> = Vec::new();
            cells.extend((lane_z + 1..=wz).map(|zz| (rx, Y_LANE, zz)));
            cells.extend((cx..rx).rev().map(|x| (x, Y_LANE, wz)));
            cells.extend([(cx, 6, wz + 1), (cx, Y_GATE, wz + 2), (cx, Y_COLL, wz + 3)]);
            cells.extend((wz + 4..dz - 3).map(|zz| (cx, Y_COLL, zz)));
            // Land on the rail HORIZONTALLY. Dropping the last step diagonally
            // from y=2 cannot work: the block above the rail dust is the
            // rail's own lid, and blocking that diagonal is the entire point
            // of the lid.
            cells.extend([(cx, 3, dz - 3), (cx, 2, dz - 2), (cx, 1, dz - 1), (cx, 1, dz)]);
            self.run(&cells, &r.sig)?;
        }
        Ok(())
    }

    fn run(&mut self, cells: &[(i32, i32, i32)], label: &str) -> Result<(), HdlError> {
        let repeat_every = ROUTE_REPEAT;
        // The SOURCE is lane dust that has already decayed a few levels, so
        // the first refresh keeps the old <=6-cell spacing; only
        // refresh-to-refresh spans run at full pitch.
        let mut since = (repeat_every - 6).max(0);
        for i in 0..cells.len() {
            let (x, y, z) = cells[i];
            // Shared trunk: a second route of the SAME signal may re-walk
            // cells the first already laid. Existing dust is reused as-is;
            // `since` is primed so a repeater lands at the first legal cell
            // after the shared prefix ends.
            if self
                .b
                .cells
                .get(&(x, y, z))
                .is_some_and(|c| c.contains("redstone_wire"))
            {
                if self.b.labels.get(&(x, y, z)).map(String::as_str) != Some(label) {
                    return Err(HdlError::Layout(format!(
                        "route {label} crosses net {} at ({x},{y},{z})",
                        self.b
                            .labels
                            .get(&(x, y, z))
                            .map_or("<unlabelled>", String::as_str)
                    )));
                }
                since = repeat_every;
                continue;
            }
            self.b.stone(x, y - 1, z, "route")?;
            let prev = if i > 0 { Some(cells[i - 1]) } else { None };
            let nxt = cells.get(i + 1).copied();
            let straight = matches!(
                (prev, nxt),
                (Some(p), Some(n))
                    if p.1 == y && n.1 == y
                        && ((p.0 == x && n.0 == x) || (p.2 == z && n.2 == z))
            );
            // The tail after a straight run (stairs, rail landings) cannot
            // host repeaters, so bank a refresh at the run's LAST straight
            // cell if the budget is already half spent.
            let nxt2 = cells.get(i + 2).copied();
            let cont = matches!(
                (nxt, nxt2),
                (Some(n), Some(n2))
                    if n.1 == y && n2.1 == y
                        && ((n.0 == x && n2.0 == x) || (n.2 == z && n2.2 == z))
            );
            since += 1;
            if straight && since >= (if cont { repeat_every } else { repeat_every - 6 }) {
                let facing = facing_toward((x, y, z), prev.expect("straight implies prev"))?;
                self.b.put(x, y, z, &repeater(facing))?;
                since = 0;
            } else {
                self.dust(x, y, z, label)?;
            }
        }
        Ok(())
    }

    // -- sequential: DFF bank, clock spine, D spurs, Q wrap corridors --------
    //
    // Topology (documented in seq_README/DESIGN_SPEC): the bank is the LAST
    // stage band. D signals are raised to the top combinational level, so
    // their delivery is an ordinary next-stage route onto a bank rail; each
    // DFF taps its D rail with a y3 flyover spur (support blocks double as
    // caps over every rail row crossed). The clock is a dedicated y1 spine
    // north of the DFF row — one lever, one dust branch per cell by
    // adjacency. Q leaves east and wraps AROUND the fabric (south, then
    // west of x=0, then north past stage 0) and re-enters its stage-0 input
    // rail from the north — the rail is slot 0, the northernmost row of the
    // input band, so the corridor lands on its west-end dust without
    // crossing anything. Corridor rows/columns are pitched 2 apart with
    // depths ordered by bank/stage-0 slice, which makes the whole wrap
    // planar: no two corridors ever cross (same argument as the counter4
    // feedback corridors, generalised).
    fn build_seq(&mut self) -> Result<(), HdlError> {
        let si = match self.stages.iter().position(|p| !p.st.seq.is_empty()) {
            Some(i) => i,
            None => return Ok(()),
        };
        let cells: Vec<crate::seq::SeqCell> = self.stages[si].st.seq.clone();
        let zb = self.stages[si].z;
        let nslots = self.stages[si]
            .slot
            .values()
            .max()
            .map_or(0, |m| m + 1);
        let dz = zb + 3 * nslots + 6;
        let spine_z = dz - 2;
        let lo_slice = cells.iter().map(|c| c.slice).min().expect("seq not empty");
        let hi_slice = cells.iter().map(|c| c.slice).max().expect("seq not empty");

        // -- clock: lever + spine, one dust branch per DFF by adjacency ------
        let lx = sx(lo_slice) - 3;
        self.b.stone(lx, 0, spine_z, "route")?;
        self.b.force(lx, 1, spine_z, LEVER_OFF);
        self.clock_lever = Some((lx, 1, spine_z));
        let mut since = 0;
        for x in lx + 1..=sx(hi_slice) + 10 {
            self.b.stone(x, 0, spine_z, "route")?;
            since += 1;
            // dense refresh (>=8) keeps every branch cell at ss>=6 so the
            // in-cell clock column stays alive; the cell directly north of a
            // clk_in port must stay dust so the branch is fed by adjacency
            if since >= 8 && x.rem_euclid(SLICE_W) != 10 {
                self.b.put(x, 1, spine_z, &repeater("west"))?;
                since = 0;
            } else {
                self.dust(x, 1, spine_z, "clk")?;
            }
        }

        for (k, sc) in cells.iter().enumerate() {
            let x0 = sx(sc.slice);

            // -- the DFF template, baked at its declared initial state ------
            for ((cx, cy, cz), state, lab) in crate::seq::dff_cells(sc.init) {
                let (gx, gy, gz) = (x0 + cx, cy, dz + cz);
                if state.contains("concrete") {
                    // structure: first role to claim a cell keeps its colour
                    if !self.b.solid_at(gx, gy, gz) {
                        self.b.put(gx, gy, gz, &state)?;
                    }
                } else {
                    self.b.put(gx, gy, gz, &state)?;
                }
                if let Some(l) = lab {
                    self.b
                        .labels
                        .insert((gx, gy, gz), format!("{}.{l}", sc.label));
                }
            }
            let d_port = (x0, 1, dz);
            let q_port = (x0 + 12, 1, dz);
            self.seq_ports.push((d_port, q_port));
            self.probe.insert(format!("{}.d", sc.label), d_port);
            self.probe.insert(format!("{}.q", sc.label), q_port);

            // -- D: spur off the bank rail, or a constant tie ----------------
            match &sc.d {
                Some(d) => {
                    let rz = zb + 3 * self.stages[si].slot[d];
                    // guarantee a fresh signal at the tap: force a rail
                    // repeater just west of this slice (offset 16 is never a
                    // tap/inverter column)
                    self.b.force(x0 - 1, 1, rz, &repeater("west"));
                    self.b.labels.remove(&(x0 - 1, 1, rz));
                    // y3 flyover spur: supports double as caps over every
                    // rail row crossed (incl. the clock spine), which also
                    // severs the descent diagonals into foreign dust
                    let mut path: Vec<(i32, i32, i32)> = vec![
                        (x0, 1, rz + 1),
                        (x0, 2, rz + 2),
                        (x0, 3, rz + 3),
                    ];
                    path.extend((rz + 4..=dz - 2).map(|z| (x0, 3, z)));
                    path.push((x0, 2, dz - 1));
                    self.run(&path, d)?;
                }
                None => {
                    if sc.d_const == 1 {
                        // tie D high: torch replacing the D port dust (the
                        // dust at local (1,1,0) carries it into the master)
                        self.b.force(x0, 1, dz, TORCH);
                        self.b.labels.remove(&(x0, 1, dz));
                    }
                    // d_const == 0: the D run floats at 0 by itself
                }
            }

            // -- Q: wrap corridor east -> south -> west -> north -> into the
            //    stage-0 input rail's west end from the north ---------------
            let row_s = dz + 7 + 2 * k as i32; // south row (east cells deeper)
            let col_w = -4 - 2 * k as i32; // west column
            let row_n = -2 - 2 * k as i32; // north row (east entries deeper)
            let ex = sx(sc.q_slice); // stage-0 entry column
            let mut path: Vec<(i32, i32, i32)> = Vec::new();
            path.extend((13..=15).map(|dx| (x0 + dx, 1, dz)));
            path.extend((dz + 1..=row_s).map(|z| (x0 + 15, 1, z)));
            path.extend((col_w..=x0 + 14).rev().map(|x| (x, 1, row_s)));
            path.extend((row_n..row_s).rev().map(|z| (col_w, 1, z)));
            path.extend((col_w + 1..=ex).map(|x| (x, 1, row_n)));
            path.extend((row_n + 1..=-1).map(|z| (ex, 1, z)));
            self.run(&path, &sc.q_rail)?;
        }
        Ok(())
    }

    /// A rail lid is only load-bearing where a wire could actually drop into
    /// the rail: any dust in the four y=2 neighbours of a rail cell.
    fn place_lids(&mut self) -> Result<usize, HdlError> {
        let mut n = 0;
        let rail_dust: Vec<(i32, i32)> = self.rail_dust.iter().copied().collect();
        for (x, z) in rail_dust {
            if self.b.cells.contains_key(&(x, 2, z)) {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nb = self.b.cells.get(&(x + dx, 2, z + dz));
                if nb.is_some_and(|c| c.contains("redstone_wire")) {
                    self.b.stone(x, 2, z, "lid")?;
                    n += 1;
                    break;
                }
            }
        }
        Ok(n)
    }

    /// Emit rails, inverters, node columns, routes, the DFF bank (when a
    /// stage carries sequential cells) and lids.
    pub fn build(&mut self) -> Result<(), HdlError> {
        self.build_rails()?;
        self.build_inverters()?;
        self.build_nodes()?;
        self.build_routes()?;
        self.build_seq()?;
        self.lids = self.place_lids()?;
        Ok(())
    }
}
