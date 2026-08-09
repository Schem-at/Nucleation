//! Design: the composition document over cells, loose blocks and buses.
//!
//! See `redstone-eda/DESIGN_SPEC.md` for the co-designed model. A `Design`
//! is a Photoshop-style layer stack over a shared coordinate space:
//!
//! - a **loose block layer** (the base schematic — endpoint hardware placed
//!   with plain `set_block`),
//! - **instance layers** referencing typed cells (schematic + embedded
//!   [`CellContract`]) through a transform,
//! - **bus layers** owning their routed voxel fragments, with per-bus state
//!   `intended | routed | FAILED(reason)` — unroutability is a *state*, not
//!   an exception.
//!
//! `flatten()` collapses the stack into ONE self-describing
//! [`UniversalSchematic`] with a named region per layer and the merged
//! contract embedded — the artifact is itself placeable as a cell.
//!
//! Buses realize the verified vertical 2y-pitch form
//! (`redstone-eda/bus8_*.py`): axis-aligned runs (with one implicit L
//! corner per waypoint pair) between the driver, optional gates and the
//! primary sink, implicit dip-under crossings ported from `bus8_cross.py`
//! v2 as template data, fanout branches for extra sinks and explicit
//! wired-OR merges for extra drivers (`route_bus_or`).
//!
//! Phase 2 Lane A adds the INTERFERENCE model and drag APIs: a spatial
//! [`OccupancyIndex`] (loose blocks + instance footprints + influence
//! halos + routed fragments), [`Design::move_instance`] /
//! [`Design::move_gate`] computing the affected bus set and co-rerouting
//! it deterministically in bounded rounds, and [`Design::check`] gaining
//! STA/skew plus per-bus net-class rule enforcement. Moves always succeed
//! (they are the document's truth); buses fail into a visible
//! `BusState::Failed`, never a half-routed fragment.

use crate::io_contract::{CellContract, IoType, LayoutFunction, NetClassRule, PortDirection};
use crate::routing::engine::blocks as rblocks;
use crate::UniversalSchematic;
use std::collections::{BTreeMap, BTreeSet};

/// Position triple, kept plain so the module stays wasm-safe and
/// serde-friendly.
pub type P3 = (i32, i32, i32);

fn add(a: P3, b: P3) -> P3 {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

fn scale(a: P3, k: i32) -> P3 {
    (a.0 * k, a.1 * k, a.2 * k)
}

/// A cell's TRUE block extent, min and max corner inclusive.
pub struct CellBounds {
    pub min: P3,
    pub max: P3,
}

/// The cell's real occupied extent — NOT `UniversalSchematic::get_bounding_box`.
///
/// `get_bounding_box()` reports the region's allocated envelope, which for a
/// programmatically built schematic is its growth capacity, not its contents: a
/// 10x18x6 cell built with `set_block_from_string` reports
/// `(0,0,0)..(10,65,65)`. [`transform_pos`] derives the rotation's footprint
/// size from that box, so an over-reported extent flings every ROTATED
/// instance — its blocks, its ports, its halo — tens of blocks away from where
/// the user placed it, while rot_y=0 (where only `min` matters) looks fine.
/// That asymmetry is exactly what "the bus fails as soon as I rotate a cell"
/// looks like from the studio.
///
/// Air is excluded so a padded schematic does not inflate the footprint.
fn cell_bounds(sch: &UniversalSchematic) -> CellBounds {
    let mut min: Option<P3> = None;
    let mut max: Option<P3> = None;
    for (bp, bs) in sch.iter_blocks() {
        if bs.to_string().contains("minecraft:air") {
            continue;
        }
        let p = (bp.x, bp.y, bp.z);
        min = Some(match min {
            None => p,
            Some(m) => (m.0.min(p.0), m.1.min(p.1), m.2.min(p.2)),
        });
        max = Some(match max {
            None => p,
            Some(m) => (m.0.max(p.0), m.1.max(p.1), m.2.max(p.2)),
        });
    }
    match (min, max) {
        (Some(min), Some(max)) => CellBounds { min, max },
        _ => {
            // An empty cell: fall back to the declared envelope.
            let bb = sch.get_bounding_box();
            CellBounds {
                min: bb.min,
                max: bb.max,
            }
        }
    }
}

/// The single straight run realizing an axis-aligned anchor pair, or `None`
/// when the pair needs a corner (differs on both horizontal axes, or on y).
fn axis_run(a: P3, b: P3, width: u8) -> Option<RunInfo> {
    if a.1 != b.1 {
        return None;
    }
    if a.2 == b.2 && a.0 != b.0 {
        return Some(RunInfo {
            along_x: true,
            fixed: a.2,
            y0: a.1,
            from: a.0,
            to: b.0,
            width,
        });
    }
    if a.0 == b.0 && a.2 != b.2 {
        return Some(RunInfo {
            along_x: false,
            fixed: a.0,
            y0: a.1,
            from: a.2,
            to: b.2,
            width,
        });
    }
    None
}

/// Dust cells between refresh repeaters. 7 keeps the worst joint-spanning
/// gap (tail + joint column + head = 15) inside dust's 15-cell reach.
const REFRESH_AT: u32 = 7;

/// Dust cells tolerated between refresh stations INSIDE a level shift. A
/// staircase cell cannot host a repeater (a repeater does not power
/// diagonally), so a slope spends the signal 2 cells per level; 12 leaves the
/// exit cell plus a joint column inside dust's 15-cell reach.
const SHIFT_DUST_CAP: u32 = 12;

/// One column of the verified BUS LEVEL-SHIFT tile ([`shift_plan`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ShiftCell {
    /// The dust the stack steps OFF toward the next level.
    Step,
    /// The dust the stack LANDS on, one level away.
    Land,
    /// Flat dust resuming the bus form at the shifted level.
    Flat,
    /// Refresh station entry block (weak-powered by the dust pointing in).
    Entry,
    /// Refresh station repeater, on a conducting floor.
    Rep,
    /// Refresh station exit block (strongly powered: a fresh 15 out).
    Exit,
}

/// The column plan of a `k`-level shift of a dense 2y-pitch stack:
/// `(offset along the axis, level RELATIVE to the entry level, cell)`, plus
/// the dust-since-refresh count on the way out.
///
/// A level costs TWO columns — step off, land — so the slope is 1 y per 2
/// cells. A continuous 1y-per-1x staircase is IMPOSSIBLE for a dense stack:
/// every cell would be both the step-UPPER of the next diagonal (its support
/// must CONDUCT, the diode law) and the cap over bit n-1's in-use lower
/// diagonal (its support must INSULATE, the cut law) — an over-constrained
/// cell, i.e. the geometry itself is wrong. Landing flat for one cell splits
/// those two roles across two columns: THE ALTERNATION.
///
/// Because stairs cannot host repeaters, a station is inserted before any
/// level whose two dust cells would blow [`SHIFT_DUST_CAP`] — which also
/// refreshes a stale arrival on entry. `k` is therefore unbounded.
///
/// Verified in mc-tick by `redstone-eda/bus_levelshift.py`: 8-bit stack,
/// k in {1,2,3,5,8} x {down,up} x {fresh,stale} arrival, walking-ones /
/// all-on / alternating / 8 random patterns, 3040 output checks, zero
/// crosstalk. All-solid, all-glass and swapped-parity variants all FAIL,
/// so the alternation below is load-bearing in both directions.
fn shift_plan(k: u32, down: bool, since0: u32) -> (Vec<(i32, i32, ShiftCell)>, u32) {
    let sgn = if down { -1 } else { 1 };
    let mut cols = Vec::new();
    let (mut o, mut dy, mut since) = (0i32, 0i32, since0);
    for _ in 0..k {
        if since + 2 > SHIFT_DUST_CAP {
            for kind in [ShiftCell::Entry, ShiftCell::Rep, ShiftCell::Exit] {
                cols.push((o, dy, kind));
                o += 1;
            }
            since = 0;
        }
        cols.push((o, dy, ShiftCell::Step));
        o += 1;
        dy += sgn;
        cols.push((o, dy, ShiftCell::Land));
        o += 1;
        since += 2;
    }
    cols.push((o, dy, ShiftCell::Flat));
    (cols, since + 1)
}

/// Cells of straight run a `k`-level shift consumes (entry cell included).
fn shift_len(k: u32, down: bool, since0: u32) -> i32 {
    shift_plan(k, down, since0).0.last().map_or(0, |c| c.0 + 1)
}

/// The MOST cells a `k`-level shift can ever consume.
///
/// The tile's real length depends on how much of the dust budget the leg into
/// it already spent — a stale arrival buys a leading refresh station, three
/// cells the caller cannot know about before planning that leg. Placement is
/// therefore checked against the worst case, so a tile can never grow past the
/// room reserved for it and leave a gap at its exit.
fn shift_len_max(k: u32, down: bool) -> i32 {
    shift_len(k, down, SHIFT_DUST_CAP)
}

/// A library cell: schematic + resolved contract, stored once and shared by
/// every instance that references it.
#[derive(Clone, Debug)]
pub struct CellDef {
    /// The cell body.
    pub schematic: UniversalSchematic,
    /// The resolved contract (embedded metadata or Insign, see
    /// [`UniversalSchematic::resolve_cell_contract`]).
    pub contract: CellContract,
}

/// One placed instance layer: a cell REFERENCE plus a transform.
#[derive(Clone, Debug)]
pub struct Instance {
    /// Layer name (region `inst:{name}` on flatten).
    pub name: String,
    /// Key into the design's cell library.
    pub cell: String,
    /// Translation applied after rotation.
    pub at: P3,
    /// Y rotation in degrees (0/90/180/270), about the cell's min corner.
    pub rot_y: i32,
    /// Per-port mode overrides. A port absent here is in
    /// [`PortMode::Executor`]; see [`Design::set_port_mode`].
    pub port_modes: BTreeMap<String, PortOverride>,
}

/// How a port presents itself. The two modes are mutually exclusive by
/// physics, not by policy: a lever cannot be driven by redstone, and dust
/// cannot be flipped by a player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortMode {
    /// The shipped hardware (levers/buttons in, lamps out). Drivable by
    /// `CellExecutor`; NOT routable.
    Executor,
    /// Promoted: the hardware is replaced by a driver stub ending in dust.
    /// Routable; no longer hand-drivable.
    Bus,
}

impl PortMode {
    /// `"executor"` / `"bus"`.
    pub fn as_str(self) -> &'static str {
        match self {
            PortMode::Executor => "executor",
            PortMode::Bus => "bus",
        }
    }

    /// Parse `"executor"` / `"bus"`.
    pub fn parse(s: &str) -> Option<PortMode> {
        match s {
            "executor" => Some(PortMode::Executor),
            "bus" => Some(PortMode::Bus),
            _ => None,
        }
    }
}

/// A port's remembered BOTH-FORMS state: the current mode plus the reversible
/// patch that realizes [`PortMode::Bus`].
///
/// The patch is kept even in `Executor` mode so toggling is instant and a UI
/// can say in advance what promoting would change. The cell library itself is
/// never mutated, which is what makes Bus -> Executor a byte-exact undo.
#[derive(Clone, Debug)]
pub struct PortOverride {
    pub mode: PortMode,
    pub patch: crate::design_promote::PortPatch,
}

/// Scanned per-bit hardware capabilities of a port (derived, not declared).
#[derive(Clone, Debug, Default)]
pub struct BitHardware {
    /// The connection cell holds dust: a router may land here.
    pub connectable: bool,
    /// An adjacent lever: the executor can drive this bit (lever position).
    pub lever: Option<P3>,
    /// A lamp supports/neighbours the cell: the bit is human-readable.
    pub lamp: Option<P3>,
}

/// A design port: named typed geometry. The anchor is the bit-0
/// CONNECTION CELL; capabilities are derived by hardware scan and validated
/// loudly at declaration.
#[derive(Clone, Debug)]
pub struct DesignPort {
    pub name: String,
    /// Bit-0 connection cell.
    pub anchor: P3,
    /// Step from bit k to bit k+1.
    pub step: P3,
    /// Logical width in bits (Binary1PerWire in Phase 1: one wire per bit).
    pub width: u8,
    /// Semantic word type (bit order: position order, LSB first).
    pub ty: IoType,
    /// Signal direction as seen from the design.
    pub direction: PortDirection,
    /// Scanned hardware per bit, in bit order.
    pub bits: Vec<BitHardware>,
}

impl DesignPort {
    /// Connection cell of bit `k`.
    pub fn wire(&self, k: u8) -> P3 {
        add(self.anchor, scale(self.step, k as i32))
    }

    /// All connection cells in bit order.
    pub fn wires(&self) -> Vec<P3> {
        (0..self.width).map(|k| self.wire(k)).collect()
    }
}

/// Neighbour scan order of a connection cell (deterministic: the first dust
/// found in this order is the cell's tap).
const AROUND: [P3; 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 0, 1),
    (0, 0, -1),
    (0, 1, 0),
    (0, -1, 0),
];

/// A routing endpoint contributed by a placed instance, named
/// `{instance}.{port}`.
///
/// Instance ports are DERIVED, never declared: the cell's contract port
/// transformed by the instance transform, plus the dust CONNECTION CELLS a
/// bus can actually land on. A cell contract stores *executor-facing*
/// hardware (levers/buttons for its inputs, lamps for its outputs) while a
/// bus terminates on *dust*, so each bit's connection cell is found by
/// hardware scan: the contract position itself when it already holds dust,
/// else the first dust neighbour in [`AROUND`] order.
///
/// A port with no dust tap on every bit is reported `blocked` rather than
/// silently mis-routed — a lever input, for instance, can only ever be
/// driven by the executor, never by a bus.
#[derive(Clone, Debug)]
pub struct InstancePort {
    /// `{instance}.{port}` — the name [`Design::route_bus`] accepts.
    pub name: String,
    pub instance: String,
    pub port: String,
    /// Direction as the CELL sees it: an `Output` drives the fabric, an
    /// `Input` receives from it (the design-facing direction is the flip,
    /// see [`Design::resolve_port`]).
    pub cell_direction: PortDirection,
    pub ty: IoType,
    pub width: u8,
    /// Executor-facing hardware in bit order, transformed.
    pub hardware: Vec<P3>,
    /// Dust connection cells in bit order, when every bit has one.
    pub wires: Option<Vec<P3>>,
    /// Step between consecutive connection cells, when uniform.
    pub step: Option<P3>,
    /// Why a bus cannot terminate here; `None` when it can.
    pub blocked: Option<String>,
}

impl InstancePort {
    /// A bus may terminate on this port.
    pub fn routable(&self) -> bool {
        self.blocked.is_none()
    }

    /// `{"name","instance","port","role","ty","width","hardware","wires",
    ///   "step","routable","blocked"}` — `role` is the CELL-facing
    /// direction (`"output"` drives a bus, `"input"` receives one).
    pub fn to_json(&self) -> String {
        let pos = |ps: &[P3]| {
            let items: Vec<String> = ps
                .iter()
                .map(|p| format!("[{},{},{}]", p.0, p.1, p.2))
                .collect();
            format!("[{}]", items.join(","))
        };
        format!(
            "{{\"name\":{:?},\"instance\":{:?},\"port\":{:?},\"role\":{:?},\"ty\":{},\
             \"width\":{},\"hardware\":{},\"wires\":{},\"step\":{},\"routable\":{},\
             \"blocked\":{}}}",
            self.name,
            self.instance,
            self.port,
            match self.cell_direction {
                PortDirection::Input => "input",
                PortDirection::Output => "output",
            },
            serde_json::to_string(&self.ty).unwrap_or_else(|_| "null".to_string()),
            self.width,
            pos(&self.hardware),
            self.wires.as_ref().map(|w| pos(w)).unwrap_or("null".into()),
            self.step
                .map(|s| format!("[{},{},{}]", s.0, s.1, s.2))
                .unwrap_or("null".into()),
            self.routable(),
            self.blocked
                .as_ref()
                .map(|b| format!("{b:?}"))
                .unwrap_or("null".into()),
        )
    }
}

/// Per-bus material style. The transparent block is used ONLY where a
/// diagonal must survive (the `materials.py` predicate model).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BusStyle {
    /// Solid conductor used for supports, separators and station blocks.
    pub bus_block: String,
    /// Non-conducting sturdy support for in-use diagonals (default glass).
    pub transparent_block: String,
}

impl Default for BusStyle {
    fn default() -> Self {
        BusStyle {
            bus_block: "minecraft:gray_concrete".to_string(),
            transparent_block: "minecraft:glass".to_string(),
        }
    }
}

impl BusStyle {
    /// Loud style validation: the bus block must be a sturdy conductor, the
    /// transparent block a sturdy NON-conductor (the dip's diode law and
    /// diagonal-survival both depend on it).
    pub fn validate(&self) -> Result<(), String> {
        if !rblocks::is_solid_block(&self.bus_block) {
            return Err(format!(
                "bus_block `{}` is not a solid conductor (stone/concrete/lamp family)",
                self.bus_block
            ));
        }
        if !rblocks::is_sturdy_support(&self.transparent_block)
            || rblocks::is_solid_block(&self.transparent_block)
        {
            return Err(format!(
                "transparent_block `{}` must be a sturdy non-conductor (glass family / top slab)",
                self.transparent_block
            ));
        }
        Ok(())
    }
}

/// How a narrower word is placed inside a wider one.
///
/// A width mismatch is a LAYOUT question, not an error: 3 BCD-hundreds bits
/// genuinely fit inside a 4-bit `bcd` input, and which 3 of the 4 they drive is
/// the only thing anyone has to decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BusAlign {
    /// Bit 0 to bit 0 — the arithmetic default (a narrow value keeps its
    /// magnitude; the destination's high bits read 0).
    #[default]
    Lsb,
    /// Top bit to top bit: the source is shifted up by the width difference.
    /// This is a MULTIPLY BY 2^(ws-wd), not a reinterpretation — it is what you
    /// want when the two words are fixed-point fields, not integers.
    Msb,
    /// Place the source word `n` positions toward the MSB. Negative shifts
    /// down (and drops the bits that fall off, which needs `truncate`).
    Shift(i32),
}

/// The width-adaptation policy for one bus.
#[derive(Clone, Copy, Debug, Default)]
pub struct WidthAdapt {
    /// Where the source word sits in the destination.
    pub align: BusAlign,
    /// Permit DROPPING source bits that fall outside the destination.
    /// Refused by default: silently losing the high bits of a word is the kind
    /// of thing a router must make you ask for.
    pub truncate: bool,
}

impl WidthAdapt {
    /// LSB-aligned, no truncation — what [`Design::route_bus`] uses.
    pub fn lsb() -> Self {
        Self::default()
    }

    /// MSB-aligned.
    pub fn msb() -> Self {
        WidthAdapt {
            align: BusAlign::Msb,
            truncate: false,
        }
    }

    /// Shifted `n` toward the MSB.
    pub fn shift(n: i32) -> Self {
        WidthAdapt {
            align: BusAlign::Shift(n),
            truncate: false,
        }
    }

    /// The same policy, allowed to drop source bits.
    pub fn truncating(mut self) -> Self {
        self.truncate = true;
        self
    }
}

/// What a bus actually did about a width mismatch: the resolved bit mapping,
/// recorded so LVS pairs the right bits and a UI can show the wiring.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidthMap {
    /// Driver word width.
    pub driver_width: u8,
    /// Sink word width.
    pub sink_width: u8,
    /// Positions the source is moved toward the MSB: driver bit `i` drives sink
    /// bit `i + shift`.
    pub shift: i32,
    /// First driver bit that is actually connected.
    pub from_bit: u8,
    /// How many bits are connected (the routed stack's width).
    pub bits: u8,
    /// Sink bits nothing drives. Undriven dust IS logical 0 — no hardware is
    /// needed to tie them, which is verified in
    /// `tests/design_width_adapt.rs::an_undriven_promoted_input_reads_zero`.
    pub tied_zero: Vec<u8>,
    /// Driver bits DROPPED (only ever non-empty with `truncate`).
    pub dropped: Vec<u8>,
}

impl WidthMap {
    /// Whether this is the trivial identity (equal widths, nothing to say).
    pub fn is_identity(&self) -> bool {
        self.driver_width == self.sink_width && self.shift == 0 && self.dropped.is_empty()
    }

    /// The sentence to show the user.
    pub fn note(&self, driver: &str, sink: &str) -> String {
        let mut s = format!(
            "{driver}[{}] -> {sink}[{}], {}",
            self.driver_width,
            self.sink_width,
            match self.shift {
                0 => "lsb-aligned".to_string(),
                n if n > 0 => format!("shifted {n} toward the msb"),
                n => format!("shifted {} toward the lsb", -n),
            }
        );
        if !self.tied_zero.is_empty() {
            s.push_str(&format!(
                "; sink bit(s) {} left undriven, which reads 0",
                ranges(&self.tied_zero)
            ));
        }
        if !self.dropped.is_empty() {
            s.push_str(&format!(
                "; driver bit(s) {} TRUNCATED away",
                ranges(&self.dropped)
            ));
        }
        s
    }

    /// `{"driver_width":n,"sink_width":n,"shift":n,"from_bit":n,"bits":n,
    ///   "tied_zero":[..],"dropped":[..],"pairs":[[dbit,sbit],..]}`
    pub fn to_json(&self) -> String {
        let list = |v: &[u8]| -> String {
            v.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",")
        };
        let pairs: Vec<String> = (0..self.bits)
            .map(|k| {
                let d = self.from_bit + k;
                format!("[{},{}]", d, d as i32 + self.shift)
            })
            .collect();
        format!(
            "{{\"driver_width\":{},\"sink_width\":{},\"shift\":{},\"from_bit\":{},\
             \"bits\":{},\"tied_zero\":[{}],\"dropped\":[{}],\"pairs\":[{}]}}",
            self.driver_width,
            self.sink_width,
            self.shift,
            self.from_bit,
            self.bits,
            list(&self.tied_zero),
            list(&self.dropped),
            pairs.join(",")
        )
    }
}

/// Whether two types differ ONLY in width, so a resolved width adaptation makes
/// them compatible. Two integer words of different widths are different
/// `IoType`s by construction, which is exactly the case adaptation exists for;
/// an int and a float, or an int and a string, are not.
fn same_type_family(a: &IoType, b: &IoType) -> bool {
    matches!(
        (a, b),
        (IoType::UnsignedInt { .. }, IoType::UnsignedInt { .. })
            | (IoType::SignedInt { .. }, IoType::SignedInt { .. })
            | (IoType::UnsignedInt { .. }, IoType::Boolean)
            | (IoType::Boolean, IoType::UnsignedInt { .. })
    )
}

/// `[0,1,2,5]` as `"0..2, 5"` — a bit list a human can read.
fn ranges(bits: &[u8]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < bits.len() {
        let start = bits[i];
        let mut end = start;
        while i + 1 < bits.len() && bits[i + 1] == end + 1 {
            i += 1;
            end = bits[i];
        }
        out.push(if start == end {
            format!("{start}")
        } else {
            format!("{start}..{end}")
        });
        i += 1;
    }
    out.join(", ")
}

/// Resolve a width mismatch into a bit mapping, or say why it cannot be.
fn plan_width_map(
    driver_width: u8,
    sink_width: u8,
    adapt: WidthAdapt,
) -> Result<WidthMap, String> {
    let (wd, ws) = (driver_width as i32, sink_width as i32);
    let shift = match adapt.align {
        BusAlign::Lsb => 0,
        BusAlign::Msb => ws - wd,
        BusAlign::Shift(n) => n,
    };
    // Driver bits whose destination exists.
    let from = 0.max(-shift);
    let to = wd.min(ws - shift);
    if to <= from {
        return Err(format!(
            "a {wd}-bit word shifted {shift} lands entirely outside a {ws}-bit destination, so              there is nothing to connect"
        ));
    }
    let dropped: Vec<u8> = (0..wd)
        .filter(|i| *i < from || *i >= to)
        .map(|i| i as u8)
        .collect();
    if !dropped.is_empty() && !adapt.truncate {
        return Err(format!(
            "connecting a {wd}-bit driver to a {ws}-bit sink shifted {shift} would drop bits {} of the driver word — losing the high bits silently is not something a router should decide, so pass truncate to accept it (or align/shift so it fits)",
            ranges(&dropped)
        ));
    }
    let tied_zero: Vec<u8> = (0..ws)
        .filter(|j| *j < from + shift || *j >= to + shift)
        .map(|j| j as u8)
        .collect();
    Ok(WidthMap {
        driver_width,
        sink_width,
        shift,
        from_bit: from as u8,
        bits: (to - from) as u8,
        tied_zero,
        dropped,
    })
}

/// A bus-shaped waypoint splitting the bus into independently-routed
/// segments.
#[derive(Clone, Debug)]
pub struct Gate {
    pub name: String,
    /// Bit-0 cell of the gate column.
    pub anchor: P3,
    /// Step between bits (must match the bus form).
    pub step: P3,
}

/// Lifecycle state of a bus layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BusState {
    /// Declared, not yet realized.
    Intended,
    /// Realized; the fragment is committed.
    Routed,
    /// Realization failed; the reason is user-facing. The workspace was
    /// left untouched (atomic realization).
    Failed(String),
}

/// One straight realized run of a bus (crossing detection input for buses
/// routed later).
#[derive(Clone, Debug)]
pub struct RunInfo {
    /// `true`: the run varies along X (fixed z). `false`: along Z (fixed x).
    pub along_x: bool,
    /// The fixed cross-axis coordinate.
    pub fixed: i32,
    /// Bit-0 canonical dust level.
    pub y0: i32,
    /// Start coordinate along the axis (the driver-side anchor).
    pub from: i32,
    /// End coordinate along the axis (the sink-side anchor).
    pub to: i32,
    /// Bus width.
    pub width: u8,
}

impl RunInfo {
    fn sign(&self) -> i32 {
        if self.to >= self.from {
            1
        } else {
            -1
        }
    }

    /// Whether `c` lies strictly between the run's anchors with `margin`
    /// cells of slack on each side.
    fn strictly_inside(&self, c: i32, margin: i32) -> bool {
        let (lo, hi) = if self.from <= self.to {
            (self.from, self.to)
        } else {
            (self.to, self.from)
        };
        c - lo >= margin && hi - c >= margin
    }
}

/// Which part of a bus a routed [`Segment`] realizes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentKind {
    /// Trunk waypoint pair `i`: `waypoints[i] -> waypoints[i+1]` (driver,
    /// gates, primary sink).
    Trunk(usize),
    /// A branch tying the named extra endpoint (fanout sink, or wired-OR
    /// driver) into the trunk at a dust junction.
    Branch(String),
    /// A FORM ADAPTER for the named port: the row->stack pivot that lets a bus
    /// terminate on a port whose native geometry is a horizontal row.
    ///
    /// It belongs to the BUS, not to the component: promotion is minimal and
    /// in-place (the port keeps its native form), and the adapter is created
    /// and RIPPED with the bus. Owning it in a per-instance promotion patch —
    /// which is where it used to live — left it behind as orphaned geometry
    /// when the bus was ripped.
    Adapter(String),
}

/// One independently-routed piece of a bus: a trunk waypoint pair
/// (straight, or L-shaped through one implicit corner) or a branch.
/// Segments are the rip granularity of the drag APIs: `move_gate` rips
/// exactly the two segments adjacent to the gate.
#[derive(Clone, Debug)]
pub struct Segment {
    pub kind: SegmentKind,
    /// Bit-0 anchor the segment starts from.
    pub a: P3,
    /// Bit-0 anchor the segment ends at.
    pub b: P3,
    /// The straight runs realizing the segment (1, or 2 through a corner).
    pub runs: Vec<RunInfo>,
    /// Cells owned by the segment (subset of the bus fragment).
    pub cells: BTreeSet<P3>,
}

/// A bus layer: endpoints with roles, gates, style, state, and the OWNED
/// voxel fragment.
#[derive(Clone, Debug)]
pub struct BusLayer {
    pub name: String,
    /// Primary driver port name.
    pub driver: String,
    /// Additional drivers — legal only with `merge_or` (wired-OR), realized
    /// as dust-merge joins into the trunk; LVS intent stays ONE net.
    pub extra_drivers: Vec<String>,
    /// `true` when multiple drivers were explicitly declared as a wired-OR
    /// merge (`merge="or"`); multiple drivers without it are rejected.
    pub merge_or: bool,
    /// Sink port names.
    pub sinks: Vec<String>,
    pub gates: Vec<Gate>,
    pub style: BusStyle,
    pub state: BusState,
    /// The owned voxel fragment (block per cell), empty unless `Routed`.
    pub fragment: BTreeMap<P3, String>,
    /// The straight runs realized (crossing detection input; the union of
    /// every segment's runs).
    pub runs: Vec<RunInfo>,
    /// Per-segment realization (rip granularity for the drag APIs). Empty
    /// on documents loaded from formats predating segments — the drag APIs
    /// then fall back to a full reroute.
    pub segments: Vec<Segment>,
    /// The electrical joint columns of the gates, per gate name.
    pub gate_cells: BTreeMap<String, BTreeSet<P3>>,
    /// Optional net-class discipline enforced by [`Design::check`]
    /// (`max_len_rt` delay budget, `y_band` layer assignment).
    pub rule: Option<NetClassRule>,
    /// Ports this bus PROMOTED for itself on the way in, one human-readable
    /// note each (see [`Design::set_auto_promote`]). Non-empty means routing
    /// changed instance hardware, which the studio has to tell the user about —
    /// it is reversible, but it is not nothing.
    pub promotions: Vec<String>,
    /// The resolved bit mapping when the driver and sink widths DIFFER — see
    /// [`WidthMap`]. `None` means the widths matched and every bit pairs with
    /// its own index.
    pub width_map: Option<WidthMap>,
}

impl BusLayer {
    /// Every driver name, primary first.
    pub fn driver_names(&self) -> Vec<String> {
        let mut v = vec![self.driver.clone()];
        v.extend(self.extra_drivers.iter().cloned());
        v
    }
}

/// What occupies a cell in the design-wide spatial index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Occupant {
    /// The loose block layer (the base schematic).
    Loose,
    /// An instance layer's transformed footprint.
    Instance(String),
    /// A routed bus's owned fragment.
    Bus(String),
}

/// The spatial occupancy index: hard cells (loose blocks, instance
/// footprints, routed bus fragments) plus instance influence halos
/// ([`crate::io_contract::PhysicalContract`] keepouts, or the cell bounds
/// grown by one where a cell declares none). Routing refuses halo cells;
/// the drag APIs use halos to compute the affected bus set.
#[derive(Clone, Debug, Default)]
pub struct OccupancyIndex {
    /// Hard cells: block state + owner.
    pub cells: BTreeMap<P3, (String, Occupant)>,
    /// Influence halo cells -> owning instance name (never hard-occupied).
    pub halos: BTreeMap<P3, String>,
}

/// One cell a port-mode switch rewrote, with both sides of the change so a UI
/// can say exactly what happened ("removed lever at (19,5,5)").
#[derive(Clone, Debug)]
pub struct PortModeChange {
    /// World position (the instance transform already applied).
    pub at: P3,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Outcome of [`Design::set_port_mode`].
#[derive(Clone, Debug)]
pub struct PortModeReport {
    /// `{instance}.{port}`.
    pub port: String,
    /// The mode now in effect.
    pub mode: PortMode,
    /// Every cell the switch rewrote, in position order.
    pub changed: Vec<PortModeChange>,
    /// Buses deleted because they terminated on this port.
    pub removed_buses: Vec<String>,
    /// Reroute outcome for buses that merely crossed the changed space.
    pub moves: MoveReport,
    /// The reversible patch, as JSON.
    pub patch_json: String,
    /// One human sentence, ready for a toast.
    pub note: String,
}

impl PortModeReport {
    /// `{"port":..,"mode":..,"note":..,"changed":[{"at":[x,y,z],"from":..,
    ///   "to":..}],"removed_buses":[..],"moves":{..},"patch":{..}}`
    pub fn to_json(&self) -> String {
        let changed: Vec<String> = self
            .changed
            .iter()
            .map(|c| {
                format!(
                    "{{\"at\":[{},{},{}],\"from\":{},\"to\":{}}}",
                    c.at.0,
                    c.at.1,
                    c.at.2,
                    c.from.as_ref().map(|s| format!("{s:?}")).unwrap_or("null".into()),
                    c.to.as_ref().map(|s| format!("{s:?}")).unwrap_or("null".into()),
                )
            })
            .collect();
        let removed: Vec<String> = self.removed_buses.iter().map(|n| format!("{n:?}")).collect();
        format!(
            "{{\"port\":{:?},\"mode\":{:?},\"note\":{:?},\"changed\":[{}],\
             \"removed_buses\":[{}],\"moves\":{},\"patch\":{}}}",
            self.port,
            self.mode.as_str(),
            self.note,
            changed.join(","),
            removed.join(","),
            self.moves.to_json(),
            self.patch_json,
        )
    }
}

/// Outcome of a drag ([`Design::move_instance`]): the move itself always
/// succeeds (the document's truth); buses fail VISIBLY, never half-routed.
#[derive(Clone, Debug, Default)]
pub struct MoveReport {
    /// Buses ripped and successfully co-rerouted, in name order.
    pub rerouted: Vec<String>,
    /// Buses left in `FAILED(reason)` after the bounded negotiation.
    pub failed: Vec<(String, String)>,
    /// EVERY bus layer whose realized geometry was rewritten, in name order —
    /// the studio's redraw set. See [`Design::changed_layers_since`] for the
    /// guarantee: it is a superset of `rerouted` + `failed`, because it also
    /// names buses amended INDIRECTLY (a crossing station stamped into a bus
    /// that was never ripped) and buses that were deleted.
    pub changed: Vec<String>,
}

impl MoveReport {
    /// JSON: `{"rerouted": [...], "failed": {name: reason}}`.
    pub fn to_json(&self) -> String {
        let r: Vec<String> = self.rerouted.iter().map(|n| format!("{n:?}")).collect();
        let f: Vec<String> = self
            .failed
            .iter()
            .map(|(n, why)| format!("{n:?}:{why:?}"))
            .collect();
        let c: Vec<String> = self.changed.iter().map(|n| format!("{n:?}")).collect();
        format!(
            "{{\"rerouted\":[{}],\"failed\":{{{}}},\"changed\":[{}]}}",
            r.join(","),
            f.join(","),
            c.join(",")
        )
    }
}

/// Outcome of [`Design::remove_instance`].
#[derive(Clone, Debug)]
pub struct RemoveReport {
    /// Buses deleted because they terminated on the removed instance.
    pub removed_buses: Vec<String>,
    /// Reroute outcome for the buses that merely crossed its space.
    pub moves: MoveReport,
}

impl RemoveReport {
    /// `{"removed_buses":[...],"rerouted":[...],"failed":{...}}`.
    pub fn to_json(&self) -> String {
        let removed: Vec<String> = self
            .removed_buses
            .iter()
            .map(|b| format!("{b:?}"))
            .collect();
        let moves = self.moves.to_json();
        format!(
            "{{\"removed_buses\":[{}],{}",
            removed.join(","),
            &moves[1..]
        )
    }
}

/// Outcome of [`Design::move_gate`].
#[derive(Clone, Debug)]
pub struct GateMoveReport {
    /// The bus state after the move.
    pub state: BusState,
    /// Segments ripped and rerouted (2 for a partial gate drag; the full
    /// segment count when the bus needed a whole-bus reroute).
    pub rerouted_segments: usize,
    /// Every bus layer whose geometry was rewritten — the dragged bus, plus
    /// any bus a crossing amendment touched. See
    /// [`Design::changed_layers_since`].
    pub changed: Vec<String>,
}

/// Outcome of [`Design::check`].
#[derive(Clone, Debug)]
pub struct DesignCheck {
    /// No DRC violations, no LVS opens/shorts/cycles.
    pub clean: bool,
    /// Full JSON report: `{"clean", "drc": [...], "lvs": {...}}`.
    pub json: String,
}

/// The composition document. See module docs.
#[derive(Clone, Debug)]
pub struct Design {
    name: String,
    base: UniversalSchematic,
    cells: BTreeMap<String, CellDef>,
    instances: Vec<Instance>,
    ports: BTreeMap<String, DesignPort>,
    buses: BTreeMap<String, BusLayer>,
    auto_promote: bool,
    /// GEOMETRY REVISION per bus layer — see [`Design::changed_layers_since`].
    /// Runtime-only (never serialized): a reloaded document starts every layer
    /// at revision 1, which is correct, because a fresh reader has drawn
    /// nothing yet.
    bus_revs: BTreeMap<String, u64>,
    /// Monotonic clock stamped into `bus_revs`.
    rev_clock: u64,
}

impl Design {
    /// An empty design.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Design {
            base: UniversalSchematic::new(name.clone()),
            name,
            cells: BTreeMap::new(),
            instances: Vec::new(),
            ports: BTreeMap::new(),
            buses: BTreeMap::new(),
            auto_promote: true,
            bus_revs: BTreeMap::new(),
            rev_clock: 0,
        }
    }

    // ------------------------------------------------------------------
    // The CHANGED-LAYER CONTRACT
    // ------------------------------------------------------------------

    /// The current geometry revision. Read it BEFORE a mutating call, pass it
    /// to [`Design::changed_layers_since`] after, and redraw exactly the
    /// layers it names.
    pub fn layer_revision(&self) -> u64 {
        self.rev_clock
    }

    /// Bus layers whose realized geometry may have been REWRITTEN since
    /// revision `rev`, in name order. This is the studio's redraw set, and it
    /// is **complete by construction**: the revision is stamped at every
    /// single write to a layer's fragment, so no operation can change a layer
    /// without naming it. Specifically it includes
    ///
    /// - the bus an operation was aimed at, whether it ended `Routed` or
    ///   `FAILED` (both directions of the transition are stamped);
    /// - every bus RIPPED and co-rerouted because a moved instance's
    ///   footprint, influence halo, or PORTS touched it;
    /// - every bus amended INDIRECTLY — a crossing stamps a through-bus
    ///   station into a bus that was never ripped and is not otherwise named
    ///   in any report. Missing these is the classic "I moved a component and
    ///   the bus didn't update" stale-mesh bug;
    /// - every bus DELETED (a layer named here that [`Design::bus`] no longer
    ///   knows is a removal: drop the mesh).
    ///
    /// It may over-report: a bus ripped and re-routed to byte-identical
    /// geometry is still named. Redrawing it is wasted work, never a wrong
    /// picture, and `tests/design_reroute_stress.rs` pins the guarantee by
    /// comparing this set against a block-by-block diff.
    pub fn changed_layers_since(&self, rev: u64) -> Vec<String> {
        self.bus_revs
            .iter()
            .filter(|(_, r)| **r > rev)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Stamp a layer as rewritten. Called from EVERY write to a bus
    /// fragment — including the indirect ones (crossing amendments) — so
    /// [`Design::changed_layers_since`] cannot under-report.
    fn touch_bus(&mut self, name: &str) {
        self.rev_clock += 1;
        self.bus_revs.insert(name.to_string(), self.rev_clock);
    }

    /// Every bus layer's realized geometry — the independent oracle the
    /// changed-layer contract is tested against (and a cheap way for a caller
    /// that would rather diff than track revisions).
    pub fn bus_geometry(&self) -> BTreeMap<String, BTreeMap<P3, String>> {
        self.buses
            .iter()
            .map(|(n, b)| (n.clone(), b.fragment.clone()))
            .collect()
    }

    /// Bus layers whose blocks actually differ from a [`Design::bus_geometry`]
    /// snapshot, in name order — appearances and removals included.
    pub fn layers_differing_from(
        &self,
        before: &BTreeMap<String, BTreeMap<P3, String>>,
    ) -> Vec<String> {
        let now = self.bus_geometry();
        let mut out: BTreeSet<String> = BTreeSet::new();
        for (n, frag) in &now {
            if before.get(n) != Some(frag) {
                out.insert(n.clone());
            }
        }
        for n in before.keys() {
            if !now.contains_key(n) {
                out.insert(n.clone());
            }
        }
        out.into_iter().collect()
    }

    /// A design whose loose block layer is `base` (endpoint hardware placed
    /// with raw `set_block`, the `design_step1.py` workflow).
    pub fn for_schematic(name: impl Into<String>, base: UniversalSchematic) -> Self {
        let mut d = Design::new(name);
        d.base = base;
        d
    }

    /// Does [`Design::route_bus`] promote executor-only endpoints by itself?
    /// On by default.
    pub fn auto_promote(&self) -> bool {
        self.auto_promote
    }

    /// Turn automatic promotion off (or back on).
    ///
    /// With it ON — the default — routing to a community cell's LEVER input
    /// just works: `route_bus` switches the port to [`PortMode::Bus`] first and
    /// records what it did. With it OFF, such a bus is refused with the
    /// instruction to promote explicitly, which is what a UI wants when the
    /// user must confirm a hardware change before it happens.
    pub fn set_auto_promote(&mut self, on: bool) {
        self.auto_promote = on;
    }

    /// The design name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The loose block layer.
    pub fn base(&self) -> &UniversalSchematic {
        &self.base
    }

    /// A declared port.
    pub fn port(&self, name: &str) -> Option<&DesignPort> {
        self.ports.get(name)
    }

    /// A bus layer.
    pub fn bus(&self, name: &str) -> Option<&BusLayer> {
        self.buses.get(name)
    }

    /// The resolved bit mapping of a width-adapted bus, as JSON, plus the
    /// sentence to show the user — `null` when the widths matched.
    ///
    /// `{"map":{...},"note":"u1.bcd_hundreds[3] -> u3.bcd[4], msb-aligned; sink
    /// bit(s) 0 left undriven, which reads 0"}`
    pub fn bus_width_map_json(&self, name: &str) -> Result<String, String> {
        let bus = self
            .buses
            .get(name)
            .ok_or_else(|| format!("unknown bus `{name}`"))?;
        Ok(match &bus.width_map {
            None => "null".to_string(),
            Some(m) => format!(
                "{{\"map\":{},\"note\":{:?}}}",
                m.to_json(),
                m.note(&bus.driver, bus.sinks.first().map(String::as_str).unwrap_or("?"))
            ),
        })
    }

    /// The state of a bus layer.
    pub fn bus_state(&self, name: &str) -> Option<&BusState> {
        self.buses.get(name).map(|b| &b.state)
    }

    // ------------------------------------------------------------------
    // Cells + instances
    // ------------------------------------------------------------------

    /// Register a cell in the design's library. The contract is resolved
    /// from the schematic itself (embedded metadata first, Insign signs as
    /// fallback); registration fails loudly when no source defines one.
    /// Returns resolution warnings (source conflicts).
    pub fn add_cell(
        &mut self,
        name: impl Into<String>,
        schematic: UniversalSchematic,
    ) -> Result<Vec<String>, String> {
        let name = name.into();
        let (contract, warnings) = schematic
            .resolve_cell_contract()?
            .ok_or_else(|| format!("cell `{name}`: no contract (embedded metadata or Insign)"))?;
        self.cells.insert(
            name,
            CellDef {
                schematic,
                contract,
            },
        );
        Ok(warnings)
    }

    /// Register a cell with an explicitly provided contract (the explicit
    /// API wins over every embedded source).
    pub fn add_cell_with_contract(
        &mut self,
        name: impl Into<String>,
        schematic: UniversalSchematic,
        contract: CellContract,
    ) {
        self.cells.insert(
            name.into(),
            CellDef {
                schematic,
                contract,
            },
        );
    }

    /// Place an instance layer referencing a library cell. `rot_y` is in
    /// degrees, a multiple of 90, applied about the cell's min corner
    /// before translating to `at`. Two instances may reference one cell.
    pub fn place(
        &mut self,
        name: impl Into<String>,
        cell: &str,
        at: P3,
        rot_y: i32,
    ) -> Result<(), String> {
        let name = name.into();
        if !self.cells.contains_key(cell) {
            return Err(format!("unknown cell `{cell}`"));
        }
        if rot_y.rem_euclid(90) != 0 {
            return Err(format!("rot_y must be a multiple of 90, got {rot_y}"));
        }
        if self.instances.iter().any(|i| i.name == name) {
            return Err(format!("instance `{name}` already exists"));
        }
        self.instances.push(Instance {
            name,
            cell: cell.to_string(),
            at,
            rot_y: rot_y.rem_euclid(360),
            port_modes: BTreeMap::new(),
        });
        Ok(())
    }

    // ------------------------------------------------------------------
    // Port modes (promotion): executor hardware <-> routable dust
    // ------------------------------------------------------------------

    /// The mode a port currently presents in ([`PortMode::Executor`] unless it
    /// has been switched).
    pub fn port_mode(&self, instance: &str, port: &str) -> PortMode {
        self.instances
            .iter()
            .find(|i| i.name == instance)
            .and_then(|i| i.port_modes.get(port))
            .map(|o| o.mode)
            .unwrap_or(PortMode::Executor)
    }

    /// Switch a port between executor hardware and a routable dust input.
    ///
    /// This is the composability switch: community cells name LEVERS for their
    /// inputs, and nothing in redstone drives a lever, so `add.sum -> bcd.bin`
    /// is impossible until `bin` is in [`PortMode::Bus`]. The conversion is a
    /// reversible per-instance patch (see [`crate::design_promote`]) — the cell
    /// library is untouched, so switching back restores the original blocks
    /// byte-exactly.
    ///
    /// Buses attached to the port are RIPPED (their endpoint physically stops
    /// existing) and named in the report; every other affected bus is
    /// co-rerouted exactly as for a drag, so a toggle never leaves stale
    /// geometry.
    pub fn set_port_mode(
        &mut self,
        instance: &str,
        port: &str,
        mode: PortMode,
    ) -> Result<PortModeReport, String> {
        let idx = self
            .instances
            .iter()
            .position(|i| i.name == instance)
            .ok_or_else(|| format!("unknown instance `{instance}`"))?;
        if self.port_mode(instance, port) == mode {
            return Ok(PortModeReport {
                port: format!("{instance}.{port}"),
                mode,
                changed: Vec::new(),
                removed_buses: Vec::new(),
                moves: MoveReport::default(),
                patch_json: self.instances[idx]
                    .port_modes
                    .get(port)
                    .map(|o| o.patch.to_json())
                    .unwrap_or_else(|| "null".to_string()),
                note: format!("`{instance}.{port}` is already in {} mode", mode.as_str()),
            });
        }
        let rev0 = self.layer_revision();
        // Plan the patch once and remember it, so toggling is symmetric.
        if !self.instances[idx].port_modes.contains_key(port) {
            let patch = self.plan_port_patch(instance, port)?;
            self.instances[idx].port_modes.insert(
                port.to_string(),
                PortOverride {
                    mode: PortMode::Executor,
                    patch,
                },
            );
        }
        // Buses that terminate on this port lose their endpoint geometry.
        let full = format!("{instance}.{port}");
        let doomed: Vec<String> = self
            .buses
            .values()
            .filter(|b| {
                b.driver_names()
                    .iter()
                    .chain(b.sinks.iter())
                    .any(|p| *p == full)
            })
            .map(|b| b.name.clone())
            .collect();
        for b in &doomed {
            self.touch_bus(b);
            self.buses.remove(b);
        }
        // Everything that touched the instance's space is re-attempted.
        let old_region = self.instance_region(idx);
        self.instances[idx]
            .port_modes
            .get_mut(port)
            .expect("just inserted")
            .mode = mode;
        let new_region = self.instance_region(idx);
        let mut affected: BTreeSet<String> = BTreeSet::new();
        for bus in self.buses.values() {
            match &bus.state {
                BusState::Routed
                    if bus
                        .fragment
                        .keys()
                        .any(|p| old_region.contains(p) || new_region.contains(p)) =>
                {
                    affected.insert(bus.name.clone());
                }
                BusState::Failed(_) => {
                    affected.insert(bus.name.clone());
                }
                _ => {}
            }
        }
        let moves = self.co_reroute(affected, rev0);
        let over = self.instances[idx].port_modes[port].clone();
        let inst = &self.instances[idx];
        let cell = &self.cells[&inst.cell];
        let bbox = cell_bounds(&cell.schematic);
        let map = |p: P3| transform_pos(p, bbox.min, bbox.max, inst.rot_y, inst.at);
        let mut changed = Vec::new();
        for (p, want) in &over.patch.writes {
            let before = over.patch.saved.get(p).and_then(|o| o.clone());
            let (from, to) = match mode {
                PortMode::Bus => (before, want.clone()),
                PortMode::Executor => (want.clone(), before),
            };
            changed.push(PortModeChange {
                at: map(*p),
                from,
                to,
            });
        }
        let note = match mode {
            PortMode::Bus => format!(
                "`{full}` is now a BUS input: {} — bit 0 lands on dust at {:?}",
                over.patch.note,
                over.patch.wires.first().map(|w| map(*w)).unwrap_or((0, 0, 0))
            ),
            PortMode::Executor => format!(
                "`{full}` is back to EXECUTOR hardware: {} cell(s) restored exactly as shipped",
                over.patch.writes.len()
            ),
        };
        Ok(PortModeReport {
            port: full,
            mode,
            changed,
            removed_buses: doomed,
            moves,
            patch_json: over.patch.to_json(),
            note,
        })
    }

    /// Promote `endpoint` if — and only if — that is what stands between it and
    /// terminating a bus. Returns the note to report, or `None` to leave the
    /// endpoint (and its own error message) exactly as it was.
    ///
    /// Deliberately conservative: it promotes ONLY when the port does not
    /// currently resolve, is still in [`PortMode::Executor`], and promoting
    /// actually makes it resolve. Anything else — a width mismatch, an unknown
    /// port, a ceiling lever that cannot be promoted at all — is left for the
    /// caller's own error path, so auto-promotion can never mask a different
    /// problem or half-apply a patch. A promotion that does not help is rolled
    /// back to `Executor`.
    fn auto_promote_endpoint(&mut self, endpoint: &str) -> Option<String> {
        if self.resolve_port(endpoint).is_ok() {
            return None;
        }
        let (inst, port) = endpoint.split_once('.')?;
        if self.port_mode(inst, port) != PortMode::Executor {
            return None;
        }
        let (inst, port) = (inst.to_string(), port.to_string());
        // Refuse to touch anything unless the patch plans cleanly first.
        self.plan_port_patch(&inst, &port).ok()?;
        let report = self.set_port_mode(&inst, &port, PortMode::Bus).ok()?;
        if self.resolve_port(endpoint).is_err() {
            // Promotion was not the blocker. Put the hardware back.
            let _ = self.set_port_mode(&inst, &port, PortMode::Executor);
            return None;
        }
        Some(format!("auto-promoted `{endpoint}`: {}", report.note))
    }

    /// [`Design::set_port_mode`] to [`PortMode::Bus`] — the "promote this
    /// lever input to something a bus can land on" verb.
    pub fn promote_input(&mut self, instance: &str, port: &str) -> Result<PortModeReport, String> {
        self.set_port_mode(instance, port, PortMode::Bus)
    }

    /// [`Design::set_port_mode`] to [`PortMode::Bus`] for an OUTPUT port: a
    /// dust tap on its lamps, so a bus can pick the value up. The lamps stay,
    /// so the port remains readable through the typed executor too.
    pub fn promote_output(&mut self, instance: &str, port: &str) -> Result<PortModeReport, String> {
        self.set_port_mode(instance, port, PortMode::Bus)
    }

    /// Plan (but do not apply) the Bus-mode patch for a port. Useful for a UI
    /// that wants to describe the change before the user commits to it.
    pub fn plan_port_patch(
        &self,
        instance: &str,
        port: &str,
    ) -> Result<crate::design_promote::PortPatch, String> {
        let inst = self
            .instances
            .iter()
            .find(|i| i.name == instance)
            .ok_or_else(|| format!("unknown instance `{instance}`"))?;
        let cell = &self.cells[&inst.cell];
        let (dirn, mapping) = if let Some(m) = cell.contract.io.inputs.get(port) {
            (PortDirection::Input, m)
        } else if let Some(m) = cell.contract.io.outputs.get(port) {
            (PortDirection::Output, m)
        } else {
            let have: Vec<&str> = cell
                .contract
                .io
                .inputs
                .keys()
                .chain(cell.contract.io.outputs.keys())
                .map(String::as_str)
                .collect();
            return Err(format!(
                "instance `{instance}` (cell `{}`) has no port `{port}` (has: {})",
                inst.cell,
                have.join(", ")
            ));
        };
        match dirn {
            PortDirection::Input => {
                crate::design_promote::plan_input(&cell.schematic, &mapping.positions)
            }
            PortDirection::Output => {
                crate::design_promote::plan_output(&cell.schematic, &mapping.positions)
            }
        }
        .map_err(|e| format!("cannot promote `{instance}.{port}`: {e}"))
    }

    /// Every port's mode as JSON, for a UI:
    /// `[{"name":"u0.bin","mode":"bus","patch":{..}}, ..]`
    pub fn port_modes_json(&self) -> String {
        let mut items = Vec::new();
        for inst in &self.instances {
            for (port, over) in &inst.port_modes {
                items.push(format!(
                    "{{\"name\":{:?},\"mode\":{:?},\"patch\":{}}}",
                    format!("{}.{}", inst.name, port),
                    over.mode.as_str(),
                    over.patch.to_json()
                ));
            }
        }
        format!("[{}]", items.join(","))
    }

    /// Bus-mode patches in effect for an instance.
    fn active_patches<'a>(&self, inst: &'a Instance) -> Vec<&'a crate::design_promote::PortPatch> {
        inst.port_modes
            .values()
            .filter(|o| o.mode == PortMode::Bus)
            .map(|o| &o.patch)
            .collect()
    }

    /// The CELL-LOCAL blocks an instance contributes, with its Bus-mode port
    /// patches applied. This is the one place that knows a placed cell is the
    /// library body PLUS its promotions — every occupancy, flatten and scan
    /// path goes through it, so a promoted lever can never be half-visible.
    fn instance_local_blocks(&self, inst: &Instance) -> Vec<(P3, crate::BlockState)> {
        let cell = &self.cells[&inst.cell];
        let patches = self.active_patches(inst);
        if patches.is_empty() {
            return cell
                .schematic
                .iter_blocks()
                .map(|(bp, bs)| ((bp.x, bp.y, bp.z), bs.clone()))
                .collect();
        }
        let mut overlay: BTreeMap<P3, Option<String>> = BTreeMap::new();
        for pa in &patches {
            for (p, b) in &pa.writes {
                overlay.insert(*p, b.clone());
            }
        }
        let mut out = Vec::new();
        for (bp, bs) in cell.schematic.iter_blocks() {
            let p = (bp.x, bp.y, bp.z);
            match overlay.remove(&p) {
                Some(Some(b)) => {
                    if let Ok(st) = crate::BlockState::from_block_string(&b) {
                        out.push((p, st));
                    }
                }
                Some(None) => {}
                None => out.push((p, bs.clone())),
            }
        }
        for (p, b) in overlay {
            if let Some(b) = b {
                if let Ok(st) = crate::BlockState::from_block_string(&b) {
                    out.push((p, st));
                }
            }
        }
        out
    }


    /// The transformed contract an instance exposes: port cells and step
    /// vectors mapped through the instance transform; types, delays and
    /// bit order unchanged.
    pub fn instance_contract(&self, name: &str) -> Result<CellContract, String> {
        let inst = self
            .instances
            .iter()
            .find(|i| i.name == name)
            .ok_or_else(|| format!("unknown instance `{name}`"))?;
        let cell = &self.cells[&inst.cell];
        let bbox = cell_bounds(&cell.schematic);
        let mut contract = cell.contract.clone();
        // A port in Bus mode presents its promoted CONNECTION CELLS, not the
        // executor hardware it replaced.
        for (port, over) in &inst.port_modes {
            if over.mode != PortMode::Bus {
                continue;
            }
            if let Some(m) = contract.io.inputs.get_mut(port) {
                m.positions = over.patch.wires.clone();
            } else if let Some(m) = contract.io.outputs.get_mut(port) {
                m.positions = over.patch.wires.clone();
            }
        }
        let map = |p: P3| transform_pos(p, bbox.min, bbox.max, inst.rot_y, inst.at);
        for mapping in contract
            .io
            .inputs
            .values_mut()
            .chain(contract.io.outputs.values_mut())
        {
            for p in mapping.positions.iter_mut() {
                *p = map(*p);
            }
        }
        for bus in contract.io.buses.values_mut() {
            bus.bit0 = map(bus.bit0);
        }
        for keepout in contract.physical.keepouts.iter_mut() {
            let a = map(keepout.min);
            let b = map(keepout.max);
            keepout.min = (a.0.min(b.0), a.1.min(b.1), a.2.min(b.2));
            keepout.max = (a.0.max(b.0), a.1.max(b.1), a.2.max(b.2));
        }
        Ok(contract)
    }

    /// Remove an instance layer. Buses that terminate on one of its ports
    /// lose their endpoint and are DELETED (named in the report); buses that
    /// merely ran through its footprint or halo are ripped and co-rerouted,
    /// as for a drag.
    pub fn remove_instance(&mut self, name: &str) -> Result<RemoveReport, String> {
        let idx = self
            .instances
            .iter()
            .position(|i| i.name == name)
            .ok_or_else(|| format!("unknown instance `{name}`"))?;
        let rev0 = self.layer_revision();
        let region = self.instance_region(idx);
        let prefix = format!("{name}.");

        // Buses whose declaration names a port of this instance cannot
        // survive it.
        let doomed: Vec<String> = self
            .buses
            .values()
            .filter(|b| {
                b.driver_names()
                    .iter()
                    .chain(b.sinks.iter())
                    .any(|p| p.starts_with(&prefix))
            })
            .map(|b| b.name.clone())
            .collect();
        // Buses to re-attempt: anything that touched the vacated space, plus
        // every already-FAILED bus (the removal may have unblocked it).
        let mut affected: BTreeSet<String> = BTreeSet::new();
        for bus in self.buses.values() {
            if doomed.contains(&bus.name) {
                continue;
            }
            match &bus.state {
                BusState::Routed if bus.fragment.keys().any(|p| region.contains(p)) => {
                    affected.insert(bus.name.clone());
                }
                BusState::Failed(_) => {
                    affected.insert(bus.name.clone());
                }
                _ => {}
            }
        }
        for b in &doomed {
            self.touch_bus(b);
            self.buses.remove(b);
        }
        self.instances.remove(idx);
        let moves = self.co_reroute(affected, rev0);
        Ok(RemoveReport {
            removed_buses: doomed,
            moves,
        })
    }

    // ------------------------------------------------------------------
    // Instance ports (derived routing endpoints)
    // ------------------------------------------------------------------

    /// Every routing endpoint the placed instances expose, in
    /// `{instance}.{port}` name order. Ports that cannot terminate a bus
    /// are included with `blocked` set, so a UI can list them and say why.
    pub fn instance_ports(&self) -> Result<Vec<InstancePort>, String> {
        let overlay = self.instance_blocks();
        let mut out = Vec::new();
        let names: Vec<String> = self.instances.iter().map(|i| i.name.clone()).collect();
        for inst in &names {
            let contract = self.instance_contract(inst)?;
            for (dirn, mappings) in [
                (PortDirection::Input, &contract.io.inputs),
                (PortDirection::Output, &contract.io.outputs),
            ] {
                for (pname, mapping) in mappings {
                    out.push(self.derive_instance_port(
                        inst,
                        pname,
                        dirn,
                        mapping.io_type.clone(),
                        mapping.positions.clone(),
                        &overlay,
                    ));
                }
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    /// [`Design::instance_ports`] as a JSON array.
    pub fn instance_ports_json(&self) -> Result<String, String> {
        let items: Vec<String> = self.instance_ports()?.iter().map(|p| p.to_json()).collect();
        Ok(format!("[{}]", items.join(",")))
    }

    fn derive_instance_port(
        &self,
        instance: &str,
        port: &str,
        cell_direction: PortDirection,
        ty: IoType,
        hardware: Vec<P3>,
        overlay: &BTreeMap<P3, String>,
    ) -> InstancePort {
        let name = format!("{instance}.{port}");
        let width = hardware.len().min(u8::MAX as usize) as u8;
        let mut wires = Vec::with_capacity(hardware.len());
        let mut blocked = None;
        let mut step = None;
        for (k, hp) in hardware.iter().enumerate() {
            match self.dust_tap(*hp, overlay) {
                Some(w) => wires.push(w),
                None => {
                    // Actionable, because there IS an action: this is exactly
                    // what `set_port_mode(.., PortMode::Bus)` exists for.
                    blocked = Some(format!(
                        "bit {k}: no dust connection cell at or beside {:?} (holds `{}`) — this \
                         port is executor-only hardware and cannot terminate a bus. PROMOTE it \
                         (switch the port to Bus mode) to swap the hardware for a dust input; the \
                         switch is reversible",
                        hp,
                        self.block_at(*hp, overlay)
                            .unwrap_or_else(|| "air".to_string())
                    ));
                    break;
                }
            }
        }
        if blocked.is_none() {
            if wires.is_empty() {
                blocked = Some("port declares no positions".to_string());
            } else if wires.len() == 1 {
                // A 1-bit port has no measurable pitch: adopt the canonical
                // vertical bus form so bools are routable.
                step = Some((0, 2, 0));
            } else {
                let s = (
                    wires[1].0 - wires[0].0,
                    wires[1].1 - wires[0].1,
                    wires[1].2 - wires[0].2,
                );
                if s == (0, 0, 0) {
                    blocked = Some("connection cells collapse onto one cell".to_string());
                } else if !wires
                    .windows(2)
                    .all(|w| (w[1].0 - w[0].0, w[1].1 - w[0].1, w[1].2 - w[0].2) == s)
                {
                    blocked = Some(format!(
                        "connection cells {wires:?} do not lie on a uniform step"
                    ));
                } else {
                    step = Some(s);
                }
            }
        }
        InstancePort {
            name,
            instance: instance.to_string(),
            port: port.to_string(),
            cell_direction,
            ty,
            width,
            hardware,
            wires: blocked.is_none().then_some(wires),
            step,
            blocked,
        }
    }

    /// Resolve a routing endpoint name: a declared design port, or an
    /// instance port `{instance}.{port}` derived from the instance's
    /// transformed contract.
    ///
    /// The returned [`DesignPort::direction`] is DESIGN-facing — which way
    /// signal flows in the fabric — so a cell OUTPUT resolves to
    /// `PortDirection::Input` (it drives buses) and a cell INPUT resolves to
    /// `PortDirection::Output` (it receives them).
    pub fn resolve_port(&self, name: &str) -> Result<DesignPort, String> {
        if let Some(p) = self.ports.get(name) {
            return Ok(p.clone());
        }
        let Some((inst, port)) = name.split_once('.') else {
            return Err(format!("unknown port `{name}`"));
        };
        if !self.instances.iter().any(|i| i.name == inst) {
            return Err(format!(
                "unknown port `{name}`: nothing declared under that name and no instance `{inst}`"
            ));
        }
        let ports = self.instance_ports()?;
        let ip = ports.iter().find(|p| p.name == name).ok_or_else(|| {
            let have: Vec<&str> = ports
                .iter()
                .filter(|p| p.instance == inst)
                .map(|p| p.port.as_str())
                .collect();
            format!(
                "instance `{inst}` has no contract port `{port}` (has: {})",
                have.join(", ")
            )
        })?;
        if let Some(why) = &ip.blocked {
            return Err(format!("instance port `{name}` cannot terminate a bus: {why}"));
        }
        let wires = ip.wires.clone().expect("a routable port has wires");
        let overlay = self.instance_blocks();
        let bits = wires
            .iter()
            .map(|w| self.scan_bit_with(*w, &overlay))
            .collect();
        Ok(DesignPort {
            name: name.to_string(),
            anchor: wires[0],
            step: ip.step.expect("a routable port has a step"),
            width: ip.width,
            ty: ip.ty.clone(),
            direction: match ip.cell_direction {
                PortDirection::Output => PortDirection::Input,
                PortDirection::Input => PortDirection::Output,
            },
            bits,
        })
    }

    // ------------------------------------------------------------------
    // Ports
    // ------------------------------------------------------------------

    /// Declare a typed port over existing hardware: anchor = bit-0
    /// connection cell, `step` to the next bit, `width` bits of `ty`.
    /// The hardware is scanned (lever ⇒ drivable, lamp ⇒ readable, dust ⇒
    /// connectable) and validated loudly: every connection cell must hold
    /// dust; a declared input must be drivable on every bit; a declared
    /// output readable on every bit.
    pub fn declare_port(
        &mut self,
        name: impl Into<String>,
        anchor: P3,
        step: P3,
        width: u8,
        ty: IoType,
        direction: PortDirection,
    ) -> Result<&DesignPort, String> {
        let name = name.into();
        if self.ports.contains_key(&name) {
            return Err(format!("port `{name}` already declared"));
        }
        if width == 0 {
            return Err(format!("port `{name}`: width must be at least 1"));
        }
        if step == (0, 0, 0) {
            return Err(format!("port `{name}`: step must be non-zero"));
        }
        if ty.bit_count() != width as usize {
            return Err(format!(
                "port `{name}`: width {} does not match type bit count {}",
                width,
                ty.bit_count()
            ));
        }
        // Instance bodies count as hardware: a design port may be declared
        // over a placed cell's dust, not only over loose blocks.
        let overlay = self.instance_blocks();
        let mut bits = Vec::with_capacity(width as usize);
        for k in 0..width {
            let cell = add(anchor, scale(step, k as i32));
            let hw = self.scan_bit_with(cell, &overlay);
            if !hw.connectable {
                return Err(format!(
                    "port `{name}` bit {k}: connection cell {:?} holds `{}`, not dust",
                    cell,
                    self.block_at(cell, &overlay).unwrap_or_default()
                ));
            }
            match direction {
                PortDirection::Input if hw.lever.is_none() => {
                    return Err(format!(
                        "port `{name}` bit {k}: declared input but no adjacent lever at {cell:?} \
                         (not drivable)"
                    ));
                }
                PortDirection::Output if hw.lamp.is_none() => {
                    return Err(format!(
                        "port `{name}` bit {k}: declared output but no adjacent lamp at {cell:?} \
                         (not readable)"
                    ));
                }
                _ => {}
            }
            bits.push(hw);
        }
        let port = DesignPort {
            name: name.clone(),
            anchor,
            step,
            width,
            ty,
            direction,
            bits,
        };
        self.ports.insert(name.clone(), port);
        Ok(&self.ports[&name])
    }

    /// Sugar: declare a drivable input port (asserts lever capability).
    pub fn declare_input(
        &mut self,
        name: impl Into<String>,
        anchor: P3,
        step: P3,
        width: u8,
        ty: IoType,
    ) -> Result<&DesignPort, String> {
        self.declare_port(name, anchor, step, width, ty, PortDirection::Input)
    }

    /// Sugar: declare a readable output port (asserts lamp capability).
    pub fn declare_output(
        &mut self,
        name: impl Into<String>,
        anchor: P3,
        step: P3,
        width: u8,
        ty: IoType,
    ) -> Result<&DesignPort, String> {
        self.declare_port(name, anchor, step, width, ty, PortDirection::Output)
    }

    fn base_block_string(&self, p: P3) -> Option<String> {
        self.base
            .get_block(p.0, p.1, p.2)
            .map(|b| b.to_string())
            .filter(|s| !s.contains("minecraft:air"))
    }

    /// The blocks the instance layers contribute, transformed — the overlay
    /// a hardware scan needs so it sees cell bodies as well as loose
    /// hardware. Built on demand; scans are one-shot (declaration, port
    /// resolution), never per-planned-cell.
    fn instance_blocks(&self) -> BTreeMap<P3, String> {
        let mut out = BTreeMap::new();
        for inst in &self.instances {
            let cell = &self.cells[&inst.cell];
            let bbox = cell_bounds(&cell.schematic);
            for (bp, bs) in self.instance_local_blocks(inst) {
                let s = transform_state(&bs, inst.rot_y).to_string();
                if s.contains("minecraft:air") {
                    continue;
                }
                let p = transform_pos(bp, bbox.min, bbox.max, inst.rot_y, inst.at);
                out.insert(p, s);
            }
        }
        out
    }

    /// The block at `p`: loose layer first, then the instance overlay.
    fn block_at(&self, p: P3, overlay: &BTreeMap<P3, String>) -> Option<String> {
        self.base_block_string(p)
            .or_else(|| overlay.get(&p).cloned())
    }

    /// The dust CONNECTION CELL for one piece of executor hardware: the cell
    /// itself when it already holds dust, else the first dust neighbour in
    /// [`AROUND`] order.
    fn dust_tap(&self, hardware: P3, overlay: &BTreeMap<P3, String>) -> Option<P3> {
        if self
            .block_at(hardware, overlay)
            .is_some_and(|b| rblocks::is_dust(&b))
        {
            return Some(hardware);
        }
        AROUND
            .iter()
            .map(|d| add(hardware, *d))
            .find(|q| self.block_at(*q, overlay).is_some_and(|b| rblocks::is_dust(&b)))
    }

    /// Hardware scan of one connection cell, seeing `overlay` (instance
    /// bodies) as well as the loose layer.
    fn scan_bit_with(&self, cell: P3, overlay: &BTreeMap<P3, String>) -> BitHardware {
        let mut hw = BitHardware::default();
        hw.connectable = self
            .block_at(cell, overlay)
            .is_some_and(|b| rblocks::is_dust(&b));
        // Levers power adjacent dust: the 4 horizontal neighbours, the cell
        // above and the support below.
        for d in AROUND {
            let q = add(cell, d);
            if let Some(b) = self.block_at(q, overlay) {
                if hw.lever.is_none() && rblocks::is_lever(&b) {
                    hw.lever = Some(q);
                }
                if hw.lamp.is_none() && b.contains("redstone_lamp") {
                    hw.lamp = Some(q);
                }
            }
        }
        hw
    }

    // ------------------------------------------------------------------
    // Buses
    // ------------------------------------------------------------------

    /// Declare AND realize a bus from `driver` to `sinks`, threading the
    /// optional `gates` in order. One driver, N sinks: sinks beyond the
    /// first branch off the trunk at dust junctions. Declaration errors
    /// (unknown port, width mismatch, invalid style, duplicate name) are
    /// `Err`; geometric unroutability is a returned [`BusState::Failed`] —
    /// realization is atomic and never leaves a half-routed fragment.
    pub fn route_bus(
        &mut self,
        name: impl Into<String>,
        driver: &str,
        sinks: &[&str],
        gates: Vec<Gate>,
        style: BusStyle,
    ) -> Result<BusState, String> {
        self.route_bus_inner(
            name.into(),
            &[driver],
            sinks,
            gates,
            style,
            false,
            WidthAdapt::default(),
        )
    }

    /// [`Design::route_bus`] with an explicit WIDTH-ADAPTATION policy.
    ///
    /// A width mismatch is a layout question, not an error. 3 BCD-hundreds bits
    /// fit inside a 4-bit `bcd` input; the only decision is which 3 of the 4
    /// they drive, and the destination bits nothing drives read 0 with no
    /// hardware at all (undriven dust IS logical 0 — verified in-sim by
    /// `tests/design_width_adapt.rs`).
    ///
    /// - [`BusAlign::Lsb`] (the default): bit 0 to bit 0, magnitude preserved.
    /// - [`BusAlign::Msb`]: top bit to top bit — a shift up by the width
    ///   difference, which is what fixed-point fields want.
    /// - [`BusAlign::Shift`]: place the word anywhere.
    /// - `truncate`: permit dropping source bits that fall outside. Refused by
    ///   default, because losing a word's high bits is not the router's call.
    ///
    /// Adaptation applies to a single driver and a single sink; a fanout or a
    /// wired-OR merge still requires one common width, so nobody has to reason
    /// about several different mappings sharing one trunk.
    pub fn route_bus_adapted(
        &mut self,
        name: impl Into<String>,
        driver: &str,
        sinks: &[&str],
        gates: Vec<Gate>,
        style: BusStyle,
        adapt: WidthAdapt,
    ) -> Result<BusState, String> {
        self.route_bus_inner(name.into(), &[driver], sinks, gates, style, false, adapt)
    }

    /// Declare AND realize a wired-OR bus: multiple drivers are legal ONLY
    /// through this explicit merge (`merge="or"`). Extra drivers join the
    /// trunk as dust-merge branches (diode-isolated); the LVS intent stays
    /// ONE net per bit.
    pub fn route_bus_or(
        &mut self,
        name: impl Into<String>,
        drivers: &[&str],
        sinks: &[&str],
        gates: Vec<Gate>,
        style: BusStyle,
    ) -> Result<BusState, String> {
        self.route_bus_inner(
            name.into(),
            drivers,
            sinks,
            gates,
            style,
            true,
            WidthAdapt::default(),
        )
    }

    fn route_bus_inner(
        &mut self,
        name: String,
        drivers: &[&str],
        sinks: &[&str],
        gates: Vec<Gate>,
        style: BusStyle,
        merge_or: bool,
        adapt: WidthAdapt,
    ) -> Result<BusState, String> {
        if self.buses.contains_key(&name) {
            return Err(format!("bus `{name}` already exists"));
        }
        style.validate()?;
        if drivers.is_empty() {
            return Err(format!("bus `{name}` needs at least one driver"));
        }
        if drivers.len() > 1 && !merge_or {
            return Err(format!(
                "bus `{name}`: {} drivers are only legal as an explicit wired-OR merge \
                 (route_bus_or / merge=\"or\")",
                drivers.len()
            ));
        }
        // Promote executor-only endpoints before resolving them, so routing to
        // a community cell's lever input just works. Nothing else about the
        // route changes: a promoted port resolves like any other dust port.
        // Ordered and deduplicated so the promotions recorded on the bus are
        // deterministic regardless of how the endpoints were listed.
        let mut promotions: Vec<String> = Vec::new();
        if self.auto_promote {
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for ep in drivers.iter().chain(sinks.iter()) {
                if !seen.insert(ep.to_string()) {
                    continue;
                }
                if let Some(note) = self.auto_promote_endpoint(ep) {
                    promotions.push(note);
                }
            }
        }

        let mut driver_ports: Vec<DesignPort> = Vec::new();
        for dn in drivers {
            let drv = self
                .resolve_port(dn)
                .map_err(|e| format!("bus `{name}`: driver {e}"))?;
            if drv.direction == PortDirection::Output {
                return Err(format!(
                    "bus `{name}`: `{dn}` receives signal, it cannot drive — swap the endpoints \
                     (a design output, or a cell's input port, is a sink)"
                ));
            }
            if let Some(first) = driver_ports.first() {
                if drv.width != first.width {
                    return Err(format!(
                        "bus `{name}`: driver `{dn}` width {} != driver `{}` width {}",
                        drv.width, first.name, first.width
                    ));
                }
            }
            driver_ports.push(drv);
        }
        if sinks.is_empty() {
            return Err(format!("bus `{name}` needs at least one sink"));
        }
        let mut sink_ports = Vec::new();
        let mut width_map: Option<WidthMap> = None;
        for s in sinks {
            let sp = self
                .resolve_port(s)
                .map_err(|e| format!("bus `{name}`: sink {e}"))?;
            if sp.direction == PortDirection::Input {
                return Err(format!(
                    "bus `{name}`: `{s}` drives signal, it cannot be a sink — swap the endpoints \
                     (a design input, or a cell's output port, is a driver)"
                ));
            }
            // Adapt when the widths differ, and ALSO when the caller asked for
            // a specific placement: `shift(2)` between two 8-bit ports is a real
            // request, and ignoring it because the widths happen to match would
            // be the worst kind of surprise.
            let asked = !matches!(adapt.align, BusAlign::Lsb) || adapt.truncate;
            if sp.width != driver_ports[0].width || asked {
                // A width mismatch is a LAYOUT question, and the router answers
                // it — but only where one answer is unambiguous. Several sinks
                // sharing a trunk would each want their own mapping, so those
                // still need one common width.
                if sp.width != driver_ports[0].width && (sinks.len() > 1 || drivers.len() > 1) {
                    return Err(format!(
                        "bus `{name}`: sink `{s}` width {} != driver width {} — width adaptation \
                         applies to a single driver and a single sink, so a fanout or wired-OR \
                         merge needs one common width (route the odd sink as its own bus)",
                        sp.width, driver_ports[0].width
                    ));
                }
                let m = plan_width_map(driver_ports[0].width, sp.width, adapt)
                    .map_err(|e| format!("bus `{name}`: {e}"))?;
                // Equal widths, no shift, nothing dropped: nothing to say, so
                // leave the bus unadorned.
                if !m.is_identity() {
                    width_map = Some(m);
                }
            }
            // Two integer words of DIFFERENT widths are different `IoType`s by
            // construction, so the type check has to look past the width once
            // an adaptation has resolved it; everything else still has to match.
            if sp.ty != driver_ports[0].ty
                && !(width_map.is_some()
                    && same_type_family(&driver_ports[0].ty, &sp.ty))
            {
                return Err(format!(
                    "bus `{name}`: sink `{s}` type {:?} != driver `{}` type {:?}",
                    sp.ty, driver_ports[0].name, driver_ports[0].ty
                ));
            }
            sink_ports.push(sp);
        }

        let mut layer = BusLayer {
            name: name.clone(),
            driver: drivers[0].to_string(),
            extra_drivers: drivers[1..].iter().map(|s| s.to_string()).collect(),
            merge_or,
            sinks: sinks.iter().map(|s| s.to_string()).collect(),
            gates,
            style,
            state: BusState::Intended,
            fragment: BTreeMap::new(),
            runs: Vec::new(),
            segments: Vec::new(),
            gate_cells: BTreeMap::new(),
            rule: None,
            promotions,
            width_map,
        };

        match self.realize(
            Some(&name),
            &driver_ports,
            &sink_ports,
            &layer.gates,
            &layer.style,
            layer.width_map.as_ref(),
        ) {
            Ok(real) => {
                Self::fill_layer(&mut layer, real.fragment, real.segments, real.gate_cells);
                self.apply_amendments(real.amendments);
            }
            Err(reason) => {
                layer.state = BusState::Failed(reason);
            }
        }
        let state = layer.state.clone();
        self.touch_bus(&name);
        self.buses.insert(name, layer);
        Ok(state)
    }

    /// Attach a net-class discipline to a bus; [`Design::check`] enforces
    /// its `max_len_rt` delay budget and `y_band` layer assignment.
    pub fn set_bus_rule(&mut self, bus: &str, rule: NetClassRule) -> Result<(), String> {
        self.buses
            .get_mut(bus)
            .ok_or_else(|| format!("unknown bus `{bus}`"))?
            .rule = Some(rule);
        Ok(())
    }

    /// Edit the loose block layer: plain `set_block` on the base
    /// schematic. The cell participates in occupancy and flatten like any
    /// other loose hardware.
    pub fn set_block(&mut self, p: P3, block: &str) -> Result<(), String> {
        self.base
            .set_block_from_string(p.0, p.1, p.2, block)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn fill_layer(
        layer: &mut BusLayer,
        fragment: BTreeMap<P3, String>,
        segments: Vec<Segment>,
        gate_cells: BTreeMap<String, BTreeSet<P3>>,
    ) {
        layer.fragment = fragment;
        layer.runs = segments.iter().flat_map(|s| s.runs.clone()).collect();
        layer.segments = segments;
        layer.gate_cells = gate_cells;
        layer.state = BusState::Routed;
    }

    /// Apply crossing amendments to the buses they target, keeping their
    /// fragments AND per-segment cell sets consistent.
    /// An amendment rewrites a bus that was never ripped and is named in NO
    /// report, so it is stamped as changed here — otherwise the studio, which
    /// redraws only what the engine reports, leaves the crossed bus showing
    /// its pre-station geometry.
    fn apply_amendments(&mut self, amendments: Vec<(String, Vec<P3>, BTreeMap<P3, String>)>) {
        let mut touched: Vec<String> = Vec::new();
        for (bus, removals, additions) in amendments {
            self.touch_bus(&bus);
            let target = self.buses.get_mut(&bus).expect("amended bus exists");
            // The segment losing cells also receives the station cells.
            let seg_idx = removals
                .iter()
                .find_map(|p| target.segments.iter().position(|s| s.cells.contains(p)));
            for p in &removals {
                target.fragment.remove(p);
                for seg in target.segments.iter_mut() {
                    seg.cells.remove(p);
                }
            }
            if let Some(i) = seg_idx {
                target.segments[i].cells.extend(additions.keys().copied());
            }
            target.fragment.extend(additions);
            touched.push(bus);
        }
        // An amendment REMOVES dust from the bus it stations, and a removal
        // changes the connection state of every wire that pointed at it. Draw
        // the amended bus again rather than leaving dots around the station.
        if !touched.is_empty() {
            let occ = self.occupancy_index();
            for bus in touched {
                if let Some(target) = self.buses.get_mut(&bus) {
                    let mut frag = std::mem::take(&mut target.fragment);
                    Self::rewire_fragment(&mut frag, &occ);
                    self.buses.get_mut(&bus).unwrap().fragment = frag;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Drag APIs (Phase 2): moves always succeed — they are the document's
    // truth; buses fail VISIBLY (`FAILED(reason)`), never half-routed.
    // ------------------------------------------------------------------

    /// Move (and optionally re-rotate) an instance layer. Computes the
    /// AFFECTED bus set — routed buses whose fragments intersect the old
    /// or new footprint + influence halo, plus every already-`FAILED` bus
    /// (a move may have unblocked it) — rips exactly that set and
    /// co-reroutes it deterministically in name order with bounded retry
    /// rounds. Unroutable buses end `FAILED(reason)`.
    pub fn move_instance(&mut self, name: &str, at: P3, rot_y: i32) -> Result<MoveReport, String> {
        if rot_y.rem_euclid(90) != 0 {
            return Err(format!("rot_y must be a multiple of 90, got {rot_y}"));
        }
        let idx = self
            .instances
            .iter()
            .position(|i| i.name == name)
            .ok_or_else(|| format!("unknown instance `{name}`"))?;
        let rev0 = self.layer_revision();
        let old_region = self.instance_region(idx);
        // The move itself always succeeds.
        self.instances[idx].at = at;
        self.instances[idx].rot_y = rot_y.rem_euclid(360);
        let new_region = self.instance_region(idx);

        // Buses WIRED to this instance move with it: their endpoint anchors
        // are derived from the instance transform, so a bus that keeps its old
        // fragment is now wired to where the ports USED to be. This must not
        // depend on the fragment intersecting the region — a cell that
        // declares explicit `keepouts` can have a halo that excludes the very
        // cell its port's bus leaves along, and then the fragment test misses
        // it. That was the "I moved a component and the bus didn't update"
        // bug: the geometry silently kept pointing at the old position.
        let mut affected: BTreeSet<String> = self.buses_wired_to(name);
        for bus in self.buses.values() {
            match &bus.state {
                BusState::Routed => {
                    if bus
                        .fragment
                        .keys()
                        .any(|p| old_region.contains(p) || new_region.contains(p))
                    {
                        affected.insert(bus.name.clone());
                    }
                }
                // A FAILED bus is re-attempted on every move: the drag that
                // blocked it can also be the drag that unblocks it.
                BusState::Failed(_) => {
                    affected.insert(bus.name.clone());
                }
                BusState::Intended => {}
            }
        }
        Ok(self.co_reroute(affected, rev0))
    }

    /// Buses whose declaration names a port OF this instance (`inst.port`) —
    /// the buses an instance carries with it when it moves, and the ones that
    /// cannot survive its removal.
    fn buses_wired_to(&self, instance: &str) -> BTreeSet<String> {
        let prefix = format!("{instance}.");
        self.buses
            .values()
            .filter(|b| {
                b.driver_names()
                    .iter()
                    .chain(b.sinks.iter())
                    .any(|p| p.starts_with(&prefix))
            })
            .map(|b| b.name.clone())
            .collect()
    }

    /// Footprint + influence halo of an instance, as a cell set.
    fn instance_region(&self, idx: usize) -> BTreeSet<P3> {
        let inst = &self.instances[idx];
        let cell = &self.cells[&inst.cell];
        let bbox = cell_bounds(&cell.schematic);
        let map = |p: P3| transform_pos(p, bbox.min, bbox.max, inst.rot_y, inst.at);
        let mut region = BTreeSet::new();
        for (bp, bs) in self.instance_local_blocks(inst) {
            if bs.to_string().contains("minecraft:air") {
                continue;
            }
            region.insert(map(bp));
        }
        for (min, max) in Self::halo_boxes(cell, bbox.min, bbox.max) {
            let a = map(min);
            let b = map(max);
            let (lo, hi) = (
                (a.0.min(b.0), a.1.min(b.1), a.2.min(b.2)),
                (a.0.max(b.0), a.1.max(b.1), a.2.max(b.2)),
            );
            for x in lo.0..=hi.0 {
                for y in lo.1..=hi.1 {
                    for z in lo.2..=hi.2 {
                        region.insert((x, y, z));
                    }
                }
            }
        }
        region
    }

    /// The influence-halo boxes of a cell, untransformed:
    /// `PhysicalContract.keepouts` where declared, else the cell bounds
    /// grown by one (electrical clearance — dust one step up/down shorts
    /// without sharing a cell).
    fn halo_boxes(cell: &CellDef, min: P3, max: P3) -> Vec<(P3, P3)> {
        if cell.contract.physical.keepouts.is_empty() {
            vec![(
                (min.0 - 1, min.1 - 1, min.2 - 1),
                (max.0 + 1, max.1 + 1, max.2 + 1),
            )]
        } else {
            cell.contract
                .physical
                .keepouts
                .iter()
                .map(|k| (k.min, k.max))
                .collect()
        }
    }

    /// Rip and co-reroute a bus set: deterministic name order (seeded by
    /// the BTree ordering), bounded negotiation rounds — a bus that fails
    /// because a peer's fresh fragment contested its cells is retried
    /// after the peers commit. No exceptions: survivors are `Routed`,
    /// the rest `FAILED(reason)`.
    fn co_reroute(&mut self, affected: BTreeSet<String>, rev0: u64) -> MoveReport {
        const ROUNDS: usize = 3;
        for name in &affected {
            let _ = self.rip(name);
        }
        let mut pending: Vec<String> = affected.iter().cloned().collect();
        let mut report = MoveReport::default();
        for round in 0..ROUNDS {
            let mut still = Vec::new();
            for name in &pending {
                match self.reroute_one(name) {
                    BusState::Routed => report.rerouted.push(name.clone()),
                    BusState::Failed(_) => still.push(name.clone()),
                    BusState::Intended => {}
                }
            }
            let no_progress = still.len() == pending.len();
            pending = still;
            if pending.is_empty() || (no_progress && round > 0) {
                break;
            }
        }
        for name in &affected {
            if let Some(BusState::Failed(reason)) = self.bus_state(name) {
                report.failed.push((name.clone(), reason.clone()));
            }
        }
        // The redraw set is MEASURED from the revision stamps, not inferred
        // from `affected`: it also picks up buses that were never ripped but
        // had a crossing station amended into them by a peer's fresh route.
        report.changed = self.changed_layers_since(rev0);
        report
    }

    /// Re-realize one bus from its stored declaration. The bus keeps its
    /// endpoints, gates, style and rule; only the fragment is rebuilt.
    fn reroute_one(&mut self, name: &str) -> BusState {
        let Some(layer) = self.buses.get(name) else {
            return BusState::Intended;
        };
        let driver_names = layer.driver_names();
        let sink_names = layer.sinks.clone();
        let gates = layer.gates.clone();
        let style = layer.style.clone();
        // The bit mapping is part of the bus's INTENT: a reroute keeps it, so a
        // width-adapted bus never silently re-pairs its bits.
        let width_map = layer.width_map.clone();
        let mut driver_ports = Vec::new();
        for dn in &driver_names {
            match self.resolve_port(dn) {
                Ok(p) => driver_ports.push(p),
                Err(why) => {
                    let state = BusState::Failed(format!("driver port `{dn}`: {why}"));
                    self.touch_bus(name);
                    self.buses.get_mut(name).unwrap().state = state.clone();
                    return state;
                }
            }
        }
        let mut sink_ports = Vec::new();
        for sn in &sink_names {
            match self.resolve_port(sn) {
                Ok(p) => sink_ports.push(p),
                Err(why) => {
                    let state = BusState::Failed(format!("sink port `{sn}`: {why}"));
                    self.touch_bus(name);
                    self.buses.get_mut(name).unwrap().state = state.clone();
                    return state;
                }
            }
        }
        match self.realize(
            Some(name),
            &driver_ports,
            &sink_ports,
            &gates,
            &style,
            width_map.as_ref(),
        ) {
            Ok(real) => {
                self.touch_bus(name);
                let layer = self.buses.get_mut(name).unwrap();
                Self::fill_layer(layer, real.fragment, real.segments, real.gate_cells);
                self.apply_amendments(real.amendments);
                BusState::Routed
            }
            Err(reason) => {
                self.touch_bus(name);
                let layer = self.buses.get_mut(name).unwrap();
                layer.fragment.clear();
                layer.runs.clear();
                layer.segments.clear();
                layer.gate_cells.clear();
                layer.state = BusState::Failed(reason.clone());
                BusState::Failed(reason)
            }
        }
    }

    /// Add a gate to an existing bus (splitting the segment it lands in)
    /// and re-realize the bus. Returns the resulting state.
    pub fn add_gate(&mut self, bus: &str, name: &str, anchor: P3, step: P3) -> Result<BusState, String> {
        let layer = self
            .buses
            .get(bus)
            .ok_or_else(|| format!("unknown bus `{bus}`"))?;
        if layer.gates.iter().any(|g| g.name == name) {
            return Err(format!("bus `{bus}` already has a gate `{name}`"));
        }
        // Insert position: the trunk waypoint pair the anchor is nearest
        // to (Manhattan distance to the pair midpoint) — deterministic.
        //
        // Resolve through `resolve_port`, NOT the declared-port table: an
        // endpoint may be an INSTANCE port (`add0.sum` -> `bcd0.bin`), which is
        // derived from the cell's contract and never appears in `self.ports`.
        // Looking only there made `add_gate` refuse every bus between two
        // placed cells — exactly the buses a user most wants a checkpoint on —
        // while `move_gate` worked fine on the same bus.
        let (driver_name, sink0_name) = (layer.driver.clone(), layer.sinks[0].clone());
        let gate_anchors: Vec<P3> = layer.gates.iter().map(|g| g.anchor).collect();
        let driver = self
            .resolve_port(&driver_name)
            .map_err(|e| format!("bus `{bus}`: driver {e}"))?;
        let sink0 = self
            .resolve_port(&sink0_name)
            .map_err(|e| format!("bus `{bus}`: sink {e}"))?;
        let mut wps = vec![driver.anchor];
        wps.extend(gate_anchors);
        wps.push(sink0.anchor);
        let mut best = (0usize, i32::MAX);
        for (i, pair) in wps.windows(2).enumerate() {
            let mid = (
                (pair[0].0 + pair[1].0) / 2,
                (pair[0].1 + pair[1].1) / 2,
                (pair[0].2 + pair[1].2) / 2,
            );
            let dist =
                (anchor.0 - mid.0).abs() + (anchor.1 - mid.1).abs() + (anchor.2 - mid.2).abs();
            if dist < best.1 {
                best = (i, dist);
            }
        }
        let layer = self.buses.get_mut(bus).unwrap();
        layer.gates.insert(
            best.0,
            Gate {
                name: name.to_string(),
                anchor,
                step,
            },
        );
        let _ = self.rip(bus);
        Ok(self.reroute_one(bus))
    }

    /// Undo a port declaration.
    ///
    /// A DESIGN port (`declare_input` / `declare_output`) is a document entry
    /// and this removes it. An INSTANCE port (`inst.port`) is derived from the
    /// cell's contract and cannot be removed — what a UI means there is
    /// "return it to Executor mode", which is
    /// [`Design::set_port_mode`]; this says so rather than silently doing
    /// nothing.
    ///
    /// GATES AND ENDPOINTS ARE NOT THE SAME THING. Removing a gate relaxes a
    /// constraint: the bus survives and re-routes straighter
    /// ([`Design::remove_gate`]). Removing an ENDPOINT changes the netlist: the
    /// bus loses a terminal, so every bus that named this port is DELETED, and
    /// they are returned so a UI can say which. Nothing is blurred, and nothing
    /// is deleted silently: pass `force = false` to be refused with the list
    /// instead, and confirm first.
    ///
    /// Returns `(removed buses, reroute outcome for the buses that merely used
    /// the space)`.
    pub fn remove_port(
        &mut self,
        name: &str,
        force: bool,
    ) -> Result<(Vec<String>, MoveReport), String> {
        let rev0 = self.layer_revision();
        if !self.ports.contains_key(name) {
            if name.contains('.') {
                return Err(format!(
                    "`{name}` is an INSTANCE port derived from its cell's contract, not a \
                     declaration, so there is nothing to remove. To undo its promotion, set it \
                     back to Executor mode (`set_port_mode`); to remove the port itself, remove \
                     the instance"
                ));
            }
            return Err(format!(
                "unknown port `{name}` (declared: {})",
                if self.ports.is_empty() {
                    "none".to_string()
                } else {
                    self.ports.keys().cloned().collect::<Vec<_>>().join(", ")
                }
            ));
        }
        let doomed: Vec<String> = self
            .buses
            .values()
            .filter(|b| {
                b.driver_names().iter().chain(b.sinks.iter()).any(|p| p == name)
            })
            .map(|b| b.name.clone())
            .collect();
        if !force && !doomed.is_empty() {
            return Err(format!(
                "port `{name}` is an ENDPOINT of {} bus(es) ({}); removing it changes the netlist, \
                 so those buses would be deleted. Re-run with force to confirm, or re-point them \
                 first",
                doomed.len(),
                doomed.join(", ")
            ));
        }
        for b in &doomed {
            self.touch_bus(b);
            self.buses.remove(b);
        }
        self.ports.remove(name);
        // Freeing the port's space may unblock a bus that failed near it.
        let affected: BTreeSet<String> = self
            .buses
            .values()
            .filter(|b| matches!(b.state, BusState::Failed(_)))
            .map(|b| b.name.clone())
            .collect();
        Ok((doomed, self.co_reroute(affected, rev0)))
    }

    /// Remove a gate by INDEX (its position in the bus's gate list) and
    /// re-realize the bus, so the two spans it separated MERGE and are routed
    /// as one.
    ///
    /// Removing a gate RELAXES a constraint, so the result should be shorter
    /// and straighter — not the two old legs stitched together. That is why
    /// this re-plans the merged span from scratch instead of splicing: with the
    /// waypoint gone, `A -> gate -> C` becomes the single pair `A -> C`, which
    /// the templates take as one straight run or one L.
    ///
    /// The bus survives either way: an unroutable result leaves it
    /// `FAILED(reason)`, visible, never half-routed. The change is in the
    /// changed-layer report ([`Design::changed_layers_since`]) like every other
    /// mutation.
    ///
    /// See also [`Design::remove_port`]: a gate and an endpoint are NOT the
    /// same thing, and deleting them means different things.
    pub fn remove_gate(&mut self, bus: &str, index: usize) -> Result<GateMoveReport, String> {
        let rev0 = self.layer_revision();
        let layer = self
            .buses
            .get(bus)
            .ok_or_else(|| format!("unknown bus `{bus}`"))?;
        let n = layer.gates.len();
        if index >= n {
            return Err(format!(
                "bus `{bus}` has {n} gate(s) ({}), so there is no gate at index {index}",
                if n == 0 {
                    "none".to_string()
                } else {
                    layer
                        .gates
                        .iter()
                        .map(|g| format!("`{}`", g.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ));
        }
        self.buses.get_mut(bus).unwrap().gates.remove(index);
        let _ = self.rip(bus);
        let state = self.reroute_one(bus);
        Ok(GateMoveReport {
            state,
            rerouted_segments: self.buses[bus].segments.len(),
            changed: self.changed_layers_since(rev0),
        })
    }

    /// Remove a gate by NAME — the same as [`Design::remove_gate`], for a UI
    /// that tracks gates by their label rather than their order.
    pub fn remove_gate_named(&mut self, bus: &str, gate: &str) -> Result<GateMoveReport, String> {
        let layer = self
            .buses
            .get(bus)
            .ok_or_else(|| format!("unknown bus `{bus}`"))?;
        let index = layer
            .gates
            .iter()
            .position(|g| g.name == gate)
            .ok_or_else(|| {
                format!(
                    "bus `{bus}` has no gate `{gate}` (has: {})",
                    layer
                        .gates
                        .iter()
                        .map(|g| g.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        self.remove_gate(bus, index)
    }

    /// Drag a gate: the anchor moves unconditionally (the document's
    /// truth), then EXACTLY the two segments adjacent to the gate are
    /// ripped and rerouted atomically against the design-wide occupancy.
    /// An unroutable move leaves the bus `FAILED(reason)` with the
    /// fragment cleared — visible, never half-routed.
    pub fn move_gate(&mut self, bus: &str, gate: &str, anchor: P3) -> Result<GateMoveReport, String> {
        let rev0 = self.layer_revision();
        let layer = self
            .buses
            .get(bus)
            .ok_or_else(|| format!("unknown bus `{bus}`"))?;
        let gi = layer
            .gates
            .iter()
            .position(|g| g.name == gate)
            .ok_or_else(|| format!("bus `{bus}` has no gate `{gate}`"))?;
        let was_routed = layer.state == BusState::Routed && !layer.segments.is_empty();
        // Trunk waypoints BEFORE the move (gate gi is waypoint gi+1).
        let old_wps: Option<(P3, P3)> = if was_routed {
            let seg_a = layer.segments.iter().find(|s| s.kind == SegmentKind::Trunk(gi));
            let seg_b = layer
                .segments
                .iter()
                .find(|s| s.kind == SegmentKind::Trunk(gi + 1));
            match (seg_a, seg_b) {
                (Some(a), Some(b)) => Some((a.a, b.b)),
                _ => None,
            }
        } else {
            None
        };
        self.buses.get_mut(bus).unwrap().gates[gi].anchor = anchor;

        let Some((wp_before, wp_after)) = old_wps else {
            // No per-segment record (unrouted, FAILED, or a loaded
            // document): whole-bus reroute.
            let _ = self.rip(bus);
            let state = self.reroute_one(bus);
            let n = self.buses[bus].segments.len();
            return Ok(GateMoveReport {
                state,
                rerouted_segments: n,
                changed: self.changed_layers_since(rev0),
            });
        };

        // A branch whose junction rides an affected trunk segment moves
        // with it: fall back to the whole-bus reroute in that case.
        let layer = &self.buses[bus];
        let seg_a_idx = layer
            .segments
            .iter()
            .position(|s| s.kind == SegmentKind::Trunk(gi))
            .unwrap();
        let seg_b_idx = layer
            .segments
            .iter()
            .position(|s| s.kind == SegmentKind::Trunk(gi + 1))
            .unwrap();
        let branch_rides = layer.segments.iter().any(|s| {
            matches!(s.kind, SegmentKind::Branch(_))
                && [seg_a_idx, seg_b_idx].iter().any(|&i| {
                    let t = &layer.segments[i];
                    t.cells.contains(&s.a) || t.cells.contains(&s.b)
                        || t.runs.iter().any(|r| {
                            let (jx, jz) = if r.along_x { (s.a.0, s.a.2) } else { (s.a.2, s.a.0) };
                            jz == r.fixed && r.strictly_inside(jx, 0)
                        })
                })
        });
        if branch_rides {
            let _ = self.rip(bus);
            let state = self.reroute_one(bus);
            let n = self.buses[bus].segments.len();
            return Ok(GateMoveReport {
                state,
                rerouted_segments: n,
                changed: self.changed_layers_since(rev0),
            });
        }

        // Rip exactly the two adjacent segments + the gate's joint column.
        let mut ripped: BTreeSet<P3> = BTreeSet::new();
        ripped.extend(layer.segments[seg_a_idx].cells.iter().copied());
        ripped.extend(layer.segments[seg_b_idx].cells.iter().copied());
        if let Some(cells) = layer.gate_cells.get(gate) {
            ripped.extend(cells.iter().copied());
        }
        let width = layer
            .segments
            .get(seg_a_idx)
            .and_then(|s| s.runs.first())
            .map(|r| r.width)
            .unwrap_or(1);
        let style = layer.style.clone();

        let endpoints: Vec<String> = layer
            .driver_names()
            .into_iter()
            .chain(layer.sinks.iter().cloned())
            .collect();
        let occ = self.occupancy_for_plan(&ripped, &self.halo_exempt(&endpoints));
        let mut planner = Planner::new(self, Some(bus), &style, &occ);
        let plan = (|| -> Result<(), String> {
            // The refresh count entering a mid-bus segment is unknown; a
            // conservative REFRESH_AT forces the earliest legal refresh.
            planner.begin_segment(SegmentKind::Trunk(gi), wp_before, anchor);
            let since = planner.plan_pair(wp_before, anchor, width, &BTreeSet::new(), REFRESH_AT)?;
            planner.end_segment();
            planner.begin_segment(SegmentKind::Trunk(gi + 1), anchor, wp_after);
            planner.plan_pair(anchor, wp_after, width, &BTreeSet::new(), since.saturating_add(1))?;
            planner.end_segment();
            planner.plan_column(gate, anchor, width)
        })();
        let real = planner.finish();

        match plan {
            Ok(()) => {
                self.touch_bus(bus);
                let layer = self.buses.get_mut(bus).unwrap();
                for p in &ripped {
                    layer.fragment.remove(p);
                }
                layer.fragment.extend(real.fragment);
                let mut new_segments = real.segments;
                layer.segments[seg_a_idx] = new_segments.remove(0);
                layer.segments[seg_b_idx] = new_segments.remove(0);
                layer
                    .gate_cells
                    .insert(gate.to_string(), real.gate_cells.get(gate).cloned().unwrap_or_default());
                layer.runs = layer.segments.iter().flat_map(|s| s.runs.clone()).collect();
                self.apply_amendments(real.amendments);
                Ok(GateMoveReport {
                    state: BusState::Routed,
                    rerouted_segments: 2,
                    changed: self.changed_layers_since(rev0),
                })
            }
            Err(reason) => {
                let reason = format!(
                    "segment {:?} -> {:?} -> {:?} (gate `{gate}`): {reason}",
                    wp_before, anchor, wp_after
                );
                self.touch_bus(bus);
                let layer = self.buses.get_mut(bus).unwrap();
                layer.fragment.clear();
                layer.runs.clear();
                layer.segments.clear();
                layer.gate_cells.clear();
                layer.state = BusState::Failed(reason.clone());
                Ok(GateMoveReport {
                    state: BusState::Failed(reason),
                    rerouted_segments: 2,
                    changed: self.changed_layers_since(rev0),
                })
            }
        }
    }

    /// Rip a bus: clear its fragment and return it to `Intended`. Station
    /// amendments stamped into OTHER buses by crossings stay (they remain
    /// electrically sound straight-line refreshes).
    pub fn rip(&mut self, name: &str) -> Result<(), String> {
        if !self.buses.contains_key(name) {
            return Err(format!("unknown bus `{name}`"));
        }
        self.touch_bus(name);
        let bus = self.buses.get_mut(name).expect("checked above");
        bus.fragment.clear();
        bus.runs.clear();
        bus.segments.clear();
        bus.gate_cells.clear();
        bus.state = BusState::Intended;
        Ok(())
    }

    /// Re-realize a bus from its stored declaration (endpoints, gates, style
    /// and rule all kept). The counterpart to [`Design::rip`]: rip to free
    /// the space, reroute to try again once the obstacle moved.
    pub fn reroute(&mut self, name: &str) -> Result<BusState, String> {
        if !self.buses.contains_key(name) {
            return Err(format!("unknown bus `{name}`"));
        }
        let _ = self.rip(name);
        Ok(self.reroute_one(name))
    }

    /// Delete a bus outright: its fragment AND its declaration. Unlike
    /// [`Design::rip`] (which keeps the declaration so it can be rerouted),
    /// the name becomes free again.
    pub fn remove_bus(&mut self, name: &str) -> Result<(), String> {
        self.rip(name)?; // stamps the layer as changed
        self.buses.remove(name);
        Ok(())
    }

    /// The design-wide spatial occupancy index: loose blocks, instance
    /// footprints, routed bus fragments, and instance influence halos.
    pub fn occupancy_index(&self) -> OccupancyIndex {
        self.occupancy_for_plan(&BTreeSet::new(), &BTreeSet::new())
    }

    /// The instances whose influence halo a bus with these endpoints may
    /// enter: the ones that own an endpoint. Routing *into* the cell you are
    /// connecting to is legal — that is what a pin is for — while the
    /// instance's hard body cells still protect themselves, and every other
    /// instance's halo still blocks.
    fn halo_exempt(&self, endpoints: &[String]) -> BTreeSet<String> {
        endpoints
            .iter()
            .filter_map(|p| p.split_once('.').map(|(i, _)| i.to_string()))
            .filter(|i| self.instances.iter().any(|inst| &inst.name == i))
            .collect()
    }

    /// The occupancy index minus `skip` (cells a partial rip vacated), with
    /// the halos of `halo_exempt` instances suppressed (pin access).
    fn occupancy_for_plan(
        &self,
        skip: &BTreeSet<P3>,
        halo_exempt: &BTreeSet<String>,
    ) -> OccupancyIndex {
        let mut idx = OccupancyIndex::default();
        for (bp, bs) in self.base.iter_blocks() {
            let s = bs.to_string();
            if s.contains("minecraft:air") {
                continue;
            }
            let p = (bp.x, bp.y, bp.z);
            if skip.contains(&p) {
                continue;
            }
            idx.cells.insert(p, (s, Occupant::Loose));
        }
        for inst in &self.instances {
            let cell = &self.cells[&inst.cell];
            let bbox = cell_bounds(&cell.schematic);
            for (bp, bs) in self.instance_local_blocks(inst) {
                let s = transform_state(&bs, inst.rot_y).to_string();
                if s.contains("minecraft:air") {
                    continue;
                }
                let p = transform_pos(bp, bbox.min, bbox.max, inst.rot_y, inst.at);
                if skip.contains(&p) {
                    continue;
                }
                idx.cells.insert(p, (s, Occupant::Instance(inst.name.clone())));
            }
        }
        for bus in self.buses.values() {
            for (p, b) in &bus.fragment {
                if skip.contains(p) {
                    continue;
                }
                idx.cells.insert(*p, (b.clone(), Occupant::Bus(bus.name.clone())));
            }
        }
        // Influence halos never shadow hard cells.
        for inst in &self.instances {
            if halo_exempt.contains(&inst.name) {
                continue;
            }
            let cell = &self.cells[&inst.cell];
            let bbox = cell_bounds(&cell.schematic);
            let map = |p: P3| transform_pos(p, bbox.min, bbox.max, inst.rot_y, inst.at);
            for (min, max) in Self::halo_boxes(cell, bbox.min, bbox.max) {
                let a = map(min);
                let b = map(max);
                let lo = (a.0.min(b.0), a.1.min(b.1), a.2.min(b.2));
                let hi = (a.0.max(b.0), a.1.max(b.1), a.2.max(b.2));
                for x in lo.0..=hi.0 {
                    for y in lo.1..=hi.1 {
                        for z in lo.2..=hi.2 {
                            let p = (x, y, z);
                            if idx.cells.contains_key(&p) || skip.contains(&p) {
                                continue;
                            }
                            idx.halos.insert(p, inst.name.clone());
                        }
                    }
                }
            }
        }
        idx
    }

    /// Realize a bus (pure planning: no mutation). `Err` is the
    /// user-facing failure reason. `exclude` names the bus being planned
    /// so its own surviving runs are not treated as crossings.
    ///
    /// Realization shape: a TRUNK (primary driver -> gates -> primary
    /// sink; straight runs, or one implicit L corner per waypoint pair)
    /// plus a BRANCH per extra sink (fanout) and per extra driver
    /// (wired-OR), each joining the trunk at a plain-dust junction,
    /// diode-isolated by a repeater on the branch side.
    #[allow(clippy::too_many_arguments)]
    fn realize(
        &self,
        exclude: Option<&str>,
        drivers: &[DesignPort],
        sinks: &[DesignPort],
        gates: &[Gate],
        style: &BusStyle,
        width_map: Option<&WidthMap>,
    ) -> Result<Realization, String> {
        let step = (0, 2, 0);
        // WIDTH ADAPTATION is pure geometry once the mapping is resolved: route
        // the OVERLAPPING bits only, and slide each end's bit-0 anchor to the
        // first bit it actually carries. The stack pitch is unchanged, so every
        // template, crossing rule and DRC downstream sees an ordinary bus that
        // happens to be narrower. Sink bits nothing drives are simply not built
        // — undriven dust is logical 0, so tying them costs no hardware.
        let width = width_map.map_or(drivers[0].width, |m| m.bits);
        let (drivers, sinks) = match width_map {
            None => (drivers.to_vec(), sinks.to_vec()),
            Some(m) => {
                let slide = |p: &DesignPort, bit: i32| -> DesignPort {
                    let mut q = p.clone();
                    q.anchor = add(q.anchor, scale(q.step, bit));
                    q.width = m.bits;
                    q
                };
                (
                    vec![slide(&drivers[0], m.from_bit as i32)],
                    vec![slide(&sinks[0], m.from_bit as i32 + m.shift)],
                )
            }
        };
        let (drivers, sinks) = (&drivers[..], &sinks[..]);
        for p in drivers.iter().chain(sinks.iter()) {
            if p.step != step && p.step.1 != 0 {
                return Err(format!(
                    "unsupported bus form: this design realizes the verified vertical 2y-pitch \
                     stack (step (0,2,0)), and can adapt a HORIZONTAL row onto it; port `{}` has \
                     step {:?}, which is neither",
                    p.name, p.step
                ));
            }
        }
        for g in gates {
            if g.step != step {
                return Err(format!(
                    "gate `{}`: step {:?} does not match the bus form (0,2,0)",
                    g.name, g.step
                ));
            }
        }

        // Pin access: this bus may enter the halo of the instances it
        // terminates on.
        let endpoints: Vec<String> = drivers
            .iter()
            .chain(sinks.iter())
            .map(|p| p.name.clone())
            .collect();
        let mut occ = self.occupancy_for_plan(&BTreeSet::new(), &self.halo_exempt(&endpoints));

        // FORM ADAPTATION IS THE BUS'S JOB. A port whose native geometry is a
        // horizontal row gets a row->stack adapter planned into THIS BUS's
        // fragment, so it is created and ripped with the bus and the component
        // is never edited beyond its own minimal in-place promotion.
        let mut adapters: Vec<(String, P3, crate::design_promote::PivotPlan)> = Vec::new();
        let mut anchor_of: BTreeMap<String, P3> = BTreeMap::new();
        for p in drivers.iter().chain(sinks.iter()) {
            if p.step == step {
                continue;
            }
            let plan = self.plan_form_adapter(p, &occ)?;
            anchor_of.insert(p.name.clone(), plan.column[0]);
            adapters.push((p.name.clone(), p.anchor, plan));
        }
        // The adapter's cells are the bus's own: other legs must route AROUND
        // them (so add them to the occupancy the planner sees) while the
        // planner is still allowed to write them (so mark them vacated).
        let mut adapter_cells: BTreeSet<P3> = BTreeSet::new();
        for (name, _, plan) in &adapters {
            for (q, blk) in &plan.cells {
                adapter_cells.insert(*q);
                occ.cells.insert(
                    *q,
                    (blk.clone(), Occupant::Bus(exclude.unwrap_or(name).to_string())),
                );
            }
        }
        let occ = occ;
        // Downstream everything — waypoints, branch geometry, the planner —
        // works from the ADAPTED port: its anchor is the adapter's column head
        // and its form is the canonical stack. Nothing else needs to know.
        let adapt = |p: &DesignPort| -> DesignPort {
            let mut q = p.clone();
            if let Some(a) = anchor_of.get(&p.name) {
                q.anchor = *a;
                q.step = step;
            }
            q
        };
        let drivers: Vec<DesignPort> = drivers.iter().map(adapt).collect();
        let sinks: Vec<DesignPort> = sinks.iter().map(adapt).collect();
        let (drivers, sinks) = (&drivers[..], &sinks[..]);

        // Trunk waypoint chain: primary driver, gates, primary sink.
        let mut waypoints = vec![drivers[0].anchor];
        waypoints.extend(gates.iter().map(|g| g.anchor));
        waypoints.push(sinks[0].anchor);
        // A pair whose anchors sit on different levels is NOT a failure: the
        // planner inserts the verified level-shift tile (`shift_plan`) and the
        // bus changes level in form. Only a pair with too little run for the
        // tile fails, and it says so with the numbers.

        // Branch junctions must sit on the trunk the planner ACTUALLY lays,
        // and a trunk pair may now detour around an obstacle instead of
        // taking the x-first template. So when there are branches, plan the
        // trunk once as a probe to learn its real runs, then plan for real
        // with the junctions kept as plain dust. Without branches the probe
        // is pure waste, so skip it.
        let has_branches = sinks.len() > 1 || drivers.len() > 1;
        let trunk_runs = if has_branches {
            Self::probe_trunk_runs(self, exclude, style, &occ, &waypoints, width)?
        } else {
            Vec::new()
        };
        let mut branches: Vec<(String, RunInfo, P3, bool, P3)> = Vec::new();
        let mut keep: BTreeSet<P3> = BTreeSet::new();
        for sp in &sinks[1..] {
            let (run, junction) = Self::branch_geometry(&trunk_runs, sp, false)?;
            keep.insert(junction);
            branches.push((sp.name.clone(), run, junction, false, sp.anchor));
        }
        for dp in &drivers[1..] {
            let (run, junction) = Self::branch_geometry(&trunk_runs, dp, true)?;
            keep.insert(junction);
            branches.push((dp.name.clone(), run, junction, true, dp.anchor));
        }

        let mut planner = Planner::new(self, exclude, style, &occ);
        // Stamp the form adapters FIRST: they are this bus's cells, so the
        // planner must be allowed to write them even though they are in `occ`
        // (which is what makes the other legs route around them).
        planner.vacated.extend(adapter_cells.iter().copied());
        for (port, row_anchor, plan) in &adapters {
            planner.begin_segment(SegmentKind::Adapter(port.clone()), *row_anchor, plan.column[0]);
            for (q, blk) in &plan.cells {
                planner
                    .put(*q, blk)
                    .map_err(|e| format!("form adapter for `{port}`: {e}"))?;
            }
            planner.end_segment();
        }
        let mut since = 0u32;
        for (i, pair) in waypoints.windows(2).enumerate() {
            planner.begin_segment(SegmentKind::Trunk(i), pair[0], pair[1]);
            since = planner
                .plan_pair(pair[0], pair[1], width, &keep, since)
                .map_err(|e| format!("segment {:?} -> {:?}: {e}", pair[0], pair[1]))?;
            planner.end_segment();
            // The gate joint between pairs is one more dust cell.
            since = since.saturating_add(1);
        }
        for g in gates {
            planner.plan_column(&g.name, g.anchor, width)?;
        }
        for (port, run, junction, is_driver, port_anchor) in &branches {
            // The junction must have survived as plain dust on the trunk
            // (a flipped corner or a crossing window would have eaten it).
            if !planner
                .real
                .fragment
                .get(junction)
                .is_some_and(|b| rblocks::is_dust(b))
            {
                return Err(format!(
                    "branch for `{port}`: junction {:?} did not survive as plain dust on the \
                     trunk; add a gate to shift the join",
                    junction
                ));
            }
            let (a, b) = if *is_driver {
                (*port_anchor, *junction)
            } else {
                (*junction, *port_anchor)
            };
            planner.begin_segment(SegmentKind::Branch(port.clone()), a, b);
            planner
                .plan_run(run, &BTreeSet::new(), 0, !*is_driver, *is_driver)
                .map_err(|e| format!("branch for `{port}`: {e}"))?;
            planner.end_segment();
        }
        let mut real = planner.finish();

        // TWO STRUCTURAL INVARIANTS, checked here because this is the ONLY
        // place a bus's geometry comes from. A "routed" bus with nothing built
        // is the worst possible outcome — green status, empty layer, and a
        // viewer that trusts the status shows nothing and says everything is
        // fine. Make it unrepresentable rather than merely unlikely.
        if real.fragment.is_empty() {
            return Err(format!(
                "internal: the plan for {:?} -> {:?} produced NO cells. A routed bus always \
                 builds something; refusing rather than reporting an empty layer as routed",
                waypoints[0],
                waypoints[waypoints.len() - 1]
            ));
        }
        // A gate is a CHECKPOINT: the realized path must actually pass through
        // it. `plan_column` lays the joint, so a missing gate cell means the
        // column was never planned — the waypoint was silently skipped.
        for g in gates {
            for k in 0..width {
                let p = add(g.anchor, (0, 2 * k as i32, 0));
                if !real.fragment.contains_key(&p) {
                    return Err(format!(
                        "internal: gate `{}` was not realized — bit {k} of its checkpoint column \
                         at {:?} is absent from the route, so the bus does not pass through the \
                         gate it was given",
                        g.name, p
                    ));
                }
            }
        }

        // DRAW THE WIRE, not a trail of dots. Every dust cell above was
        // authored in the fully-spelled-out DEFAULT state — which is right for
        // interning (a bare `redstone_wire` interns a property-less state that
        // tick engines never normalise, and those cells sit INERT) but is
        // geometrically a DOT. Minecraft derives the connection state from the
        // neighbours at placement time; so do we, here, once the whole fragment
        // is known. No simulation, no bake: cheap and deterministic.
        Self::rewire_fragment(&mut real.fragment, &occ);
        for (_, _, additions) in real.amendments.iter_mut() {
            Self::rewire_fragment(additions, &occ);
        }
        Ok(real)
    }

    /// Give every dust cell in `cells` its geometrically derived connection
    /// state, reading the rest of the world out of `occ`.
    fn rewire_fragment(cells: &mut BTreeMap<P3, String>, occ: &OccupancyIndex) {
        let outside = |q: P3| -> Option<String> { occ.cells.get(&q).map(|(b, _)| b.clone()) };
        crate::routing::engine::wire::rewire(cells, &outside);
    }

    /// Plan the row->stack FORM ADAPTER for a port whose native geometry is a
    /// horizontal row, against the design-wide occupancy.
    ///
    /// The cells come back as a pure plan; the caller stamps them into the
    /// BUS's fragment. That ownership is the point: promotion is minimal and
    /// in-place (a horizontal row of 8 stays a horizontal row of 8 at its
    /// native pitch, inside the cell's own footprint), and the adapter — which
    /// exists only to serve one bus and reaches well outside the component —
    /// is created and RIPPED with that bus.
    fn plan_form_adapter(
        &self,
        port: &DesignPort,
        occ: &OccupancyIndex,
    ) -> Result<crate::design_promote::PivotPlan, String> {
        let row: Vec<P3> = (0..port.width as i32)
            .map(|i| {
                (
                    port.anchor.0 + port.step.0 * i,
                    port.anchor.1 + port.step.1 * i,
                    port.anchor.2 + port.step.2 * i,
                )
            })
            .collect();
        // What the adapter must treat as taken: hard cells, and any halo it is
        // not exempt from (`occ` already has the endpoint instances' halos
        // suppressed, so pin access still works).
        let at = |q: P3| -> Option<String> {
            occ.cells
                .get(&q)
                .map(|(b, _)| b.clone())
                .or_else(|| occ.halos.get(&q).map(|i| format!("the influence halo of `{i}`")))
        };
        // Grow AWAY from the owning instance's body, so the adapter leaves the
        // component instead of burrowing into it.
        let away = port
            .name
            .split_once('.')
            .and_then(|(inst, _)| self.instances.iter().find(|i| i.name == inst))
            .map(|inst| {
                let blocks = self.placed_instance_blocks(inst);
                let n = blocks.len().max(1) as i64;
                let (sx, sy, sz) = blocks.keys().fold((0i64, 0i64, 0i64), |a, q| {
                    (a.0 + q.0 as i64, a.1 + q.1 as i64, a.2 + q.2 as i64)
                });
                ((sx / n) as i32, (sy / n) as i32, (sz / n) as i32)
            })
            .unwrap_or(port.anchor);
        // A port that DRIVES the fabric flows out of the row into the column;
        // a sink flows the other way.
        let flow_out = port.direction == PortDirection::Input;
        crate::design_promote::plan_pivot(&row, port.step, away, flow_out, &at)
            .map_err(|e| format!("port `{}`: {e}", port.name))
    }

    /// Plan the trunk as a throwaway probe and report the runs it actually
    /// laid down. Branch junctions are derived from THESE runs, not from the
    /// x-first template guess: a pair that had to detour around an obstacle
    /// ends up nowhere near its template, and a junction chosen off the guess
    /// would not exist on the real trunk.
    ///
    /// The probe is deterministic, so the real pass reproduces its geometry
    /// exactly (the only difference is the `keep` set, which changes whether a
    /// refresh repeater may land on a junction cell, never the corridor).
    fn probe_trunk_runs(
        &self,
        exclude: Option<&str>,
        style: &BusStyle,
        occ: &OccupancyIndex,
        waypoints: &[P3],
        width: u8,
    ) -> Result<Vec<RunInfo>, String> {
        let mut probe = Planner::new(self, exclude, style, occ);
        let mut since = 0u32;
        for (i, pair) in waypoints.windows(2).enumerate() {
            probe.begin_segment(SegmentKind::Trunk(i), pair[0], pair[1]);
            since = probe
                .plan_pair(pair[0], pair[1], width, &BTreeSet::new(), since)
                .map_err(|e| format!("segment {:?} -> {:?}: {e}", pair[0], pair[1]))?;
            probe.end_segment();
            since = since.saturating_add(1);
        }
        Ok(probe.finish().segments.iter().flat_map(|s| s.runs.clone()).collect())
    }

    /// The trunk's straight runs from its waypoint chain, choosing the
    /// x-first corner for non-straight pairs (deterministic; the planner
    /// may flip a congested corner z-first at plan time).
    #[allow(dead_code)]
    fn trunk_geometry(waypoints: &[P3], width: u8) -> Result<Vec<RunInfo>, String> {
        let mut runs = Vec::new();
        for pair in waypoints.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a == b {
                return Err(format!("segment {:?} -> {:?}: zero length", a, b));
            }
            if a.2 == b.2 && a.0 != b.0 {
                runs.push(RunInfo {
                    along_x: true,
                    fixed: a.2,
                    y0: a.1,
                    from: a.0,
                    to: b.0,
                    width,
                });
            } else if a.0 == b.0 && a.2 != b.2 {
                runs.push(RunInfo {
                    along_x: false,
                    fixed: a.0,
                    y0: a.1,
                    from: a.2,
                    to: b.2,
                    width,
                });
            } else {
                runs.push(RunInfo {
                    along_x: true,
                    fixed: a.2,
                    y0: a.1,
                    from: a.0,
                    to: b.0,
                    width,
                });
                runs.push(RunInfo {
                    along_x: false,
                    fixed: b.0,
                    y0: a.1,
                    from: a.2,
                    to: b.2,
                    width,
                });
            }
        }
        Ok(runs)
    }

    /// Find the trunk run an extra endpoint can join with one
    /// perpendicular straight shot; returns the branch run and the
    /// junction cell (bit 0) on the trunk.
    fn branch_geometry(
        trunk: &[RunInfo],
        port: &DesignPort,
        is_driver: bool,
    ) -> Result<(RunInfo, P3), String> {
        let s = port.anchor;
        for r in trunk {
            if r.y0 != s.1 {
                continue;
            }
            let (along_coord, cross_coord) = if r.along_x { (s.0, s.2) } else { (s.2, s.0) };
            if cross_coord == r.fixed || !r.strictly_inside(along_coord, 2) {
                continue;
            }
            let junction = if r.along_x {
                (s.0, s.1, r.fixed)
            } else {
                (r.fixed, s.1, s.2)
            };
            let (from, to) = if is_driver {
                (cross_coord, r.fixed)
            } else {
                (r.fixed, cross_coord)
            };
            let run = RunInfo {
                along_x: !r.along_x,
                fixed: along_coord,
                y0: s.1,
                from,
                to,
                width: port.width,
            };
            return Ok((run, junction));
        }
        Err(format!(
            "no trunk run aligns with `{}` at {:?} (needs a perpendicular straight shot into \
             the trunk interior); add a gate",
            port.name, s
        ))
    }

    // ------------------------------------------------------------------
    // flatten / check / bake
    // ------------------------------------------------------------------

    /// One instance's non-air blocks in WORLD space, transform applied.
    ///
    /// The single definition of "where an instance's blocks actually are",
    /// shared by [`Design::flatten`] and
    /// [`Design::instance_blocks_json`] so a viewer can never disagree with an
    /// export about an instance's position.
    fn placed_instance_blocks(&self, inst: &Instance) -> BTreeMap<P3, crate::BlockState> {
        let cell = &self.cells[&inst.cell];
        let bbox = cell_bounds(&cell.schematic);
        let mut out = BTreeMap::new();
        for (bp, bs) in self.instance_local_blocks(inst) {
            if bs.to_string().contains("minecraft:air") {
                continue;
            }
            let p = transform_pos(bp, bbox.min, bbox.max, inst.rot_y, inst.at);
            out.insert(p, transform_state(&bs, inst.rot_y));
        }
        // A PROMOTED port's dust is authored before any bus exists, so in
        // isolation it is correctly a dot; once a bus lands on it, it must be
        // drawn connected. This is the single definition of where an instance's
        // blocks are — shared by `flatten` and `instance_blocks_json` — so
        // deriving here is what keeps a viewer and an export agreeing about it.
        //
        // ONLY the promotion patch's OWN cells. A placed cell is a verified
        // black box: its interior wire states were authored by whoever built
        // it, over blocks the wire model does not classify, and rewriting them
        // breaks working redstone — measured, when this pass covered every dust
        // in the body it turned the ADD007 -> BINTOBCD001 chain's arithmetic
        // wrong (1+1 read 0).
        let ours: BTreeSet<P3> = inst
            .port_modes
            .values()
            .filter(|o| o.mode == PortMode::Bus)
            .flat_map(|o| o.patch.writes.keys())
            .map(|bp| transform_pos(*bp, bbox.min, bbox.max, inst.rot_y, inst.at))
            .collect();
        let dust: Vec<P3> = out
            .iter()
            .filter(|(p, b)| ours.contains(p) && rblocks::is_dust(&b.to_string()))
            .map(|(p, _)| *p)
            .collect();
        if !dust.is_empty() {
            let mine: BTreeMap<P3, String> =
                out.iter().map(|(p, b)| (*p, b.to_string())).collect();
            let at = |q: P3| -> Option<String> { mine.get(&q).cloned().or_else(|| self.neighbour_block(q)) };
            for p in dust {
                let power = crate::routing::engine::wire::power_of(&mine[&p]);
                let derived = crate::routing::engine::wire::derive(p, power, &at);
                if let Ok(bs) = crate::BlockState::from_block_string(&derived) {
                    out.insert(p, bs);
                }
            }
        }
        out
    }

    /// The block at `p` from the layers that can sit beside an instance: the
    /// routed bus fragments and the loose layer. Other INSTANCES are skipped on
    /// purpose — influence halos keep two of them from ever touching, so paying
    /// to scan them all on every neighbour lookup would buy nothing.
    fn neighbour_block(&self, p: P3) -> Option<String> {
        for bus in self.buses.values() {
            if let Some(b) = bus.fragment.get(&p) {
                return Some(b.clone());
            }
        }
        self.base
            .get_block(p.0, p.1, p.2)
            .map(|b| b.to_string())
            .filter(|b| !b.contains("minecraft:air"))
    }

    /// `[[x,y,z,"block"],..]` for ONE bus layer's cells.
    ///
    /// Exists for live re-routing: fetching a single changed bus through
    /// [`Design::flatten`] means rebuilding every layer in the document (~22ms
    /// on a real design) to read back a few hundred cells. This reads the bus's
    /// own fragment directly. An unrouted bus has no cells and yields `[]` —
    /// that is a legal answer, not an error, so a caller can poll a bus it just
    /// failed to route without special-casing.
    ///
    /// Positional array-of-arrays rather than `{"at":..,"block":..}` on purpose:
    /// the whole point is byte volume across the bridge.
    pub fn bus_blocks_json(&self, name: &str) -> Result<String, String> {
        let bus = self
            .buses
            .get(name)
            .ok_or_else(|| format!("unknown bus `{name}`"))?;
        Ok(Self::cells_json(bus.fragment.iter().map(|(p, b)| (*p, b.as_str()))))
    }

    /// `[[x,y,z,"block"],..]` for ONE instance's placed blocks, transform
    /// applied. Same reasoning as [`Design::bus_blocks_json`].
    pub fn instance_blocks_json(&self, name: &str) -> Result<String, String> {
        let inst = self
            .instances
            .iter()
            .find(|i| i.name == name)
            .ok_or_else(|| format!("unknown instance `{name}`"))?;
        let placed: Vec<(P3, String)> = self
            .placed_instance_blocks(inst)
            .into_iter()
            .map(|(p, b)| (p, b.to_string()))
            .collect();
        Ok(Self::cells_json(
            placed.iter().map(|(p, b)| (*p, b.as_str())),
        ))
    }

    /// Render `(pos, block)` pairs as `[[x,y,z,"block"],..]`.
    fn cells_json<'a>(cells: impl Iterator<Item = (P3, &'a str)>) -> String {
        let items: Vec<String> = cells
            .map(|(p, b)| {
                // Block states are `ns:name[k=v,..]` — no quotes to escape in
                // practice, but go through the serializer rather than trusting
                // that: a malformed state must not be able to forge JSON.
                let block = serde_json::to_string(b).unwrap_or_else(|_| "\"\"".to_string());
                format!("[{},{},{},{}]", p.0, p.1, p.2, block)
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    /// Collapse the layer stack into ONE self-describing schematic: the
    /// loose layer stays in the base regions, every instance becomes region
    /// `inst:{name}`, every routed bus region `bus:{name}`, and the merged
    /// transformed contract is embedded in the metadata — the artifact is
    /// itself placeable as a cell.
    pub fn flatten(&self) -> Result<UniversalSchematic, String> {
        let mut flat = self.base.clone();
        flat.metadata.name = Some(self.name.clone());

        for inst in &self.instances {
            let region = format!("inst:{}", inst.name);
            for (p, state) in self.placed_instance_blocks(inst) {
                if !flat.set_block_in_region(&region, p.0, p.1, p.2, &state) {
                    return Err(format!(
                        "flatten: could not place {state} at {p:?} in region {region}"
                    ));
                }
            }
        }

        for bus in self.buses.values() {
            if bus.state != BusState::Routed {
                continue;
            }
            let region = format!("bus:{}", bus.name);
            for (p, b) in &bus.fragment {
                flat.try_set_block_in_region_str(&region, p.0, p.1, p.2, b)?;
            }
        }

        let contract = self.merged_contract()?;
        flat.set_cell_contract(&contract)?;
        Ok(flat)
    }

    /// The merged contract of the flattened artifact: every design port
    /// (executor-facing positions: levers for drivable inputs, connection
    /// dust for outputs) plus every instance port under `{inst}.{port}`,
    /// transformed.
    pub fn merged_contract(&self) -> Result<CellContract, String> {
        let mut builder = crate::io_contract::IoLayoutBuilder::new();
        for port in self.ports.values() {
            match port.direction {
                PortDirection::Input => {
                    let levers: Vec<P3> = port
                        .bits
                        .iter()
                        .enumerate()
                        .map(|(k, hw)| {
                            hw.lever.ok_or_else(|| {
                                format!("input `{}` bit {k} lost its lever", port.name)
                            })
                        })
                        .collect::<Result<_, String>>()?;
                    builder = builder.add_input(
                        port.name.clone(),
                        port.ty.clone(),
                        LayoutFunction::OneToOne,
                        levers,
                    )?;
                }
                PortDirection::Output => {
                    builder = builder.add_output(
                        port.name.clone(),
                        port.ty.clone(),
                        LayoutFunction::OneToOne,
                        port.wires(),
                    )?;
                }
            }
        }
        for inst in &self.instances {
            let transformed = self.instance_contract(&inst.name)?;
            for (pname, mapping) in &transformed.io.inputs {
                builder = builder.add_input(
                    format!("{}.{}", inst.name, pname),
                    mapping.io_type.clone(),
                    mapping.layout.clone(),
                    mapping.positions.clone(),
                )?;
            }
            for (pname, mapping) in &transformed.io.outputs {
                builder = builder.add_output(
                    format!("{}.{}", inst.name, pname),
                    mapping.io_type.clone(),
                    mapping.layout.clone(),
                    mapping.positions.clone(),
                )?;
            }
        }
        Ok(CellContract::new(self.name.clone(), builder.build()))
    }

    /// The LVS intent netlist of the routed buses: one net per bit, its
    /// terminals the driver and sink connection cells.
    pub fn intent_nets(&self) -> Vec<crate::routing::IntentNet> {
        use crate::routing::Pos;
        let mut nets = Vec::new();
        for bus in self.buses.values() {
            if bus.state != BusState::Routed {
                continue;
            }
            let Ok(driver) = self.resolve_port(&bus.driver) else {
                continue;
            };
            // A WIDTH-ADAPTED bus does not pair bit k with bit k: driver bit
            // `from_bit + i` drives sink bit `from_bit + i + shift`, and the
            // sink bits nothing drives are not part of any net (undriven dust is
            // logical 0, not a broken connection). Pairing by index here would
            // report the whole bus as opens and shorts.
            let (first, count, shift) = bus
                .width_map
                .as_ref()
                .map_or((0u8, driver.width, 0i32), |m| (m.from_bit, m.bits, m.shift));
            for i in 0..count {
                let bit = first + i;
                let sink_bit = bit as i32 + shift;
                // ONE net per bit: every driver (wired-OR merges stay one
                // intent net) plus every sink.
                let mut terminals = Vec::new();
                for dn in bus.driver_names() {
                    if let Some(dp) = self.ports.get(&dn) {
                        terminals.push(dp.wire(bit));
                    }
                }
                for s in &bus.sinks {
                    if let Some(sp) = self.ports.get(s) {
                        terminals.push(sp.wire(sink_bit.max(0) as u8));
                    }
                }
                nets.push(crate::routing::IntentNet {
                    name: format!("{}[{}]", bus.name, bit),
                    terminals: terminals
                        .into_iter()
                        .map(|(x, y, z)| Pos::new(x, y, z))
                        .collect(),
                });
            }
        }
        nets
    }

    /// DRC + LVS + STA/skew over the flattened artifact, plus the per-bus
    /// net-class rules. `clean` requires clean DRC, clean LVS and zero
    /// rule violations; STA numbers are advisory unless a rule bounds
    /// them (`max_len_rt`).
    pub fn check(&self) -> Result<DesignCheck, String> {
        let flat = self.flatten()?;
        let opts = crate::routing::DrcOptions {
            aliases: vec![],
            skip_decay: true,
        };
        let all = crate::routing::drc_schematic(&flat, &opts);
        // Split the DRC report at the cell boundary. A placed cell is a
        // VERIFIED BLACK BOX behind its keepout: hand-built community redstone
        // legitimately breaks the route-oriented conventions these rules
        // encode (one 8-bit community adder reports 253 "floating" cells for
        // dust our support predicate does not recognise), and blaming the
        // design for a library cell's interior means `check()` is never clean
        // and the whole report becomes noise. Interior findings are reported
        // under `cells` for information; only what the DESIGN itself owns —
        // the loose layer and the routed bus fragments — gates `clean`.
        let (violations, cell_violations): (Vec<_>, Vec<_>) =
            all.into_iter().partition(|v| !self.is_inside_an_instance(v));
        let ws = crate::routing::workspace_from_schematic(&flat);
        let lvs = crate::routing::lvs(ws.cells(), &self.intent_nets());
        let (sta_json, rule_violations) = self.sta_and_rules(&flat);
        // The cell boundary applies to LVS too, and here it is not merely
        // noise-reduction: a register's latch IS a repeater ring, so
        // `lvs.cycles` reports every memory cell in the library as an
        // "accidental latch". A ring living entirely inside one placed cell is
        // that cell's verified internal state, not a design error.
        let bodies = self.instance_bodies();
        let interior = |cells: &[crate::routing::Pos]| -> bool {
            !cells.is_empty()
                && cells.iter().all(|q| {
                    let p = (q.x, q.y, q.z);
                    bodies.iter().any(|b| b.contains(&p))
                })
        };
        let design_cycles = lvs.cycles.iter().filter(|ring| !interior(ring)).count();
        let design_shorts = lvs
            .shorts
            .iter()
            .filter(|s| !interior(&[s.at_a, s.at_b]))
            .count();
        let design_opens = lvs
            .opens
            .iter()
            .filter(|o| !interior(&o.fragments.concat()))
            .count();
        // A bus reported ROUTED must have built something. Nothing upstream can
        // produce this any more (`realize` refuses an empty plan outright), but
        // it is asserted here too because it is the single worst state to ship:
        // green status, empty layer, and a viewer that trusts the status draws
        // nothing and reports success.
        let empty_routed: Vec<String> = self
            .buses
            .values()
            .filter(|b| b.state == BusState::Routed && b.fragment.is_empty())
            .map(|b| b.name.clone())
            .collect();
        let clean = violations.is_empty()
            && design_opens == 0
            && design_shorts == 0
            && design_cycles == 0
            && rule_violations.is_empty()
            && empty_routed.is_empty();
        let bus_states: Vec<String> = self
            .buses
            .values()
            .map(|b| {
                let state = match &b.state {
                    BusState::Intended => "\"intended\"".to_string(),
                    BusState::Routed => "\"routed\"".to_string(),
                    BusState::Failed(r) => format!("{{\"failed\":{:?}}}", r),
                };
                format!("{:?}:{state}", b.name)
            })
            .collect();
        let mut rules: Vec<String> = rule_violations.iter().map(|r| format!("{r:?}")).collect();
        for name in &empty_routed {
            rules.push(format!(
                "{:?}",
                format!("bus `{name}` reports ROUTED but built no cells")
            ));
        }
        let json = format!(
            "{{\"clean\":{clean},\"drc\":{},\"cells\":{},\"lvs\":{},\"buses\":{{{}}},\
             \"sta\":{sta_json},\"rules\":[{}]}}",
            crate::routing::violations_json(&violations),
            crate::routing::violations_json(&cell_violations),
            crate::routing::lvs_report_json(&lvs),
            bus_states.join(","),
            rules.join(",")
        );
        Ok(DesignCheck { clean, json })
    }

    /// Whether every cell a DRC violation points at lies inside some placed
    /// instance's own footprint — i.e. the finding is about a library cell's
    /// interior, not about anything this design laid down.
    ///
    /// A violation straddling the boundary (a cell's dust shorting against a
    /// bus, say) is NOT interior: it stays in the gating report, which is
    /// exactly where an integration error belongs.
    fn is_inside_an_instance(&self, v: &crate::routing::Violation) -> bool {
        let cells = Self::violation_cells(v);
        if cells.is_empty() {
            return false;
        }
        let bodies = self.instance_bodies();
        cells
            .iter()
            .all(|c| bodies.iter().any(|body| body.contains(c)))
    }

    /// Every placed instance's occupied cells, one set per instance.
    fn instance_bodies(&self) -> Vec<BTreeSet<P3>> {
        (0..self.instances.len()).map(|i| self.instance_region(i)).collect()
    }

    /// The cells a DRC violation implicates.
    fn violation_cells(v: &crate::routing::Violation) -> Vec<P3> {
        use crate::routing::Violation as V;
        let p = |q: &crate::routing::Pos| (q.x, q.y, q.z);
        match v {
            V::Short { at_a, at_b, .. } => vec![p(at_a), p(at_b)],
            V::Floating { at, .. } => vec![p(at)],
            V::UnattachedWallTorch { at, anchor } => vec![p(at), p(anchor)],
            V::RepeaterCycle { diodes } => diodes.iter().map(p).collect(),
            V::PowerStarved { at, .. } => vec![p(at)],
        }
    }

    /// Per-bit repeater delay (redstone ticks) of a routed bus, from its
    /// fragment: a repeater at even offset from the canonical level
    /// belongs to that bit's straight run, at odd offset to the bit's dip
    /// station one level down.
    pub fn bus_bit_delays(&self, bus: &BusLayer) -> Vec<u64> {
        let Ok(driver) = self.resolve_port(&bus.driver) else {
            return Vec::new();
        };
        let width = driver.width as i32;
        let y0 = driver.anchor.1;
        let mut per = vec![0u64; width as usize];
        for (p, b) in &bus.fragment {
            if !rblocks::is_repeater(b) {
                continue;
            }
            let off = p.1 - y0;
            let bit = if off.rem_euclid(2) == 0 {
                off / 2
            } else {
                (off + 1) / 2
            };
            if (0..width).contains(&bit) {
                per[bit as usize] += rblocks::repeater_delay(b) as u64;
            }
        }
        per
    }

    /// Per-bus skew as JSON: `{"per_bit_rt": [...], "skew_rt", "max_rt"}`.
    pub fn bus_skew_json(&self, name: &str) -> Option<String> {
        let bus = self.buses.get(name)?;
        let per = self.bus_bit_delays(bus);
        let max = per.iter().max().copied().unwrap_or(0);
        let min = per.iter().min().copied().unwrap_or(0);
        Some(format!(
            "{{\"per_bit_rt\":{:?},\"skew_rt\":{},\"max_rt\":{max}}}",
            per,
            max - min
        ))
    }

    /// STA over the design (buses as nets, drivers as sources, repeaters
    /// charged per net by the existing sta machinery) plus per-bus skew
    /// and the net-class rule violations.
    fn sta_and_rules(&self, flat: &UniversalSchematic) -> (String, Vec<String>) {
        use crate::routing::Pos;
        let mut ws = crate::routing::workspace_from_schematic(flat);
        let mut sinks_of: BTreeSet<String> = BTreeSet::new();
        let mut net_gates: Vec<crate::routing::sta::Gate> = Vec::new();
        let mut bus_json: Vec<String> = Vec::new();
        let mut rules: Vec<String> = Vec::new();
        for bus in self.buses.values() {
            if bus.state != BusState::Routed {
                continue;
            }
            // Label the fragment with the driving signal so the sta
            // machinery attributes the bus's repeaters to its net.
            for p in bus.fragment.keys() {
                ws.set_label(Pos::new(p.0, p.1, p.2), &bus.driver);
            }
            for s in &bus.sinks {
                sinks_of.insert(s.clone());
                net_gates.push(crate::routing::sta::Gate {
                    out: s.clone(),
                    ins: bus.driver_names(),
                    delay_rt: 0,
                });
            }
            let per = self.bus_bit_delays(bus);
            let max = per.iter().max().copied().unwrap_or(0);
            let min = per.iter().min().copied().unwrap_or(0);
            let skew = max - min;
            let mut len_ok = true;
            if let Some(rule) = &bus.rule {
                if let Some(limit) = rule.max_len_rt {
                    if max > limit as u64 {
                        len_ok = false;
                        rules.push(format!(
                            "bus `{}`: max bit delay {max}rt exceeds max_len_rt {limit}rt",
                            bus.name
                        ));
                    }
                }
                if let Some((lo, hi)) = rule.y_band {
                    if let Some(p) = bus.fragment.keys().find(|p| p.1 < lo || p.1 > hi) {
                        rules.push(format!(
                            "bus `{}`: cell {:?} outside y_band {lo}..={hi}",
                            bus.name, p
                        ));
                    }
                }
            }
            bus_json.push(format!(
                "{:?}:{{\"per_bit_rt\":{:?},\"skew_rt\":{skew},\"max_rt\":{max},\"len_ok\":{len_ok}}}",
                bus.name, per
            ));
        }
        let inputs: Vec<String> = self
            .buses
            .values()
            .filter(|b| b.state == BusState::Routed)
            .flat_map(|b| b.driver_names())
            .filter(|d| !sinks_of.contains(d))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let core = if net_gates.is_empty() {
            "{\"arrival_rt\":{},\"critical\":[]}".to_string()
        } else {
            match crate::routing::sta::sta(&ws, &inputs, &net_gates) {
                Ok(rep) => {
                    let arr: Vec<String> = rep
                        .arrival_rt
                        .iter()
                        .map(|(k, v)| format!("{k:?}:{v}"))
                        .collect();
                    let crit: Vec<String> =
                        rep.critical.iter().map(|s| format!("{s:?}")).collect();
                    format!(
                        "{{\"arrival_rt\":{{{}}},\"critical\":[{}]}}",
                        arr.join(","),
                        crit.join(",")
                    )
                }
                Err(e) => format!("{{\"error\":{:?}}}", format!("{e:?}")),
            }
        };
        let json = format!("{{\"design\":{core},\"buses\":{{{}}}}}", bus_json.join(","));
        (json, rules)
    }

    /// Settle the flattened artifact in the vanilla-accurate tick engine,
    /// write every settled state back, stamp `InitialState::Baked` into the
    /// embedded contract and return the baked artifact — the cell deploys
    /// trusting its saved states.
    #[cfg(all(feature = "simulation", feature = "bridge", feature = "mc-tick"))]
    pub fn bake(&self, budget: u32) -> Result<UniversalSchematic, String> {
        use crate::io_contract::InitialState;
        use crate::simulation::typed_executor::BackendCircuitExecutor;

        let mut flat = self.flatten()?;
        let mut contract = flat
            .embedded_cell_contract()?
            .ok_or("flatten() embeds a contract; none found")?;
        let extra = executor_extra_states();
        let extra_refs: Vec<&str> = extra.iter().map(String::as_str).collect();
        let mut cell = BackendCircuitExecutor::for_cell(flat.clone(), &contract, &extra_refs)?;
        cell.settle(budget);
        cell.bake_to(&mut flat)?;

        let mut port_values = std::collections::BTreeMap::new();
        let output_names: Vec<String> = contract.io.outputs.keys().cloned().collect();
        for name in output_names {
            if let Ok(v) = cell.read_output(&name) {
                port_values.insert(name, value_to_u64(&v));
            }
        }
        contract.physical.initial_state = InitialState::Baked { port_values };
        flat.set_cell_contract(&contract)?;
        Ok(flat)
    }

    /// Flatten and save the artifact as `.schem` bytes (the flat-artifact
    /// serialization tier; the embedded contract rides along).
    /// The flattened artifact as a SINGLE-REGION composite.
    ///
    /// `.schem` has no layer concept, and a layered schematic pushed through
    /// the region merge loses every named-layer cell that the loose layer's
    /// bounding box shadows — the merge mirrors [`UniversalSchematic::get_block`],
    /// which answers from the default region first whenever a coordinate falls
    /// inside its (dense) bounds, so a bus fragment threading the endpoint
    /// hardware's own bounding box reads back as air. Compositing the layers
    /// explicitly, topmost non-air wins, keeps the artifact whole. Layers are
    /// disjoint by construction (the planner refuses collisions), so the
    /// union is unambiguous.
    pub fn flatten_composite(&self) -> Result<UniversalSchematic, String> {
        let layered = self.flatten()?;
        let mut flat = UniversalSchematic::new(self.name.clone());
        for (pos, state) in layered.iter_blocks() {
            if state.name == "minecraft:air" {
                continue;
            }
            let s = state.to_string();
            flat.set_block_from_string(pos.x, pos.y, pos.z, &s)
                .map_err(|e| format!("composite: {s} at {:?}: {e}", (pos.x, pos.y, pos.z)))?;
        }
        flat.metadata = layered.metadata.clone();
        flat.metadata.name = Some(self.name.clone());
        Ok(flat)
    }

    pub fn to_schem_bytes(&self) -> Result<Vec<u8>, String> {
        let flat = self.flatten_composite()?;
        crate::formats::schematic::to_schematic(&flat).map_err(|e| e.to_string())
    }

    /// Flatten and save to a file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.to_schem_bytes()?).map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------------
    // Serialization hooks (`src/design_io.rs` owns the document formats:
    // `.nucm` project tier + `.litematic` layered interchange tier)
    // ------------------------------------------------------------------

    /// The document's parts, for the serialization module.
    #[allow(clippy::type_complexity)]
    pub(crate) fn io_parts(
        &self,
    ) -> (
        &str,
        &UniversalSchematic,
        &BTreeMap<String, CellDef>,
        &[Instance],
        &BTreeMap<String, DesignPort>,
        &BTreeMap<String, BusLayer>,
    ) {
        (
            &self.name,
            &self.base,
            &self.cells,
            &self.instances,
            &self.ports,
            &self.buses,
        )
    }

    /// Rebuild a design from deserialized parts (serialization module
    /// only — the loading format is responsible for internal consistency).
    pub(crate) fn from_io_parts(
        name: String,
        base: UniversalSchematic,
        cells: BTreeMap<String, CellDef>,
        instances: Vec<Instance>,
        ports: BTreeMap<String, DesignPort>,
        buses: BTreeMap<String, BusLayer>,
    ) -> Self {
        Design {
            name,
            base,
            cells,
            instances,
            ports,
            buses,
            // Policy, not document state: a reloaded design routes the same way
            // a fresh one does.
            auto_promote: true,
            // A reloaded document starts every layer at revision 0: a fresh
            // reader has drawn nothing yet, so nothing needs reporting until
            // the first edit.
            bus_revs: BTreeMap::new(),
            rev_clock: 0,
        }
    }
}

/// A planned realization: the new bus's fragment (broken down per
/// segment), the gate joint columns, plus amendments (removals +
/// additions) to the buses it crosses.
#[derive(Clone, Default)]
struct Realization {
    fragment: BTreeMap<P3, String>,
    amendments: Vec<(String, Vec<P3>, BTreeMap<P3, String>)>,
    segments: Vec<Segment>,
    gate_cells: BTreeMap<String, BTreeSet<P3>>,
}

/// The bus planner: plans cells against the design-wide occupancy index
/// (collisions and instance halos fail loudly AT PLAN TIME, keeping
/// realization atomic), tracks which segment owns each cell, and carries
/// the crossing logic — dip-under tiles where we cross a routed bus,
/// through-bus stations where a routed bus already dips under us.
struct Planner<'a> {
    design: &'a Design,
    /// The bus being planned (its own surviving runs are not crossings).
    exclude: Option<&'a str>,
    style: &'a BusStyle,
    occ: &'a OccupancyIndex,
    /// Cells this plan's amendments vacate (free for re-use).
    vacated: BTreeSet<P3>,
    real: Realization,
    cur_seg: Option<Segment>,
    cur_gate: Option<(String, BTreeSet<P3>)>,
    /// Level hops taken on the current pair. The hop rung plans cross-level
    /// legs, which recurse into `plan_pair`; without this a pair that cannot be
    /// routed at ANY level would hop forever.
    hops: u32,
}

impl<'a> Planner<'a> {
    fn new(
        design: &'a Design,
        exclude: Option<&'a str>,
        style: &'a BusStyle,
        occ: &'a OccupancyIndex,
    ) -> Self {
        Planner {
            design,
            exclude,
            style,
            occ,
            vacated: BTreeSet::new(),
            real: Realization::default(),
            cur_seg: None,
            cur_gate: None,
            hops: 0,
        }
    }

    fn begin_segment(&mut self, kind: SegmentKind, a: P3, b: P3) {
        self.cur_seg = Some(Segment {
            kind,
            a,
            b,
            runs: Vec::new(),
            cells: BTreeSet::new(),
        });
    }

    fn end_segment(&mut self) {
        if let Some(seg) = self.cur_seg.take() {
            self.real.segments.push(seg);
        }
    }

    fn finish(self) -> Realization {
        self.real
    }

    #[allow(clippy::type_complexity)]
    fn snapshot(&self) -> (Realization, Option<Segment>, BTreeSet<P3>) {
        (self.real.clone(), self.cur_seg.clone(), self.vacated.clone())
    }

    fn restore(&mut self, s: (Realization, Option<Segment>, BTreeSet<P3>)) {
        self.real = s.0;
        self.cur_seg = s.1;
        self.vacated = s.2;
    }

    /// Add a planned cell. Identical double-writes are fine (shared
    /// supports); diverging ones are a planner bug; cells hard-occupied by
    /// anything else, or inside an instance's influence halo, fail with
    /// the owner named.
    fn put(&mut self, p: P3, block: &str) -> Result<(), String> {
        if let Some(existing) = self.real.fragment.get(&p) {
            if existing != block {
                return Err(format!(
                    "internal plan conflict at {:?}: `{existing}` vs `{block}`",
                    p
                ));
            }
            return Ok(());
        }
        if !self.vacated.contains(&p) {
            if let Some((existing, owner)) = self.occ.cells.get(&p) {
                if existing != block {
                    let owner = match owner {
                        Occupant::Loose => "loose block layer".to_string(),
                        Occupant::Instance(n) => format!("instance `{n}`"),
                        Occupant::Bus(n) => format!("bus `{n}`"),
                    };
                    return Err(format!(
                        "collision at {:?}: `{existing}` ({owner}) already there, wanted \
                         `{block}`",
                        p
                    ));
                }
            } else if let Some(inst) = self.occ.halos.get(&p) {
                return Err(format!(
                    "cell {:?} lies inside the influence halo of instance `{inst}`",
                    p
                ));
            }
        }
        self.real.fragment.insert(p, block.to_string());
        if let Some(seg) = self.cur_seg.as_mut() {
            seg.cells.insert(p);
        } else if let Some((_, cells)) = self.cur_gate.as_mut() {
            cells.insert(p);
        }
        Ok(())
    }

    /// Plan one trunk waypoint pair, climbing a retry ladder instead of
    /// giving up on the first blocked template:
    ///
    /// 1. the straight run (axis-aligned pairs) or the two single-corner Ls
    ///    (x-first then z-first) — the deterministic templates, tried first so
    ///    an unobstructed design realizes byte-for-byte as it always did;
    /// 2. a real rectilinear corridor from [`crate::design_corridor`], which
    ///    routes AROUND whatever blocked the templates (a foreign instance's
    ///    body or halo, a loose wall, another bus), at three escalating
    ///    efforts — tidier corridors first, then wider search bounds.
    ///
    /// Every rung's failure is collected, so the reason a pair is finally
    /// unroutable names each shape that was tried and why it lost.
    ///
    /// `since0` is the dust count since the last refresh entering the pair;
    /// the exit count is returned so refresh spacing stays sound across
    /// joints.
    fn plan_pair(
        &mut self,
        a: P3,
        b: P3,
        width: u8,
        keep: &BTreeSet<P3>,
        since0: u32,
    ) -> Result<u32, String> {
        if a == b {
            return Err("zero-length segment".to_string());
        }
        // Anchors on different levels are not a failure: the bus changes
        // level BY CONSTRUCTION with the verified level-shift tile.
        if a.1 != b.1 {
            return self.plan_pair_across_levels(a, b, width, keep, since0);
        }
        let snap = self.snapshot();
        let mut tried: Vec<String> = Vec::new();

        if let Some(run) = axis_run(a, b, width) {
            match self.plan_run(&run, keep, since0, false, false) {
                Ok(out) => return Ok(out),
                Err(e) => {
                    self.restore(snap.clone());
                    tried.push(format!("straight run: {e}"));
                }
            }
        } else {
            for (corner, x_first) in [((b.0, a.1, a.2), true), ((a.0, a.1, b.2), false)] {
                match self.plan_l(a, b, corner, x_first, width, keep, since0) {
                    Ok(out) => return Ok(out),
                    Err(e) => {
                        self.restore(snap.clone());
                        tried.push(format!(
                            "L corner {}: {e}",
                            if x_first { "x-first" } else { "z-first" }
                        ));
                    }
                }
            }
        }

        // The templates are out of ideas; search for an actual corridor.
        //
        // A corridor may loop back near itself. That is electrically harmless
        // here: every cell a bus lays belongs to ONE net and, at a given
        // level, to one BIT, so two nearby legs carry the same signal — a
        // redundant path, not a short. Different bits cannot meet because the
        // 2y pitch keeps their dust two levels apart, and dust never reads the
        // block above it. `Design::check` (DRC + LVS) is the backstop.
        let mut seen: Vec<Vec<P3>> = Vec::new();
        for (rung, effort) in crate::design_corridor::LADDER.iter().enumerate() {
            let Some(chain) = crate::design_corridor::search(self.occ, a, b, width, *effort) else {
                continue;
            };
            // A rung that reproduces a corridor an earlier rung already failed
            // on cannot do better; skip the replan.
            if seen.contains(&chain) {
                continue;
            }
            seen.push(chain.clone());
            match self.plan_chain(&chain, width, keep, since0) {
                Ok(out) => return Ok(out),
                Err(e) => {
                    self.restore(snap.clone());
                    tried.push(format!(
                        "corridor (effort {}, {} legs via {:?}): {e}",
                        rung + 1,
                        chain.len() - 1,
                        &chain[1..chain.len().saturating_sub(1)]
                    ));
                }
            }
        }
        // LAST RUNG: HOP TO A CLEAR LEVEL.
        //
        // The pair's own level is congested, but the diagnostic's cross-level
        // probe often proves a clear corridor exists a few blocks up or down.
        // That used to be the end of the road — the reason literally said "the
        // bus form cannot ramp between levels, so shift an endpoint's instance
        // in y". It CAN ramp now, so use the level shift to go get that lane
        // instead of asking the user to move a component: hop up, cross at the
        // clear level, hop back down onto the sink.
        //
        // Each leg is cross-level, so it recurses through
        // `plan_pair_across_levels`; `hops` bounds that.
        if self.hops == 0 {
            for dy in [2i32, -2, 4, -4, 6, -6, 8, -8] {
                let y2 = a.1 + dy;
                if y2 < 1 {
                    continue; // the stack's supports would go under the world
                }
                let k = dy.unsigned_abs();
                // Each end needs a shift, plus a cell so neither lands on a port.
                let leg = shift_len_max(k, dy < 0) + 1;
                let along_x = (b.0 - a.0).abs() >= (b.2 - a.2).abs();
                let (span, sign) = if along_x {
                    ((b.0 - a.0).abs(), (b.0 - a.0).signum())
                } else {
                    ((b.2 - a.2).abs(), (b.2 - a.2).signum())
                };
                if span < 2 * leg + 1 {
                    continue;
                }
                let (m1, m2) = if along_x {
                    (
                        (a.0 + sign * leg, y2, a.2),
                        (b.0 - sign * leg, y2, b.2),
                    )
                } else {
                    (
                        (a.0, y2, a.2 + sign * leg),
                        (b.0, y2, b.2 - sign * leg),
                    )
                };
                self.hops += 1;
                let hop = (|| -> Result<u32, String> {
                    let s1 = self.plan_pair(a, m1, width, keep, since0)?;
                    let s2 = self.plan_pair(m1, m2, width, keep, s1.saturating_add(1))?;
                    self.plan_pair(m2, b, width, keep, s2.saturating_add(1))
                })();
                self.hops -= 1;
                match hop {
                    Ok(out) => return Ok(out),
                    Err(e) => {
                        self.restore(snap.clone());
                        tried.push(format!("level hop to y={y2}: {e}"));
                    }
                }
            }
        }

        Err(crate::design_corridor::diagnose(self.occ, a, b, width, &tried))
    }

    /// Plan a waypoint pair whose anchors sit on DIFFERENT levels: two FLAT
    /// legs (planned by the ordinary machinery — templates, corners,
    /// corridors, crossings) joined by the verified level-shift tile.
    ///
    /// The tile needs `shift_len` cells of straight run on one horizontal
    /// axis, so every placement is a `(axis, position)` pair; they are tried
    /// cheapest-clearance first (the end with more room), and each failure is
    /// collected so an unroutable pair names every placement it tried.
    fn plan_pair_across_levels(
        &mut self,
        a: P3,
        b: P3,
        width: u8,
        keep: &BTreeSet<P3>,
        since0: u32,
    ) -> Result<u32, String> {
        let k = (b.1 - a.1).unsigned_abs();
        let down = b.1 < a.1;
        let len = shift_len_max(k, down);
        let places = Self::shift_placements(self.occ, a, b, len, width, keep);
        if places.is_empty() {
            return Err(format!(
                "the anchors are {k} level(s) apart, so the bus must change level: the verified \
                 level-shift tile needs {len} cells of straight run (plus 1 to clear the port) on \
                 one horizontal axis, but the pair only spans {} in x and {} in z between {:?} and \
                 {:?}. Lengthen the run, or split it with a gate so one leg has room",
                (b.0 - a.0).abs(),
                (b.2 - a.2).abs(),
                a,
                b
            ));
        }
        let snap = self.snapshot();
        let mut tried: Vec<String> = Vec::new();
        for (entry, exit, along_x, sign, what) in places {
            match self.plan_shift_route(a, b, entry, along_x, sign, k, down, width, keep, since0) {
                Ok(out) => return Ok(out),
                Err(e) => {
                    self.restore(snap.clone());
                    tried.push(format!("level shift {what} ({entry:?} -> {exit:?}): {e}"));
                }
            }
        }
        Err(format!(
            "the anchors are {k} level(s) apart and every placement of the {len}-cell level-shift \
             tile was blocked: {}",
            tried.join("; ")
        ))
    }

    /// One placement attempt: flat leg into the tile, the tile, flat leg out.
    #[allow(clippy::too_many_arguments)]
    fn plan_shift_route(
        &mut self,
        a: P3,
        b: P3,
        entry: P3,
        along_x: bool,
        sign: i32,
        k: u32,
        down: bool,
        width: u8,
        keep: &BTreeSet<P3>,
        since0: u32,
    ) -> Result<u32, String> {
        let mut since = since0;
        if a != entry {
            since = self
                .plan_pair(a, entry, width, keep, since)
                .map_err(|e| format!("leg into the shift: {e}"))?;
        }
        let (out, exit) = self.plan_level_shift(entry, along_x, sign, k, down, width, since)?;
        since = out;
        if exit != b {
            // The tile's exit cell is one more dust cell between refreshes.
            since = self
                .plan_pair(exit, b, width, keep, since)
                .map_err(|e| format!("leg out of the shift: {e}"))?;
        }
        Ok(since)
    }

    /// Candidate placements of a `len`-cell level-shift tile between `a` and
    /// `b`, ordered by clearance (emptiest footprint first). On each axis with
    /// room: hard against the driver end, mid-run, and hard against the sink
    /// end — so a congested endpoint falls through to the open one.
    fn shift_placements(
        occ: &OccupancyIndex,
        a: P3,
        b: P3,
        len: i32,
        width: u8,
        keep: &BTreeSet<P3>,
    ) -> Vec<(P3, P3, bool, i32, &'static str)> {
        let mut out: Vec<(usize, P3, P3, bool, i32, &'static str)> = Vec::new();
        // Longer axis first, so the tie-break after scoring is the roomier one.
        let mut axes = [(true, (b.0 - a.0).abs()), (false, (b.2 - a.2).abs())];
        axes.sort_by_key(|(_, span)| std::cmp::Reverse(*span));
        for (along_x, span) in axes {
            // One cell must stay clear at each end so the tile never lands on
            // a port's own anchor cell.
            if span < len + 1 {
                continue;
            }
            let sign = if along_x {
                (b.0 - a.0).signum()
            } else {
                (b.2 - a.2).signum()
            };
            let a_c = if along_x { a.0 } else { a.2 };
            let slack = span - len - 1;
            for (offset, what) in [
                (1, "at the driver end"),
                (1 + slack / 2, "mid-run"),
                (1 + slack, "at the sink end"),
            ] {
                let start = a_c + sign * offset;
                // Driver-end and mid placements ride the driver's cross-axis
                // row; the sink-end one rides the sink's, so the flat leg out
                // is a straight shot.
                let cross = if offset == 1 + slack {
                    if along_x {
                        b.2
                    } else {
                        b.0
                    }
                } else if along_x {
                    a.2
                } else {
                    a.0
                };
                let (entry, exit) = if along_x {
                    ((start, a.1, cross), (start + sign * (len - 1), b.1, cross))
                } else {
                    ((cross, a.1, start), (cross, b.1, start + sign * (len - 1)))
                };
                let Some(score) = Self::shift_clearance(occ, entry, exit, along_x, width, keep)
                else {
                    continue; // a branch junction falls inside the footprint
                };
                if out.iter().any(|c| c.1 == entry && c.3 == along_x) {
                    continue; // slack 0/1 collapses the three positions
                }
                out.push((score, entry, exit, along_x, sign, what));
            }
        }
        out.sort_by_key(|c| c.0);
        out.into_iter()
            .map(|(_, e, x, ax, s, w)| (e, x, ax, s, w))
            .collect()
    }

    /// How congested a candidate tile footprint is (occupied + halo cells), or
    /// `None` when a branch junction we must keep as plain dust falls inside
    /// it — the tile would eat the junction, so the placement is void.
    fn shift_clearance(
        occ: &OccupancyIndex,
        entry: P3,
        exit: P3,
        along_x: bool,
        width: u8,
        keep: &BTreeSet<P3>,
    ) -> Option<usize> {
        let (lo_a, hi_a) = if along_x {
            (entry.0.min(exit.0), entry.0.max(exit.0))
        } else {
            (entry.2.min(exit.2), entry.2.max(exit.2))
        };
        // The stack sweeps from the lower level's support to the top bit.
        let lo_y = entry.1.min(exit.1) - 1;
        let hi_y = entry.1.max(exit.1) + 2 * (width as i32 - 1);
        let mut score = 0usize;
        for c in lo_a..=hi_a {
            for y in lo_y..=hi_y {
                let p = if along_x {
                    (c, y, entry.2)
                } else {
                    (entry.0, y, c)
                };
                if keep.contains(&p) {
                    return None;
                }
                if occ.cells.contains_key(&p) {
                    score += 4; // a hard cell is worse than a soft halo
                } else if occ.halos.contains_key(&p) {
                    score += 1;
                }
            }
        }
        Some(score)
    }

    /// Stamp the verified BUS LEVEL-SHIFT tile: the whole `width`-bit
    /// 2y-pitch stack changes level by `k`, in form, over
    /// [`shift_len`] cells of straight run from `entry` (bit 0) along the
    /// axis. See [`shift_plan`] for the column plan and its verification.
    ///
    /// Every bit moves in LOCKSTEP, so the 2 y pitch — and with it the
    /// interleave that isolates the bits — is invariant through the shift; an
    /// odd `k` changes the stack's absolute y PARITY, which matters only to
    /// crossings, and those are evaluated per run at the level it actually
    /// runs on.
    #[allow(clippy::too_many_arguments)]
    fn plan_level_shift(
        &mut self,
        entry: P3,
        along_x: bool,
        sign: i32,
        k: u32,
        down: bool,
        width: u8,
        since0: u32,
    ) -> Result<(u32, P3), String> {
        let bus_block = self.style.bus_block.clone();
        let transparent = self.style.transparent_block.clone();
        // Repeater INPUT side faces the driver.
        let toward_driver = if along_x {
            rblocks::facing_name(-sign, 0)
        } else {
            rblocks::facing_name(0, -sign)
        }
        .expect("axis-aligned unit step");
        let (cols, since_out) = shift_plan(k, down, since0);
        let pos_at = |o: i32, y: i32| -> P3 {
            if along_x {
                (entry.0 + o * sign, y, entry.2)
            } else {
                (entry.0, y, entry.2 + o * sign)
            }
        };
        for bit in 0..width {
            let base = entry.1 + 2 * bit as i32;
            for &(o, dy, kind) in &cols {
                let y = base + dy;
                match kind {
                    // The station blocks float, exactly as in the dip tile.
                    ShiftCell::Entry | ShiftCell::Exit => self.put(pos_at(o, y), &bus_block)?,
                    ShiftCell::Rep => {
                        self.put(pos_at(o, y - 1), &bus_block)?;
                        self.put(pos_at(o, y), &rblocks::repeater(toward_driver, 1))?;
                    }
                    ShiftCell::Flat => {
                        self.put(pos_at(o, y - 1), &bus_block)?;
                        self.put(pos_at(o, y), rblocks::DUST)?;
                    }
                    ShiftCell::Step | ShiftCell::Land => {
                        // THE ALTERNATION, and it FLIPS WITH DIRECTION.
                        //
                        // Descending, the step-off dust is the diagonal's
                        // UPPER end, so its support must CONDUCT (a 1y step
                        // passes down only over a conducting upper support —
                        // the diode law), while the landed dust's support is
                        // the cap sitting directly above bit n-1's in-use
                        // diagonal and must NOT cut it.
                        //
                        // Ascending, up-steps conduct over anything, so the
                        // two roles SWAP: the step-off support becomes the
                        // cap that must insulate, and the landed support is
                        // the conductor that severs the cross-bit diagonal
                        // (bit n's step-off dust and bit n-1's landed dust
                        // are 1 y apart in adjacent columns — an unintended
                        // connection, cut by the conductor above the lower).
                        //
                        // Bit 0 has nothing below to protect, so it is
                        // unconstrained: solid, per "transparency only where
                        // a diagonal must survive".
                        let insulate = bit > 0 && (kind == ShiftCell::Land) == down;
                        let support = if insulate { &transparent } else { &bus_block };
                        self.put(pos_at(o, y - 1), support)?;
                        self.put(pos_at(o, y), rblocks::DUST)?;
                    }
                }
            }
        }
        // The EXIT is wherever the plan actually ended, not where a
        // pre-computed length said it would: the caller continues from here.
        let last = cols.last().expect("a shift plan is never empty");
        Ok((since_out, pos_at(last.0, entry.1 + last.1)))
    }

    /// Plan a multi-leg corridor: a straight run per leg with a joint column
    /// at every interior corner — the same vocabulary [`Planner::plan_l`] uses
    /// for its single corner, generalized to N.
    fn plan_chain(
        &mut self,
        chain: &[P3],
        width: u8,
        keep: &BTreeSet<P3>,
        since0: u32,
    ) -> Result<u32, String> {
        let mut since = since0;
        for (i, leg) in chain.windows(2).enumerate() {
            let (a, b) = (leg[0], leg[1]);
            let run = axis_run(a, b, width)
                .ok_or_else(|| format!("corridor leg {:?} -> {:?} is not axis-aligned", a, b))?;
            since = self.plan_run(&run, keep, since, false, false)?;
            if i + 2 < chain.len() {
                self.column(b, width)?;
                // The corner joint is one more dust cell between refreshes.
                since = since.saturating_add(1);
            }
        }
        Ok(since)
    }

    /// One L-shaped pair through `corner`.
    #[allow(clippy::too_many_arguments)]
    fn plan_l(
        &mut self,
        a: P3,
        b: P3,
        corner: P3,
        x_first: bool,
        width: u8,
        keep: &BTreeSet<P3>,
        since0: u32,
    ) -> Result<u32, String> {
        let (r1, r2) = if x_first {
            (
                RunInfo {
                    along_x: true,
                    fixed: a.2,
                    y0: a.1,
                    from: a.0,
                    to: corner.0,
                    width,
                },
                RunInfo {
                    along_x: false,
                    fixed: corner.0,
                    y0: a.1,
                    from: a.2,
                    to: b.2,
                    width,
                },
            )
        } else {
            (
                RunInfo {
                    along_x: false,
                    fixed: a.0,
                    y0: a.1,
                    from: a.2,
                    to: corner.2,
                    width,
                },
                RunInfo {
                    along_x: true,
                    fixed: corner.2,
                    y0: a.1,
                    from: a.0,
                    to: b.0,
                    width,
                },
            )
        };
        let mid = self.plan_run(&r1, keep, since0, false, false)?;
        self.column(corner, width)?;
        // The corner joint is one more dust cell between refreshes.
        self.plan_run(&r2, keep, mid.saturating_add(1), false, false)
    }

    /// A gate's electrical joint column, recorded under the gate's name.
    fn plan_column(&mut self, gate: &str, anchor: P3, width: u8) -> Result<(), String> {
        self.cur_gate = Some((gate.to_string(), BTreeSet::new()));
        let r = self.column(anchor, width);
        if let Some((name, cells)) = self.cur_gate.take() {
            self.real.gate_cells.entry(name).or_default().extend(cells);
        }
        r
    }

    /// A dust-on-support joint column at every bit (gates and L corners).
    fn column(&mut self, anchor: P3, width: u8) -> Result<(), String> {
        let bus_block = self.style.bus_block.clone();
        for k in 0..width {
            let p = add(anchor, (0, 2 * k as i32, 0));
            self.put(p, rblocks::DUST)?;
            self.put((p.0, p.1 - 1, p.2), &bus_block)?;
        }
        Ok(())
    }

    /// Does `other` already dip under our line at the shared column?
    /// (Its dip dust sits one level below its canonical dust level, one
    /// cell before / three after the crossing center along its own axis.)
    fn dips_at(other: &BusLayer, orun: &RunInfo, center_on_their_axis: i32) -> bool {
        let sa = orun.sign();
        let poso = |c: i32, y: i32| -> P3 {
            if orun.along_x {
                (c, y, orun.fixed)
            } else {
                (orun.fixed, y, c)
            }
        };
        let d = orun.y0 - 1;
        // The dip cells hold DUST one level down; a straight run's support
        // blocks at the same level must not count.
        other
            .fragment
            .get(&poso(center_on_their_axis - sa, d))
            .is_some_and(|b| rblocks::is_dust(b))
            || other
                .fragment
                .get(&poso(center_on_their_axis + 3 * sa, d))
                .is_some_and(|b| rblocks::is_dust(b))
    }

    /// Plan one straight run: supports + dust, refresh repeaters, forced
    /// diode repeaters for branch joins (`first_rep` right after the
    /// junction on sink branches, `last_rep` right before it on wired-OR
    /// driver branches), implicit dip-under crossings against
    /// already-routed buses, and through-bus stations where a routed bus
    /// already dips under this line.
    fn plan_run(
        &mut self,
        run: &RunInfo,
        keep: &BTreeSet<P3>,
        since0: u32,
        first_rep: bool,
        last_rep: bool,
    ) -> Result<u32, String> {
        let sign = run.sign();
        let bus_block = self.style.bus_block.clone();
        let transparent = self.style.transparent_block.clone();
        let pos_at = |c: i32, y: i32| -> P3 {
            if run.along_x {
                (c, y, run.fixed)
            } else {
                (run.fixed, y, c)
            }
        };
        // Repeater INPUT side faces the driver.
        let toward_driver = if run.along_x {
            rblocks::facing_name(-sign, 0)
        } else {
            rblocks::facing_name(0, -sign)
        }
        .expect("axis-aligned unit step");

        // Crossings against every OTHER routed bus's perpendicular runs.
        let mut dips: Vec<i32> = Vec::new(); // centers along OUR axis
        let mut stations: Vec<i32> = Vec::new();
        for other in self.design.buses.values() {
            if Some(other.name.as_str()) == self.exclude || other.state != BusState::Routed {
                continue;
            }
            for orun in &other.runs {
                if orun.along_x == run.along_x {
                    continue; // parallel: overlap shows up as a collision
                }
                let center = orun.fixed;
                if !run.strictly_inside(center, 3) || !orun.strictly_inside(run.fixed, 3) {
                    continue;
                }
                // Two lines that cross in plan view but occupy DISJOINT
                // vertical bands never touch: no dip, no station, nothing to
                // adapt. Only an actual overlap needs the crossing tile, and
                // only then does a level/width mismatch matter.
                let ours = (run.y0 - 1, run.y0 + 2 * (run.width as i32 - 1));
                let theirs = (orun.y0 - 1, orun.y0 + 2 * (orun.width as i32 - 1));
                if theirs.1 < ours.0 || ours.1 < theirs.0 {
                    continue;
                }
                if orun.y0 != run.y0 || orun.width != run.width {
                    return Err(format!(
                        "crossing with bus `{}` at {:?}: the two stacks overlap vertically (ours \
                         y {}..={} at {} bits, theirs y {}..={} at {} bits) but do not share a \
                         level, so the dip-under tile does not apply. Give the two buses \
                         non-overlapping y bands (a `y_band` net-class rule), or make their \
                         widths and bit-0 levels match — vertical level adapters are not \
                         implemented yet",
                        other.name,
                        (run.fixed, orun.fixed),
                        ours.0,
                        ours.1,
                        run.width,
                        theirs.0,
                        theirs.1,
                        orun.width
                    ));
                }
                if Self::dips_at(other, orun, run.fixed) {
                    // They already dip under this line: WE are the through
                    // bus and pay the station (one repeater per bit).
                    stations.push(center);
                } else {
                    dips.push(center);
                    self.plan_station_amendment(other, orun, run.fixed)?;
                }
            }
        }
        dips.sort();
        dips.dedup();
        stations.sort();
        stations.dedup();
        let mut centers: Vec<i32> = dips.iter().chain(stations.iter()).copied().collect();
        centers.sort();
        for w in centers.windows(2) {
            if (w[1] - w[0]).abs() < 8 {
                return Err(format!(
                    "crossing windows at {} and {} overlap (need 8 cells of spacing)",
                    w[0], w[1]
                ));
            }
        }

        // Cell coordinates strictly between the anchors.
        let in_dip =
            |c: i32| dips.iter().any(|&cc| ((c - cc) * sign) >= -2 && ((c - cc) * sign) <= 4);
        let near_dip =
            |c: i32| dips.iter().any(|&cc| ((c - cc) * sign) >= -4 && ((c - cc) * sign) <= 6);
        let in_station = |c: i32| {
            stations
                .iter()
                .any(|&cc| ((c - cc) * sign) >= 0 && ((c - cc) * sign) <= 2)
        };
        let near_station = |c: i32| {
            stations
                .iter()
                .any(|&cc| ((c - cc) * sign) >= -2 && ((c - cc) * sign) <= 4)
        };

        // A kept-dust junction inside a crossing window cannot merge.
        for k in keep {
            let on_this_run = k.1 == run.y0
                && if run.along_x {
                    k.2 == run.fixed
                } else {
                    k.0 == run.fixed
                };
            if !on_this_run {
                continue;
            }
            let c = if run.along_x { k.0 } else { k.2 };
            if in_dip(c) || in_station(c) || near_dip(c) {
                return Err(format!("junction {:?} falls inside a crossing window", k));
            }
        }

        let mut since_out = since0;
        for bit in 0..run.width {
            let y = run.y0 + 2 * bit as i32;
            let d = y - 1;
            let mut since_refresh = since0;
            let mut c = run.from + sign;
            while c != run.to {
                if in_dip(c) || in_station(c) {
                    since_refresh = 0; // the window's own repeater refreshes
                    c += sign;
                    continue;
                }
                let is_first = c == run.from + sign;
                let is_last = c == run.to - sign;
                let keep_here = keep.contains(&pos_at(c, run.y0));
                let steps_from_ends = (c - run.from).abs().min((run.to - c).abs());
                let force_rep = (first_rep && is_first) || (last_rep && is_last);
                if force_rep
                    || (since_refresh >= REFRESH_AT
                        && steps_from_ends >= 2
                        && !keep_here
                        && !near_dip(c)
                        && !near_station(c))
                {
                    self.put(pos_at(c, y - 1), &bus_block)?;
                    self.put(pos_at(c, y), &rblocks::repeater(toward_driver, 1))?;
                    since_refresh = 0;
                } else {
                    self.put(pos_at(c, y - 1), &bus_block)?;
                    self.put(pos_at(c, y), rblocks::DUST)?;
                    since_refresh += 1;
                }
                c += sign;
            }

            // Dip-under windows (bus8_cross.py v2 canonical tile): step
            // down, dip dust, entry block / repeater / exit block on the
            // shared column's interleaved levels, dip dust, step up.
            // Transparent supports appear ONLY at the dip's slope
            // transitions for bits > 0, where bit n-1's diagonal below
            // must survive.
            for &cc in &dips {
                let at = |o: i32, y: i32| pos_at(cc + o * sign, y);
                // o = -2 and 4: the step upper cells (conductor support —
                // the diode law — which also severs cross-bit diagonals).
                self.put(at(-2, y - 1), &bus_block)?;
                self.put(at(-2, y), rblocks::DUST)?;
                self.put(at(4, y - 1), &bus_block)?;
                self.put(at(4, y), rblocks::DUST)?;
                // o = -1 and 3: the dip dust cells; their supports sit
                // directly above bit n-1's dip dust diagonal.
                let dip_support = if bit > 0 {
                    transparent.as_str()
                } else {
                    bus_block.as_str()
                };
                self.put(at(-1, d - 1), dip_support)?;
                self.put(at(-1, d), rblocks::DUST)?;
                self.put(at(3, d - 1), dip_support)?;
                self.put(at(3, d), rblocks::DUST)?;
                // o = 0..2: the block-sandwich station at the dip level.
                self.put(at(0, d), &bus_block)?;
                self.put(at(1, d - 1), &bus_block)?;
                self.put(at(1, d), &rblocks::repeater(toward_driver, 1))?;
                self.put(at(2, d), &bus_block)?;
            }

            // Through-bus stations (the other bus dips): entry block,
            // repeater on a conductor floor, exit block — the same cells
            // the amendment would have stamped, planned first-hand.
            for &cc in &stations {
                let at = |o: i32, y: i32| pos_at(cc + o * sign, y);
                self.put(at(0, y), &bus_block)?;
                self.put(at(1, y - 1), &bus_block)?;
                self.put(at(1, y), &rblocks::repeater(toward_driver, 1))?;
                self.put(at(2, y), &bus_block)?;
            }
            if bit == 0 {
                since_out = since_refresh;
            }
        }
        if let Some(seg) = self.cur_seg.as_mut() {
            seg.runs.push(run.clone());
        }
        Ok(since_out)
    }

    /// The station upgrade a crossing stamps into the THROUGH bus at the
    /// crossing column: its plain dust run over o = 0..2 becomes entry
    /// block / repeater / exit block (the crossed bus pays one repeater
    /// per bit, doubling as its refresh).
    fn plan_station_amendment(
        &mut self,
        other: &BusLayer,
        orun: &RunInfo,
        center: i32,
    ) -> Result<(), String> {
        let sa = orun.sign();
        let toward_driver = if orun.along_x {
            rblocks::facing_name(-sa, 0)
        } else {
            rblocks::facing_name(0, -sa)
        }
        .expect("axis-aligned unit step");
        let pos_at = |c: i32, y: i32| -> P3 {
            if orun.along_x {
                (c, y, orun.fixed)
            } else {
                (orun.fixed, y, c)
            }
        };
        let mut removals = Vec::new();
        let mut additions = BTreeMap::new();
        for bit in 0..orun.width {
            let y = orun.y0 + 2 * bit as i32;
            for o in 0..=2 {
                let c = center + o * sa;
                // Vacate the through bus's dust + support at the station.
                for p in [pos_at(c, y), pos_at(c, y - 1)] {
                    if other.fragment.contains_key(&p) {
                        removals.push(p);
                    }
                }
            }
            let bus_block = other.style.bus_block.clone();
            // Entry block (floats: it is also the cap severing every read
            // to/from the dipping bus one level down), repeater on a
            // conductor floor, exit block (fresh 15).
            additions.insert(pos_at(center, y), bus_block.clone());
            additions.insert(pos_at(center + sa, y - 1), bus_block.clone());
            additions.insert(pos_at(center + sa, y), rblocks::repeater(toward_driver, 1));
            additions.insert(pos_at(center + 2 * sa, y), bus_block);
        }
        self.vacated.extend(removals.iter().copied());
        self.real
            .amendments
            .push((other.name.clone(), removals, additions));
        Ok(())
    }
}

/// Map a position through an instance transform: Y-rotation in quarter
/// turns about the cell bounding box's min corner (the same convention as
/// `Region::rotate_y`), then translation to `at`.
fn transform_pos(p: P3, min: P3, max: P3, rot_y: i32, at: P3) -> P3 {
    let mut rel = (p.0 - min.0, p.1 - min.1, p.2 - min.2);
    let mut size = (max.0 - min.0 + 1, max.2 - min.2 + 1); // (sx, sz)
    for _ in 0..(rot_y.rem_euclid(360) / 90) {
        // (rx, rz) -> (sz - 1 - rz, rx); sizes swap.
        rel = (size.1 - 1 - rel.2, rel.1, rel.0);
        size = (size.1, size.0);
    }
    (at.0 + rel.0, at.1 + rel.1, at.2 + rel.2)
}

/// Rotate a block state's orientation properties to match the instance
/// transform.
fn transform_state(bs: &crate::BlockState, rot_y: i32) -> crate::BlockState {
    let mut out = bs.clone();
    for _ in 0..(rot_y.rem_euclid(360) / 90) {
        out = crate::transforms::transform_block_state_rotate(&out, crate::transforms::Axis::Y, 90);
    }
    out
}

/// The block states a design's endpoints and routes can visit at runtime —
/// interned up front so late-placed cells never sit inert in the tick
/// engine. Delegates to the executor backend's standard list.
#[cfg(feature = "simulation")]
pub fn executor_extra_states() -> Vec<String> {
    crate::simulation::typed_executor::standard_io_extra_states()
}

#[cfg(all(feature = "simulation", feature = "bridge", feature = "mc-tick"))]
fn value_to_u64(v: &crate::io_contract::Value) -> u64 {
    use crate::io_contract::Value;
    match v {
        Value::Bool(b) => *b as u64,
        Value::U32(x) => *x as u64,
        Value::U64(x) => *x,
        Value::I32(x) => *x as u64,
        Value::I64(x) => *x as u64,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_contract::IoType;

    const STONE: &str = "minecraft:stone";
    const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
    const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";

    /// The design_step1.py endpoint hardware: 8 levers stacked at 2y pitch,
    /// each with its connection dust one step toward the field.
    fn lever_bank(s: &mut UniversalSchematic, x: i32, z: i32, dx: i32, dz: i32) -> P3 {
        for i in 0..8 {
            let y = 2 + 2 * i;
            s.set_block_from_string(x, y - 1, z, STONE).unwrap();
            s.set_block_from_string(x, y, z, LEVER).unwrap();
            s.set_block_from_string(x + dx, y - 1, z + dz, STONE).unwrap();
            s.set_block_from_string(x + dx, y, z + dz, rblocks::DUST).unwrap();
        }
        (x + dx, 2, z + dz)
    }

    /// 8 lamps stacked at 2y pitch, each lamp supporting its own dust.
    fn lamp_bank(s: &mut UniversalSchematic, x: i32, z: i32) -> P3 {
        for i in 0..8 {
            let y = 2 + 2 * i;
            s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
            s.set_block_from_string(x, y, z, rblocks::DUST).unwrap();
        }
        (x, 2, z)
    }

    fn crossing_design() -> Design {
        let mut s = UniversalSchematic::new("crossing".to_string());
        let a_in = lever_bank(&mut s, 0, 8, 1, 0);
        let a_out = lamp_bank(&mut s, 16, 8);
        let b_in = lever_bank(&mut s, 8, 0, 0, 1);
        let b_out = lamp_bank(&mut s, 8, 16);
        let mut d = Design::for_schematic("crossing", s);
        let step = (0, 2, 0);
        let ty = IoType::UnsignedInt { bits: 8 };
        d.declare_input("a_in", a_in, step, 8, ty.clone()).unwrap();
        d.declare_output("a_out", a_out, step, 8, ty.clone()).unwrap();
        d.declare_input("b_in", b_in, step, 8, ty.clone()).unwrap();
        d.declare_output("b_out", b_out, step, 8, ty).unwrap();
        d
    }

    #[test]
    fn capability_scan_validates_loudly() {
        let mut s = UniversalSchematic::new("caps".to_string());
        lever_bank(&mut s, 0, 8, 1, 0);
        let mut d = Design::for_schematic("caps", s);
        // Anchor on the lever's support (not dust): refused, names the block.
        let err = d
            .declare_input("bad", (0, 1, 8), (0, 2, 0), 8, IoType::UnsignedInt { bits: 8 })
            .unwrap_err();
        assert!(err.contains("not dust"), "{err}");
        // Output on lever hardware: not readable.
        let err = d
            .declare_output("bad", (1, 2, 8), (0, 2, 0), 8, IoType::UnsignedInt { bits: 8 })
            .unwrap_err();
        assert!(err.contains("no adjacent lamp"), "{err}");
        // Input over the same cells: drivable, fine.
        d.declare_input("good", (1, 2, 8), (0, 2, 0), 8, IoType::UnsignedInt { bits: 8 })
            .unwrap();
    }

    /// A library cell shaped like the community ones: its contract names
    /// EXECUTOR hardware (levers in, lamps out), not dust. Bit 0 of `dead`
    /// is a bare lamp with no dust anywhere near it.
    fn buffer_cell() -> UniversalSchematic {
        let mut s = UniversalSchematic::new("buf".to_string());
        let d_hw: Vec<P3> = (0..8).map(|i| (0, 2 + 2 * i, 0)).collect();
        let q_hw: Vec<P3> = (0..8).map(|i| (4, 1 + 2 * i, 0)).collect();
        for i in 0..8 {
            let y = 2 + 2 * i;
            // input lever + its connection dust one step in
            s.set_block_from_string(0, y - 1, 0, STONE).unwrap();
            s.set_block_from_string(0, y, 0, LEVER).unwrap();
            s.set_block_from_string(1, y - 1, 0, STONE).unwrap();
            s.set_block_from_string(1, y, 0, rblocks::DUST).unwrap();
            // output lamp supporting its connection dust
            s.set_block_from_string(4, y - 1, 0, LAMP).unwrap();
            s.set_block_from_string(4, y, 0, rblocks::DUST).unwrap();
        }
        // A lamp with no dust neighbour at all: executor-readable, never
        // bus-routable.
        s.set_block_from_string(4, 40, 0, LAMP).unwrap();
        let layout = crate::io_contract::IoLayoutBuilder::new()
            .add_input(
                "d".to_string(),
                IoType::UnsignedInt { bits: 8 },
                LayoutFunction::OneToOne,
                d_hw,
            )
            .unwrap()
            .add_output(
                "q".to_string(),
                IoType::UnsignedInt { bits: 8 },
                LayoutFunction::OneToOne,
                q_hw,
            )
            .unwrap()
            .add_output(
                "dead".to_string(),
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(4, 40, 0)],
            )
            .unwrap()
            .build();
        s.set_cell_contract(&CellContract::new("buf".to_string(), layout))
            .unwrap();
        s
    }

    /// The blocker this module exists to remove: a PLACED INSTANCE exposes
    /// its contract ports as routable endpoints under `{inst}.{port}`,
    /// transformed, with the dust connection cell derived by hardware scan —
    /// so `route_bus` takes `u0.q` directly.
    #[test]
    fn instance_ports_are_first_class_routing_endpoints() {
        let mut s = UniversalSchematic::new("host".to_string());
        let out = lamp_bank(&mut s, 24, 8);
        let mut d = Design::for_schematic("host", s);
        d.add_cell("buf", buffer_cell()).unwrap();
        // The cell's own bbox starts at y=1, so `at` y=1 lands bit 0's
        // connection dust on the design's canonical y=2 bus level.
        d.place("u0", "buf", (0, 1, 8), 0).unwrap();
        d.declare_output("q_out", out, (0, 2, 0), 8, IoType::UnsignedInt { bits: 8 })
            .unwrap();

        let ports = d.instance_ports().unwrap();
        let names: Vec<&str> = ports.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["u0.d", "u0.dead", "u0.q"], "{names:?}");

        // The contract names the lamp column; the routable endpoint is the
        // dust ON those lamps, one cell up, transformed by the placement.
        let q = ports.iter().find(|p| p.name == "u0.q").unwrap();
        assert!(q.routable(), "{:?}", q.blocked);
        assert_eq!(q.hardware[0], (4, 1, 8));
        assert_eq!(q.wires.as_ref().unwrap()[0], (4, 2, 8));
        assert_eq!(q.step, Some((0, 2, 0)));

        // A lamp with no dust anywhere near it is reported, not mis-routed.
        let dead = ports.iter().find(|p| p.name == "u0.dead").unwrap();
        assert!(!dead.routable());
        assert!(
            dead.blocked.as_ref().unwrap().contains("no dust connection cell"),
            "{:?}",
            dead.blocked
        );

        // resolve_port flips to the DESIGN-facing direction: the cell's
        // output drives the fabric.
        let rq = d.resolve_port("u0.q").unwrap();
        assert_eq!(rq.direction, PortDirection::Input);
        assert_eq!(d.resolve_port("u0.d").unwrap().direction, PortDirection::Output);
        let err = d.resolve_port("u0.nope").unwrap_err();
        assert!(err.contains("no contract port `nope`"), "{err}");

        // ...and a bus takes the instance port by name. The endpoint sits on
        // the instance's own body, so pin access must beat its halo.
        let state = d
            .route_bus("chain", "u0.q", &["q_out"], vec![], BusStyle::default())
            .unwrap();
        assert_eq!(state, BusState::Routed, "{state:?}");
        assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);

        // Driving into a cell input is legal; driving OUT of one is not.
        let err = d
            .route_bus("bad", "u0.d", &["q_out"], vec![], BusStyle::default())
            .unwrap_err();
        assert!(err.contains("cannot drive"), "{err}");

        // Removing the instance takes its bus with it, loudly.
        let report = d.remove_instance("u0").unwrap();
        assert_eq!(report.removed_buses, vec!["chain".to_string()]);
        assert!(d.instance_ports().unwrap().is_empty());
    }

    #[test]
    fn two_crossing_buses_route_with_an_implicit_dip_under() {
        let mut d = crossing_design();
        let a = d
            .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
            .unwrap();
        assert_eq!(a, BusState::Routed);
        let mut style_b = BusStyle::default();
        style_b.bus_block = "minecraft:cyan_concrete".to_string();
        let b = d
            .route_bus("bus_b", "b_in", &["b_out"], vec![], style_b)
            .unwrap();
        assert_eq!(b, BusState::Routed, "{:?}", d.bus_state("bus_b"));

        // The crossing is implicit: bus_b dips (dust at the odd level) and
        // bus_a got its station upgrade (a repeater in its fragment).
        let bus_b = d.bus("bus_b").unwrap();
        assert!(bus_b.fragment.contains_key(&(8, 1, 7)), "dip dust");
        let bus_a = d.bus("bus_a").unwrap();
        assert!(
            bus_a
                .fragment
                .values()
                .any(|b| rblocks::is_repeater(b)),
            "station repeater in the through bus"
        );

        // Flatten: one region per layer + embedded merged contract.
        let flat = d.flatten().unwrap();
        let regions = flat.get_region_names();
        assert!(regions.iter().any(|r| r == "bus:bus_a"), "{regions:?}");
        assert!(regions.iter().any(|r| r == "bus:bus_b"), "{regions:?}");
        let contract = flat.embedded_cell_contract().unwrap().unwrap();
        assert!(contract.io.get_input("a_in").is_some());
        assert!(contract.io.get_output("b_out").is_some());

        // check(): DRC + LVS come back clean.
        let check = d.check().unwrap();
        assert!(check.clean, "{}", check.json);
    }

    #[test]
    fn unroutable_is_a_state_not_an_exception() {
        let mut d = crossing_design();
        // A SEALED stone wall across the a-line. A single blocking column is
        // no longer enough — the corridor search routes around one of those —
        // so unroutability now needs a wall the bus genuinely cannot get past.
        // The reason must name the blocker and what to do about it.
        for z in -260..=260 {
            for y in 0..=20 {
                d.set_block((12, y, z), STONE).unwrap();
            }
        }
        let state = d
            .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
            .unwrap();
        match state {
            BusState::Failed(reason) => {
                assert!(reason.contains("no corridor"), "{reason}");
                assert!(reason.contains("(12,"), "names the blocker location: {reason}");
                assert!(reason.contains("loose block"), "names the owner: {reason}");
                // And it must say whether the LEVEL is the problem or the
                // workspace is genuinely full — the two need opposite fixes, and
                // "no path" told the user neither. Here the wall spans y 0..=20,
                // so no level is clear.
                assert!(
                    reason.contains("No level within 8 blocks"),
                    "must report the cross-level verdict: {reason}"
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
        assert!(d.bus("bus_a").unwrap().fragment.is_empty());
        // The failed layer does not poison later routing.
        let b = d
            .route_bus("bus_b", "b_in", &["b_out"], vec![], BusStyle::default())
            .unwrap();
        assert_eq!(b, BusState::Routed, "{:?}", d.bus_state("bus_b"));
    }

    #[test]
    fn a_dogleg_routes_through_an_implicit_corner() {
        let mut d = crossing_design();
        // a_in (1,2,8) -> b_out (8,2,16): not axis-aligned; the planner
        // bends through one implicit corner joint instead of failing.
        let state = d
            .route_bus("diag", "a_in", &["b_out"], vec![], BusStyle::default())
            .unwrap();
        assert_eq!(state, BusState::Routed, "{:?}", d.bus_state("diag"));
        let bus = d.bus("diag").unwrap();
        // One trunk segment realized as two perpendicular runs + the
        // corner joint dust at (8, 2, 8).
        assert_eq!(bus.segments.len(), 1);
        assert_eq!(bus.segments[0].runs.len(), 2);
        assert!(bus.fragment.contains_key(&(8, 2, 8)), "corner joint dust");
        let check = d.check().unwrap();
        assert!(check.clean, "{}", check.json);
    }

    #[test]
    fn embedded_contract_round_trips_through_schem_bytes() {
        let mut d = crossing_design();
        d.route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
            .unwrap();
        let bytes = d.to_schem_bytes().unwrap();
        let back = crate::formats::schematic::from_schematic(&bytes).unwrap();
        let (contract, warnings) = back.resolve_cell_contract().unwrap().unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(contract.name, "crossing");
        // Executor-facing positions: inputs are the LEVER cells.
        let a_in = contract.io.get_input("a_in").unwrap();
        assert_eq!(a_in.positions[0], (0, 2, 8));
        let a_out = contract.io.get_output("a_out").unwrap();
        assert_eq!(a_out.positions[0], (16, 2, 8));
    }
}
