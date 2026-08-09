# nucleation-routing: crate design sketch

Evidence base: every API below maps to something the adder/ALU/multiplier/cell
sessions needed and built ad hoc in Python (`router.py`, `nets.py`, `audit.py`,
`timing.py`, `cells.py`). Every design rule listed was paid for by a real bug.

## Positioning

`crates/nucleation-routing`, sibling of `crates/mc-tick`. Depends on the core
schematic types; *optionally* on mc-tick (verification oracle) and the mchprs
compile graph (LVS). Nothing in core depends on it.

```
UniversalSchematic  <-- reads/writes -->  nucleation-routing
        |                                     |
   RedstoneGraph  (extracted netlist)  <--  LVS
        |                                     |
     mc-tick      (golden model)       <--  verify
```

## Core types

```rust
/// A net: one electrical signal, N terminals, a label.
pub struct Net { pub name: SmolStr, pub terminals: Vec<Terminal> }
pub enum Terminal { Cell(BlockPosition), AnyOf(Vec<BlockPosition>) }   // route-to-net

/// Occupancy + claims over a schematic region. All routing is transactional:
/// claim cells, commit or roll back (rip-up needs cheap undo).
pub struct Workspace<'a> { /* schematic view + per-net claims + label map */ }

/// A verified placeable template: blocks, ports, keepouts, delay.
pub struct CellTemplate {
    pub blocks: Vec<(BlockPosition, BlockState)>,
    pub ports: Vec<Port>,                // name, position, face, direction (in/out)
    pub keepout: Region,                 // no foreign routing inside
    pub edge_contract: EdgeContract,     // which boundary cells may carry nets
    pub delay_rt: HashMap<(PortId, PortId), u32>,
}
pub struct Placement { pub template: CellTemplateId, pub at: BlockPosition, pub rotate: Rotation }
```

## The router

```rust
pub struct Router { pub rules: RuleSet, pub moves: MoveSet, pub budget: SignalBudget }

impl Router {
    /// Single net, A* with design rules. Same as the Python prototype.
    pub fn route(&mut self, ws: &mut Workspace, net: &Net) -> Result<Route, RouteError>;

    /// All nets with PathFinder negotiated congestion: route greedily, then
    /// iterate rip-up-and-reroute with escalating history costs on contested
    /// cells. The Python prototype's ordering whack-a-mole (3 separate
    /// sessions) is the argument for this being the default entry point.
    pub fn route_all(&mut self, ws: &mut Workspace, nets: &[Net]) -> Result<Vec<Route>, RouteReport>;
}
```

**MoveSet is data, not code**: horizontal step, stair (max-chain constrained),
and *via templates* — the verified torch ladder, and later droppers/observers —
each with cost, occupied cells, entry contract ("needs straight dead-end dust
pointing at base"), delay, and signal-strength effect (ladder cap = fresh 15).
New vertical primitives should be a template registration, not a router fork.

**SignalBudget is first-class**: decay tracked per path cell, repeater
insertion is the router's job, stairs can't host repeaters, switchbacks
forbidden (support cuts own diagonal), refresh interval from the budget — all
as checked invariants of the emitted route, not conventions.

## The rule set (all verified in-sim this week)

| rule | bug that mandated it |
|---|---|
| electrical clearance (dust adjacency incl. cut diagonals) | every braid short |
| support may not cap diagonal-using dust | broken neighbours' stairs |
| stair corner cells must be clear | silent opens |
| stair chains <= 4, refresh <= 5 | 15-cell staircase decayed to 0 |
| no switchback stairs | route cut its own diagonal |
| torch/comparator base needs *pointing* dust | dead ladder climbs |
| footprint bounds / keepouts | cell routed through composer's space |
| edge contracts (port columns only on seams) | cross-cell seam shorts |
| **no directed cycles through repeaters on a net** | the cout/cin ring latch |

## Static analysis (the DRC/LVS story)

```rust
pub fn drc(ws: &Workspace, rules: &RuleSet) -> Vec<Violation>;      // shorts, support, decay, cycles

/// LVS: extract the conduction netlist from geometry and compare to intent.
/// The extractor ALREADY EXISTS: the mchprs compile graph (RedstoneGraph /
/// export_graph). LVS = labelled-graph comparison intended-vs-extracted;
/// the WL fingerprint work is reusable for fast mismatch triage.
pub fn lvs(schem: &UniversalSchematic, intent: &Netlist) -> LvsReport;

/// The single most valuable primitive: who powers this cell and why.
/// mc-tick knows; expose it. Every debugging session this week was
/// reconstructing this by hand, one probe at a time.
pub fn conduction_trace(sim: &TickSimulation, at: BlockPosition) -> PowerTree;
```

`lvs` + `conduction_trace` close the two blind spots simulation-plus-shorts
checking has: **opens** (dead route reads as quiescent) and **cycles** (the
repeater ring latch passed every check and held a phantom carry).

## Timing

```rust
pub fn sta(ws: &Workspace, netlist: &Netlist) -> TimingReport;   // arrivals, critical path, slack
```
Port of `timing.py` (validated 1.4x bound), upgraded to exact via the compile
graph. Later: slack-aware routing (order/detour by criticality).

## Verification harness

```rust
pub fn verify(schem, io: &IoContract, oracle: impl Fn(&Inputs) -> Outputs, cases) -> VerifyReport;
```
Lever driving (toggle-to-target, settle-per-flip), probe reads (batched — this
is the throughput ceiling today), truth-table/reference-model comparison, and
**bake**: write settled states back so saved files carry real wire geometry.

## Missing for state of the art (priority order)

1. **conduction_trace + LVS** — turns days of probe-debugging into one call
2. **Negotiated congestion** (PathFinder) — ordering provably doesn't scale
3. **Placement annealer** over `Placement`s — cost = wirelength+bbox+timing,
   feasibility = DRC; needed the moment netlists come from synthesis (Yosys)
4. **Global routing** — region/corridor assignment before detailed (the PLA
   compiler's channel allocation, generalized)
5. **Sequential support** — DFF cell template + clock nets + setup/hold in STA
6. **Steiner nets** — proper multi-terminal trees (today: incremental join)

## Non-goals

Logic synthesis (Yosys + genlib does it), the PLA compiler itself (a client),
cell *design* (hand-crafted, verified, then registered as templates).

## Unifying CellTemplate with TypedCircuitExecutor + Insign

Three existing representations of "a circuit with named IO" should become
layers over ONE contract, not a fourth representation:

```
IoContract           (exists as IoLayout: name -> IoType + ordered positions)
  ^ compiled from      Insign signs (in-world authoring, already implemented)
  ^ consumed by        TypedCircuitExecutor (typed set/read/execute)
  ^ embedded in        CellTemplate = Schematic + IoContract + PhysicalContract
                       PhysicalContract = port faces/directions, keepout,
                                          edge contract, per-port delays_rt,
                                          drive strength (output level under load)
```

Key moves:

1. **IoLayout is the shared core.** It already does typed words over ordered
   position lists -- exactly the word-bus port grouping the routing crate
   needs. Extend, don't duplicate: add optional `face`/`direction` per port
   and a physical sidecar, so the router and the executor read the same
   contract. A saved cell = schematic + contract; a cell library is a
   directory of them.

2. **Insign becomes the cell-authoring front door.** Hand-build a cell
   in-game, annotate ports with signs, import -> verified CellTemplate.
   Needs small DSL additions: direction (in/out), face, bit order, keepout
   regions, and a `#cell` header (name, version). This closes the loop the
   Python prototype lacked: community-built cells enter the same pipeline as
   compiler-generated ones, and `verify()` gates both identically.

3. **TypedCircuitExecutor grows a backend trait.**
   ```rust
   trait SimBackend {
       fn load(schem, opts) -> Self;          // settle mode, interning
       fn drive(&mut self, port, value);      // lever bank / signal injection
       fn settle(&mut self, budget) -> bool;
       fn read(&self, port) -> Value;
       fn bake_to(&self, schem);
   }
   ```
   `MchprsBackend` (today's, fast, custom-IO injection) and `McTickBackend`
   (vanilla-accurate; the verification oracle). Known semantic deltas the
   trait must own, all hit this week: mc-tick has no signal injection (drive
   via lever banks, toggle-to-target, settle per flip), `extra_states`
   interning, settle-mode selection, and mchprs's `is_lit` latching. The
   executor's typed word conversion is reused as-is -- it becomes the
   `verify()` harness for free, on either backend.

4. **Then the routing crate's `verify()` is not new code** -- it is
   TypedCircuitExecutor(McTickBackend) + a reference model, and cell
   characterization (delay/drive measurement for PhysicalContract) is a
   scripted executor run per port pair.

Migration note: TypedCircuitExecutor/CircuitBuilder currently sit behind the
`simulation` (mchprs) feature; the trait split lets `bridge-full` expose the
same typed API with whichever backends are compiled in.

## Bus as a first-class concept

A bus is not just N ports -- it is a geometry + ordering + timing contract:

```rust
pub struct BusSpec {
    pub width: u8,
    pub ty: IoType,               // reuses the executor's type system
    pub pitch: Pitch,             // axis + spacing between bits
    pub face: Face,               // which cell face it presents on
    pub encoding: Encoding,       // Binary1PerWire | HexAnalog (0-15/wire) | ...
}
pub struct BusPort { pub bus: BusSpecId, pub bit0: BlockPosition, pub dir: InOut }
```

What it buys, concretely:

1. **Abutment compatibility is checkable at placement time**: two BusPorts mate
   iff their BusSpecs match (width/pitch/face/encoding). Today that contract
   lives in my head and failed twice (seam shorts, PITCH mismatch).
2. **Bus routing**: route bit 0, replicate at pitch offsets -- with skew as a
   constraint the router must respect (equal repeater counts / matched lengths),
   and STA reporting per-bus skew, since a word is valid only when its slowest
   bit lands.
3. **Free arithmetic by geometry**: offsetting a bus tap by k pitches IS a
   shift-by-k -- zero blocks. The 4x4 multiplier's acc>>1 wiring did this
   implicitly; BusSpec makes it a named, checkable operation.
4. **Executor binding is 1:1**: TypedCircuitExecutor's word set/read maps to a
   BusPort directly -- drive a bus, read a bus, no per-bit bookkeeping. The
   Insign DSL annotates a whole bus with one sign (name, width, bit order).
5. **Bus utility cells become library entries**: ripper (bit extract), joiner,
   width adapter, pitch adapter, and encoder/decoder between Binary and
   HexAnalog -- the analog encoding (a hex digit per wire, comparator
   arithmetic) is redstone's native density advantage and deserves a slot in
   the spec from day one even if v1 only implements Binary.

## The two products

1. **verilog-to-redstone**: Yosys JSON -> genlib-mapped netlist -> placement ->
   route_all -> DRC/LVS/STA -> verify -> .schem. Blocked on: cell library
   trust, DFF cell (sequential), placement annealer.
2. **The compositor**: place existing circuits (imported via Insign or from
   the library), auto-connect matching BusPorts by abutment or routed bus,
   then analyse (DRC/LVS/STA/skew) and optimise (anneal placements, re-route).
   This is the same crate API -- the compositor is a UI over Workspace +
   CellTemplate + BusSpec + route_all + the analysis passes. mult4.py was a
   hand-run of exactly this loop.

## Abstract (non-voxel) representation for placement

Standard EDA split (LEF abstract vs DEF detail), and the right call here:

```rust
pub struct CellAbstract {
    pub hull: Aabb,                       // or a small set of boxes
    pub pins: Vec<(PortId, Point3)>,      // port positions only
    pub keepout: Vec<Aabb>,
    pub delay: DelayTable,                // from characterization
}
```

The annealer moves CellAbstracts and scores HPWL wirelength + hull overlap +
congestion estimate + STA-on-abstracts -- O(cells + nets) per move, never
touching voxels. Voxel reality enters only at detailed routing, and a
placement is accepted only if route_all + DRC succeed. Fragment already
carries bbox/ports/keepout; this is a formalization, not new machinery.

## Backend validity from palettes

`SimBackend::coverage(palette) -> Supported | Partial(unsupported: Vec<Block>)`.
Nucleation already extracts palettes cheaply. Auto-pick the fastest valid
backend; report exactly which blocks forced the accurate one. Also a lint:
"this design is mchprs-clean" is a useful property to preserve deliberately.

## Poly-backend / mixed-level simulation

Partitioning one world across two block-level engines in lockstep is the hard
version: mchprs is deliberately NOT tick-accurate to vanilla (that is its
speed), so block-level co-sim would chase phantom timing bugs at the seam.

The tractable version is **mixed-LEVEL, not just mixed-engine** -- the same
move real EDA made (RTL/gate-level cosim): a *verified* cell does not need
block simulation at all. Its characterization (function + port delays, both
measured against mc-tick at verification time) IS a behavioural model.

  - verified cells        -> behavioural: function + delay at ports
  - glue routing          -> conduction graph (cheap, static + event-driven)
  - unverified/piston-y   -> mc-tick islands, exchanged at port boundaries

Port contracts (BusPort + delay) are the exchange interface, which is why they
must carry timing from day one. mchprs then becomes one possible executor for
the behavioural tier rather than half of a fragile split-world.

## Sequential logic (prerequisite for both products)

DFF cell: design + verify a latch/register cell (the accidental repeater-ring
IS a latch; do it on purpose, gated). Clock as a BusSpec with its own
distribution rules (skew budget), setup/hold added to STA, executor gains
clocked test protocol (drive inputs, pulse clock, read after settle).

## Roadmap: priority order, grouped, with parallel lanes

Three independent lanes until Phase 2 forces them together.
[SIM] = needs simulation debugging skills; [CORE] = Rust core/engine work;
[CRATE] = new-crate work, Python prototypes are the spec.

### Phase 0 -- unblock (all three in parallel)
- A0 [SIM]  Comparator-side micro-probe (rep->side, both orientations, standalone
            AND embedded) then fix rca_cells 18/32. Nothing maps onto an
            untrusted cell library. (Suspects banked in memory.)
- B0 [CORE] conduction_trace ("who powers this cell and why") + batched probe
            reads + attach last_error_detail to exceptions. Small, huge leverage:
            B0 retroactively accelerates every other lane's debugging.
- C0 [CRATE] Scaffold nucleation-routing; port Workspace/rules/router/DRC/STA
            from the Python prototypes (they are the spec + test vectors).

### Phase 1 -- foundations (parallel pairs)
- B1 [CORE] LVS: intended-netlist vs RedstoneGraph extraction (+ WL-fingerprint
            triage). Depends loosely on B0.
- B2 [CORE] SimBackend trait: McTickBackend + MchprsBackend under
            TypedCircuitExecutor; owns settle/interning/injection deltas.
- D1 [CORE] Contracts: IoLayout + face/direction, BusSpec (+encoding enum),
            Insign DSL additions (#cell, bus, keepout). Pure design + bindings.
- C1 [CRATE] verify() harness on B2; bake_to; palette coverage() lint.

### Phase 2 -- the library and the optimizer (first hard join point)
- A2 [SIM]  Cell library v1 on trusted physics: NOT/NOR/AND/XOR/BUF +
            characterization (delay, drive) via C1 harness -> CellTemplates.
            Needs A0 + C1 + D1.
- C2 [CRATE] Placement annealer over CellAbstracts (HPWL + overlap + STA);
            accept only if route_all + DRC pass. Needs C0; benefits from A2.
- C3 [CRATE] Negotiated congestion (PathFinder) in route_all. Needs C0 only --
            parallel with C2.

### Phase 3 -- the products' cores
- A3 [SIM]  DFF cell + clock BusSpec + setup/hold in STA + clocked test
            protocol. Needs A2, C1.
- E1 [BOTH] Compositor MVP: import (Insign) -> place -> auto-bus -> analyse ->
            optimise -> export. Needs A2, C2, C3, D1. First user-facing win.

### Phase 4 -- endgame (parallel again)
- E2  Yosys frontend: JSON ingest + genlib for the library -> full HDL flow.
      Needs A2/A3, C2, C3.
- E3  Mixed-level simulation (behavioural cells + mc-tick islands). Needs A2
      characterization + B2.
- E4  HexAnalog encoding + adapter cells; sparse/analog arithmetic revisit.

Critical path to "verilog in, verified .schem out":
A0 -> A2 -> A3 -> E2, with C0->C2/C3 and D1 feeding in. Everything on the B
lane shortens every debugging loop and can proceed independently right now.

## Integration map into the rest of Nucleation

| existing asset | role in this system |
|---|---|
| `DefinitionRegion` / `RegionBounds` | THE carrier for wiring constraints: named include/exclude zones, authorable in-world or via API |
| Insign | authoring front door for cells AND region constraints (sign a zone) |
| `RedstoneGraph` (mchprs compile graph) | LVS extraction + exact STA |
| mc-tick | golden model, conduction_trace, characterization |
| `TypedCircuitExecutor` / `CircuitBuilder` | typed drive/read = verify harness + compositor probes |
| Fingerprint / diff | cell dedup + recognition (find known cells in imported worlds), LVS mismatch triage, library versioning |
| `world_segment` (WOL) | compositor import path: extract circuits from worlds -> cells |
| store module | cell-library storage backend (fs/s3/...) |
| meshing / rendering / to_animated_glb | compositor visualization; colour-coded overlays already proven |
| Autostack / voxel-period work | regular stamping of pitched cell arrays |
| Diplomat bridge + apps/ pattern | compositor UI = wasm web app over the same bindings |

## Wiring-area constraints (region model)

DEF-style region constraints, layered on DefinitionRegion:

```rust
pub struct RoutingRegion { pub include: Vec<Aabb>, pub exclude: Vec<Aabb> }
pub struct NetClassRule {
    pub region: Option<RoutingRegionId>,   // confine nets of this class here
    pub y_band: Option<(i32, i32)>,        // layer assignment
    pub direction_bias: Option<Axis>,      // corridor discipline
    pub spacing: u8,                       // extra clearance (e.g. clock)
    pub max_len_rt: Option<u32>,           // delay budget
}
```

Router legality gains one check (`region.contains(cell)` per net class);
authoring is `DefinitionRegion` + an Insign sign ("#route_zone bus_north
include"). Cell keepouts, the FA edge contract, and my ad-hoc router `bounds`
all become special cases of the same mechanism.

## Advanced bussing techniques (in adoption order)

1. **Bus channels / highways**: pre-reserved corridors (a RoutingRegion) with
   repeater stations at fixed intervals; global routing assigns buses to
   channels, detailed routing only does BREAKOUT (port -> channel escape).
   This is the PLA compiler's rail discipline, generalized -- and it is what
   made 32-bit feasible there.
2. **Skew-matched buses**: equal repeater counts per bit enforced by
   construction in channels; STA verifies.
3. **Vertical bus risers**: ladder banks at pitch (verified: 2-block spacing,
   no crosstalk) as a via template for whole buses.
4. **Bus pivots / pitch adapters / rippers / joiners**: library cells.
5. **Crossbars & muxed buses**: bus-select cells built from the gate library;
   compositor instantiates on demand.
6. **Serialized buses (needs DFF + clock)**: parallel->serial->parallel cells
   to cross congested cuts on 1-2 wires; classic when channels run out.
7. **HexAnalog trunks**: 4 bits/wire via signal strength + comparator
   arithmetic; encoder/decoder cells at trunk ends. Highest density, latest.

## The DFF cell (sequential design brief)

**Primary design: the repeater-lock latch.** A repeater whose SIDE is driven
by another repeater/comparator becomes `locked=true` and freezes its output --
a transparent D latch in a ~2x1 footprint, flat, no torches. Two in series
with opposite clock phases = master-slave edge-triggered DFF. This is the
standard modern redstone register, it is tiny, and mc-tick models `locked`
(the settle-mode notes call out locked-flag re-derivation explicitly).
The accidental cout/cin repeater ring was the unclocked version of exactly
this storage mechanism.

Candidates, in order:
1. repeater-lock master-slave DFF (primary: flat, dense, fast)
2. RS-NOR torch latch (fallback: taller, well understood)
3. **comparator self-loop register** -- stores an ANALOG value: a 4-bit
   register in one cell. The HexAnalog datapath's natural partner (v2).

Cell contract: ports D, CLK, Q (+ optional nQ). **Initial state is carried by
the schematic itself, not by a reset**: cells are BAKED at a chosen state and
deployed under InWorld settle (trust the saved block states -- locked flags,
comparator outputs, wire power). Every artifact this project bakes already
reloads InWorld quiescent in 0 ticks; a register template baked with Q=0 wakes
holding 0, exactly like an FPGA bitstream carries initial register values.
Consequences:
- RST becomes a per-design OPTION (runtime re-init), not a correctness need.
- The flow must preserve states end to end: stamping copies exact state
  strings (it does), routing must not disturb stored cells, final load is
  InWorld. Verify checks "wakes quiescent with declared state" per cell.
- Placement-mode robustness ("paste-safe") is a separate, recorded property:
  the placement pass re-derives locked flags and pulses observers, so a cell
  either proves its state survives that (verify under Placement too) or is
  marked InWorld-only. Both results live in the CellTemplate.

**Verification changes shape** (this is the real work):
- The harness must step `run(n_ticks)`, NOT `run_until_quiescent` -- a clocked
  system with an oscillator never goes quiescent; today's Levers/verify
  discipline is combinational-only.
- Clocked protocol: reset, drive D, wait, pulse CLK (measured width), read Q,
  then flip D and confirm Q holds.
- **Empirical setup/hold**: mc-tick is deterministic, so sweep the D-to-edge
  offset tick by tick and read off the exact setup window; same for minimum
  pulse width and clk->Q. These measured numbers go into the CellTemplate
  DelayTable, and STA gains setup/hold checks:
  data_arrival + setup <= clock_arrival at every DFF, per clock skew.
- Two-tick alignment: repeater delays quantize to redstone ticks; clock
  edges land on that grid -- characterization must state phases explicitly.

Clock distribution: a `BusSpec` with a skew budget; spine-and-ribs with
matched repeater counts per rib, skew verified by STA, gated regions later.

Milestone ladder: latch cell (hold verified) -> DFF (edge behaviour + setup/
hold characterized) -> 4-bit register w/ reset (bus-ported) -> counter
(self-feedback closes the sequential loop) -> then E2's Yosys flow may emit
flops.

## Naming, crate split, and backport plan

**Umbrella feature / branch**: `feat/redstone-eda` (matches repo convention:
feat/mc-tick, feat/wol-ingestion). The name covers both products; "routing"
undersells it, "eda" is exactly what it is.

**Two new crates, not one** -- generic algorithms must not know about redstone:

```
crates/pnr-core            # fabric-agnostic P&R algorithms, zero MC deps
  grid A* / negotiated congestion (PathFinder history costs)
  simulated annealing engine (moves/cost/feasibility as traits)
  interval colouring, Steiner heuristics, union-find net checking
  generic STA over a delay graph
  trait Fabric { moves(), legal(), cost(), budget() }   // the seam

crates/nucleation-routing  # the redstone Fabric + everything MC-specific
  design rules (the verified table), via templates (torch ladder...),
  SignalBudget, CellTemplate/CellAbstract, BusSpec, RoutingRegion,
  Workspace over UniversalSchematic, DRC, LVS glue, verify harness
```

`pnr-core` gets pure unit tests (synthetic grids) and could even be published
independently; `nucleation-routing` carries the sim-verified integration tests
ported from the Python prototypes.

**Backports into existing code** (separate small branches -- the bridge is a
high-conflict area and another agent is mid-feature there):

| branch | touches | contents |
|---|---|---|
| `feat/sim-backend-trait` | src/simulation, bridge | SimBackend trait; McTickBackend for TypedCircuitExecutor/CircuitBuilder; settle/interning/injection policy per backend |
| `feat/mc-tick-introspection` | crates/mc-tick, bridge | conduction_trace, batched probe reads, error detail on exceptions, bake_to |
| `feat/io-contracts` | core, insign, bridge | IoLayout face/direction + physical sidecar, BusSpec, Insign DSL additions (#cell, bus, route_zone), DefinitionRegion constraint attachment |
| `feat/backend-coverage` | core | palette -> backend coverage() lint |

Order: introspection first (accelerates everything), io-contracts second
(interfaces others build against), sim-backend-trait third (waits for the
in-flight bridge feature to land), coverage anytime.
