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
Verified form templates ready to port (cell listings in `pivot_tiles.md`):
vertical↔flat adapters `pivot_v2h`/`pivot_h2v` and the flat 90° corner
`pivot_flat90` (`pivot_tiles.py`, 96/96 each); an 8-bit word as TWO analog
wires — the HexAnalog bus form — is verified 256/256 exhaustive end to end
(`compositor/hexanalog_bus8.py`, comparator-sandwich trunk stations).

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

## 11. API surface — generated wire core + idiomatic veneer

The user-facing API is TWO layers, and only the top one is typed by hand:

1. **Wire core (generated).** The Diplomat bridge (`src/bridge/design.rs`)
   speaks the wire format: positional coordinate splats and JSON-string
   arguments/returns (`route_bus(name, driver, sinks_json, gates_json,
   style_json) -> state`). It is regenerated per binding and is NEVER the
   surface users type. In the Python wheel it ships as the
   `nucleation.nucleation` submodule (re-exported as `nucleation.core`).
2. **Idiomatic veneer (hand-written, thin).** One small module per
   language marshals native shapes into exactly those wire calls — zero
   design/routing logic on the veneer side. Python:
   `bindings/python/nucleation/design.py` (the package `__init__` star-
   re-exports the core, then overlays the veneer), the Python sibling of
   the C++ `bindings/python/custom/` extension point.

The veneer surface (acceptance-tested by `compositor/design_demo2/3/4`):

```python
d = n.Design.for_schematic("crossing", s)
d.declare_input("a_in", anchor=(1, 2, 8), step=(0, 2, 0), width=8, ty="uint")
bus_a = d.route_bus("bus_a", driver="a_in", sinks=["a_out"],
                    gates=[n.Gate(anchor=(8, 2, 8), step=(0, 2, 0))],
                    style=n.Style(bus_block="minecraft:lime_concrete"))
bus_a.state                      # live "intended" / "routed" / "failed: …"
bus_a.move_gate(0, (8, 2, 12))   # by index or name -> reroute report dict
bus_a.skew; bus_a.rule(max_len_rt=100); bus_a.rip()
report = d.check()               # CheckReport: .clean/.drc/.lvs/.rules
d.check(strict=True)             # raises DesignCheckError when dirty
baked = d.bake(4000)             # Flat: core Schematic + .save()/.executor()
ex = baked.executor()            # ex["a_in"] = 0x55; ex.settle(); ex["a_out"]
d.move_instance("c0", at=(4, 0, 8))            # -> co-reroute report dict
d.save("x.nucm"); d.save("x.litematic")        # tier dispatched by suffix
```

Dataclasses `Gate`/`Style` (plain dicts accepted), `Bus`/`CheckReport`/
`Flat`/`Executor` handles; explicit wire methods stay reachable via
`d.raw` / `nucleation.core`. **The JS veneer must mirror this module
1:1** (same names, same shapes, `Object` literals where Python takes
dataclasses) over the same generated wasm core.

## 12. Sequential cells — `.latch` compiles to a DFF bank (landed)

The HDL compiler (`crates/nucleation-hdl`) accepts rising-edge `.latch`
lines (single clock domain). Chosen topology, end to end:

* **State = stage-0 rails.** Each latch Q is an extra stage-0 input slice
  whose rail has no lever ("ext"): the fabric reads state exactly like a
  primary input, dual-rail included.
* **Bank = the last stage.** D nets are buffer-raised to the top level, so
  every D delivery is an ordinary next-stage route onto a rail in the bank
  band; the DFF taps it with a y3 flyover spur (supports double as caps
  over crossed rails). One DFF per latch, one slice each, east of every
  combinational slice.
* **Cell = the verified master-slave repeater-lock DFF** (13x4x7, from
  `seq_cells.py`, frozen as data in `nucleation-hdl/src/seq.rs`). Timing
  (characterized, exact): setup 0 / hold 3 / min pulse 3 / clk->Q 10 /
  min period 20 gt.
* **Clock = a dedicated spine**, not a data rail: y1 dust row north of the
  DFF row, one lever, repeaters at least every 8 cells (2 gt skew each),
  one branch per DFF by adjacency; the cell's own clk column refreshes
  once more inside. Clock-into-logic is a compile error.
* **Feedback = planar wrap corridors.** Q leaves east, runs south, wraps
  west of x<0, north past stage 0 and enters its rail's west-end dust from
  the north (the rail is slot 0 = the band's northernmost row). Rows and
  columns pitch 2 apart, depths ordered by slice — no two corridors ever
  cross (the counter4 corridor argument, generalised and mechanised).
* **Init-by-construction.** The slave repeater and its lock path are
  authored at the declared init (Q=1 bakes the powered dust ladder down
  the Q path). At rest the slave is locked, which cuts every sequential
  loop: the deployed build settles to the declared state and is quiescent
  until the first edge. The schematic IS the reset state.
* **Contract**: the clock is a real Boolean input port at the spine lever,
  plus a `sequential` sidecar (clock_port/clock=true, DFF table,
  `est_min_period_gt`, `initial_state`) that the shared `CellContract`
  parser ignores by design.
* **Verification is fixed-tick only** after the placement settle
  (`verify_clocked`, mirrored in `hdl/hdl2redstone.py --rust`): margins are
  MEASURED by polling D ports / Q rails against the stepped model, never
  assumed. Gate results: counter4 24 steps (period 140 gt), fsm 30 steps
  (130 gt), toggle1 boots at baked Q=1 (72 gt), uart_tx one full 8N1 frame
  bit-by-bit (562 gt).

## Phase 2 (in scope of the model; ✅ = landed on this branch)

- ✅ interference co-reroute + drag APIs (`move_gate`, `move_instance`) —
  `src/design.rs`: `OccupancyIndex` (footprints + keepout/bounds+1 halos +
  fragments), per-segment rip (`BusLayer::segments`, gate drag = exactly
  its 2 adjacent segments), deterministic bounded co-reroute rounds,
  implicit L corners for doglegs; acceptance `tests/design_drag.rs` +
  `tests/design_typed_drag.rs`, demo
  `compositor/design_demo3_drag.py`. Cancellation hooks remain open;
- form-pivot adapter tiles (vertical stack ↔ flat lane at a gate) —
  hardware DONE and sim-verified (`pivot_tiles.py`/`pivot_tiles.md`,
  incl. the flat 90° corner); remaining work is the router-side stamping;
- ✅ multi-driver wired-OR (`merge="or"`) — `route_bus_or`: dust-merge
  branches, diode-isolated, ONE LVS intent net; multi-sink fanout trunks
  landed with it (`route_bus` with N sinks = shared trunk + branches);
- ✅ loose-layer editing APIs on the design document — `Design::set_block`
  (tracked as the loose layer; participates in occupancy + flatten);
- design-document serialization: `.nucm` project tier and `.litematic`
  layered-interchange tier (section 8);
- HexAnalog drive/read in the executor (contract already models it);
- ✅ STA/skew as a first-class `check()` stage — per-bus `per_bit_rt` /
  `skew_rt` from fragment repeaters, design-level arrival + critical path
  over the existing sta machinery, and `NetClassRule` enforcement
  (`max_len_rt`, `y_band`) folded into `clean`.

## Port modes: executor hardware ⇄ routable dust (promotion)

A community cell's contract names **executor hardware** — inputs are levers or
buttons, outputs lamps. Nothing in redstone drives a lever, so
`ADD007.sum -> BINTOBCD001.bin` was impossible however good the router became.
That is not a routing gap; it is the composability blocker, and it is why the
studio could only ever bus `ADD007.sum` (the one port in the whole enhanced
library with dust beside its lamps) into a loose lamp readout.

Every instance port therefore has a **mode**, and the design remembers both
forms:

| mode | hardware | drivable by `CellExecutor` | routable |
| --- | --- | --- | --- |
| `Executor` (default) | as shipped | ✅ | ❌ |
| `Bus` (promoted) | driver stub ending in dust | ❌ | ✅ |

```rust
d.set_port_mode("u1", "bin", PortMode::Bus)?;   // or d.promote_input("u1","bin")
d.set_port_mode("u1", "bin", PortMode::Executor)?;  // byte-exact undo
```

- The switch is a **reversible per-instance patch** (`design_promote::PortPatch`:
  `writes` + `saved` + the new connection cells), *not* an edit to the shared
  cell body. Toggling back restores the original block states exactly, which
  `tests/design_promotion.rs` asserts by byte-comparing the flattened layers.
- Modes are persisted in `.nucm` (`InstanceCore::port_modes`, format v2). The
  payload is bincode, so the field is unconditional and the version gate rejects
  v1 documents rather than misreading them.
- Toggling a port that carries a bus **rips** that bus (its endpoint physically
  stops existing) and names it in `PortModeReport::removed_buses`; buses that
  merely crossed the changed space are co-rerouted, exactly as for a drag. This
  follows the FAILED-state philosophy: the document's truth wins, buses fail or
  heal visibly.
- `PortModeReport::to_json` carries a per-cell before/after list in WORLD
  coordinates, so a UI can say *"removed lever at (19,5,5); bin[0] now lands on
  dust at (0,5,-20)"*.

### Why the strategy depends on the lever's face

A lever **strongly** powers its attachment block; everything downstream reads
that block. Dust only ever powers a block **weakly**, and weak power does not
reach dust. So:

- `face=floor` — the attachment block is directly below, and dust in the
  lever's own cell sits on it and powers it from above. Verified:
  `BINTOBCD001.bin`, 8/8 vectors identical to lever drive.
- `face=wall` — a **repeater** in the lever's cell pointing into the attachment
  block reproduces the lever's strong power exactly; the connection dust goes
  one cell further out. Verified: `ADD007.a` 8/8 and `NUMDISPLAY001.bcd` 10/10.
  Plain dust here is *not* enough — `ADD007.a` feeds bare dust and reads 0
  forever, which is precisely the weak-power rule.
- `face=ceiling` — refused with that reason: nothing can sit above a block to
  power it.

Outputs are easier: a lamp driven by a repeater is already strongly powered, so
dust **on top of the lamp** taps the value without touching the lamp — the port
stays executor-READABLE *and* becomes routable.

### The form pivot

Promotion is only half the job. A bus realizes the verified vertical 2y-pitch
stack, and community IO is often a horizontal ROW (`BINTOBCD001`'s `bin` levers
march along x at pitch 2). Such a port is dust, routable in principle, and still
unusable: its step is `(2,0,0)`.

`design_promote::pivot_row_to_stack` grows a **form adapter**. Bit `i` leaves the
row in its own private lane (lanes are 2 apart on the row axis, so no two bits
are ever plan-adjacent), climbs `2i` blocks on a dust staircase, runs out to a
depth every lane shares, then gathers back along the row axis so all bits land
in one vertical 2y-pitch column — a textbook bus stack. Refresh repeaters go in
every 6 dust cells; dust cannot climb out of a repeater, so the staircase pauses
on a flat landing, repeats, and resumes. The shared depth must equal the deepest
lane's actual end (`refresh_pauses`), and `lay_pivot` asserts it: off by one and
only the TOP bit of the port goes dead.

**Acceptance (`tests/design_promotion.rs`)** — the canonical demo, end to end in
the tick engine: `ADD007.sum -> BINTOBCD001.bin -> NUMDISPLAY001.bcd`, both
buses `Routed`, driven through the adder's own levers. 8/8 BCD values exact
(0, 2, 42, 127, 16, 137, 255, 255) and 8/8 seven-segment patterns exact against
the verified `REPORT.md` reference.
