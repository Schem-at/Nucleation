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
//! Phase 1 realizes the verified vertical 2y-pitch bus form
//! (`redstone-eda/bus8_*.py`): straight axis-aligned segments between the
//! driver, optional gates and one sink, with implicit dip-under crossings
//! ported from `bus8_cross.py` v2 as template data. Everything else the
//! model describes (multi-sink trunks, interference co-reroute, drag APIs)
//! is Phase 2 and fails into a clear `BusState::Failed` rather than
//! guessing.

use crate::io_contract::{CellContract, IoType, LayoutFunction, PortDirection};
use crate::routing::engine::blocks as rblocks;
use crate::UniversalSchematic;
use std::collections::BTreeMap;

/// Position triple, kept plain so the module stays wasm-safe and
/// serde-friendly.
pub type P3 = (i32, i32, i32);

fn add(a: P3, b: P3) -> P3 {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

fn scale(a: P3, k: i32) -> P3 {
    (a.0 * k, a.1 * k, a.2 * k)
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

/// A bus layer: endpoints with roles, gates, style, state, and the OWNED
/// voxel fragment.
#[derive(Clone, Debug)]
pub struct BusLayer {
    pub name: String,
    /// Driver port name (exactly one in Phase 1; wired-OR merge is Phase 2).
    pub driver: String,
    /// Sink port names.
    pub sinks: Vec<String>,
    pub gates: Vec<Gate>,
    pub style: BusStyle,
    pub state: BusState,
    /// The owned voxel fragment (block per cell), empty unless `Routed`.
    pub fragment: BTreeMap<P3, String>,
    /// The straight runs realized (crossing detection input).
    pub runs: Vec<RunInfo>,
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
    /// optional `gates` in order. Declaration errors (unknown port, width
    /// mismatch, invalid style, duplicate name) are `Err`; geometric
    /// unroutability is a returned [`BusState::Failed`] — realization is
    /// atomic and never leaves a half-routed fragment.
    pub fn route_bus(
        &mut self,
        name: impl Into<String>,
        driver: &str,
        sinks: &[&str],
        gates: Vec<Gate>,
        style: BusStyle,
    ) -> Result<BusState, String> {
        let name = name.into();
        if self.buses.contains_key(&name) {
            return Err(format!("bus `{name}` already exists"));
        }
        style.validate()?;
        let drv = self
            .ports
            .get(driver)
            .ok_or_else(|| format!("unknown driver port `{driver}`"))?;
        if drv.direction == PortDirection::Output {
            return Err(format!(
                "bus `{name}`: driver `{driver}` is a declared output; bidirectional buses are \
                 modeled but reserved (Phase 2), and outputs cannot drive"
            ));
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
            if sp.width != drv.width {
                return Err(format!(
                    "bus `{name}`: sink `{s}` width {} != driver width {}",
                    sp.width, drv.width
                ));
            }
            sink_ports.push(sp.clone());
        }
        let drv = drv.clone();

        let mut layer = BusLayer {
            name: name.clone(),
            driver: driver.to_string(),
            sinks: sinks.iter().map(|s| s.to_string()).collect(),
            gates,
            style,
            state: BusState::Intended,
            fragment: BTreeMap::new(),
            runs: Vec::new(),
        };

        match self.realize(&drv, &sink_ports, &layer.gates, &layer.style) {
            Ok(real) => {
                // Atomic commit: fragment + amendments to crossed buses.
                layer.fragment = real.fragment;
                layer.runs = real.runs;
                layer.state = BusState::Routed;
                for (bus, removals, additions) in real.amendments {
                    let target = self.buses.get_mut(&bus).expect("amended bus exists");
                    for p in removals {
                        target.fragment.remove(&p);
                    }
                    target.fragment.extend(additions);
                }
            }
            Err(reason) => {
                layer.state = BusState::Failed(reason);
            }
        }
        let state = layer.state.clone();
        self.buses.insert(name, layer);
        Ok(state)
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
        bus.state = BusState::Intended;
        Ok(())
    }

    /// The design-wide occupancy: base + instances + routed fragments.
    fn occupancy(&self) -> Result<BTreeMap<P3, String>, String> {
        let mut occ = BTreeMap::new();
        for (bp, bs) in self.base.iter_blocks() {
            let s = bs.to_string();
            if !s.contains("minecraft:air") {
                occ.insert((bp.x, bp.y, bp.z), s);
            }
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
                occ.insert(p, s);
            }
        }
        for bus in self.buses.values() {
            for (p, b) in &bus.fragment {
                occ.insert(*p, b.clone());
            }
        }
        Ok(occ)
    }

    /// Realize a bus (pure planning: no mutation). `Err` is the
    /// user-facing failure reason.
    fn realize(
        &self,
        driver: &DesignPort,
        sinks: &[DesignPort],
        gates: &[Gate],
        style: &BusStyle,
    ) -> Result<Realization, String> {
        if sinks.len() != 1 {
            return Err(format!(
                "{} sinks: shared trunk + branch realization lands in Phase 2; route one sink \
                 per bus for now",
                sinks.len()
            ));
        }
        let sink = &sinks[0];
        let step = (0, 2, 0);
        if driver.step != step || sink.step != step {
            return Err(format!(
                "unsupported bus form: Phase 1 realizes the verified vertical 2y-pitch stack \
                 (step (0,2,0)); got driver {:?}, sink {:?}",
                driver.step, sink.step
            ));
        }
        for g in gates {
            if g.step != step {
                return Err(format!(
                    "gate `{}`: step {:?} does not match the bus form (0,2,0)",
                    g.name, g.step
                ));
            }
        }

        // Waypoint chain: driver anchor, gates, sink anchor. Each
        // consecutive pair is one independently-routed straight segment.
        let mut waypoints = vec![driver.anchor];
        waypoints.extend(gates.iter().map(|g| g.anchor));
        waypoints.push(sink.anchor);

        let occ = self.occupancy()?;
        let width = driver.width;
        let mut real = Realization::default();

        for pair in waypoints.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a.1 != b.1 {
                return Err(format!(
                    "segment {:?} -> {:?}: endpoints differ in y; level changes land in Phase 2",
                    a, b
                ));
            }
            let run = if a.2 == b.2 && a.0 != b.0 {
                RunInfo {
                    along_x: true,
                    fixed: a.2,
                    y0: a.1,
                    from: a.0,
                    to: b.0,
                    width,
                }
            } else if a.0 == b.0 && a.2 != b.2 {
                RunInfo {
                    along_x: false,
                    fixed: a.0,
                    y0: a.1,
                    from: a.2,
                    to: b.2,
                    width,
                }
            } else {
                return Err(format!(
                    "segment {:?} -> {:?}: not a straight axis-aligned run; add a gate to bend \
                     the bus",
                    a, b
                ));
            };
            self.plan_segment(&run, style, &mut real)?;
            real.runs.push(run);
        }

        // Gate columns: dust on supports at every bit (the waypoint is a
        // real electrical joint between segments).
        for g in gates {
            for k in 0..width {
                let p = add(g.anchor, (0, 2 * k as i32, 0));
                real.put(p, rblocks::DUST)?;
                real.put((p.0, p.1 - 1, p.2), &style.bus_block)?;
            }
        }

        // Atomicity: every planned cell must be free (or identical) in the
        // design-wide occupancy, side-stepping the cells this plan's
        // amendments vacate.
        let vacated: std::collections::BTreeSet<P3> = real
            .amendments
            .iter()
            .flat_map(|(_, removals, _)| removals.iter().copied())
            .collect();
        let added: Vec<(P3, String)> = real
            .fragment
            .iter()
            .map(|(p, b)| (*p, b.clone()))
            .chain(
                real.amendments
                    .iter()
                    .flat_map(|(_, _, adds)| adds.iter().map(|(p, b)| (*p, b.clone()))),
            )
            .collect();
        for (p, b) in added {
            if vacated.contains(&p) {
                continue;
            }
            if let Some(existing) = occ.get(&p) {
                if existing != &b {
                    return Err(format!(
                        "collision at {:?}: `{existing}` already there, wanted `{b}`",
                        p
                    ));
                }
            }
        }
        Ok(real)
    }

    /// Plan one straight segment: straight-run cells, refresh repeaters and
    /// implicit dip-under crossings against already-routed buses.
    fn plan_segment(
        &self,
        run: &RunInfo,
        style: &BusStyle,
        real: &mut Realization,
    ) -> Result<(), String> {
        let sign = run.sign();

        // Crossings against every routed bus's perpendicular runs.
        let mut crossings: Vec<i32> = Vec::new(); // centers along OUR axis
        for other in self.buses.values() {
            if other.state != BusState::Routed {
                continue;
            }
            for orun in &other.runs {
                if orun.along_x == run.along_x {
                    continue; // parallel: overlap shows up as a collision
                }
                // Perpendicular: our crossing coordinate along our axis is
                // THEIR fixed coordinate, and vice versa.
                let center = orun.fixed;
                if !run.strictly_inside(center, 3) || !orun.strictly_inside(run.fixed, 3) {
                    continue;
                }
                if orun.y0 != run.y0 || orun.width != run.width {
                    return Err(format!(
                        "crossing with bus `{}` at {:?}: levels/widths differ (theirs y0={} \
                         w={}, ours y0={} w={}); level adapters land in Phase 2",
                        other.name,
                        (run.fixed, orun.fixed),
                        orun.y0,
                        orun.width,
                        run.y0,
                        run.width
                    ));
                }
                crossings.push(center);
                // The crossed (through) bus gets its station upgrade.
                self.plan_station_amendment(other, orun, run.fixed, style, real)?;
            }
        }
        crossings.sort();
        crossings.dedup();
        for w in crossings.windows(2) {
            if (w[1] - w[0]).abs() < 8 {
                return Err(format!(
                    "crossing windows at {} and {} overlap (need 8 cells of spacing)",
                    w[0], w[1]
                ));
            }
        }

        // Cell coordinates strictly between the anchors.
        let in_window = |c: i32| crossings.iter().any(|&cc| ((c - cc) * sign) >= -2 && ((c - cc) * sign) <= 4);
        let near_window = |c: i32| crossings.iter().any(|&cc| ((c - cc) * sign) >= -4 && ((c - cc) * sign) <= 6);

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

        for bit in 0..run.width {
            let y = run.y0 + 2 * bit as i32;
            let d = y - 1;
            let mut since_refresh = 0u32;
            let mut c = run.from + sign;
            while c != run.to {
                if in_window(c) {
                    since_refresh = 0; // the window's own repeater refreshes
                    c += sign;
                    continue;
                }
                let steps_from_ends = (c - run.from).abs().min((run.to - c).abs());
                if since_refresh >= 9 && steps_from_ends >= 2 && !near_window(c) {
                    real.put(pos_at(c, y - 1), &style.bus_block)?;
                    real.put(pos_at(c, y), &rblocks::repeater(toward_driver, 1))?;
                    since_refresh = 0;
                } else {
                    real.put(pos_at(c, y - 1), &style.bus_block)?;
                    real.put(pos_at(c, y), rblocks::DUST)?;
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
            for &cc in &crossings {
                let at = |o: i32, y: i32| pos_at(cc + o * sign, y);
                // o = -2 and 4: the step upper cells (conductor support —
                // the diode law — which also severs cross-bit diagonals).
                real.put(at(-2, y - 1), &style.bus_block)?;
                real.put(at(-2, y), rblocks::DUST)?;
                real.put(at(4, y - 1), &style.bus_block)?;
                real.put(at(4, y), rblocks::DUST)?;
                // o = -1 and 3: the dip dust cells; their supports sit
                // directly above bit n-1's dip dust diagonal.
                let dip_support = if bit > 0 {
                    style.transparent_block.as_str()
                } else {
                    style.bus_block.as_str()
                };
                real.put(at(-1, d - 1), dip_support)?;
                real.put(at(-1, d), rblocks::DUST)?;
                real.put(at(3, d - 1), dip_support)?;
                real.put(at(3, d), rblocks::DUST)?;
                // o = 0..2: the block-sandwich station at the dip level.
                real.put(at(0, d), &style.bus_block)?;
                real.put(at(1, d - 1), &style.bus_block)?;
                real.put(at(1, d), &rblocks::repeater(toward_driver, 1))?;
                real.put(at(2, d), &style.bus_block)?;
            }
        }
        Ok(())
    }

    /// The station upgrade a crossing stamps into the THROUGH bus at the
    /// crossing column: its plain dust run over o = 0..2 becomes entry
    /// block / repeater / exit block (the crossed bus pays one repeater
    /// per bit, doubling as its refresh).
    fn plan_station_amendment(
        &self,
        other: &BusLayer,
        orun: &RunInfo,
        center: i32,
        _style: &BusStyle,
        real: &mut Realization,
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
            additions.insert(
                pos_at(center + sa, y),
                rblocks::repeater(toward_driver, 1),
            );
            additions.insert(pos_at(center + 2 * sa, y), bus_block);
        }
        real.amendments
            .push((other.name.clone(), removals, additions));
        Ok(())
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
                let mut terminals = vec![driver.wire(bit)];
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

    /// DRC + LVS over the flattened artifact. STA/skew is a Phase 2 stage.
    pub fn check(&self) -> Result<DesignCheck, String> {
        let flat = self.flatten()?;
        let opts = crate::routing::DrcOptions {
            aliases: vec![],
            skip_decay: true,
        };
        let violations = crate::routing::drc_schematic(&flat, &opts);
        let ws = crate::routing::workspace_from_schematic(&flat);
        let lvs = crate::routing::lvs(ws.cells(), &self.intent_nets());
        let clean =
            violations.is_empty() && lvs.opens.is_empty() && lvs.shorts.is_empty() && lvs.cycles.is_empty();
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
        let json = format!(
            "{{\"clean\":{clean},\"drc\":{},\"lvs\":{},\"buses\":{{{}}}}}",
            crate::routing::violations_json(&violations),
            crate::routing::lvs_report_json(&lvs),
            bus_states.join(",")
        );
        Ok(DesignCheck { clean, json })
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
}

/// A planned realization: the new bus's fragment plus amendments
/// (removals + additions) to the buses it crosses.
#[derive(Default)]
struct Realization {
    fragment: BTreeMap<P3, String>,
    amendments: Vec<(String, Vec<P3>, BTreeMap<P3, String>)>,
    runs: Vec<RunInfo>,
}

impl Realization {
    /// Add a planned cell; identical double-writes are fine (shared
    /// supports), diverging ones are a planner bug surfaced as failure.
    fn put(&mut self, p: P3, block: &str) -> Result<(), String> {
        if let Some(existing) = self.fragment.get(&p) {
            if existing != block {
                return Err(format!(
                    "internal plan conflict at {:?}: `{existing}` vs `{block}`",
                    p
                ));
            }
            return Ok(());
        }
        self.fragment.insert(p, block.to_string());
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
        // A dogleg without a gate: not a straight run -> FAILED, no panic,
        // no partial fragment.
        let state = d
            .route_bus("diag", "a_in", &["b_out"], vec![], BusStyle::default())
            .unwrap();
        match state {
            BusState::Failed(reason) => assert!(reason.contains("axis-aligned"), "{reason}"),
            other => panic!("expected Failed, got {other:?}"),
        }
        assert!(d.bus("diag").unwrap().fragment.is_empty());
        // The failed layer does not poison later routing.
        let a = d
            .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
            .unwrap();
        assert_eq!(a, BusState::Routed);
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
