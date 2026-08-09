//! Design-document serialization: the `.nucm` project tier and the
//! `.litematic` layered-interchange tier of `redstone-eda/DESIGN_SPEC.md`
//! §8. The flat `.schem` artifact tier already lives on
//! [`Design::to_schem_bytes`].
//!
//! - **`.nucm`** (magic `NUCM`, versioned, bincode payload): full fidelity —
//!   cell schematics embedded ONCE per content hash (two cells with
//!   identical bodies share one blob), instance transforms, ports with
//!   their scanned hardware, bus layers including fragments, runs, gates,
//!   styles and the `intended | routed | FAILED(reason)` state, and the
//!   loose base layer. Reopenable mid-edit: a reloaded design reroutes.
//!   Cell schematics and the base ride as NUSN snapshot payloads
//!   (`src/formats/snapshot.rs`), so region caches rebuild on load.
//! - **`.litematic`** layered interchange: `flatten()`'s named regions
//!   (`inst:{name}`, `bus:{name}`, loose base) written by the existing
//!   litematic writer, plus a design manifest (instances, transforms,
//!   ports, bus metadata, merged contract) as a root-level
//!   `NucleationDesign` string tag beside `Metadata` — root-level so
//!   Litematica ignores it, following the `NucleationTest` pattern. The
//!   file opens in Litematica as a plain multi-region litematic. On
//!   import, cell REFERENCES degrade to embedded copies: each instance
//!   becomes its own single-instance cell holding the already-transformed
//!   region blocks (identity transform), as the spec documents for the
//!   interchange tier. Block entities and entities of the layers are not
//!   reconstructed by this tier — use `.nucm` for full fidelity.
//!
//! Everything here is bytes-in/bytes-out and wasm-safe; filesystem
//! convenience wrappers are gated off wasm32.

use crate::design::{
    BitHardware, BusLayer, BusState, BusStyle, CellDef, Design, DesignPort, Gate, Instance,
    RunInfo, Segment, SegmentKind, P3, WidthMap,};
use crate::io_contract::{CellContract, IoType, NetClassRule, PortDirection};
use crate::UniversalSchematic;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const NUCM_MAGIC: &[u8; 4] = b"NUCM";
// v2 adds `InstanceCore::port_modes` (promotion). bincode is positional, so
// the version gate is what keeps a v1 document from being misread.
const NUCM_VERSION: u32 = 2;

/// Root-level NBT tag carrying the design manifest in a layered
/// `.litematic` export (beside `Metadata`, like `NucleationTest`).
pub const DESIGN_MANIFEST_TAG: &str = "NucleationDesign";
const MANIFEST_FORMAT: &str = "nucleation-design";
const MANIFEST_VERSION: u32 = 1;

// ---------------------------------------------------------------------
// Shared serde mirrors of the design model (kept explicit so the file
// format is versioned independently of the in-memory types).
// ---------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BitDoc {
    connectable: bool,
    lever: Option<P3>,
    lamp: Option<P3>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PortDoc {
    anchor: P3,
    step: P3,
    width: u8,
    ty: IoType,
    direction: PortDirection,
    bits: Vec<BitDoc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GateDoc {
    name: String,
    anchor: P3,
    step: P3,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StyleDoc {
    bus_block: String,
    transparent_block: String,
}

/// `"intended"` / `"routed"` / `{"failed":{"reason":"..."}}` in the JSON
/// manifest (externally tagged: bincode needs knowable sizes, so no
/// `serde(tag)`/`serde(flatten)` here).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
enum StateDoc {
    Intended,
    Routed,
    Failed { reason: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RunDoc {
    along_x: bool,
    fixed: i32,
    y0: i32,
    from: i32,
    to: i32,
    width: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
enum SegmentKindDoc {
    Trunk(usize),
    Branch(String),
    /// The bus-owned row->stack form adapter for the named port.
    Adapter(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SegmentDoc {
    kind: SegmentKindDoc,
    a: P3,
    b: P3,
    runs: Vec<RunDoc>,
    cells: BTreeSet<P3>,
}

/// Bus metadata shared by both tiers (the fragment itself is tier-specific:
/// inline in `.nucm`, region blocks in `.litematic`).
#[derive(Serialize, Deserialize, Clone, Debug)]
struct BusMeta {
    driver: String,
    #[serde(default)]
    extra_drivers: Vec<String>,
    #[serde(default)]
    merge_or: bool,
    sinks: Vec<String>,
    gates: Vec<GateDoc>,
    style: StyleDoc,
    state: StateDoc,
    runs: Vec<RunDoc>,
    #[serde(default)]
    segments: Vec<SegmentDoc>,
    #[serde(default)]
    gate_cells: BTreeMap<String, BTreeSet<P3>>,
    #[serde(default)]
    rule: Option<NetClassRule>,
    #[serde(default)]
    width_map: Option<WidthMapDoc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InstanceCore {
    name: String,
    cell: String,
    at: P3,
    rot_y: i32,
    /// Per-port mode overrides (promotion). Always written: the `.nucm`
    /// payload is bincode, which is not self-describing, so a `serde(default)`
    /// field would desynchronise the stream rather than default. Documents
    /// from before port modes existed are rejected by the version check.
    port_modes: BTreeMap<String, PortModeDoc>,
}

/// One cell of a promotion patch: the position and what goes there
/// (`None` = clear the cell).
///
/// A LIST of these, not a map keyed by position, and that is load-bearing:
/// `P3` is a tuple, and **serde_json cannot serialize a map with a non-string
/// key** — it fails with "key must be a string". The `.nucm` payload is bincode
/// and never cared, but the `.litematic` manifest is JSON, so a `BTreeMap<P3,
/// _>` here made `Design::to_litematic_layered_bytes` fail for EVERY design with
/// a promoted port. See `promoting_a_port_does_not_break_any_export`.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct PatchCellDoc {
    at: P3,
    block: Option<String>,
}

fn patch_cells(m: &BTreeMap<P3, Option<String>>) -> Vec<PatchCellDoc> {
    m.iter()
        .map(|(at, block)| PatchCellDoc {
            at: *at,
            block: block.clone(),
        })
        .collect()
}

fn patch_map(v: Vec<PatchCellDoc>) -> BTreeMap<P3, Option<String>> {
    v.into_iter().map(|c| (c.at, c.block)).collect()
}

/// A port's remembered both-forms state (see `design::PortOverride`).
#[derive(Serialize, Deserialize, Clone, Debug)]
struct PortModeDoc {
    mode: String,
    writes: Vec<PatchCellDoc>,
    saved: Vec<PatchCellDoc>,
    wires: Vec<P3>,
    hardware: Vec<P3>,
    step: P3,
    pivoted: bool,
    note: String,
}

impl PortModeDoc {
    fn of(o: &crate::design::PortOverride) -> Self {
        PortModeDoc {
            mode: o.mode.as_str().to_string(),
            writes: patch_cells(&o.patch.writes),
            saved: patch_cells(&o.patch.saved),
            wires: o.patch.wires.clone(),
            hardware: o.patch.hardware.clone(),
            step: o.patch.step,
            pivoted: o.patch.pivoted,
            note: o.patch.note.clone(),
        }
    }

    fn into_override(self) -> crate::design::PortOverride {
        crate::design::PortOverride {
            mode: crate::design::PortMode::parse(&self.mode)
                .unwrap_or(crate::design::PortMode::Executor),
            patch: crate::design_promote::PortPatch {
                writes: patch_map(self.writes),
                saved: patch_map(self.saved),
                wires: self.wires,
                hardware: self.hardware,
                step: self.step,
                pivoted: self.pivoted,
                note: self.note,
            },
        }
    }
}

// ---------------------------------------------------------------------
// .nucm project document
// ---------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct NucmCell {
    /// Key into `NucmDoc::blobs` (content-hash dedup).
    blob: u64,
    /// The resolved contract, as its stable JSON form.
    contract: String,
}

#[derive(Serialize, Deserialize)]
struct NucmBus {
    meta: BusMeta,
    fragment: BTreeMap<P3, String>,
}

#[derive(Serialize, Deserialize)]
struct NucmDoc {
    name: String,
    /// The loose base layer as an NUSN snapshot payload.
    base: Vec<u8>,
    /// Deduped cell bodies: content hash -> NUSN snapshot payload.
    blobs: BTreeMap<u64, Vec<u8>>,
    cells: BTreeMap<String, NucmCell>,
    instances: Vec<InstanceCore>,
    ports: BTreeMap<String, PortDoc>,
    buses: BTreeMap<String, NucmBus>,
}

// ---------------------------------------------------------------------
// .litematic manifest document
// ---------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct ManifestInstance {
    #[serde(flatten)]
    core: InstanceCore,
    /// The TRANSFORMED instance contract (matches the region blocks), so
    /// import can degrade the reference to an embedded copy.
    contract: String,
}

#[derive(Serialize, Deserialize)]
struct ManifestDoc {
    format: String,
    version: u32,
    name: String,
    instances: Vec<ManifestInstance>,
    ports: BTreeMap<String, PortDoc>,
    buses: BTreeMap<String, BusMeta>,
    /// The merged flattened contract (also embedded in `.schem` exports;
    /// litematic has no cell-contract carrier, so it rides here).
    merged_contract: Option<String>,
}

// ---------------------------------------------------------------------
// Mirror conversions
// ---------------------------------------------------------------------

fn bit_doc(hw: &BitHardware) -> BitDoc {
    BitDoc {
        connectable: hw.connectable,
        lever: hw.lever,
        lamp: hw.lamp,
    }
}

fn bit_from(doc: BitDoc) -> BitHardware {
    BitHardware {
        connectable: doc.connectable,
        lever: doc.lever,
        lamp: doc.lamp,
    }
}

fn port_doc(p: &DesignPort) -> PortDoc {
    PortDoc {
        anchor: p.anchor,
        step: p.step,
        width: p.width,
        ty: p.ty.clone(),
        direction: p.direction,
        bits: p.bits.iter().map(bit_doc).collect(),
    }
}

fn port_from(name: &str, doc: PortDoc) -> DesignPort {
    DesignPort {
        name: name.to_string(),
        anchor: doc.anchor,
        step: doc.step,
        width: doc.width,
        ty: doc.ty,
        direction: doc.direction,
        bits: doc.bits.into_iter().map(bit_from).collect(),
    }
}

fn state_doc(s: &BusState) -> StateDoc {
    match s {
        BusState::Intended => StateDoc::Intended,
        BusState::Routed => StateDoc::Routed,
        BusState::Failed(reason) => StateDoc::Failed {
            reason: reason.clone(),
        },
    }
}

fn state_from(doc: StateDoc) -> BusState {
    match doc {
        StateDoc::Intended => BusState::Intended,
        StateDoc::Routed => BusState::Routed,
        StateDoc::Failed { reason } => BusState::Failed(reason),
    }
}

fn run_doc(r: &RunInfo) -> RunDoc {
    RunDoc {
        along_x: r.along_x,
        fixed: r.fixed,
        y0: r.y0,
        from: r.from,
        to: r.to,
        width: r.width,
    }
}

fn run_from(r: RunDoc) -> RunInfo {
    RunInfo {
        along_x: r.along_x,
        fixed: r.fixed,
        y0: r.y0,
        from: r.from,
        to: r.to,
        width: r.width,
    }
}

fn segment_doc(s: &Segment) -> SegmentDoc {
    SegmentDoc {
        kind: match &s.kind {
            SegmentKind::Trunk(i) => SegmentKindDoc::Trunk(*i),
            SegmentKind::Branch(name) => SegmentKindDoc::Branch(name.clone()),
            SegmentKind::Adapter(name) => SegmentKindDoc::Adapter(name.clone()),
        },
        a: s.a,
        b: s.b,
        runs: s.runs.iter().map(run_doc).collect(),
        cells: s.cells.clone(),
    }
}

fn segment_from(s: SegmentDoc) -> Segment {
    Segment {
        kind: match s.kind {
            SegmentKindDoc::Trunk(i) => SegmentKind::Trunk(i),
            SegmentKindDoc::Branch(name) => SegmentKind::Branch(name),
            SegmentKindDoc::Adapter(name) => SegmentKind::Adapter(name),
        },
        a: s.a,
        b: s.b,
        runs: s.runs.into_iter().map(run_from).collect(),
        cells: s.cells,
    }
}

fn bus_meta(b: &BusLayer) -> BusMeta {
    BusMeta {
        driver: b.driver.clone(),
        extra_drivers: b.extra_drivers.clone(),
        merge_or: b.merge_or,
        sinks: b.sinks.clone(),
        gates: b
            .gates
            .iter()
            .map(|g| GateDoc {
                name: g.name.clone(),
                anchor: g.anchor,
                step: g.step,
            })
            .collect(),
        style: StyleDoc {
            bus_block: b.style.bus_block.clone(),
            transparent_block: b.style.transparent_block.clone(),
        },
        state: state_doc(&b.state),
        runs: b.runs.iter().map(run_doc).collect(),
        segments: b.segments.iter().map(segment_doc).collect(),
        gate_cells: b.gate_cells.clone(),
        rule: b.rule.clone(),
        width_map: b.width_map.as_ref().map(width_map_doc),
    }
}

/// Serializable mirror of [`nucleation::design::WidthMap`]. PERSISTED, unlike
/// `promotions`: the bit mapping is part of the bus's INTENT (it is what LVS
/// pairs and what the UI shows), so a reloaded document must carry it or it
/// would reroute to a different wiring.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct WidthMapDoc {
    driver_width: u8,
    sink_width: u8,
    shift: i32,
    from_bit: u8,
    bits: u8,
    #[serde(default)]
    tied_zero: Vec<u8>,
    #[serde(default)]
    dropped: Vec<u8>,
}

fn width_map_doc(m: &WidthMap) -> WidthMapDoc {
    WidthMapDoc {
        driver_width: m.driver_width,
        sink_width: m.sink_width,
        shift: m.shift,
        from_bit: m.from_bit,
        bits: m.bits,
        tied_zero: m.tied_zero.clone(),
        dropped: m.dropped.clone(),
    }
}

fn width_map_from(d: WidthMapDoc) -> WidthMap {
    WidthMap {
        driver_width: d.driver_width,
        sink_width: d.sink_width,
        shift: d.shift,
        from_bit: d.from_bit,
        bits: d.bits,
        tied_zero: d.tied_zero,
        dropped: d.dropped,
    }
}

fn bus_from(name: &str, meta: BusMeta, fragment: BTreeMap<P3, String>) -> BusLayer {
    BusLayer {
        // Not persisted, and correctly so: `promotions` reports what a
        // `route_bus` CALL changed. A reloaded document already carries the
        // promotion in the instance's port modes, so this load promoted nothing.
        promotions: Vec::new(),
        width_map: meta.width_map.map(width_map_from),
        name: name.to_string(),
        driver: meta.driver,
        extra_drivers: meta.extra_drivers,
        merge_or: meta.merge_or,
        sinks: meta.sinks,
        gates: meta
            .gates
            .into_iter()
            .map(|g| Gate {
                name: g.name,
                anchor: g.anchor,
                step: g.step,
            })
            .collect(),
        style: BusStyle {
            bus_block: meta.style.bus_block,
            transparent_block: meta.style.transparent_block,
        },
        state: state_from(meta.state),
        fragment,
        runs: meta.runs.into_iter().map(run_from).collect(),
        segments: meta.segments.into_iter().map(segment_from).collect(),
        gate_cells: meta.gate_cells,
        rule: meta.rule,
    }
}

/// FNV-1a 64: the blob content hash (equal bytes -> equal key; colliding
/// unequal bytes are linear-probed to the next free key, so the map stays
/// exact whatever the hash does).
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// ---------------------------------------------------------------------
// Region helpers (litematic tier)
// ---------------------------------------------------------------------

fn all_regions(s: &UniversalSchematic) -> impl Iterator<Item = &crate::region::Region> {
    std::iter::once(&s.default_region).chain(s.other_regions.values())
}

/// Every non-air block of a region, in absolute coordinates.
fn region_blocks(r: &crate::region::Region) -> Vec<(P3, String)> {
    let bbox = r.get_bounding_box();
    let mut out = Vec::new();
    for y in bbox.min.1..=bbox.max.1 {
        for z in bbox.min.2..=bbox.max.2 {
            for x in bbox.min.0..=bbox.max.0 {
                if let Some(b) = r.get_block(x, y, z) {
                    let s = b.to_string();
                    if !s.contains("minecraft:air") {
                        out.push(((x, y, z), s));
                    }
                }
            }
        }
    }
    out
}

fn is_layer_region(name: &str) -> bool {
    name.starts_with("inst:") || name.starts_with("bus:")
}

// ---------------------------------------------------------------------
// Litematic root-tag surgery: the manifest is added AFTER the writer runs
// (and read before the parser), so the existing litematic module stays
// untouched and rebuild-from-scratch metadata can never drop it.
// ---------------------------------------------------------------------

fn read_litematic_root(data: &[u8]) -> Result<quartz_nbt::NbtCompound, String> {
    let mut gz = flate2::read::GzDecoder::new(std::io::Cursor::new(data));
    let (root, _) = quartz_nbt::io::read_nbt(&mut gz, quartz_nbt::io::Flavor::Uncompressed)
        .map_err(|e| format!("not a litematic (gzip NBT): {e}"))?;
    Ok(root)
}

fn write_litematic_root(root: &quartz_nbt::NbtCompound) -> Result<Vec<u8>, String> {
    let mut enc =
        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(3));
    quartz_nbt::io::write_nbt(&mut enc, None, root, quartz_nbt::io::Flavor::Uncompressed)
        .map_err(|e| e.to_string())?;
    enc.finish().map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------
// The Design serialization surface
// ---------------------------------------------------------------------

impl Design {
    /// Serialize the full design document to `.nucm` bytes (project tier):
    /// cells deduped by content hash, transforms, ports, every bus layer
    /// with its state (including `FAILED(reason)`), fragment and runs, and
    /// the loose base layer. Wasm-safe: bytes out, no filesystem.
    pub fn to_nucm_bytes(&self) -> Result<Vec<u8>, String> {
        let (name, base, cells, instances, ports, buses) = self.io_parts();

        let mut blobs: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
        let mut cell_docs = BTreeMap::new();
        for (cname, cell) in cells {
            let bytes = crate::formats::snapshot::to_snapshot(&cell.schematic)
                .map_err(|e| format!("cell `{cname}`: {e}"))?;
            let mut key = fnv1a64(&bytes);
            while blobs.get(&key).is_some_and(|b| b != &bytes) {
                key = key.wrapping_add(1);
            }
            blobs.entry(key).or_insert(bytes);
            cell_docs.insert(
                cname.clone(),
                NucmCell {
                    blob: key,
                    contract: cell.contract.to_json()?,
                },
            );
        }

        let doc = NucmDoc {
            name: name.to_string(),
            base: crate::formats::snapshot::to_snapshot(base).map_err(|e| e.to_string())?,
            blobs,
            cells: cell_docs,
            instances: instances
                .iter()
                .map(|i| InstanceCore {
                    name: i.name.clone(),
                    cell: i.cell.clone(),
                    at: i.at,
                    rot_y: i.rot_y,
                    port_modes: i
                        .port_modes
                        .iter()
                        .map(|(k, v)| (k.clone(), PortModeDoc::of(v)))
                        .collect(),
                })
                .collect(),
            ports: ports.iter().map(|(n, p)| (n.clone(), port_doc(p))).collect(),
            buses: buses
                .iter()
                .map(|(n, b)| {
                    (
                        n.clone(),
                        NucmBus {
                            meta: bus_meta(b),
                            fragment: b.fragment.clone(),
                        },
                    )
                })
                .collect(),
        };

        let payload = bincode::serialize(&doc).map_err(|e| e.to_string())?;
        let mut buf = Vec::with_capacity(8 + payload.len());
        buf.extend_from_slice(NUCM_MAGIC);
        buf.extend_from_slice(&NUCM_VERSION.to_le_bytes());
        buf.extend_from_slice(&payload);
        Ok(buf)
    }

    /// Reopen a `.nucm` project document. The reloaded design is the same
    /// model mid-edit: ports keep their scanned hardware, buses their
    /// fragments and states, and rerouting works.
    pub fn from_nucm_bytes(data: &[u8]) -> Result<Design, String> {
        if data.len() < 8 || &data[0..4] != NUCM_MAGIC {
            return Err("not a .nucm design document (bad magic)".to_string());
        }
        let version = u32::from_le_bytes(data[4..8].try_into().map_err(|_| "short header")?);
        if version != NUCM_VERSION {
            return Err(format!("unsupported .nucm version {version} (this build reads {NUCM_VERSION})"));
        }
        let doc: NucmDoc = bincode::deserialize(&data[8..]).map_err(|e| e.to_string())?;

        let base =
            crate::formats::snapshot::from_snapshot(&doc.base).map_err(|e| e.to_string())?;
        let mut cells = BTreeMap::new();
        for (cname, c) in doc.cells {
            let blob = doc
                .blobs
                .get(&c.blob)
                .ok_or_else(|| format!("cell `{cname}`: missing blob {:#x}", c.blob))?;
            let schematic =
                crate::formats::snapshot::from_snapshot(blob).map_err(|e| e.to_string())?;
            let contract = CellContract::from_json(&c.contract)
                .map_err(|e| format!("cell `{cname}`: {e}"))?;
            cells.insert(
                cname,
                CellDef {
                    schematic,
                    contract,
                },
            );
        }
        for inst in &doc.instances {
            if !cells.contains_key(&inst.cell) {
                return Err(format!(
                    "instance `{}` references missing cell `{}`",
                    inst.name, inst.cell
                ));
            }
        }
        let ports = doc
            .ports
            .into_iter()
            .map(|(n, p)| {
                let port = port_from(&n, p);
                (n, port)
            })
            .collect();
        let buses = doc
            .buses
            .into_iter()
            .map(|(n, b)| {
                let bus = bus_from(&n, b.meta, b.fragment);
                (n, bus)
            })
            .collect();
        Ok(Design::from_io_parts(
            doc.name,
            base,
            cells,
            doc.instances
                .into_iter()
                .map(|i| Instance {
                    name: i.name,
                    cell: i.cell,
                    at: i.at,
                    rot_y: i.rot_y,
                    port_modes: i
                        .port_modes
                        .into_iter()
                        .map(|(k, v)| (k, v.into_override()))
                        .collect(),
                })
                .collect(),
            ports,
            buses,
        ))
    }

    /// The design manifest JSON a layered `.litematic` export carries.
    fn manifest_json(&self) -> Result<String, String> {
        let (name, _base, _cells, instances, ports, buses) = self.io_parts();
        let mut manifest_instances = Vec::new();
        for inst in instances {
            manifest_instances.push(ManifestInstance {
                core: InstanceCore {
                    name: inst.name.clone(),
                    cell: inst.cell.clone(),
                    at: inst.at,
                    rot_y: inst.rot_y,
                    port_modes: inst
                        .port_modes
                        .iter()
                        .map(|(k, v)| (k.clone(), PortModeDoc::of(v)))
                        .collect(),
                },
                contract: self.instance_contract(&inst.name)?.to_json()?,
            });
        }
        let doc = ManifestDoc {
            format: MANIFEST_FORMAT.to_string(),
            version: MANIFEST_VERSION,
            name: name.to_string(),
            instances: manifest_instances,
            ports: ports.iter().map(|(n, p)| (n.clone(), port_doc(p))).collect(),
            buses: buses.iter().map(|(n, b)| (n.clone(), bus_meta(b))).collect(),
            merged_contract: self.merged_contract().ok().and_then(|c| c.to_json().ok()),
        };
        serde_json::to_string(&doc).map_err(|e| e.to_string())
    }

    /// Export the design as a LAYERED `.litematic` (interchange tier): the
    /// flattened stack's named regions (`inst:{name}`, `bus:{name}`, loose
    /// base) plus the design manifest as a root-level `NucleationDesign`
    /// tag. The file opens in Litematica as a plain multi-region litematic
    /// and reimports as a design whose cell references have degraded to
    /// embedded copies.
    pub fn to_litematic_layered_bytes(&self) -> Result<Vec<u8>, String> {
        let flat = self.flatten()?;
        let bytes =
            crate::formats::litematic::to_litematic(&flat).map_err(|e| e.to_string())?;
        let mut root = read_litematic_root(&bytes)?;
        root.insert(
            DESIGN_MANIFEST_TAG,
            quartz_nbt::NbtTag::String(self.manifest_json()?),
        );
        write_litematic_root(&root)
    }

    /// Import a layered `.litematic` produced by
    /// [`Design::to_litematic_layered_bytes`]. Errs loudly when the file
    /// carries no `NucleationDesign` manifest (a plain litematic is not a
    /// design document — open it with `from_litematic` instead).
    ///
    /// Degradations of the interchange tier (per DESIGN_SPEC §8): each
    /// instance's cell REFERENCE becomes an embedded single-instance cell
    /// holding the already-transformed region blocks under an identity
    /// transform (`rot_y = 0`, `at` = the copy's min corner), with the
    /// transformed instance contract attached; the loose layer's own
    /// region naming collapses into the default region; block entities and
    /// entities are not reconstructed. Ports, bus metadata, fragments,
    /// states (including `FAILED(reason)`) and runs survive, so rerouting
    /// works on the imported design.
    pub fn from_litematic_layered_bytes(data: &[u8]) -> Result<Design, String> {
        let root = read_litematic_root(data)?;
        let manifest = root
            .get::<_, &str>(DESIGN_MANIFEST_TAG)
            .map_err(|_| {
                format!(
                    "no `{DESIGN_MANIFEST_TAG}` manifest: a plain litematic is not a design \
                     document (open it with from_litematic)"
                )
            })?
            .to_string();
        let m: ManifestDoc =
            serde_json::from_str(&manifest).map_err(|e| format!("bad design manifest: {e}"))?;
        if m.format != MANIFEST_FORMAT {
            return Err(format!("unknown manifest format `{}`", m.format));
        }
        if m.version != MANIFEST_VERSION {
            return Err(format!(
                "unsupported manifest version {} (this build reads {MANIFEST_VERSION})",
                m.version
            ));
        }
        let flat =
            crate::formats::litematic::from_litematic(data).map_err(|e| e.to_string())?;

        // Loose base: every region that is not an instance/bus layer.
        let mut base = UniversalSchematic::new(m.name.clone());
        for region in all_regions(&flat) {
            if is_layer_region(&region.name) {
                continue;
            }
            for ((x, y, z), block) in region_blocks(region) {
                base.set_block_from_string(x, y, z, &block)
                    .map_err(|e| format!("base block at ({x},{y},{z}): {e}"))?;
            }
        }

        let region_by_name = |name: &str| all_regions(&flat).find(|r| r.name == name);

        // Instances degrade to embedded copies (see the doc comment).
        let mut cells = BTreeMap::new();
        let mut instances = Vec::new();
        for inst in m.instances {
            let mut body = UniversalSchematic::new(inst.core.name.clone());
            if let Some(region) = region_by_name(&format!("inst:{}", inst.core.name)) {
                for ((x, y, z), block) in region_blocks(region) {
                    body.set_block_from_string(x, y, z, &block)
                        .map_err(|e| format!("instance `{}` block: {e}", inst.core.name))?;
                }
            }
            let contract = CellContract::from_json(&inst.contract)
                .map_err(|e| format!("instance `{}` contract: {e}", inst.core.name))?;
            let at = body.get_bounding_box().min; // identity placement
            let cell_key = inst.core.name.clone();
            cells.insert(
                cell_key.clone(),
                CellDef {
                    schematic: body,
                    contract,
                },
            );
            instances.push(Instance {
                name: inst.core.name,
                cell: cell_key,
                at,
                rot_y: 0,
                port_modes: BTreeMap::new(),
            });
        }

        let ports: BTreeMap<String, DesignPort> = m
            .ports
            .into_iter()
            .map(|(n, p)| {
                let port = port_from(&n, p);
                (n, port)
            })
            .collect();

        let mut buses = BTreeMap::new();
        for (bname, meta) in m.buses {
            let mut fragment = BTreeMap::new();
            if let Some(region) = region_by_name(&format!("bus:{bname}")) {
                for (p, block) in region_blocks(region) {
                    fragment.insert(p, block);
                }
            }
            let bus = bus_from(&bname, meta, fragment);
            buses.insert(bname, bus);
        }

        Ok(Design::from_io_parts(
            m.name, base, cells, instances, ports, buses,
        ))
    }

    /// Save the `.nucm` project document to a file (fs convenience; the
    /// wasm-safe core is [`Design::to_nucm_bytes`]).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_nucm(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.to_nucm_bytes()?).map_err(|e| e.to_string())
    }

    /// Load a `.nucm` project document from a file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_nucm(path: &str) -> Result<Design, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        Design::from_nucm_bytes(&data)
    }

    /// Export the layered `.litematic` to a file (fs convenience; the
    /// wasm-safe core is [`Design::to_litematic_layered_bytes`]).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn export_litematic(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.to_litematic_layered_bytes()?).map_err(|e| e.to_string())
    }

    /// Import a layered `.litematic` from a file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn import_litematic(path: &str) -> Result<Design, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        Design::from_litematic_layered_bytes(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_contract::{IoLayoutBuilder, IoType, LayoutFunction};
    use crate::routing::engine::blocks as rblocks;

    const STONE: &str = "minecraft:stone";
    const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
    const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";

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

    fn lamp_bank(s: &mut UniversalSchematic, x: i32, z: i32) -> P3 {
        for i in 0..8 {
            let y = 2 + 2 * i;
            s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
            s.set_block_from_string(x, y, z, rblocks::DUST).unwrap();
        }
        (x, 2, z)
    }

    /// A tiny contract-carrying cell (one boolean input on a lever).
    fn tiny_cell() -> (UniversalSchematic, CellContract) {
        let mut s = UniversalSchematic::new("tiny".to_string());
        s.set_block_from_string(0, 0, 0, STONE).unwrap();
        s.set_block_from_string(0, 1, 0, LEVER).unwrap();
        let layout = IoLayoutBuilder::new()
            .add_input(
                "x".to_string(),
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(0, 1, 0)],
            )
            .unwrap()
            .build();
        (s, CellContract::new("tiny".to_string(), layout))
    }

    /// The crossing fixture from `src/design.rs` tests, plus two instances
    /// of one embedded cell and one FAILED bus.
    fn full_design() -> Design {
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

        let (cs, contract) = tiny_cell();
        d.add_cell_with_contract("tiny", cs, contract);
        d.place("u0", "tiny", (30, 0, 0), 0).unwrap();
        d.place("u1", "tiny", (34, 0, 0), 90).unwrap();

        let a = d
            .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
            .unwrap();
        assert_eq!(a, BusState::Routed);
        let mut style_b = BusStyle::default();
        style_b.bus_block = "minecraft:cyan_concrete".to_string();
        let b = d
            .route_bus("bus_b", "b_in", &["b_out"], vec![], style_b)
            .unwrap();
        assert_eq!(b, BusState::Routed);
        // A zero-length segment (gate on the driver anchor): FAILED(reason)
        // — a state to round-trip. (Doglegs route through implicit corners
        // now, so they no longer produce a FAILED fixture.)
        let f = d
            .route_bus(
                "diag",
                "a_in",
                &["b_out"],
                vec![Gate {
                    name: "g0".to_string(),
                    anchor: (1, 2, 8),
                    step: (0, 2, 0),
                }],
                BusStyle::default(),
            )
            .unwrap();
        assert!(matches!(f, BusState::Failed(_)));
        d
    }

    fn blocks_of(s: &UniversalSchematic) -> BTreeMap<P3, String> {
        s.iter_blocks()
            .map(|(p, b)| ((p.x, p.y, p.z), b.to_string()))
            .filter(|(_, b)| !b.contains("minecraft:air"))
            .collect()
    }

    fn assert_ports_eq(a: &DesignPort, b: &DesignPort) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.anchor, b.anchor);
        assert_eq!(a.step, b.step);
        assert_eq!(a.width, b.width);
        assert_eq!(a.ty, b.ty);
        assert_eq!(a.direction, b.direction);
        assert_eq!(a.bits.len(), b.bits.len());
        for (x, y) in a.bits.iter().zip(&b.bits) {
            assert_eq!(x.connectable, y.connectable);
            assert_eq!(x.lever, y.lever);
            assert_eq!(x.lamp, y.lamp);
        }
    }

    fn assert_bus_eq(a: &BusLayer, b: &BusLayer) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.driver, b.driver);
        assert_eq!(a.extra_drivers, b.extra_drivers);
        assert_eq!(a.merge_or, b.merge_or);
        assert_eq!(a.sinks, b.sinks);
        assert_eq!(a.style, b.style);
        assert_eq!(a.state, b.state);
        assert_eq!(a.fragment, b.fragment);
        assert_eq!(a.gate_cells, b.gate_cells);
        assert_eq!(a.rule, b.rule);
        assert_eq!(a.segments.len(), b.segments.len());
        for (x, y) in a.segments.iter().zip(&b.segments) {
            assert_eq!(x.kind, y.kind);
            assert_eq!((x.a, x.b), (y.a, y.b));
            assert_eq!(x.cells, y.cells);
            assert_eq!(x.runs.len(), y.runs.len());
        }
        assert_eq!(a.gates.len(), b.gates.len());
        for (x, y) in a.gates.iter().zip(&b.gates) {
            assert_eq!((&x.name, x.anchor, x.step), (&y.name, y.anchor, y.step));
        }
        assert_eq!(a.runs.len(), b.runs.len());
        for (x, y) in a.runs.iter().zip(&b.runs) {
            assert_eq!(
                (x.along_x, x.fixed, x.y0, x.from, x.to, x.width),
                (y.along_x, y.fixed, y.y0, y.from, y.to, y.width)
            );
        }
    }

    /// Deep model equality across a save/load cycle.
    fn assert_design_eq(a: &Design, b: &Design) {
        let (an, ab, ac, ai, ap, abus) = a.io_parts();
        let (bn, bb, bc, bi, bp, bbus) = b.io_parts();
        assert_eq!(an, bn);
        assert_eq!(blocks_of(ab), blocks_of(bb), "base layers differ");
        assert_eq!(
            ac.keys().collect::<Vec<_>>(),
            bc.keys().collect::<Vec<_>>()
        );
        for (name, cell) in ac {
            let other = &bc[name];
            assert_eq!(
                blocks_of(&cell.schematic),
                blocks_of(&other.schematic),
                "cell `{name}` bodies differ"
            );
            assert_eq!(cell.contract, other.contract, "cell `{name}` contracts differ");
        }
        assert_eq!(ai.len(), bi.len());
        for (x, y) in ai.iter().zip(bi.iter()) {
            assert_eq!(
                (&x.name, &x.cell, x.at, x.rot_y),
                (&y.name, &y.cell, y.at, y.rot_y)
            );
        }
        assert_eq!(ap.keys().collect::<Vec<_>>(), bp.keys().collect::<Vec<_>>());
        for (name, port) in ap {
            assert_ports_eq(port, &bp[name]);
        }
        assert_eq!(
            abus.keys().collect::<Vec<_>>(),
            bbus.keys().collect::<Vec<_>>()
        );
        for (name, bus) in abus {
            assert_bus_eq(bus, &bbus[name]);
        }
    }

    #[test]
    fn nucm_round_trips_the_full_document_including_a_failed_bus() {
        let d = full_design();
        let bytes = d.to_nucm_bytes().unwrap();
        let back = Design::from_nucm_bytes(&bytes).unwrap();
        assert_design_eq(&d, &back);
        // The FAILED state carried its reason.
        match back.bus_state("diag").unwrap() {
            BusState::Failed(reason) => assert!(!reason.is_empty()),
            other => panic!("expected Failed, got {other:?}"),
        }
        // The reloaded document is live: flatten + check still work…
        let check = back.check().unwrap();
        assert!(check.clean, "{}", check.json);
        // …and rerouting works after reload (rip + route again).
        let mut back = back;
        back.rip("bus_b").unwrap();
        let mut style_b = BusStyle::default();
        style_b.bus_block = "minecraft:cyan_concrete".to_string();
        // The ripped name still exists as a layer; reroute under a new name
        // exercises fresh planning against the reloaded occupancy.
        assert_eq!(back.bus_state("bus_b"), Some(&BusState::Intended));
        let rerouted = back
            .route_bus("bus_b2", "b_in", &["b_out"], vec![], style_b)
            .unwrap();
        assert_eq!(rerouted, BusState::Routed, "{:?}", back.bus_state("bus_b2"));
    }

    #[test]
    fn nucm_dedupes_identical_cell_bodies_by_content_hash() {
        let mut d = Design::new("dedup");
        let (c1, k1) = tiny_cell();
        let (c2, k2) = tiny_cell();
        d.add_cell_with_contract("first", c1, k1);
        d.add_cell_with_contract("second", c2, k2);
        let bytes = d.to_nucm_bytes().unwrap();
        let doc: NucmDoc = bincode::deserialize(&bytes[8..]).unwrap();
        assert_eq!(doc.cells.len(), 2);
        assert_eq!(doc.blobs.len(), 1, "identical bodies must share one blob");
        assert_eq!(doc.cells["first"].blob, doc.cells["second"].blob);
        let back = Design::from_nucm_bytes(&bytes).unwrap();
        assert_design_eq(&d, &back);
    }

    #[test]
    fn nucm_rejects_bad_magic_and_future_versions() {
        assert!(Design::from_nucm_bytes(b"NOPE0000").is_err());
        let mut bytes = full_design().to_nucm_bytes().unwrap();
        bytes[4..8].copy_from_slice(&99u32.to_le_bytes());
        let err = Design::from_nucm_bytes(&bytes).unwrap_err();
        assert!(err.contains("version 99"), "{err}");
    }

    #[test]
    fn litematic_layered_export_round_trips_and_opens_plain() {
        let d = full_design();
        let bytes = d.to_litematic_layered_bytes().unwrap();

        // The very same file opens as a PLAIN multi-region litematic.
        let plain = crate::formats::litematic::from_litematic(&bytes).unwrap();
        let regions = plain.get_region_names();
        assert!(regions.iter().any(|r| r == "bus:bus_a"), "{regions:?}");
        assert!(regions.iter().any(|r| r == "bus:bus_b"), "{regions:?}");
        assert!(regions.iter().any(|r| r == "inst:u0"), "{regions:?}");
        assert!(regions.iter().any(|r| r == "inst:u1"), "{regions:?}");

        // And reimports as a design.
        let back = Design::from_litematic_layered_bytes(&bytes).unwrap();
        let (name, _, _, instances, ports, buses) = back.io_parts();
        assert_eq!(name, "crossing");
        assert_eq!(ports.len(), 4);
        assert_eq!(instances.len(), 2);
        assert_eq!(buses.len(), 3);
        // References degraded to embedded copies: identity transforms.
        for inst in instances {
            assert_eq!(inst.rot_y, 0, "degraded copy is identity-placed");
            assert_eq!(inst.name, inst.cell);
        }
        // The FAILED bus survived the trip with its reason.
        match back.bus_state("diag").unwrap() {
            BusState::Failed(reason) => assert!(!reason.is_empty()),
            other => panic!("expected Failed, got {other:?}"),
        }
        // Same flattened artifact block-for-block (the degradation is a
        // sharing-semantics change, not a geometry change).
        assert_eq!(
            blocks_of(&d.flatten().unwrap()),
            blocks_of(&back.flatten().unwrap())
        );
        // Ports kept their scanned hardware.
        for (pname, port) in ports {
            assert_ports_eq(port, d.port(pname).unwrap());
        }
        // The imported design is live: reroute works.
        let mut back = back;
        back.rip("bus_b").unwrap();
        let mut style_b = BusStyle::default();
        style_b.bus_block = "minecraft:cyan_concrete".to_string();
        let rerouted = back
            .route_bus("bus_b2", "b_in", &["b_out"], vec![], style_b)
            .unwrap();
        assert_eq!(rerouted, BusState::Routed, "{:?}", back.bus_state("bus_b2"));
    }

    #[test]
    fn a_plain_litematic_is_refused_loudly_on_design_import() {
        let mut s = UniversalSchematic::new("plain".to_string());
        s.set_block_from_string(0, 0, 0, STONE).unwrap();
        let bytes = crate::formats::litematic::to_litematic(&s).unwrap();
        let err = Design::from_litematic_layered_bytes(&bytes).unwrap_err();
        assert!(err.contains("NucleationDesign"), "{err}");
    }
}
