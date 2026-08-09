//! Mechanism-level signal transport: what CARRIES a signal, what a carrier
//! needs under it, what it ENERGISES, and whether two placements belonging to
//! different nets actually interfere.
//!
//! Port of `redstone-eda/materials.py` §2b; the prose table, and the probe
//! that fixed each field, are in `redstone-eda/TRANSPORT_MODEL.md`. Nothing
//! here is invented: every row is probe-backed against the tick engine.
//!
//! # Why this exists
//!
//! The fabric used to ask one question — "is this cell free, and is any
//! foreign redstone next to it?" — and answer it with a scalar halo: a
//! one-cell shell dilated around every placed instance. That halo conflates
//! *occupancy* with *interference*, and the two are very different:
//!
//! ```text
//! an inert stone block emits NOTHING and reads NOTHING
//! ```
//!
//! so a bus may lay its dust flush against a cell's stone flank all day. The
//! halo forbade it anyway, which is the single largest exclusion bucket in
//! `tests/design_routability.rs`.
//!
//! # The one line worth memorising
//!
//! **Dust does not read WEAK.** That asymmetry is the whole model: it is why a
//! live weakly-powered block may sit under a foreign dust line, why a repeater
//! may stand on a hard-powered block, and why an inert solid support is
//! electrically invisible to a neighbouring bus.
//!
//! Interference is therefore *not* a distance: it is
//! `one side's emission lands where the other side reads, in a kind the other
//! side can actually read`.

use pnr_core::grid::Pos;
use std::collections::BTreeMap;

use crate::blocks;

/// The four kinds of power a placement can put into a cell.
///
/// The kind matters because consumers are selective — see [`Reader::kinds`].
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// Hard power: a block strong-powered by a repeater/comparator front or a
    /// lever's attachment. Read by dust AND devices; never chains block to
    /// block (probe `S1`/`S3`).
    Strong,
    /// Weak power: what dust puts into the block beneath it and the blocks it
    /// points at. Read by DEVICE inputs only — **never by dust** (`W1`).
    Weak,
    /// The wire itself. Two cells sharing WIRE are the same net by
    /// construction, so it never counts as foreign interference.
    Wire,
    /// A permanent source (torch body, redstone block, lever). Read by
    /// everything.
    Source,
}

/// Which emission kinds actually reach a consumer.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Reader {
    /// Reads nothing — an inert block, a support, a pure source.
    None,
    /// Dust: scans its 6 neighbours and picks up STRONG, SOURCE and WIRE.
    /// **Not WEAK** (probe `W1`).
    Dust,
    /// A device input (repeater/comparator back, torch attachment): picks up
    /// everything, including WEAK carried through a conducting block (`W2`).
    Device,
}

impl Reader {
    /// The emission kinds this reader can pick up.
    pub fn kinds(self) -> &'static [Kind] {
        match self {
            Reader::None => &[],
            Reader::Dust => &[Kind::Strong, Kind::Source, Kind::Wire],
            Reader::Device => &[Kind::Strong, Kind::Weak, Kind::Source, Kind::Wire],
        }
    }

    /// Whether this reader picks up `kind`.
    pub fn reads(self, kind: Kind) -> bool {
        self.kinds().contains(&kind)
    }
}

/// The six face neighbours, in the canonical order the probes used.
pub const NB6: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

/// The four horizontal directions.
pub const NB4: [(i32, i32, i32); 4] = [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)];

/// One way a signal MOVES. Eleven rows; `dust_step` shares dust's electrical
/// row and differs only in the step law ([`step_reads`]), so it is not a
/// separate variant here.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mechanism {
    /// Redstone dust on a sturdy support. Decays 1 per cell.
    Dust,
    /// A block hard-powered by a repeater/comparator front. Needs no support,
    /// refreshes to 15, and — the fact both crossing tiles exploit — strong
    /// power never chains block to block, so two of these may be neighbours.
    StrongBlock,
    /// A block weakly powered by dust. Invisible to dust; reaches device
    /// inputs only.
    WeakBlock,
    /// A repeater: reads its BACK only, emits STRONG out its front.
    Repeater,
    /// A comparator: reads its back and both sides, emits STRONG out its front.
    Comparator,
    /// A floor-standing redstone torch: the only inverter and the only compact
    /// vertical carrier.
    TorchFloor,
    /// A redstone block: always on.
    RedstoneBlock,
    /// A lever: strong-powers its attachment block.
    Lever,
    /// Sturdy AND conducting (stone, concrete). Carries dust, separates
    /// stacked runs, lids a live run, and severs a foreign diagonal — all four
    /// at once, which is what the crossings exploit.
    SolidSupport,
    /// Sturdy and NON-conducting (glass, top slabs). Supports dust without
    /// conducting, which makes it a one-way diode uphill.
    TransparentSupport,
    /// Anything else: not a carrier, not a support, electrically dead.
    Inert,
}

impl Mechanism {
    /// Which emission kinds this mechanism can pick up.
    pub fn reader(self) -> Reader {
        match self {
            Mechanism::Dust => Reader::Dust,
            Mechanism::Repeater | Mechanism::Comparator | Mechanism::TorchFloor => Reader::Device,
            _ => Reader::None,
        }
    }

    /// Whether the cell directly beneath must be a sturdy support.
    pub fn needs_sturdy_support(self) -> bool {
        matches!(
            self,
            Mechanism::Dust | Mechanism::Repeater | Mechanism::Comparator
        )
    }

    /// The offset (canonical frame) of a block this mechanism hangs off, if any.
    pub fn attach(self) -> Option<(i32, i32, i32)> {
        match self {
            Mechanism::TorchFloor | Mechanism::Lever => Some((0, -1, 0)),
            _ => None,
        }
    }

    /// Game ticks of delay this mechanism adds.
    pub fn delay_gt(self) -> u32 {
        match self {
            Mechanism::Repeater | Mechanism::Comparator | Mechanism::TorchFloor => 2,
            _ => 0,
        }
    }

    /// Signal strength lost per cell traversed.
    pub fn decay(self) -> u32 {
        match self {
            Mechanism::Dust => 1,
            _ => 0,
        }
    }

    /// Whether this mechanism re-emits 15 regardless of its input strength.
    pub fn refreshes(self) -> bool {
        matches!(
            self,
            Mechanism::Repeater | Mechanism::TorchFloor | Mechanism::StrongBlock
        )
    }

    /// Whether this mechanism inverts.
    pub fn inverts(self) -> bool {
        matches!(self, Mechanism::TorchFloor)
    }

    /// `{cell: kind}` this placement energises.
    ///
    /// `fwd` is the mechanism's forward unit vector (a device's OUTPUT
    /// direction); the canonical frame's forward is `+X`. `pointing` supplies
    /// dust's connection axes, which are data-dependent — see
    /// [`dust_pointing`].
    pub fn emission(
        self,
        cell: Pos,
        fwd: (i32, i32, i32),
        pointing: &[(i32, i32, i32)],
    ) -> Vec<(Pos, Kind)> {
        let mut out: Vec<(Pos, Kind)> = Vec::new();
        let mut push = |p: Pos, k: Kind| out.push((p, k));
        match self {
            Mechanism::Dust => {
                // WIRE in its own cell, WEAK into the block below and into
                // every block it POINTS at. It never powers the block ABOVE
                // it (probe W3) — that is what lets a solid lid over a live
                // run carry a foreign line.
                push(cell, Kind::Wire);
                push(add(cell, (0, -1, 0)), Kind::Weak);
                for d in pointing {
                    push(add(cell, *d), Kind::Weak);
                }
            }
            Mechanism::StrongBlock => {
                push(cell, Kind::Strong);
                for d in NB6 {
                    push(add(cell, d), Kind::Strong);
                }
            }
            Mechanism::WeakBlock => {
                push(cell, Kind::Weak);
                for d in NB6 {
                    push(add(cell, d), Kind::Weak);
                }
            }
            Mechanism::Repeater | Mechanism::Comparator => {
                push(add(cell, xform((1, 0, 0), fwd)), Kind::Strong);
            }
            Mechanism::TorchFloor => {
                push(add(cell, (0, 1, 0)), Kind::Strong);
                push(cell, Kind::Source);
                for d in NB4 {
                    push(add(cell, d), Kind::Source);
                }
            }
            Mechanism::RedstoneBlock => {
                push(cell, Kind::Source);
                for d in NB6 {
                    push(add(cell, d), Kind::Source);
                }
            }
            Mechanism::Lever => {
                push(add(cell, (0, -1, 0)), Kind::Strong);
                push(cell, Kind::Source);
                for d in NB6 {
                    push(add(cell, d), Kind::Source);
                }
            }
            Mechanism::SolidSupport | Mechanism::TransparentSupport | Mechanism::Inert => {}
        }
        out
    }

    /// The cells this placement takes input FROM (canonical frame rotated by
    /// `fwd`).
    pub fn inputs(self, cell: Pos, fwd: (i32, i32, i32)) -> Vec<Pos> {
        let offs: &[(i32, i32, i32)] = match self {
            Mechanism::Dust => &NB6,
            Mechanism::Repeater => &[(-1, 0, 0)],
            Mechanism::Comparator => &[(-1, 0, 0), (0, 0, 1), (0, 0, -1)],
            Mechanism::TorchFloor => &[(0, -1, 0)],
            _ => &[],
        };
        offs.iter().map(|o| add(cell, xform(*o, fwd))).collect()
    }

    /// The cells where a FOREIGN emission energises this placement.
    ///
    /// Dust is energised by anything emitting INTO ITS OWN CELL (it scans its
    /// 6 neighbours, which is the same relation seen from the source). A
    /// device is energised only at its declared input cells — that asymmetry
    /// is why a repeater may stand on a hard-powered block, and why a repeater
    /// beside a live rail is inert (probes `S4`, `probe_station S`).
    pub fn sensitive(self, cell: Pos, fwd: (i32, i32, i32)) -> Vec<Pos> {
        match self.reader() {
            Reader::None => Vec::new(),
            Reader::Dust => vec![cell],
            Reader::Device => self.inputs(cell, fwd),
        }
    }
}

/// Rotate a canonical-frame offset (forward = `+X`) so forward becomes `fwd`.
pub fn xform(off: (i32, i32, i32), fwd: (i32, i32, i32)) -> (i32, i32, i32) {
    let (ox, oy, oz) = off;
    match fwd {
        (1, 0, 0) => (ox, oy, oz),
        (-1, 0, 0) => (-ox, oy, -oz),
        (0, 0, 1) => (-oz, oy, ox),
        (0, 0, -1) => (oz, oy, -ox),
        _ => (ox, oy, oz),
    }
}

/// Offset a position.
pub fn add(p: Pos, d: (i32, i32, i32)) -> Pos {
    Pos::new(p.x + d.0, p.y + d.1, p.z + d.2)
}

// ---------------------------------------------------------------------------
// Material predicates. Sturdiness is NOT conductivity: glass and top slabs are
// full-cube supports that conduct nothing.
// ---------------------------------------------------------------------------

/// Material class of a block-state string. `None` is air.
///
/// Faithful port of `materials.py::classify`, which is the probed table's own
/// classifier. Getting this list right is load-bearing: the verified crossing
/// tiles are built out of **wool**, and a classifier that does not know wool is
/// solid reads their cut cells as air and "discovers" shorts that are not
/// there.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Material {
    /// Air, or no block at all.
    Air,
    /// A full cube that conducts: dust sits on it, weak power crosses it, and
    /// it severs a diagonal.
    Solid,
    /// A full cube that does NOT conduct (glass, stained glass): dust sits on
    /// it, but it carries no power and severs nothing.
    Transparent,
    /// A top slab: sturdy, and treated as a conductor's equal for support.
    SlabTop,
    /// A bottom slab: dust cannot sit on it.
    SlabBottom,
    /// Anything else — not a support.
    Other,
}

/// Classify a block state. Mirrors `materials.py::classify` case for case.
pub fn classify(block: Option<&str>) -> Material {
    let Some(b) = block else {
        return Material::Air;
    };
    if b.starts_with("minecraft:air") {
        return Material::Air;
    }
    if b.starts_with("minecraft:glass") || b.contains("stained_glass") {
        return Material::Transparent;
    }
    if b.contains("_slab") {
        if b.contains("type=double") {
            return Material::Solid;
        }
        return if b.contains("type=top") {
            Material::SlabTop
        } else {
            Material::SlabBottom
        };
    }
    // REDSTONE COMPONENTS ARE NOT BUILDING MATERIAL, and they must be settled
    // before any substring test, because `redstone_wire` and `redstone_torch`
    // both contain "stone". That is not a hypothetical: broadening the family
    // list below made `is_solid_block(DUST)` true and a unit test caught it
    // immediately — the same over-match that had been calling `stone_button` a
    // conductor, just pointed at ourselves.
    //
    // `redstone_lamp` and `redstone_block` are deliberately NOT here: both are
    // full cubes that conduct, and they fall through to Solid below.
    if blocks::is_dust(b)
        || blocks::is_repeater(b)
        || blocks::is_comparator(b)
        || blocks::is_torch(b)
        || blocks::is_lever(b)
    {
        return Material::Other;
    }
    // Full cubes that do NOT conduct and are not sturdy supports for our
    // purposes: the movable blocks. Checked before the `_block` family.
    if b.contains("slime_block") || b.contains("honey_block") {
        return Material::Other;
    }
    // Shapes that are NOT full cubes, checked before the family list so a
    // `stone_button` or `stone_stairs` cannot be mistaken for stone. This order
    // is the fix for a silent over-match: the old test was
    // `contains("minecraft:stone")`, which called buttons, pressure plates,
    // stairs and bottom slabs solid conductors.
    for shape in [
        "_button",
        "_pressure_plate",
        "_stairs",
        "_fence",
        "_pane",
        "_door",
        "_sign",
        "_wall",
        "_carpet",
        "_rail",
        "_trapdoor",
        "cutter",
    ] {
        if b.contains(shape) {
            return Material::Other;
        }
    }
    if b.contains("concrete")
        || b.contains("lamp")
        || b.contains("_wool")
        || b.contains("planks")
        || b.contains("terracotta")
        // FULL CUBES THAT WERE READING AS AIR. The list used to be four
        // families plus exact `minecraft:stone`, and everything else fell to
        // `Other` — "not a support, conducts nothing". That default is wrong in
        // both directions at once: the router refuses to lay dust on a block it
        // does not recognise (which is why `O03_flat_obstacle` detoured around a
        // polished-andesite wall it could simply have run over), and the
        // interference model thinks a foreign solid severs nothing (so a
        // diagonal that vanilla cuts reads as live). Same class as the
        // wool/planks/terracotta bug: silent, and wrong whichever way it lands.
        || b.contains("stone")      // stone, smooth_stone, stone_bricks, blackstone
        || b.contains("deepslate")
        || b.contains("cobble")
        || b.contains("andesite")
        || b.contains("diorite")
        || b.contains("granite")
        || b.contains("bricks")
        || b.contains("obsidian")
        || b.contains("_log")
        || b.contains("_block")     // iron_block, gold_block, ... (NOT redstone_block: caller checks that first)
        || b.contains("dirt")
        || b.contains("_ore")
        || b.contains("sandstone")
        || b.contains("prismarine")
        || b.contains("purpur")
        || b.contains("basalt")
        || b.contains("tuff")
        || b.contains("calcite")
        || b.contains("quartz")
    {
        return Material::Solid;
    }
    Material::Other
}

/// May dust or a repeater legally sit on this block? (`canSurviveOn`.)
pub fn sturdy(block: Option<&str>) -> bool {
    matches!(
        classify(block),
        Material::Solid | Material::Transparent | Material::SlabTop
    )
}

/// Does this block carry weak power into a device back?
/// (`is_redstone_conductor`.)
pub fn conducts(block: Option<&str>) -> bool {
    classify(block) == Material::Solid
}

/// Does a block in the CUT cell sever a 1-y dust step?
///
/// The cut cell is the one directly ABOVE THE LOWER dust. A CONDUCTOR there
/// severs the step; glass and top slabs do NOT.
///
/// This read `sturdy` — "any sturdy block, conducting or not" — which is
/// backwards for exactly the blocks the crossing tiles are built out of. Vanilla
/// gates the diagonal on `isRedstoneConductor`, which glass fails, and
/// `materials.py` has `cuts_step = conducts`. `wire.rs::caps_climb` already
/// excluded glass and top slabs, so the crate contradicted itself; the tests
/// missed it because they only ever passed STONE as the cut cell, never glass.
///
/// The consequence of the old answer was that a glass support — placed
/// PRECISELY so the diagonal beneath it survives, which is the whole mechanism
/// of `crossing_dipunder` — was modelled as severing that diagonal.
pub fn cuts_step(block: Option<&str>) -> bool {
    conducts(block)
}

/// Does a block in the DIODE cell let a step conduct DOWNHILL?
///
/// The diode cell is the upper dust's support. A conductor there makes the
/// step two-way; a non-conductor (glass) makes it one-way UPHILL. This is a
/// *different cell* from the cut cell, which is exactly what the old
/// conflated predicate could not express.
pub fn gates_downhill(block: Option<&str>) -> bool {
    conducts(block)
}

/// The full 1-y step law, both cells at once (probe group `C`, 8/8).
///
/// * uphill conducts iff the CUT cell is clear;
/// * downhill conducts iff the CUT cell is clear AND the DIODE cell conducts.
pub fn step_reads(cut_block: Option<&str>, upper_support: Option<&str>, downhill: bool) -> bool {
    if cuts_step(cut_block) {
        return false;
    }
    !downhill || gates_downhill(upper_support)
}

// ---------------------------------------------------------------------------
// A read-only view of placed geometry.
// ---------------------------------------------------------------------------

/// Read-only access to whatever holds the placed blocks, so the predicates run
/// against a plain map in tests and against the design's occupancy index in
/// the router without either side knowing about the other.
pub trait BlockView {
    /// The block state at `p`, or `None` for air.
    fn block_at(&self, p: Pos) -> Option<&str>;
}

impl BlockView for BTreeMap<Pos, String> {
    fn block_at(&self, p: Pos) -> Option<&str> {
        self.get(&p).map(|s| s.as_str())
    }
}

/// Classify a block state into the mechanism it acts as, judged from static
/// geometry alone.
///
/// A plain solid block classifies as [`Mechanism::SolidSupport`] — *inert*.
/// [`Mechanism::StrongBlock`] and [`Mechanism::WeakBlock`] are POWER STATES a
/// block is put into by something else, not blocks you can recognise on
/// sight; they are reached by asking the driver (a repeater front, a lever
/// attachment), never by looking at the block itself. Treating a stone block
/// as inert until a driver is found is what makes hugging a cell's flank
/// legal.
pub fn mech_of(block: &str) -> Mechanism {
    if blocks::is_dust(block) {
        Mechanism::Dust
    } else if blocks::is_repeater(block) {
        Mechanism::Repeater
    } else if blocks::is_comparator(block) {
        Mechanism::Comparator
    } else if blocks::is_torch(block) {
        Mechanism::TorchFloor
    } else if blocks::is_lever(block) {
        Mechanism::Lever
    } else if block.contains("redstone_block") {
        Mechanism::RedstoneBlock
    } else {
        match classify(Some(block)) {
            Material::Solid | Material::SlabTop => Mechanism::SolidSupport,
            Material::Transparent => Mechanism::TransparentSupport,
            _ => Mechanism::Inert,
        }
    }
}

/// The forward (output) unit vector of a placed block, from its `facing`
/// property. Repeaters and comparators face their INPUT, so their output is
/// the opposite way.
pub fn fwd_of(block: &str) -> (i32, i32, i32) {
    let facing = blocks::facing_of(block).and_then(blocks::facing_vec);
    match facing {
        Some(v) if blocks::is_repeater(block) || blocks::is_comparator(block) => {
            (-v.0, -v.1, -v.2)
        }
        Some(v) => v,
        None => (1, 0, 0),
    }
}

/// Dust's connection axes at `cell`: the directions it POINTS, and therefore
/// the blocks it weak-powers (the pointing law, probes `A`/`B`/`C`).
///
/// A dot with no connections powers all four sides; a single connection
/// extends to the opposite side as well.
pub fn dust_pointing(view: &dyn BlockView, cell: Pos) -> Vec<(i32, i32, i32)> {
    let mut hit: Vec<(i32, i32, i32)> = Vec::new();
    for d in NB4 {
        let side = add(cell, d);
        let connects = view.block_at(side).is_some_and(|b| {
            blocks::is_dust(b) || blocks::is_repeater(b) || blocks::is_comparator(b)
                || blocks::is_lever(b)
        })
            // a step up or down still connects
            || view.block_at(add(side, (0, 1, 0))).is_some_and(blocks::is_dust)
            || view.block_at(add(side, (0, -1, 0))).is_some_and(blocks::is_dust);
        if connects {
            hit.push(d);
        }
    }
    match hit.len() {
        0 => NB4.to_vec(),
        1 => {
            let d = hit[0];
            vec![d, (-d.0, -d.1, -d.2)]
        }
        _ => hit,
    }
}

/// Do two dust cells belong to the same net? Exactly vanilla's
/// `calculateTargetStrength` neighbour scan, which is where the CUT cell and
/// the DIODE cell come from (probe group `C`).
pub fn wire_connects(view: &dyn BlockView, p: Pos, q: Pos) -> bool {
    if p == q {
        return true;
    }
    let (dx, dy, dz) = (q.x - p.x, q.y - p.y, q.z - p.z);
    if dx.abs() + dz.abs() != 1 || dy.abs() > 1 {
        return false; // dust has no planar diagonal (P3)
    }
    let side = Pos::new(p.x + dx, p.y, p.z + dz);
    if dy == 0 {
        return true;
    }
    if dy == 1 {
        // `p` is the LOWER dust reading up.
        return gates_downhill(view.block_at(side))
            && !cuts_step(view.block_at(Pos::new(p.x, p.y + 1, p.z)));
    }
    // `p` is the UPPER dust reading down.
    !cuts_step(view.block_at(side))
}

// ---------------------------------------------------------------------------
// The two predicates the router actually calls.
// ---------------------------------------------------------------------------

/// One placement under consideration: a mechanism, where it sits, which way it
/// faces, and which net owns it.
#[derive(Copy, Clone, Debug)]
pub struct Placement<'a> {
    /// The transport mechanism.
    pub mech: Mechanism,
    /// Where it sits.
    pub cell: Pos,
    /// Its forward (output) unit vector.
    pub fwd: (i32, i32, i32),
    /// The owning net; two placements of the SAME net never interfere.
    pub net: &'a str,
}

impl<'a> Placement<'a> {
    /// A placement with the canonical `+X` forward.
    pub fn new(mech: Mechanism, cell: Pos, net: &'a str) -> Self {
        Placement {
            mech,
            cell,
            fwd: (1, 0, 0),
            net,
        }
    }
}

/// May `mech` sit at `cell`? `Err` names the cell and the reason.
pub fn can_occupy(
    mech: Mechanism,
    cell: Pos,
    fwd: (i32, i32, i32),
    view: &dyn BlockView,
) -> Result<(), String> {
    if let Some(b) = view.block_at(cell) {
        return Err(format!("cell {cell:?} is occupied by {b}"));
    }
    if mech.needs_sturdy_support() {
        let below = add(cell, (0, -1, 0));
        if !sturdy(view.block_at(below)) {
            return Err(format!(
                "support cell {below:?} is {:?}, not sturdy",
                view.block_at(below)
            ));
        }
    }
    if let Some(off) = mech.attach() {
        let at = add(cell, xform(off, fwd));
        if !sturdy(view.block_at(at)) {
            return Err(format!(
                "attachment cell {at:?} is {:?}, not sturdy",
                view.block_at(at)
            ));
        }
    }
    Ok(())
}

/// Do two placements of DIFFERENT nets interact? `Some(reason)` if they do.
///
/// Interference is exactly *one side's emission lands where the other side
/// reads, in a kind the other side can actually read*. Everything the crossing
/// tiles exploit falls out of that: STRONG never lands on a block, WEAK never
/// lands on dust, and a repeater only reads one cell.
///
/// Dust-to-dust shorting is NOT an emission relation (dust emits WEAK, which
/// dust cannot read) — it is a wire-connection relation, so it is tested
/// separately with [`wire_connects`].
pub fn interferes(a: &Placement, b: &Placement, view: &dyn BlockView) -> Option<String> {
    if a.net == b.net {
        return None;
    }
    for (src, dst) in [(a, b), (b, a)] {
        let pointing = if src.mech == Mechanism::Dust {
            dust_pointing(view, src.cell)
        } else {
            Vec::new()
        };
        let emit = src.mech.emission(src.cell, src.fwd, &pointing);
        let reader = dst.mech.reader();
        for c in dst.mech.sensitive(dst.cell, dst.fwd) {
            for (ec, kind) in &emit {
                if *ec == c && *kind != Kind::Wire && reader.reads(*kind) {
                    return Some(format!(
                        "{:?} at {:?} emits {:?} into {:?}'s {:?}",
                        src.mech, src.cell, kind, dst.mech, c
                    ));
                }
            }
        }
    }
    if a.mech.reader() == Reader::Dust
        && b.mech.reader() == Reader::Dust
        && wire_connects(view, a.cell, b.cell)
    {
        return Some(format!(
            "the two dust cells are wire-connected at {:?}/{:?}",
            a.cell, b.cell
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(cells: &[(Pos, &str)]) -> BTreeMap<Pos, String> {
        cells.iter().map(|(p, b)| (*p, b.to_string())).collect()
    }

    const GLASS: &str = "minecraft:glass";

    #[test]
    fn dust_does_not_read_weak() {
        // The single most useful line in the model (probe W1).
        assert!(!Reader::Dust.reads(Kind::Weak));
        assert!(Reader::Device.reads(Kind::Weak));
    }

    #[test]
    fn an_inert_solid_block_is_electrically_invisible() {
        // THE unlock: a bus may lay dust flush against a cell's stone flank.
        // The stone emits nothing and reads nothing, so there is no relation
        // for `interferes` to find — no matter how close it sits.
        let g = view(&[(Pos::new(0, 0, 0), blocks::STONE)]);
        let stone = Placement::new(Mechanism::SolidSupport, Pos::new(0, 0, 0), "theirs");
        for d in NB6 {
            let ours = Placement::new(Mechanism::Dust, add(Pos::new(0, 0, 0), d), "ours");
            assert_eq!(
                interferes(&stone, &ours, &g),
                None,
                "inert stone must not interfere with dust at {d:?}"
            );
        }
    }

    #[test]
    fn two_strong_blocks_may_be_neighbours() {
        // S1/S3: strong power never chains block to block. This is the fact
        // `CROSSWIRE002_classic` is built on.
        let g = view(&[]);
        let a = Placement::new(Mechanism::StrongBlock, Pos::new(0, 0, 0), "a");
        let b = Placement::new(Mechanism::StrongBlock, Pos::new(1, 0, 0), "b");
        assert_eq!(interferes(&a, &b, &g), None);
    }

    #[test]
    fn a_repeater_may_stand_on_a_hard_powered_block() {
        // S4: a repeater reads its BACK only — no quasi-connectivity — so the
        // X-line is electrically blind to the cell it stands on.
        let g = view(&[]);
        let block = Placement::new(Mechanism::StrongBlock, Pos::new(0, 0, 0), "z_line");
        let rep = Placement::new(Mechanism::Repeater, Pos::new(0, 1, 0), "x_line");
        assert_eq!(interferes(&block, &rep, &g), None);
    }

    #[test]
    fn a_weak_block_may_sit_under_a_foreign_dust_line() {
        // W1: dust cannot see weak power, on any face including on top.
        let g = view(&[(Pos::new(0, -1, 0), blocks::STONE)]);
        let weak = Placement::new(Mechanism::WeakBlock, Pos::new(0, 0, 0), "theirs");
        let dust = Placement::new(Mechanism::Dust, Pos::new(0, 1, 0), "ours");
        assert_eq!(interferes(&weak, &dust, &g), None);
    }

    #[test]
    fn a_strong_block_does_energise_neighbouring_dust() {
        // The other half of the same rule: STRONG *is* read by dust, so the
        // predicate still forbids the illegal adjacency.
        let g = view(&[]);
        let block = Placement::new(Mechanism::StrongBlock, Pos::new(0, 0, 0), "theirs");
        let dust = Placement::new(Mechanism::Dust, Pos::new(1, 0, 0), "ours");
        assert!(interferes(&block, &dust, &g).is_some());
    }

    #[test]
    fn two_adjacent_dusts_of_different_nets_short() {
        // Not an emission relation — a wire-connection one.
        let g = view(&[
            (Pos::new(0, 0, 0), blocks::DUST),
            (Pos::new(1, 0, 0), blocks::DUST),
        ]);
        let a = Placement::new(Mechanism::Dust, Pos::new(0, 0, 0), "a");
        let b = Placement::new(Mechanism::Dust, Pos::new(1, 0, 0), "b");
        assert!(interferes(&a, &b, &g).is_some());
    }

    #[test]
    fn dust_has_no_planar_diagonal() {
        // P3: two dusts diagonally apart on one level are separate nets.
        let g = view(&[
            (Pos::new(0, 0, 0), blocks::DUST),
            (Pos::new(1, 0, 1), blocks::DUST),
        ]);
        let a = Placement::new(Mechanism::Dust, Pos::new(0, 0, 0), "a");
        let b = Placement::new(Mechanism::Dust, Pos::new(1, 0, 1), "b");
        assert_eq!(interferes(&a, &b, &g), None);
    }

    #[test]
    fn a_solid_lid_over_a_live_run_carries_a_foreign_line() {
        // W3: dust never powers the block ABOVE it, so a lid is harmless and
        // may itself support another net's dust. This is what makes the
        // stacked crossings tile.
        let g = view(&[
            (Pos::new(0, -1, 0), blocks::STONE),
            (Pos::new(0, 0, 0), blocks::DUST),
            (Pos::new(0, 1, 0), blocks::STONE),
        ]);
        let theirs = Placement::new(Mechanism::Dust, Pos::new(0, 0, 0), "theirs");
        let lid = Placement::new(Mechanism::SolidSupport, Pos::new(0, 1, 0), "ours");
        assert_eq!(interferes(&theirs, &lid, &g), None);
        // And the dust two levels up, sitting ON that lid, is a different net
        // that stays isolated: the lid cuts the step.
        let ours = Placement::new(Mechanism::Dust, Pos::new(0, 2, 0), "ours");
        assert_eq!(interferes(&theirs, &ours, &g), None);
    }

    #[test]
    fn the_step_law_splits_the_cut_cell_from_the_diode_cell() {
        // Probe group C, the 2x2x2 matrix: uphill needs a clear cut cell;
        // downhill needs that AND a conducting diode cell.
        assert!(step_reads(None, Some(blocks::STONE), false)); // uphill, clear
        assert!(!step_reads(Some(blocks::STONE), Some(blocks::STONE), false)); // cut
        assert!(step_reads(None, Some(blocks::STONE), true)); // downhill, conducts
        assert!(!step_reads(None, Some(GLASS), true)); // glass = one-way uphill
        assert!(step_reads(None, Some(GLASS), false)); // ...still fine uphill
    }

    #[test]
    fn wool_planks_and_terracotta_are_solid() {
        // The verified crossing tiles are built out of WOOL: its cut cells are
        // `blue_wool` and its supports are `red_wool`. A classifier that does
        // not know that reads them as air and reports shorts that do not
        // exist. Matches `materials.py::classify`.
        for b in [
            "minecraft:blue_wool",
            "minecraft:red_wool",
            "minecraft:oak_planks",
            "minecraft:white_terracotta",
            "minecraft:gray_concrete",
            blocks::STONE,
            "minecraft:smooth_stone_slab[type=double,waterlogged=false]",
        ] {
            assert_eq!(classify(Some(b)), Material::Solid, "{b} must be solid");
            assert!(sturdy(Some(b)), "{b} must be sturdy");
            assert!(conducts(Some(b)), "{b} must conduct");
            assert!(cuts_step(Some(b)), "{b} must sever a step");
            assert_eq!(mech_of(b), Mechanism::SolidSupport, "{b}");
        }
    }

    #[test]
    fn slabs_are_sturdy_only_top_side_up() {
        let top = "minecraft:smooth_stone_slab[type=top,waterlogged=false]";
        let bot = "minecraft:smooth_stone_slab[type=bottom,waterlogged=false]";
        assert_eq!(classify(Some(top)), Material::SlabTop);
        assert_eq!(classify(Some(bot)), Material::SlabBottom);
        assert!(sturdy(Some(top)));
        assert!(!sturdy(Some(bot)), "dust cannot sit on a bottom slab");
    }

    #[test]
    fn air_and_unknown_blocks_are_not_supports() {
        assert_eq!(classify(None), Material::Air);
        assert_eq!(classify(Some("minecraft:air")), Material::Air);
        assert!(!sturdy(None));
        assert!(!sturdy(Some("minecraft:torch")));
        assert!(!conducts(Some("minecraft:air")));
    }

    #[test]
    fn glass_is_sturdy_but_does_not_conduct() {
        assert!(sturdy(Some(GLASS)));
        assert!(!conducts(Some(GLASS)));
        assert!(sturdy(Some(blocks::STONE)));
        assert!(conducts(Some(blocks::STONE)));
        assert!(!sturdy(None));
    }

    #[test]
    fn a_repeater_beside_a_live_rail_is_inert() {
        // probe_station S: the repeater reads its back cell only, so dust
        // running past its flank does nothing.
        let g = view(&[(Pos::new(0, 0, 1), blocks::DUST)]);
        // Repeater at origin facing +X (output +X, back at -X).
        let rep = Placement {
            mech: Mechanism::Repeater,
            cell: Pos::new(0, 0, 0),
            fwd: (1, 0, 0),
            net: "ours",
        };
        let rail = Placement::new(Mechanism::Dust, Pos::new(0, 0, 1), "theirs");
        assert_eq!(interferes(&rep, &rail, &g), None);
    }

    #[test]
    fn a_repeater_front_does_drive_the_dust_it_points_at() {
        let g = view(&[(Pos::new(1, 0, 0), blocks::DUST)]);
        let rep = Placement {
            mech: Mechanism::Repeater,
            cell: Pos::new(0, 0, 0),
            fwd: (1, 0, 0),
            net: "ours",
        };
        let dust = Placement::new(Mechanism::Dust, Pos::new(1, 0, 0), "theirs");
        assert!(interferes(&rep, &dust, &g).is_some());
    }

    #[test]
    fn same_net_never_interferes() {
        let g = view(&[]);
        let a = Placement::new(Mechanism::Dust, Pos::new(0, 0, 0), "n");
        let b = Placement::new(Mechanism::Dust, Pos::new(1, 0, 0), "n");
        assert_eq!(interferes(&a, &b, &g), None);
    }

    #[test]
    fn can_occupy_wants_a_sturdy_support_under_dust() {
        let empty = view(&[]);
        assert!(can_occupy(Mechanism::Dust, Pos::new(0, 0, 0), (1, 0, 0), &empty).is_err());
        let on_stone = view(&[(Pos::new(0, -1, 0), blocks::STONE)]);
        assert!(can_occupy(Mechanism::Dust, Pos::new(0, 0, 0), (1, 0, 0), &on_stone).is_ok());
        // A strong block needs nothing at all under it.
        assert!(can_occupy(Mechanism::StrongBlock, Pos::new(0, 0, 0), (1, 0, 0), &empty).is_ok());
        // An occupied cell is occupied.
        assert!(can_occupy(Mechanism::Dust, Pos::new(0, -1, 0), (1, 0, 0), &on_stone).is_err());
    }

    #[test]
    fn mech_classification_calls_a_plain_solid_inert() {
        assert_eq!(mech_of(blocks::STONE), Mechanism::SolidSupport);
        assert_eq!(mech_of(GLASS), Mechanism::TransparentSupport);
        assert_eq!(mech_of(blocks::DUST), Mechanism::Dust);
        assert_eq!(mech_of(&blocks::repeater("west", 1)), Mechanism::Repeater);
        assert_eq!(mech_of(blocks::TORCH), Mechanism::TorchFloor);
        assert_eq!(mech_of(blocks::LEVER_OFF), Mechanism::Lever);
        assert_eq!(mech_of("minecraft:redstone_block"), Mechanism::RedstoneBlock);
    }

    #[test]
    fn a_repeater_outputs_opposite_its_facing() {
        // `repeater[facing=west]` conducts toward +X.
        assert_eq!(fwd_of(&blocks::repeater("west", 1)), (1, 0, 0));
        assert_eq!(fwd_of(&blocks::repeater("north", 1)), (0, 0, 1));
    }

    #[test]
    fn xform_rotates_the_canonical_frame() {
        assert_eq!(xform((1, 0, 0), (0, 0, 1)), (0, 0, 1));
        assert_eq!(xform((1, 0, 0), (-1, 0, 0)), (-1, 0, 0));
        assert_eq!(xform((0, 0, 1), (0, 0, 1)), (-1, 0, 0));
        assert_eq!(xform((0, -1, 0), (0, 0, -1)), (0, -1, 0));
    }
}
