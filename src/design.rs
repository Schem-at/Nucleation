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

/// Dust cells between refresh repeaters. 7 keeps the worst joint-spanning
/// gap (tail + joint column + head = 15) inside dust's 15-cell reach.
const REFRESH_AT: u32 = 7;

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

/// Outcome of a drag ([`Design::move_instance`]): the move itself always
/// succeeds (the document's truth); buses fail VISIBLY, never half-routed.
#[derive(Clone, Debug, Default)]
pub struct MoveReport {
    /// Buses ripped and successfully co-rerouted, in name order.
    pub rerouted: Vec<String>,
    /// Buses left in `FAILED(reason)` after the bounded negotiation.
    pub failed: Vec<(String, String)>,
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
        format!(
            "{{\"rerouted\":[{}],\"failed\":{{{}}}}}",
            r.join(","),
            f.join(",")
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
        }
    }

    /// A design whose loose block layer is `base` (endpoint hardware placed
    /// with raw `set_block`, the `design_step1.py` workflow).
    pub fn for_schematic(name: impl Into<String>, base: UniversalSchematic) -> Self {
        let mut d = Design::new(name);
        d.base = base;
        d
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
        });
        Ok(())
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
        let bbox = cell.schematic.get_bounding_box();
        let mut contract = cell.contract.clone();
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
        let mut bits = Vec::with_capacity(width as usize);
        for k in 0..width {
            let cell = add(anchor, scale(step, k as i32));
            let hw = self.scan_bit(cell);
            if !hw.connectable {
                return Err(format!(
                    "port `{name}` bit {k}: connection cell {:?} holds `{}`, not dust",
                    cell,
                    self.base_block_string(cell).unwrap_or_default()
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

    /// Hardware scan of one connection cell.
    fn scan_bit(&self, cell: P3) -> BitHardware {
        let mut hw = BitHardware::default();
        hw.connectable = self
            .base_block_string(cell)
            .is_some_and(|b| rblocks::is_dust(&b));
        // Levers power adjacent dust: the 4 horizontal neighbours, the cell
        // above and the support below.
        let around = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 0, 1),
            (0, 0, -1),
            (0, 1, 0),
            (0, -1, 0),
        ];
        for d in around {
            let q = add(cell, d);
            if let Some(b) = self.base_block_string(q) {
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
        self.route_bus_inner(name.into(), &[driver], sinks, gates, style, false)
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
        self.route_bus_inner(name.into(), drivers, sinks, gates, style, true)
    }

    fn route_bus_inner(
        &mut self,
        name: String,
        drivers: &[&str],
        sinks: &[&str],
        gates: Vec<Gate>,
        style: BusStyle,
        merge_or: bool,
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
        let mut driver_ports = Vec::new();
        for dn in drivers {
            let drv = self
                .ports
                .get(*dn)
                .ok_or_else(|| format!("unknown driver port `{dn}`"))?;
            if drv.direction == PortDirection::Output {
                return Err(format!(
                    "bus `{name}`: driver `{dn}` is a declared output; bidirectional buses are \
                     modeled but reserved (Phase 2), and outputs cannot drive"
                ));
            }
            if drv.width != self.ports[drivers[0]].width {
                return Err(format!(
                    "bus `{name}`: driver `{dn}` width {} != driver `{}` width {}",
                    drv.width, drivers[0], self.ports[drivers[0]].width
                ));
            }
            driver_ports.push(drv.clone());
        }
        if sinks.is_empty() {
            return Err(format!("bus `{name}` needs at least one sink"));
        }
        let mut sink_ports = Vec::new();
        for s in sinks {
            let sp = self
                .ports
                .get(*s)
                .ok_or_else(|| format!("unknown sink port `{s}`"))?;
            if sp.width != driver_ports[0].width {
                return Err(format!(
                    "bus `{name}`: sink `{s}` width {} != driver width {}",
                    sp.width, driver_ports[0].width
                ));
            }
            sink_ports.push(sp.clone());
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
        };

        match self.realize(Some(&name), &driver_ports, &sink_ports, &layer.gates, &layer.style) {
            Ok(real) => {
                Self::fill_layer(&mut layer, real.fragment, real.segments, real.gate_cells);
                self.apply_amendments(real.amendments);
            }
            Err(reason) => {
                layer.state = BusState::Failed(reason);
            }
        }
        let state = layer.state.clone();
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
    fn apply_amendments(&mut self, amendments: Vec<(String, Vec<P3>, BTreeMap<P3, String>)>) {
        for (bus, removals, additions) in amendments {
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
        let old_region = self.instance_region(idx);
        // The move itself always succeeds.
        self.instances[idx].at = at;
        self.instances[idx].rot_y = rot_y.rem_euclid(360);
        let new_region = self.instance_region(idx);

        let mut affected: BTreeSet<String> = BTreeSet::new();
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
        Ok(self.co_reroute(affected))
    }

    /// Footprint + influence halo of an instance, as a cell set.
    fn instance_region(&self, idx: usize) -> BTreeSet<P3> {
        let inst = &self.instances[idx];
        let cell = &self.cells[&inst.cell];
        let bbox = cell.schematic.get_bounding_box();
        let map = |p: P3| transform_pos(p, bbox.min, bbox.max, inst.rot_y, inst.at);
        let mut region = BTreeSet::new();
        for (bp, bs) in cell.schematic.iter_blocks() {
            if bs.to_string().contains("minecraft:air") {
                continue;
            }
            region.insert(map((bp.x, bp.y, bp.z)));
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
    fn co_reroute(&mut self, affected: BTreeSet<String>) -> MoveReport {
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
        let mut driver_ports = Vec::new();
        for dn in &driver_names {
            match self.ports.get(dn) {
                Some(p) => driver_ports.push(p.clone()),
                None => {
                    let state = BusState::Failed(format!("driver port `{dn}` no longer exists"));
                    self.buses.get_mut(name).unwrap().state = state.clone();
                    return state;
                }
            }
        }
        let mut sink_ports = Vec::new();
        for sn in &sink_names {
            match self.ports.get(sn) {
                Some(p) => sink_ports.push(p.clone()),
                None => {
                    let state = BusState::Failed(format!("sink port `{sn}` no longer exists"));
                    self.buses.get_mut(name).unwrap().state = state.clone();
                    return state;
                }
            }
        }
        match self.realize(Some(name), &driver_ports, &sink_ports, &gates, &style) {
            Ok(real) => {
                let layer = self.buses.get_mut(name).unwrap();
                Self::fill_layer(layer, real.fragment, real.segments, real.gate_cells);
                self.apply_amendments(real.amendments);
                BusState::Routed
            }
            Err(reason) => {
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
        let driver = self
            .ports
            .get(&layer.driver)
            .ok_or_else(|| format!("driver port `{}` no longer exists", layer.driver))?;
        let sink0 = self
            .ports
            .get(&layer.sinks[0])
            .ok_or_else(|| format!("sink port `{}` no longer exists", layer.sinks[0]))?;
        let mut wps = vec![driver.anchor];
        wps.extend(layer.gates.iter().map(|g| g.anchor));
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

    /// Drag a gate: the anchor moves unconditionally (the document's
    /// truth), then EXACTLY the two segments adjacent to the gate are
    /// ripped and rerouted atomically against the design-wide occupancy.
    /// An unroutable move leaves the bus `FAILED(reason)` with the
    /// fragment cleared — visible, never half-routed.
    pub fn move_gate(&mut self, bus: &str, gate: &str, anchor: P3) -> Result<GateMoveReport, String> {
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

        let occ = self.occupancy_for_plan(&ripped);
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
                })
            }
            Err(reason) => {
                let reason = format!(
                    "segment {:?} -> {:?} -> {:?} (gate `{gate}`): {reason}",
                    wp_before, anchor, wp_after
                );
                let layer = self.buses.get_mut(bus).unwrap();
                layer.fragment.clear();
                layer.runs.clear();
                layer.segments.clear();
                layer.gate_cells.clear();
                layer.state = BusState::Failed(reason.clone());
                Ok(GateMoveReport {
                    state: BusState::Failed(reason),
                    rerouted_segments: 2,
                })
            }
        }
    }

    /// Rip a bus: clear its fragment and return it to `Intended`. Station
    /// amendments stamped into OTHER buses by crossings stay (they remain
    /// electrically sound straight-line refreshes).
    pub fn rip(&mut self, name: &str) -> Result<(), String> {
        let bus = self
            .buses
            .get_mut(name)
            .ok_or_else(|| format!("unknown bus `{name}`"))?;
        bus.fragment.clear();
        bus.runs.clear();
        bus.segments.clear();
        bus.gate_cells.clear();
        bus.state = BusState::Intended;
        Ok(())
    }

    /// The design-wide spatial occupancy index: loose blocks, instance
    /// footprints, routed bus fragments, and instance influence halos.
    pub fn occupancy_index(&self) -> OccupancyIndex {
        self.occupancy_for_plan(&BTreeSet::new())
    }

    /// The occupancy index minus `skip` (cells a partial rip vacated).
    fn occupancy_for_plan(&self, skip: &BTreeSet<P3>) -> OccupancyIndex {
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
            let bbox = cell.schematic.get_bounding_box();
            for (bp, bs) in cell.schematic.iter_blocks() {
                let s = transform_state(bs, inst.rot_y).to_string();
                if s.contains("minecraft:air") {
                    continue;
                }
                let p = transform_pos((bp.x, bp.y, bp.z), bbox.min, bbox.max, inst.rot_y, inst.at);
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
            let cell = &self.cells[&inst.cell];
            let bbox = cell.schematic.get_bounding_box();
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
    fn realize(
        &self,
        exclude: Option<&str>,
        drivers: &[DesignPort],
        sinks: &[DesignPort],
        gates: &[Gate],
        style: &BusStyle,
    ) -> Result<Realization, String> {
        let step = (0, 2, 0);
        let width = drivers[0].width;
        for p in drivers.iter().chain(sinks.iter()) {
            if p.step != step {
                return Err(format!(
                    "unsupported bus form: this design realizes the verified vertical 2y-pitch \
                     stack (step (0,2,0)); port `{}` has step {:?}",
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

        // Trunk waypoint chain: primary driver, gates, primary sink.
        let mut waypoints = vec![drivers[0].anchor];
        waypoints.extend(gates.iter().map(|g| g.anchor));
        waypoints.push(sinks[0].anchor);
        for pair in waypoints.windows(2) {
            if pair[0].1 != pair[1].1 {
                return Err(format!(
                    "segment {:?} -> {:?}: endpoints differ in y; level changes land in a later \
                     phase",
                    pair[0], pair[1]
                ));
            }
        }

        // Trunk geometry (x-first corners) for the branch junction search.
        let trunk_runs = Self::trunk_geometry(&waypoints, width)?;
        let mut branches: Vec<(String, RunInfo, P3, bool)> = Vec::new();
        let mut keep: BTreeSet<P3> = BTreeSet::new();
        for sp in &sinks[1..] {
            let (run, junction) = Self::branch_geometry(&trunk_runs, sp, false)?;
            keep.insert(junction);
            branches.push((sp.name.clone(), run, junction, false));
        }
        for dp in &drivers[1..] {
            let (run, junction) = Self::branch_geometry(&trunk_runs, dp, true)?;
            keep.insert(junction);
            branches.push((dp.name.clone(), run, junction, true));
        }

        let occ = self.occupancy_for_plan(&BTreeSet::new());
        let mut planner = Planner::new(self, exclude, style, &occ);
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
        for (port, run, junction, is_driver) in &branches {
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
                (self.ports[port].anchor, *junction)
            } else {
                (*junction, self.ports[port].anchor)
            };
            planner.begin_segment(SegmentKind::Branch(port.clone()), a, b);
            planner
                .plan_run(run, &BTreeSet::new(), 0, !*is_driver, *is_driver)
                .map_err(|e| format!("branch for `{port}`: {e}"))?;
            planner.end_segment();
        }
        Ok(planner.finish())
    }

    /// The trunk's straight runs from its waypoint chain, choosing the
    /// x-first corner for non-straight pairs (deterministic; the planner
    /// may flip a congested corner z-first at plan time).
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

    /// Collapse the layer stack into ONE self-describing schematic: the
    /// loose layer stays in the base regions, every instance becomes region
    /// `inst:{name}`, every routed bus region `bus:{name}`, and the merged
    /// transformed contract is embedded in the metadata — the artifact is
    /// itself placeable as a cell.
    pub fn flatten(&self) -> Result<UniversalSchematic, String> {
        let mut flat = self.base.clone();
        flat.metadata.name = Some(self.name.clone());

        for inst in &self.instances {
            let cell = &self.cells[&inst.cell];
            let bbox = cell.schematic.get_bounding_box();
            let region = format!("inst:{}", inst.name);
            for (bp, bs) in cell.schematic.iter_blocks() {
                let s = bs.to_string();
                if s.contains("minecraft:air") {
                    continue;
                }
                let p = transform_pos((bp.x, bp.y, bp.z), bbox.min, bbox.max, inst.rot_y, inst.at);
                let state = transform_state(bs, inst.rot_y);
                if !flat.set_block_in_region(&region, p.0, p.1, p.2, &state) {
                    return Err(format!(
                        "flatten: could not place {s} at {:?} in region {region}",
                        p
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
            let Some(driver) = self.ports.get(&bus.driver) else {
                continue;
            };
            for bit in 0..driver.width {
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
                        terminals.push(sp.wire(bit));
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
        let violations = crate::routing::drc_schematic(&flat, &opts);
        let ws = crate::routing::workspace_from_schematic(&flat);
        let lvs = crate::routing::lvs(ws.cells(), &self.intent_nets());
        let (sta_json, rule_violations) = self.sta_and_rules(&flat);
        let clean = violations.is_empty()
            && lvs.opens.is_empty()
            && lvs.shorts.is_empty()
            && lvs.cycles.is_empty()
            && rule_violations.is_empty();
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
        let rules: Vec<String> = rule_violations.iter().map(|r| format!("{r:?}")).collect();
        let json = format!(
            "{{\"clean\":{clean},\"drc\":{},\"lvs\":{},\"buses\":{{{}}},\"sta\":{sta_json},\"rules\":[{}]}}",
            crate::routing::violations_json(&violations),
            crate::routing::lvs_report_json(&lvs),
            bus_states.join(","),
            rules.join(",")
        );
        Ok(DesignCheck { clean, json })
    }

    /// Per-bit repeater delay (redstone ticks) of a routed bus, from its
    /// fragment: a repeater at even offset from the canonical level
    /// belongs to that bit's straight run, at odd offset to the bit's dip
    /// station one level down.
    pub fn bus_bit_delays(&self, bus: &BusLayer) -> Vec<u64> {
        let Some(driver) = self.ports.get(&bus.driver) else {
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
    pub fn to_schem_bytes(&self) -> Result<Vec<u8>, String> {
        let flat = self.flatten()?;
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

    /// Plan one trunk waypoint pair: a straight run, or an L through one
    /// implicit corner joint (x-first, deterministically flipped z-first
    /// when the first choice collides). `since0` is the dust count since
    /// the last refresh entering the pair; the exit count is returned so
    /// refresh spacing stays sound across joints.
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
        if a.2 == b.2 && a.0 != b.0 {
            let run = RunInfo {
                along_x: true,
                fixed: a.2,
                y0: a.1,
                from: a.0,
                to: b.0,
                width,
            };
            return self.plan_run(&run, keep, since0, false, false);
        }
        if a.0 == b.0 && a.2 != b.2 {
            let run = RunInfo {
                along_x: false,
                fixed: a.0,
                y0: a.1,
                from: a.2,
                to: b.2,
                width,
            };
            return self.plan_run(&run, keep, since0, false, false);
        }
        let corner_x_first = (b.0, a.1, a.2);
        let corner_z_first = (a.0, a.1, b.2);
        let snap = self.snapshot();
        match self.plan_l(a, b, corner_x_first, true, width, keep, since0) {
            Ok(out) => Ok(out),
            Err(e1) => {
                self.restore(snap);
                self.plan_l(a, b, corner_z_first, false, width, keep, since0)
                    .map_err(|e2| format!("L corner x-first: {e1}; z-first: {e2}"))
            }
        }
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
                if orun.y0 != run.y0 || orun.width != run.width {
                    return Err(format!(
                        "crossing with bus `{}` at {:?}: levels/widths differ (theirs y0={} \
                         w={}, ours y0={} w={}); level adapters land in a later phase",
                        other.name,
                        (run.fixed, orun.fixed),
                        orun.y0,
                        orun.width,
                        run.y0,
                        run.width
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
        // A stone wall across the a-line: the straight run collides ->
        // FAILED, no panic, no partial fragment.
        for y in 0..=18 {
            d.set_block((12, y, 8), STONE).unwrap();
        }
        let state = d
            .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
            .unwrap();
        match state {
            BusState::Failed(reason) => assert!(reason.contains("collision"), "{reason}"),
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
