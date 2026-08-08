# Redstone EDA — Composition Model (DESIGN_SPEC)

Status: co-designed model, authoritative. Phase 1 is IMPLEMENTED on
`feat/redstone-eda` (`src/design.rs`, bridge `Design` + `CellExecutor`
opaques, acceptance in `tests/design_bus_cross.rs` and
`redstone-eda/compositor/design_demo2.py`); Phase 2 items are listed at
the end. Every concept below
is grounded in a type or file that exists on this branch (grounding table,
section 9).

The one-sentence model: **a CELL is a schematic that carries its own typed
contract; a DESIGN is a Photoshop-style layer stack of cell instances, loose
blocks, and owned bus fragments; flatten() collapses the stack into one
self-describing schematic that is itself a cell** — hierarchy by recursion,
not by a separate netlist format.

---

## 1. CELL = UniversalSchematic + embedded CellContract

A cell is *one artifact*, not a schematic-plus-sidecar-file. The contract
(`CellContract`: name, version, `IoLayout` of typed ports/buses,
`PhysicalContract` sidecar — `src/io_contract/physical.rs`) is embedded in
the schematic's metadata and **autodetected on open**.

Contract sources merge with strict precedence, and conflicts are loud:

1. **explicit API** — `Schematic.set_cell_contract(json)` wins always;
2. **embedded metadata** — the `NucleationCellContract` JSON string carried
   in the `.schem` `Metadata` compound (beside the existing
   `NucleationDefinitions`, `src/formats/schematic.rs`), surfaced as the
   transient `Metadata.cell_contract` field (same file-carried-provenance
   pattern as `Metadata.embedded_test`, `src/metadata.rs`);
3. **Insign signs** — derived from in-world signs via the existing
   `io_contract::insign_ext` machinery (`contracts_json`,
   `parse_cell_header`; already bridged as
   `Schematic.compile_io_contracts_json`, `src/bridge/schematic.rs`).

`UniversalSchematic::resolve_cell_contract()` performs the merge and returns
`(contract, warnings)`; a sign-derived contract that disagrees with the
embedded one produces a warning naming both sides — never a silent pick.

The embedded contract round-trips through `.schem` save/open and drives the
typed executor directly: `BackendCircuitExecutor::for_cell(schem, &contract,
extra_states)` (`src/simulation/typed_executor/backend.rs`) — **cells must be
baked** (settled states saved; `InitialState::Baked`), because the mc-tick
backend trusts saved block states and an unbaked build sits inert until the
first lever flip.

## 2. DESIGN = composition document (Photoshop layers)

`Design` (new, `src/design.rs`, `routing` feature, wasm-safe core) is a
document of layers over a shared coordinate space:

- **instance layers** `{name, cell REFERENCE, transform(at, rot)}` — two
  instances may reference one cell; the cell is stored once in the design's
  cell library. An instance exposes the **transformed contract**: port
  cells/faces/step vectors mapped through the transform (Y-rotation in 90°
  steps + translation, same convention as
  `UniversalSchematic::rotate_schematic_y`); types, delays and bit order are
  unchanged by transforms.
- an optional **loose block layer** — raw endpoint hardware placed with
  plain `set_block` (the `design_step1.py` workflow);
- **BUS layers** — see section 4. A bus layer OWNS its voxel fragment
  (schematic-shaped block set) plus metadata `{name, endpoints, gates,
  style, state}` where `state ∈ {intended, routed, FAILED(reason)}`;
- **zones** — the existing tagged `DefinitionRegion`s /
  `RoutingRegion` / `NetClassRule` machinery
  (`src/io_contract/routing.rs`) carried through unchanged.

Occupancy and atomic commit reuse the routing crate's transactional
`Workspace` (claim / commit / rollback,
`crates/nucleation-routing/src/workspace.rs`).

## 3. PORT = named typed geometry + derived capabilities

A design port is **geometry first**:

- anchor = the bit-0 **connection cell** (a dust cell the router may land
  on), step vector, width — this is exactly `BusSpec`/`BusPort`'s
  `bit0 + Pitch × width` shape (`src/io_contract/bus.rs`), generalized to an
  arbitrary step vector;
- `ty` = `IoType` (`src/io_contract/io_type.rs`); encoding
  `Binary1PerWire | HexAnalog` (`BusEncoding`); bit order = position order,
  the `IoLayout` ordering rule (`src/io_contract/io_layout_builder.rs`).

**Capabilities are derived by hardware scan, not declared**: for each wire
cell, adjacent lever ⇒ *drivable*, lamp support ⇒ *readable*, bare dust ⇒
*connectable*. They are validated loudly at declaration (declaring a port
whose cells aren't dust, or an input with no lever, is an immediate error
naming the offending bit). `declare_input` / `declare_output` are sugar that
assert the capability.

**Routing never requires termination hardware; the executor consumes
capabilities.** Concretely: the merged contract's `IoMapping.positions` are
executor-facing — lever cells for drivable inputs (mc-tick's `drive`
requires levers: `McTickBackend::drive`,
`src/simulation/typed_executor/backend.rs`), the connection dust for
readable/connectable outputs (`read` returns power at the cell). The
connection cells stay in the design port and drive routing.

## 4. BUS = roles + gates + segments + style, atomically realized

A bus is an endpoint list with **roles**: `driver | sink | bidi`.

- 1 driver + N sinks ⇒ shared trunk + branches.
- Multiple drivers are legal **only** with explicit `merge="or"` (wired-OR);
  otherwise rejected at declaration.
- `bidi` is modeled but **rejected clearly** (reserved; the error says so).

**Gates** are bus-shaped waypoints (anchor + step + optional form) that
split the bus into independently-routed **SEGMENTS**. Form (vertical stack,
flat lane, …) is inferred from the port geometry at each end of a segment;
Phase 1 implements the verified vertical 2y-pitch form (`bus8_*.py`).

**Style** is per-bus: `{bus_block, transparent_block}` (default glass).
The transparent block is used ONLY where a diagonal must survive — the
`materials.py` predicate model (`pick_support(need_insulator=…)`,
`notes-material-model.md`: the cut rule runs on conductivity; "transparent
only where a diagonal must survive").

**Crossings are implicit.** When two routed buses intersect, the crossing is
realized from template data ported from `bus8_cross.py` v2 (canonical
levels): the crossed bus dips 1 down, passes under through its own
block-sandwich station interleaved in the shared column's odd levels, and
climbs back — glass appears only at the two dip supports per bit (bits > 0),
per the predicates. Straight runs use the station pattern (`canon_bus_a`):
solid separators, one repeater per bit doubling as refresh. Anything the
templates don't cover falls back to the generic per-net router
(`route_all`, negotiated congestion via pnr-core,
`crates/nucleation-routing/src/router.rs`).

**Segment realization is ATOMIC**: each segment is built in a transaction
seeded with the design's full occupancy; it either commits into the bus's
owned fragment or the bus enters `FAILED(reason)` with the workspace
untouched. Never half-routed; **unroutability is a state, not an
exception**.

## 5. INTERFERENCE (Phase 2)

Spatial occupancy = instance footprints + influence halos
(`PhysicalContract.keepouts` / `EdgeContract` windows,
`src/io_contract/physical.rs`) + route fragments + electrical clearance
(the router's `electrical_conflicts` — adjacency one step up/down shorts
without sharing a cell). Moving anything computes the **affected bus set**:
buses whose fragments intersect the old or new footprint, plus buses whose
DRC would newly fail. The affected set is co-rerouted under negotiated
congestion (pnr-core PathFinder), deterministic, seeded and bounded;
failures become `FAILED` states on the buses, never exceptions.

## 6. flatten / check / bake — the artifact is a cell

- `flatten()` → ONE `UniversalSchematic` with **named regions per layer**
  (`inst:alu0`, `bus:data`, loose blocks in the default region) via the
  existing named-region APIs (`set_block_in_region`, `get_region_names`,
  `translate_region` — `src/universal_schematic.rs`), plus the **merged
  transformed contract** (design ports + surviving instance ports) embedded
  in metadata. The artifact is self-describing and itself placeable as a
  cell — hierarchy recursion.
- `check()` = DRC + LVS + STA/skew over the flattened artifact, reusing
  `drc_schematic` / `lvs_schematic` / `sta_schematic` (`src/routing.rs`,
  wrapping `crates/nucleation-routing/src/{drc,lvs,sta}.rs`). LVS intent
  nets come from the bus layers (driver wires + sink wires per bit).
- `bake()` (mc-tick feature) = settle in the vanilla-accurate engine + write
  settled states back (`BackendCircuitExecutor::{settle, bake_to}`), then
  stamp `InitialState::Baked` into the embedded contract.

## 7. Independent consumers

The typed executor and the router are **independent consumers of the same
contract**: `for_cell` binds names→positions for execution;
`route_all`/`Design` bind the same geometry for realization. Neither knows
the other exists.

Web-app requirements (recorded for Phase 2): dragging a gate reroutes
exactly the 2 adjacent segments; dragging a component co-reroutes the
affected bus set; both calls cancellable and bounded; identical behavior via
the wasm bindings (`bindings/js`, regenerated by `tools/gen-bindings.sh`).

## 8. Serialization — three tiers mapping the layer model

| tier | format | analogy | carries | grounding |
|---|---|---|---|---|
| project | `.nucm` (Nucleation-native snapshot container, magic `NUSN`, `src/formats/snapshot.rs`, `to_snapshot`/`from_snapshot`) | PSD | full fidelity: cell REFERENCES, instance transforms, bus fragment layers incl. `intended/routed/FAILED` states, gates, zones, contract sources — reopenable mid-edit, non-destructive; the natural schemat.io document format | Phase 2: serialize `Design` itself into the snapshot container |
| interchange | `.litematic` | layered TIFF | litematic is natively multi-region (`create_regions` over `get_all_regions`, `src/formats/litematic.rs`): each design layer exports as a named region (`inst:alu0`, `bus:data`) + a design manifest (instances/transforms/bus metadata/merged contract) in litematic metadata NBT (the `NucleationDefinitions`-in-`Metadata` and root-level `NucleationTest` patterns already exist in that file). Opens in Litematica in-game with layers visible as regions. Cell references degrade to embedded copies on export — correct *sharing* semantics, documented | Phase 2; thin: `flatten()` already produces exactly these named regions |
| artifact | `.schem` | PNG | flattened + baked + embedded contract only | **Phase 1** — this is the flatten/bake output |

## 9. Grounding table

| concept | existing type / file | what's new |
|---|---|---|
| CellContract / PhysicalContract / InitialState | `src/io_contract/physical.rs` | embedded in `.schem` metadata; autodetect + precedence merge |
| IoLayout / IoMapping / bit order | `src/io_contract/io_layout_builder.rs`, `io_mapping.rs` | merged transformed contract on flatten |
| BusSpec / BusPort / Pitch / BusEncoding | `src/io_contract/bus.rs` (typed), `crates/nucleation-routing/src/bus.rs` (router-side) | DesignPort: anchor + arbitrary step vector + scanned capabilities |
| metadata carrier | `src/metadata.rs` (`embedded_test` pattern), `src/formats/schematic.rs` (`NucleationDefinitions`) | `Metadata.cell_contract` + `NucleationCellContract` key, both `.schem` writers + reader |
| Insign-derived contracts | `src/io_contract/insign_ext.rs` (`contracts_json`, `parse_cell_header`), `src/bridge/schematic.rs::compile_io_contracts_json` | fallback source in `resolve_cell_contract`, conflict warnings |
| typed executor | `src/simulation/typed_executor/backend.rs` (`for_cell`, `McTickBackend` lever-drive / power-read, `bake_to`) | consumes the embedded contract; capability-facing positions |
| workspace / transactions | `crates/nucleation-routing/src/workspace.rs` | seeds per-segment atomic realization |
| router / route_all / negotiation | `crates/nucleation-routing/src/router.rs`, `fabric.rs`, pnr-core | generic fallback; Phase 2 co-reroute |
| DRC / LVS / STA | `crates/nucleation-routing/src/{drc,lvs,sta}.rs` via `src/routing.rs` glue | `Design::check()` aggregation |
| zones / net classes | `src/io_contract/routing.rs` (`RoutingRegion`, `NetClassRule`), `DefinitionRegion` | carried as design zones |
| named regions | `src/universal_schematic.rs` (`set_block_in_region`, `get_region_names`, `translate_region`, `rotate_region_y`) | one region per layer on flatten |
| crossing tile | `redstone-eda/bus8_cross.py` v2 (`canon_bus_a`/`canon_bus_b`), `materials.py` predicates, `notes-material-model.md` | ported to Rust template data (`src/design.rs`) |
| bus form (vertical 2y) | `redstone-eda/bus8_*.py`, `compositor/design_step1.py` endpoint hardware | `route_bus` station emission |
| bridge | `src/bridge/{routing,hdl}.rs` (opaque + JSON results, PORTING rule 9), regen per commit `ee0142c3` | `Design` opaque (`src/bridge/design.rs`) |
| project container | `src/formats/snapshot.rs` (`NUSN`) | Phase 2 `.nucm` design document |
| layered interchange | `src/formats/litematic.rs` multi-region + metadata NBT | Phase 2 export mapping |

## 10. Acceptance sketches (final API)

### (1) bus_example — two crossing 8-bit buses, typed walking-ones

```python
import nucleation as n

s = n.Schematic.create("crossing")

# endpoint hardware with RAW set_block (loose layer): 8-lever banks with a
# connection dust per bit, 8-lamp banks whose lamp supports its own dust
# (design_step1.py geometry) — bus A runs +X at z=8, bus B runs +Z at x=8,
# so the two buses MUST cross.
place_lever_bank(s, x=0,  z=8,  dust_toward=+1)   # drives bus A
place_lamp_bank (s, x=16, z=8)                    # reads  bus A
place_lever_bank(s, x=8,  z=0,  dust_toward=+1)   # drives bus B
place_lamp_bank (s, x=8,  z=16)                   # reads  bus B

d = n.Design.for_schematic("crossing", s)

# ports: anchor = bit-0 connection dust, step, width, type.
# capabilities are scanned (lever ⇒ drivable, lamp ⇒ readable) and validated.
d.declare_input ("a_in",  1, 2, 8,  0, 2, 0,  8, "uint")
d.declare_output("a_out", 16, 2, 8, 0, 2, 0,  8, "uint")
d.declare_input ("b_in",  8, 2, 1,  0, 2, 0,  8, "uint")
d.declare_output("b_out", 8, 2, 16, 0, 2, 0,  8, "uint")

# two buses; the crossing is IMPLICIT (dip-under tile, per-bus styles)
d.route_bus("bus_a", "a_in", ["a_out"], style_json='{"bus_block":"minecraft:lime_concrete"}')
d.route_bus("bus_b", "b_in", ["b_out"], style_json='{"bus_block":"minecraft:cyan_concrete","transparent_block":"minecraft:cyan_stained_glass"}')
assert d.bus_state("bus_a") == "routed" and d.bus_state("bus_b") == "routed"

flat = d.flatten()          # named regions bus:bus_a / bus:bus_b + merged contract
report = d.check()          # DRC + LVS, clean
d.bake(4000)                # settle + bake_to (mc-tick)
flat = d.flatten()

# typed walking-ones through the EMBEDDED contract — no coordinates
cell = n.CellExecutor.for_schematic(flat)       # contract autodetected
for i in range(8):
    cell.set_input("a_in", 1 << i); cell.set_input("b_in", 0)
    cell.settle(400)
    assert cell.read_output("a_out") == 1 << i
    assert cell.read_output("b_out") == 0        # isolation, zero crosstalk

flat.save_to_file("showcase/bus_cross8_design.schem")   # contract embedded
n.Renderer.render_to_file_with_pack(flat, pack, config, "build.png")
```

### (2) gate drag → 2-segment reroute (Phase 2 API, recorded now)

```python
g = d.add_gate("bus_a", "g0", anchor=(6, 2, 8), step=(0, 2, 0))
d.move_gate("bus_a", "g0", anchor=(6, 2, 11))
# ONLY the two segments adjacent to g0 are ripped and rerouted, atomically;
# an unroutable move leaves bus_a in FAILED("segment a_in->g0: ...") — the
# UI shows the red layer, nothing raises.
```

### (3) component C dragged through the A–B bus (Phase 2 API, recorded now)

```python
d.place("c0", cmp4_cell, at=(4, 0, 4))
r = d.move_instance("c0", at=(4, 0, 8))   # now overlaps bus_a's fragment
# affected set = buses intersecting old+new footprint + halos, plus any bus
# whose DRC newly fails; co-rerouted under negotiated congestion,
# deterministic/seeded/bounded. r.rerouted == ["bus_a"]; on failure bus_a
# is FAILED(reason) and c0 IS moved (the truth of the document), never a
# half-routed fragment.
```

## Phase 2 (deferred, in scope of the model)

- interference co-reroute + drag APIs (`move_gate`, `move_instance`,
  cancellable, wasm-identical);
- form-pivot adapter tiles (vertical stack ↔ flat lane at a gate);
- multi-driver wired-OR (`merge="or"`);
- loose-layer editing APIs on the design document;
- design-document serialization: `.nucm` project tier and `.litematic`
  layered-interchange tier (section 8);
- HexAnalog drive/read in the executor (contract already models it);
- STA/skew as a first-class `check()` stage with per-bus skew budgets.
