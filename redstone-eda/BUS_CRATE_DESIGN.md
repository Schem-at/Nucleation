# `nucleation-bus` — extracting bus planning into its own crate

Status: **design proposal, nothing implemented.** Every type and file named
below exists today unless marked *speculative*. Written to serve two consumers:

1. **schematic-time routing** — `Design::route_bus` in the studio (today's only
   consumer), and
2. **a Minecraft server plugin** doing live autobussing in a world full of
   other people's redstone (does not exist; shapes the API anyway).

The second consumer is the reason this is a crate and not a module. A plugin
cannot depend on `nucleation` (188k lines, 159 deps, `UniversalSchematic`,
serde, NBT); it needs a small, deterministic, allocation-conscious planner it
can call inside a tick budget. That is a hard architectural constraint, and it
is the same constraint that already produced `pnr-core` and `mc-tick`.

> **Prerequisite.** The integration work currently in flight on `src/design.rs`,
> `src/design_corridor.rs` and `crates/nucleation-routing/**` must land and be
> green **before step 1 of the migration plan starts.** `design.rs` is ~5.4k
> lines and `design_corridor.rs` is being edited right now; extracting from a
> moving file is a guaranteed merge disaster. See §8 step 0.

---

## 1. Crate boundary

### 1.1 The layering today

```
pnr-core                    zero deps, wasm-clean, deterministic (SplitMix64)
  └─ nucleation-routing     the Minecraft fabric; does NOT depend on nucleation
       └─ nucleation        Design, UniversalSchematic glue, Diplomat bridge
```

`src/design.rs` currently plays three roles at once: the layer-stack /
contract / undo model (legitimately `Design`'s job), **and** the bus geometry
planner, **and** the bus cost model. `src/design_corridor.rs` is a second
pathfinder living in the main crate (`Cargo.toml:161-164` says so out loud:
"`design.rs` routes bus corridors with A\* rather than growing a second
pathfinder"). Neither of those two belongs to a schematic editor.

### 1.2 The layering proposed

```
pnr-core                    generic algorithms  (UNCHANGED)
  └─ nucleation-routing     mechanism model, per-net router, DRC/LVS/STA
       └─ nucleation-bus    forms, tiles, planning, cost, BusWorld    ← NEW
            ├─ nucleation        Design = composition, layers, contracts, glue
            └─ mc-plugin         live autobussing                (speculative)
```

**Dependency direction is one-way and acyclic**, extending the rule already
written into `Cargo.toml:66-69`. In particular:

* `nucleation-bus` **must not** depend on `nucleation`. No `UniversalSchematic`,
  no `CellContract`, no `Region`, no serde requirement, no fs, no threads, no
  clocks. Same contract as `pnr-core` and `mc-tick`.
* `nucleation-bus` **may** depend on `nucleation-routing`, because bus planning
  genuinely needs the mechanism model (`transport::can_occupy` / `interferes` /
  `step_reads`), the per-net router (`RedstoneRouter`) to fill a corridor, and
  the checkers (`drc`, `lvs`, `sta`) to validate a plan.
* `nucleation-routing` **must not** gain a dependency on `nucleation-bus`. This
  is the constraint that decides several of the moves below.

### 1.3 What MOVES into `nucleation-bus`

**From `crates/nucleation-routing/src/`:**

| moves | contents | why it can move |
|---|---|---|
| `bus.rs` → `nucleation_bus::form` | `Axis`, `Face`, `InOut`, `Encoding`, `Pitch`, `BusSpec`, `BusPort::{bit, mates}` — and **generalized on arrival** per §4: `Pitch` dissolves into `CrossSection::offsets`, `BusSpec` gains `BusType`/`LaneOrder`/`Form` | grep says the *only* in-crate consumer is `pivot.rs`, which is also moving |
| `pivot.rs` → `nucleation_bus::tiles::pivot` + `nucleation_bus::plan` | `PIVOT_BITS`, `PivotKind`, `pivot_fragment`, `input_ports`/`output_ports`, `BusForm`, `bus_form`, `pivot_for`, `BusRouteReport`, `route_bus` — the pivot tiles become `FormTransition` templates (§4.7) and `BusForm` is demoted to a classifier (§4.4) | it is the bus planner; its imports (`blocks`, `cell::Fragment`, `router`, `workspace`) all stay behind it, so the dependency points the right way |
| tests `pivot_integration.rs`, `pivot_sim.rs` | move with the code; `pivot_sim` keeps its `required-features = ["mc-tick"]` gate | |

**From `src/design.rs`** (all of these are private `fn`/`const` today, i.e. no
external breakage):

| moves | contents |
|---|---|
| → `nucleation_bus::levelshift` | `ShiftCell` (the `StepOff`/`Land`/`Flat`/`StationEntry`/`StationRepeater`/`StationExit` column enum), `shift_plan`, `shift_len`, `shift_len_max`, `SHIFT_DUST_CAP = 12`, `REFRESH_AT = 7` — the verified level-shift tile and the station-insertion policy (`bus_levelshift.py`, 3040 output checks) |
| → `nucleation_bus::cost` | `BusCost`, `BusCostVector { length, delay_rt, skew_rt, coherence, footprint }` — already exactly the cost vector this design needs; keep the doc comment explaining why it is a vector |
| → `nucleation_bus::plan` | `RunInfo`, `Segment`, `SegmentKind`, `BusStyle` (+`validate`), the inline-`Gate` threading model |
| → `nucleation_bus::ty` | `WidthMap` (bit mapping / truncate / promote is bus semantics) — lands on the **type** layer, §4.1, and generalizes from scalar widths to `BusType` adaptation |

**From `src/design_corridor.rs`** — the whole file → `nucleation_bus::corridor`:
`MIN_STRAIGHT`, `Effort { corner_cost, slack, node_budget }`, `LADDER`,
`Heading`, `Leg`, `BusFabric` (+ its `impl Fabric`), `column_cells`,
`column_free`, `blocker`, `search`, `self_clearance_ok`, `leg_distance`,
`compress`, `diagnose`, `blocking_layers`, `first_blocker_on_line`.

**New in `nucleation-bus`** (does not exist anywhere yet):

* the **data model** (§4) — `BusType`, `LaneOrder`, `CrossSection`, `Form`,
  `LayoutPolicy` + policy registry, `Carrier` / `CarrierProfile` + carrier
  registry, `FormTransition`;
* `BusWorld` (§2) and the world-side plumbing `BusPlan` / `ReadSet` / `WriteSet`;
* `TileSpec` / `TileRegistry` / `Preconditions` / `Postconditions` /
  `select_tile` (§5);
* `DecayLedger` — station and decay budgeting as an explicit running ledger
  rather than the two constants above;
* `Planner::plan_step` and the resumable state machine (§3).

### 1.4 What STAYS, and why

**`pnr-core` — unchanged.** It is already the right thing: `astar`,
`congestion::route_all`, `anneal`, `color`, `netcheck`, `sta`, `grid`, and the
`Fabric` trait seam (`moves`/`legal`/`cost`/`budget`, `State`, `Candidate`,
`RouteCtx`, `Budget`). Nothing bus-shaped leaks into it. `nucleation-bus` is
just another `Fabric` implementor, exactly as `BusFabric` is today.

**`nucleation-routing` keeps the fabric and the checkers:**

* `transport.rs` — the probe-backed mechanism model: `Kind`, `Reader`,
  `Mechanism` (11 rows), `NB4`/`NB6`, `sturdy`, `conducts`, `cuts_step`,
  `gates_downhill`, `step_reads(cut_block, upper_support, downhill)`,
  `mech_of`, `fwd_of`, `dust_pointing`, `wire_connects`, `Placement`,
  `can_occupy`, `interferes`, and the `BlockView` trait. **This is physics, not
  bus policy.** It is needed by DRC and by the single-net router as much as by
  bus planning, so it stays at the lower layer and `nucleation-bus` re-exports
  what its callers need.
* `via.rs` (`ViaRegistry`, `ViaTemplate`) — **stays**, despite being
  template-shaped: `fabric.rs:10` and `router.rs:14` both import it, so moving
  it would invert the dependency. `nucleation_bus::tiles` *wraps* `ViaTemplate`
  in a `TileSpec` instead.
* `cell.rs` (`Fragment`, `EdgeContract`, `CellTemplate { name, fragment,
  keepout, edge_contract, delay_rt }`) — **stays.** It is the generic
  "verified placeable template" and it is paired with `Workspace`.
  `nucleation_bus::TileSpec` composes it (`template: CellTemplate` plus
  preconditions), which keeps the move small and the concept in one place.
* `budget.rs` (`SignalBudget`, `core() -> pnr_core::Budget`) — **stays**,
  because `RedstoneFabric::budget()` needs it. The bus-level *policy* on top of
  it (where stations go, `REFRESH_AT`, `SHIFT_DUST_CAP`) is what moves.
* `workspace.rs` (`Workspace`, `Collision`, `begin`/`commit`/`rollback`),
  `blocks.rs`, `nets.rs`, `audit.rs`, `wire.rs`, `fabric.rs`
  (`RedstoneFabric`, `NetSpec`, `RMove`, `StairMem`, `H_MOVES`),
  `router.rs` (`RedstoneRouter::{route, route_to_net, route_all}`),
  `drc.rs`, `lvs.rs`, `sta.rs`, `region.rs` — all stay.

**`nucleation` (`src/`) keeps composition, contracts, and glue:**
`Design`, `CellDef`, `Instance`, `BusLayer`, `BusState`, `DesignPort`,
`InstancePort`, `BitHardware`, `Occupant`, `OccupancyIndex`, `DesignCheck`, the
report types (`MoveReport`, `RemoveReport`, `GateMoveReport`,
`PortModeReport`, `PortModeChange`), `CellBounds`, `io_contract` resolution,
`src/design_promote.rs`, `src/design_io.rs`, the `UniversalSchematic ↔
Workspace` glue in `src/routing.rs`, and the Diplomat bridge
(`src/bridge/design.rs`, `src/bridge/routing.rs`).

After the move, `Design::route_bus` becomes roughly: build a `DesignWorld`
view, call `nucleation_bus::plan(...)`, apply the returned `BusPlan` into the
`bus:<name>` layer, map failure to `BusState::Failed(reason)`. The atomicity
guarantee stated in its doc comment ("realization is atomic and never leaves a
half-routed fragment") is *strengthened* by this, because the plan is now a
pure value computed before any write happens.

---

## 2. `BusWorld` — the world-access abstraction

This is the whole key to plugin reuse. Today the corridor router is hard-wired
to one world representation: `BusFabric<'a>` borrows
`&'a crate::design::OccupancyIndex` and calls `owner_name(&Occupant)` on it. A
plugin has no `OccupancyIndex`; it has chunks.

### 2.1 Sketch

```rust
/// Everything a bus planner needs to know about a world, and nothing else.
///
/// Implemented once over `UniversalSchematic` + `Design`'s layer stack, once
/// over an in-memory block map (the test/bench harness), and later once over a
/// live server world.
pub trait BusWorld {
    // ---- READ ------------------------------------------------------------
    /// Block-state string at `p`, or `None` for air.
    /// MUST be O(1) amortized. Called on the A* inner loop.
    fn block_at(&self, p: Pos) -> Option<&str>;

    /// False when the planner is not allowed to reason about `p` at all
    /// (unloaded chunk, outside the permitted region, outside the schematic).
    /// A plan may never read a cell for which this is false.
    fn known(&self, p: Pos) -> bool;

    /// The planning envelope. Plans are clipped to it.
    fn bounds(&self) -> Aabb;

    // ---- OCCUPANCY -------------------------------------------------------
    /// Who owns `p`. `Immovable` is the plugin's default for anything the
    /// planner did not create.
    fn occupant(&self, p: Pos) -> Occupancy;

    /// Is the whole column slice `[y0, y1]` at `(x, z)` free?
    /// THE HOT PATH — see §2.3.
    fn column_free(&self, x: i32, z: i32, y0: i32, y1: i32) -> bool;

    /// True when `p` holds a mechanism the transport model does not cover
    /// (piston, observer, dispenser, target, rail, tripwire, …). The planner
    /// treats these as hard keepouts with a declared halo and REPORTS them;
    /// it never assumes they are inert. See TRANSPORT_MODEL.md "NOT MODELLED".
    fn unmodelled(&self, p: Pos) -> bool;

    // ---- WRITE (transactional) -------------------------------------------
    fn begin(&mut self) -> TxId;
    fn place(&mut self, tx: TxId, p: Pos, block: &str, owner: OwnerId)
        -> Result<(), Conflict>;
    /// Applies at most `budget.blocks` writes and returns whether more remain.
    fn apply_step(&mut self, tx: TxId, budget: ApplyBudget) -> ApplyProgress;
    fn commit(&mut self, tx: TxId) -> Result<Applied, CommitError>;
    fn rollback(&mut self, tx: TxId);
}

pub enum Occupancy {
    Free,
    Mine(NetId),        // this planner's own earlier claim — rip-up candidate
    Foreign(OwnerId),   // another layer / another player's build
    Immovable,          // never touch (bedrock, protected region, unmodelled halo)
}
```

**The one line that makes the mechanism model free:**

```rust
impl<W: BusWorld + ?Sized> nucleation_routing::transport::BlockView for W {
    fn block(&self, p: Pos) -> Option<&str> { self.block_at(p) }
}
```

With that blanket impl, `can_occupy(mech, cell, view)`,
`interferes(a, b, view)`, `step_reads(...)`, `dust_pointing(view, cell)` and
`wire_connects(view, p, q)` — the entire probe-backed legality layer — work
unchanged over a live server world, a schematic, or a synthetic bench world.
Nothing about the physics needs a second implementation. That is the payoff of
`transport.rs` already being written against `BlockView` rather than against
`Workspace`.

### 2.2 Foreign nets without labels

`interferes` takes two `Placement`s, each carrying a net name. In a plugin we
cannot label a player's redstone. Convention: **every foreign live cell is its
own singleton net**, id `foreign@x,y,z`. Then `interferes` works verbatim — no
foreign cell is ever the same net as ours, so every legal-adjacency exemption in
`TRANSPORT_MODEL.md §B` (strong↔strong at distance 1, weak↔dust at distance 0,
strong-block-under-repeater) still applies, and every illegal one still trips.

### 2.3 What must be cheap, and how cheap

| operation | called | required cost | plugin implementation |
|---|---|---|---|
| `block_at` | ~10–30× per A\* node expansion (`interferes` reads ≤ 27 cells) | **O(1), no locks** | read from an immutable **snapshot** of the region, captured once on the main thread; never touch the live world during planning |
| `column_free` | once per A\* node expansion | **O(1)**, not O(height) | per-`(x,z)` **bitmask** — one `u64` per 64 y-levels. Today `BusFabric::column_cells` walks the occupancy map; that is fine for a 6-cell studio grid and unacceptable at world scale |
| `occupant` | per node | O(1) hash | ditto, packed into the same column structure |
| `known` | per node | O(1) | chunk-resident bitset |
| `unmodelled` | per candidate cell | O(1) | precomputed during snapshot |
| `interferes` / `can_occupy` | per candidate placement | bounded (≤ 27 reads) | already bounded; no change |
| `apply_step` | per tick during commit | **bounded by the caller** | N blocks per tick |

The per-`(x,z)` y-bitmask is worth noting twice: it is simultaneously the
plugin's performance requirement **and** `TRANSPORT_MODEL.md` ranked unlock #3
("drop *a column is claimed for its whole height*; per-y column masks"), which
is the enabling change for every density trick. One change, two payoffs.

### 2.4 Transactional commit in a live world

Planning and applying are separate phases with different threading and
different failure modes.

**PLAN** — pure, reads the snapshot only, writes nothing, cancellable at any
point, may run off the main thread. Produces:

```rust
pub struct BusPlan {
    pub writes: Vec<(Pos, String)>,   // ordered: see below
    pub read_set: Vec<(Pos, BlockId)>,// every cell a legality decision depended on
    pub cost: BusCostVector,
    pub form: Form,                   // the chosen topology + carrier (§4)
    pub tiles: Vec<Chosen>,           // which tile, where, and WHY (§5)
    pub rejections: Vec<Rejection>,   // and why the others lost
}
```

**APPLY** — main thread, bounded per tick:

1. **Optimistic re-validation.** Re-read `read_set` against the live world. Any
   mismatch ⇒ abort with `Stale { cell, was, now }` and re-plan. Players build
   while we think; this is the only honest answer.
2. **Pre-image capture.** For every write, record `(Pos, Option<BlockState>)`
   before overwriting. Rollback = replay pre-images in reverse order. The
   pre-image log must survive across ticks, because `apply_step` spans ticks.
3. **Ordered writes.** Supports and inert blocks first, then dust, then
   repeaters/torches last. A half-applied bus is a **live circuit**: an
   unfinished wire with a powered head can fire whatever it happens to touch.
   Where the platform allows it, apply the whole write set with physics
   suppressed and then trigger one update pass — *speculative*, the exact API
   (`setBlock(..., applyPhysics=false)` on Bukkit; something else on Fabric)
   depends on the platform choice in §9.
4. **Rollback triggers:** stale read set, `Conflict` on a write, cancellation
   mid-apply, or a post-apply `drc` failure. Yes: run
   `nucleation_routing::drc` on the applied region as a **commit gate**, and
   roll back on a violation. The plugin should never be the reason a player's
   build starts glitching.

For the schematic consumer, `Workspace::{begin, commit, rollback}` already
implements exactly this shape for the non-live case, and `Design::route_bus`'s
existing atomicity guarantee is the same contract. The live case adds only the
read-set re-validation and the cross-tick pre-image log.

---

## 3. Plugin-driven requirements, and how they shape the API

### 3.1 Incremental, resumable planning under a tick budget

The plugin has ~10–20 ms per tick, shared with everything else. A bus plan in a
congested world can take much longer. So planning must be a resumable state
machine, not a blocking call.

```rust
pub struct Planner { /* explicit frontier, ladder rung, congestion history, … */ }

impl Planner {
    pub fn new(req: BusRequest, cfg: PlanConfig) -> Self;
    pub fn plan_step(&mut self, w: &dyn BusWorld, budget: StepBudget) -> Progress;
    pub fn cancel(&mut self);            // never wrote anything, so nothing to undo
}

pub struct StepBudget {
    /// A* nodes to expand this step. The primary knob — deterministic.
    pub nodes: u32,
    /// Tile-precondition checks this step.
    pub tiles: u16,
    /// OPTIONAL host escape hatch. The crate never reads a clock itself.
    pub should_yield: Option<&'a dyn Fn() -> bool>,
}

pub enum Progress { Working { nodes_spent: u32 }, Done(BusPlan), Failed(Diagnosis) }
```

The budget is expressed in **work units, not time**, because `pnr-core`'s
contract forbids clocks and because a node budget is reproducible while a
deadline is not. The host converts its remaining tick time into a node count
(and may additionally supply `should_yield` for a hard cutoff, accepting that
this makes *that particular run* non-reproducible — the crate must document
that clearly).

What this demands of the moved code: `design_corridor::search` is today a
single blocking A\* with `Effort.node_budget`, wrapped in a `LADDER` of retry
rungs, driven from `design.rs:3185`. The ladder becomes the state machine's
outer state (`rung: usize`), and the A\* frontier becomes a field rather than a
local. `pnr_core::astar::route` therefore needs either a resumable variant or
to be driven in bounded slices — **the one real change required in `pnr-core`**,
and it should be additive (`route_resumable`) so nothing existing shifts.

### 3.2 Cancellation

`cancel()` is trivially safe because PLAN never writes. Cancellation *during
APPLY* is the interesting case and is handled by the pre-image log (§2.4).
State it as an invariant and test it: **no partial geometry ever survives a
cancelled or failed bus.**

### 3.3 Determinism

Same world snapshot + same request + same seed ⇒ byte-identical `BusPlan`.
Inherited from `pnr-core`'s rule (ordered containers, explicit tie-breaking,
seeded SplitMix64, no `rand`). Two concrete audits during the move:

* `src/design_corridor.rs:30` imports `std::collections::HashMap`. Ordered
  containers are the crate rule; either prove that map's iteration cannot
  affect results (memo lookup only, deterministic tie-break) or convert it to
  `BTreeMap`. Do not let it cross the crate boundary unexamined.
* `select_tile` (§5) must iterate the registry in a declared order, not in
  insertion or hash order.

### 3.4 No fs, no threads, no clocks

Same contract as `pnr-core`, `mc-tick` and `nucleation-routing`. The plugin may
run the planner on its own thread; the crate must not spawn one. Must build for
`wasm32-unknown-unknown` (so the studio can plan in the browser). No serde
requirement in the core — a `serde` feature is fine, a `serde` dependency is
not. `no_std + alloc` is desirable but **not** a v1 requirement (*speculative*:
worth checking whether it costs anything, since the crate already cannot use
fs/threads/clocks).

### 3.5 Arbitrary pre-existing redstone

The studio's world is our own verified geometry. The plugin's world is a
player's base. Four consequences, each an API requirement:

**(a) There are no contracts. Endpoint discovery must work from raw geometry.**
`Design` gets its endpoints from `CellContract` / `InstancePort`; a plugin gets
"the player clicked here". So:

```rust
pub fn discover_ports(w: &dyn BusWorld, seed: Seed) -> Vec<DiscoveredPort>;
pub enum Seed { At(Pos), Within(Aabb) }

pub struct DiscoveredPort {
    pub ty: BusType,            // inferred leaf count; structure usually Scalar
    pub order: LaneOrder,       // spatial order; significance often `Ambiguous`
    pub form: Form,             // measured CrossSection offsets + Carrier + axis + roll
    pub encoding: Encoding,     // Binary unless the carrier proves value-preserving
    pub face: Face,
    pub lanes: Vec<Pos>,        // in inferred lane order
    pub confidence: Confidence, // Certain | Likely(reason) | Ambiguous(alternatives)
    pub evidence: Vec<String>,  // user-facing: "8 parallel dust runs, 2y pitch, +X"
}
```

All four layers plus the carrier are inferred — see §4.9 for what each is read
from, and note that the measured offsets may be **irregular**, so discovery must
produce a `CrossSection::from_offsets` rather than trying to fit a lattice.

Inference is built from primitives that already exist:
`transport::dust_pointing` gives a dust cell's connection axes;
`transport::fwd_of` gives a repeater/comparator's direction (remembering that
`facing` names the **input** side); `transport::wire_connects` gives electrical
adjacency; `nets::neighbours` walks a run. Clustering parallel runs yields
width and `Pitch`; `pivot::bus_form` is the contract-based version of the form
classification that must be generalized to work from geometry alone.
`Confidence` is not optional decoration — a plugin that silently guesses an
8-bit port from a 7-bit build will produce a wrong circuit, so ambiguity must
be reportable to the player.

**(b) Everything not ours is `Immovable` by default.** So the planner has to be
good at *congestion*, not merely at empty space. This promotes
`pnr_core::congestion::route_all` (negotiated congestion, rip-up and reroute
with escalating history costs) from "would be nice" — it is literally the
`IMPROVEMENTS.md` "Still open P2: the remaining 4 *no corridor* failures are
genuine congestion" item — to a core requirement.

**(c) Foreign redstone is live.** Handled by the singleton-foreign-net
convention in §2.2.

**(d) Unmodelled mechanisms exist.** Pistons, observers, quasi-connectivity,
target blocks, rails, tripwire — `TRANSPORT_MODEL.md` explicitly records these
as not modelled. `BusWorld::unmodelled` plus a declared halo, treated as hard
keepout and **reported in the plan**, is the honest behaviour. Never silently
route past a piston we cannot reason about.

---

## 4. The bus data model — four layers and an orthogonal carrier axis

> **Requirement (user).** *"We want a generalised enough model to support
> different bus topologies like vertical, horizontal, diagonal. Or even for
> example a square 4x4 bus, or like a vec3 of vertical float32 for example where
> we get 3 32 bit vertical buses."*
>
> *"That bus could very much be horizontal for example too, or even a weird
> topology."*
>
> *"We need different kind of transports; if we take a hex bus for example [it]
> is a typical hex bus that uses repeaters to transport the signal — it's wider
> and less easy to fit but way faster than alternating comparators dusts and
> blocks."* — **since measured and confirmed with numbers**;
> `redstone-eda/notes-hex-transport.md` is authoritative, and §4.6 records both the
> profile and the rule it refuted.

Today's model is **one hardcoded shape**: N bits stacked at 2y pitch (the
`bus8` v2 form), single-level, one carrier, with `BusForm` a small closed enum
and `Pitch` a scalar. That single assumption is *why* "N parallel wires" was
never in the search space, and why a congruent fast path had to be bolted on
**outside** the search. Do not preserve it. Replace it with four strictly
separated layers plus one orthogonal axis, each independently testable, each with
exactly one job.

```
BusType        logical, no geometry          vec3<f32> = Array{3, Scalar{32}}
   │                                                     96 leaf bits
   ▼
LaneOrder      leaf bits -> lane indices     significance, endianness, padding
   │
   ▼  LayoutPolicy  (Nest | Tile | Linear | Interleave | hand-authored | …)
   ▼                 recursive, per-level orientation frame
CrossSection   footprint PERPENDICULAR       an ARBITRARY set of lane -> (u,v)
   │           to travel                     offsets + scaffold
   │
   │  ×  Carrier   ── an INDEPENDENT axis ─────────────────────────────────
   │        dust / comparator-chain / repeater-chain / strong-block / torch
   │        tower / …  each with its own (footprint, latency, decay, support,
   │        value-preserving?) profile.  Legality and legal lane pitch are
   │        DERIVED from interferes() FOR A GIVEN CARRIER — never hardcoded.
   ▼
Form           CrossSection × Carrier + travel axis + roll  ← in the search state
```

The four layers are *descriptive* (what the bus **is**); `Carrier` is
*realizational* (how the bits **move**). They are orthogonal: the same
`BusType` + `LaneOrder` + cross-section shape can be realized by several
carriers with wildly different cost profiles, and choosing between them is a
cost decision (§4.6), not a semantic one.

### 4.1 Layer 1 — `BusType`: logical, no geometry

```rust
pub enum BusType {
    Scalar { width: u16 },                  // f32 -> Scalar{32}; a carry -> Scalar{1}
    Array  { n: u16, elem: Box<BusType> },  // vec3<f32> -> Array{3, Scalar{32}}
    Struct { fields: Vec<(Name, BusType)> },
}
impl BusType { pub fn leaf_bits(&self) -> u32; }   // vec3<f32> => 96
```

No positions, no axes, no pitch, no world. **This is where the existing adapter
work belongs**: width mismatch, MSB alignment, shifting, truncation and
promotion are properties of the *type*, not of the geometry. So `WidthMap` moves
onto this layer and generalizes from "driver width vs sink width" to "does type
A adapt to type B, and how". The semantics `design_width_adapt.rs` pins down
(`shift`, `first`, `count`, dropped driver bits, "an undriven promoted input
reads zero") are type-level facts and survive unchanged — they simply stop being
scalar-only.

Two types are assignment-compatible iff their leaf-bit sequences map under a
declared adaptation policy. A pure function: no world, no geometry, cheap,
exhaustively testable.

### 4.2 Layer 2 — `LaneOrder`: flatten the leaves to indices

```rust
pub struct LaneOrder { pub lanes: Vec<Lane> }   // vector index == lane index
pub struct Lane {
    pub path: LeafPath,      // which field/element this bit came from
    pub bit: u16,            // bit within that leaf
    pub significance: u16,   // declared, not inferred
    pub pad: bool,           // a reserved lane carrying no bit
}
```

Explicit and inspectable: bit significance, endianness, field order, and any
padding/alignment lanes. `vec3<f32>` flattens to 96 lanes; whether lane 0 is
`x`'s LSB or `x`'s MSB is a **declared choice recorded here**, not folklore.
Padding lanes are first class so a policy may reserve a separator lane without
pretending it carries a bit.

This layer exists so layers 1 and 3 never have to agree on anything but an
integer count.

### 4.3 Layer 3 — `CrossSection`: the footprint perpendicular to travel

The generalization that unlocks every requested topology.

```rust
pub struct CrossSection {
    /// lane index -> offset in the (u, v) plane NORMAL to the travel axis.
    /// An ARBITRARY set. There is no lattice type, no pitch parameter, no
    /// topology enum — just offsets.
    pub offsets: Vec<(i16, i16)>,
    /// Cells that must / must not exist in the same plane, each with the
    /// material predicate it must satisfy: supports, separators, insulators,
    /// lids. Same `Clearance` vocabulary as tile preconditions (§5.1).
    pub scaffold: Vec<((i16, i16), Clearance)>,
    /// Bounding (u, v) extent — feeds `BusCostVector::footprint`.
    pub extent: (i16, i16),
}
```

**The offset set is arbitrary and that is the point.** Regular lattices are
*constructors* that happen to produce offset sets; they are not a type:

```rust
impl CrossSection {
    pub fn line(n: u16, dir: (i16, i16), pitch: i16) -> Self;   // vertical, horizontal, diagonal
    pub fn lattice(nu: u16, nv: u16, pitch: (i16, i16)) -> Self;// the 4x4 square
    pub fn from_offsets(offsets: Vec<(i16, i16)>) -> Self;      // hand-authored / irregular / "weird"
    pub fn union(parts: &[(CrossSection, Frame, (i16, i16))]) -> Self; // composition, §4.5
}
```

Named topologies are therefore **values produced by constructors**, never
variants:

| topology | how |
|---|---|
| vertical stack (today's only form) | `line(n, (0,1), pitch)` |
| horizontal row | `line(n, (1,0), pitch)` |
| diagonal | `line(n, (1,1), pitch)` |
| **4×4 square** | `lattice(4, 4, (pu, pv))` |
| multi-trunk | `union` of disjoint parts |
| **irregular / "weird"** | `from_offsets(...)` — a **first-class citizen**, not a fallback |

A hand-authored irregular cross-section must be exactly as routable as a
lattice one. Nothing downstream may branch on regularity: legality is decided by
running `interferes()` over *the actual offsets*, so an irregular bundle is not a
special case — it either passes or it does not. Any code path that assumes
uniform pitch is a bug, and §6.4 carries a property test that says so.

**"Pitch 2 in y" is not an axiom.** It is the *solution* to the interference
constraint for the dust-on-solid-block carrier: `interferes()` says two dust
lanes one cell apart cross-talk and a separating support is required, so the
minimum legal vertical pitch is 2. A **different carrier gives a different
answer** — a strongly powered block may sit *immediately adjacent* to another
net's identical cell (`TRANSPORT_MODEL.md` row 3, probes S1/S3), so a
strong-block carrier's legal pitch is **1**; a repeater-based carrier is *wider*
per lane and so pushes the pitch up; a torch tower has its own spacing and an
inversion parity. Generalized: **pitch is a function of (carrier, neighbours)**,
and the model must **discover** it, never assume it:

```rust
/// Is this cross-section, realized with this carrier, internally legal here?
/// DERIVED from the mechanism model — never hardcoded.
pub fn validate(xs: &CrossSection, carrier: Carrier,
                w: &dyn BusWorld, at: Pos, axis: Axis, roll: Roll)
    -> Result<(), Vec<Interference>>;

/// The tightest legal packing FOR A GIVEN CARRIER — solved, not asserted.
pub fn min_pitch(carrier: Carrier, dir: (i16, i16), w: &dyn BusWorld) -> i16;
```

`validate` instantiates each lane as the carrier's `transport::Placement`
set at its offset and runs `can_occupy` plus pairwise `interferes` across the
plane (and against whatever the world already holds there). `min_pitch` searches
for the smallest packing that validates. Two consequences worth stating plainly:

* The `2` disappears from the code and becomes a **derived, cached** result, with
  a test asserting it still equals 2 for dust — so the verified `bus8` v2 form is
  *reproduced by* the model rather than *assumed by* it.
* Ranked unlock #5 ("`strong_block` as a routable carrier … 1-cell-separated
  parallel signals") becomes a `Carrier` **value**, not a rewrite. Density work
  turns into a search over (cross-section × carrier).

### 4.4 Layer 4 — `Form`: cross-section × carrier + travel axis + orientation

```rust
pub struct Form {
    pub xs: CrossSection,
    pub carrier: Carrier,    // the orthogonal axis of §4.6, bound in here
    pub axis: Axis,          // travel direction
    pub roll: Roll,          // which world axes (u, v) map to: 0/90/180/270 + mirror
    pub origin_lane: u16,    // the lane the route's `position` refers to
}
```

`Form` is the *only* place the four layers and the carrier axis are bound
together, which is what makes each of them independently testable and makes
"same bus, different carrier" or "same bus, transposed" a change to one field.

A route's search state becomes **`position + form`**. That is exactly the
`state = (position, form, bundle geometry)` growth already on the roadmap
(`TRANSPORT_MODEL.md`, "What the search state must become") — `Form` **is** form
+ bundle geometry, unified into one value. Two notes:

* **This subsumes the congruent fast path.** The only reason a "both endpoints
  have identical geometry ⇒ N straight parallel wires" shortcut had to be bolted
  on outside the search is that "N parallel wires" was not expressible as a
  search state. With `Form` in the state it is simply the trivial route: one
  straight segment, no `FormTransition`, `coherence = 0`. **Delete the fast path
  when this lands** — a shortcut the general model reproduces is a liability, not
  an optimisation.
* `BusForm` (the closed enum in `pivot.rs`) is demoted to a **classifier over
  `Form`**, used for naming and diagnostics, not a type the router branches on.
  `Pitch` (the scalar in `bus.rs`) is absorbed into `CrossSection::offsets`.

### 4.5 `LayoutPolicy`: composing a type tree into a cross-section

The `vec3<f32>` example is a **composition** requirement. Note carefully what it
is *not*: it is **not** "three vertical 32-bit stacks". Vertical was one
instance. Each of the three sub-bundles may itself be horizontal, diagonal or
irregular, and **the outer composition axis is independent of the inner one** —
three horizontal 32-lane ribbons stacked vertically is as valid as three vertical
columns placed side by side, or three irregular blobs arranged diagonally. So
composition must take a sub-cross-section **and its own orientation frame**, and
must recurse with a per-level frame:

```rust
pub trait LayoutPolicy {
    fn layout(&self, ty: &BusType, order: &LaneOrder, carrier: Carrier,
              w: &dyn BusWorld) -> Result<CrossSection, LayoutError>;
}

/// A sub-bundle's own orientation within its parent's (u, v) plane.
/// Composition = place child cross-sections, each under its OWN frame.
pub struct Frame { pub rot: Rot4, pub mirror: bool }   // independent per nesting level
```

Provided policies — an **open set**; new topologies must be addable without
touching the crate:

| policy | behaviour | example `vec3<f32>` result |
|---|---|---|
| `Linear { dir, pitch }` | all lanes along one direction | one 96-lane run (tall or wide, usually impractical) |
| `Nest { outer, inner, inner_frame }` | recurse the type tree: `inner` lays out each element under `inner_frame`, `outer` places the elements | 3 × 32 sub-bundles — vertical-in-horizontal, horizontal-in-vertical, or anything else the frames say |
| `Tile { aspect }` | pack lanes into a 2-D lattice of the given aspect | the **4×4 square** bus |
| `Interleave` | bit-interleave leaves across fields | one bundle, lane *k* = bit *k* of each field in turn |
| `Explicit(offsets)` | hand-authored, arbitrary | whatever the author drew |

`Nest` is recursive and each level carries its own `Frame`, so
`Nest{ outer: Linear{(0,1)}, inner: Linear{(1,0)}, .. }` gives three *horizontal*
ribbons stacked *vertically*, while swapping the two directions gives three
*vertical* columns side by side. Neither is privileged. The cost vector decides
(§4.8), not the type.

`Nest` vs `Interleave` is precisely **AoS vs SoA**: field-blocked
(`x[0..32] y[0..32] z[0..32]`) versus bit-interleaved (`x0 y0 z0 x1 y1 z1 …`) —
two *policies* over one *type*. Which is better is a routing question, not a
semantic one: `Interleave` minimizes `coherence` when the three fields are
consumed together; `Nest` minimizes `FormTransition` cost when they are consumed
by three separate cells.

**Named topologies are policies, not enum variants.** Vertical, horizontal,
diagonal, square and multi-trunk are `LayoutPolicy` values (or compositions of
them), addressed by a `PolicyId` in a registry — the same open-registry
discipline as the tile registry (§5.1), so a user, a plugin, or a future crate
can add one. Nothing in the router may match on a fixed topology enum, and
nothing may assume a sub-bundle shares its parent's orientation.

### 4.6 The `Carrier` axis — how the bits actually move

> **Requirement (user), from a real build** — now **MEASURED**. The build is
> `TRANSMIT002_hex_transmit_flat`; the characterization is
> `redstone-eda/notes-hex-transport.md` (probes `probe_hex_transmit.py` 66/66 and
> `probe_hex_vs_comparator.py` 22/22, corpus copy in
> `redstone-eda/corpus/`). The user's claim: *"we need different kind of
> transports, if we take a hex bus for example [it] is a typical hex bus that uses
> repeaters to transport the signal — it's wider and less easy to fit but way
> faster than alternating comparators dusts and blocks."*
>
> **Confirmed with numbers: 4.00× faster at spans 16/32/48, exactly, at 3× the
> width and 2.8× the blocks — and analog-lossless over all 16 levels.**

The same `BusType`, the same `LaneOrder` and the same cross-section *shape* can
be realized by **different carriers**, each with its own profile. This is a
fourth, independent axis — not a property of the type and not a property of the
topology.

```rust
pub struct CarrierProfile {
    pub id: CarrierId,                // open registry, same discipline as tiles
    /// The (u,v) footprint ONE lane of this carrier occupies. Note this is the
    /// whole bundle for a multi-column carrier: the hex comb is 3 wide and its
    /// middle column is DEVICES, not wire.
    pub lane_width: (i16, i16),
    /// How this carrier is priced. NOT always per-cell — see (d).
    pub pricing: Pricing,
    /// Support/scaffold demands, expressed as `Clearance` predicates so
    /// `CrossSection::scaffold` can be generated from the carrier.
    pub scaffold: Vec<((i16, i16), Clearance)>,
    /// Mechanism rows this carrier is built from (TRANSPORT_MODEL.md §A/§B).
    pub mechanisms: &'static [Mechanism],
    /// Does the internal conduction path depend on the transported VALUE? See (e).
    pub value_dependent_conduction: bool,
    /// *** CORRECTNESS, not cost. MEASURED, never inferred. See (c). ***
    pub value_preserving: ValuePreserving,
    pub inverts: bool,                // torch-based carriers flip parity
    pub provenance: &'static str,     // the probe run that earned these numbers
}

pub enum Pricing {
    /// Genuinely additive: cost accumulates per cell of travel. Safe as A* edges.
    PerCell { delay_gt: Rational, decay_per_cell: u8, refresh: Refresh },
    /// An indivisible stage with an entry/exit CONTRACT. Priced as one unit.
    AtomicStage(StageContract),
}

/// A carrier stage that cannot be subdivided: you place the whole thing or none
/// of it, and its ss delta may be ZERO or NEGATIVE (a gain).
pub struct StageContract {
    pub span: u16,                 // cells of travel consumed, FIXED
    pub footprint: (i16, i16, u16),// the stage's whole envelope
    pub delay_gt: u32,             // for the whole stage
    pub entry_ss: SsReq,           // what must arrive
    /// exit = f(entry). May be identity, or a GAIN. Not a per-cell decay.
    pub exit_ss: SsLaw,
    pub pipeline_depth_gt: u32,    // how soon the next value may be injected
    pub min_separation_gt: u32,    // shorter gaps between values are SWALLOWED
}
```

Seed carriers (an **open** set — pistons, observers, torch towers,
strong-powered-block chains and anything else follow the same
declared-profile-plus-probe discipline as the eleven mechanism rows). Only the
rows citing a probe are trustworthy; the rest are *estimates awaiting
characterization* and are marked so:

| carrier | lane footprint | speed | ss law | value-preserving | pricing | evidence |
|---|---|---|---|---|---|---|
| `DustOnSolid` | 1 × 2 | 0 gt | −1/cell, 15-cell reach | yes | `PerCell` | today's only carrier |
| `ComparatorChain` | 1 × 2 | **1.000 gt/block** | refreshes each link; pitch 2 is its best legal packing (pitch 3 costs a level) | **yes** | `PerCell` | measured, `probe_hex_vs_comparator` C0/C1/C3 |
| **`HexCombStage`** | **3 × 2**, 16 z per stage | **0.250 gt/block** chained (0.143 for the bare transport section) | **`out = min(15, v + (15 − comb_len))`** — identity at the full 15-rung comb, a **GAIN** for a short one | **YES**, all 16 levels | **`AtomicStage`** | measured, `probe_hex_transmit` H2.2/H4.8/H6.3/H7.5 |
| `RepeaterLine` (binary only) | 1 × 2 | 0.125 gt/block | arrives normalised to 15 | **no** | `PerCell` | measured, C3b |
| `StrongBlockChain` | 1 × 2, pitch 1 legal | 0 gt | refreshes | unmeasured | *estimate* | ranked unlock #5 |
| `TorchTower` | 1 × 2, vertical | 2 gt/level | refreshes | unmeasured | *estimate* | `inverts = true`, so parity joins the state |

Five things follow, and all five are load-bearing.

**(a) Carrier crosses with cross-section.** A wider carrier inflates the per-lane
footprint and therefore *changes the legal lane pitch*. So cross-section legality
is only meaningful **for a given carrier** — which is why `validate` and
`min_pitch` in §4.3 both take `Carrier`. This is the same statement as "pitch 2
is not an axiom", now fully generalized: **pitch is a function of (carrier,
neighbours)**, computed from the mechanism rows via `interferes()`, never written
down as a constant. Two distinct sources of width, and they must not be conflated:
a carrier's **intrinsic** lane footprint (`HexCombStage` is 3 × 2 because a stage
*is* two dust lanes plus a comb column between them — measured, H0.13/H7.5, and
constant for any number of chained stages), and the **derived** neighbour pitch
(2 lateral for that stage, because its output lane runs hot at 15 and leaks ss 14
at gap 1 — H8.6/H8.7). The first is declared by the profile; only the second is
solved by `min_pitch`. Neither is a typed-in constant.

**(b) Carrier selection is a genuine cost decision, and it validates the cost
vector.** Repeater-vs-comparator is *literally* `delay_rt` traded against
`footprint` — the two terms `BusCostVector` already carries. So carrier is chosen
by the same ranked search as everything else (§5.3):

```rust
// candidates = LayoutPolicy × Carrier × Roll, ranked by the weighted cost vector
for (policy, carrier, roll) in ctx.candidate_forms(port_a, port_b) { … }
```

and **it may differ per segment**: a fast `HexCombStage` down the long open trunk
(0.25 gt/block, 3 × 2), a compact `ComparatorChain` through a congested pinch
where the 3-wide bundle does not fit (1.0 gt/block, 1 × 2), joined by a
`Recarrier` transition (§4.7). Both are `value_preserving = Measured`, so the
swap is legal for an analog payload — that is a *measured* fact about both
carriers, not an assumption. The measured trade is stark enough to make the
per-segment choice obviously worth having: **4.00× latency for 3× width and 2.8×
blocks**, constant across spans 16/32/48. Segmented choice is only expressible
because carrier is per-`Form` and `Form` is per-segment.

**(c) ANALOG vs BINARY is a correctness rule — and `value_preserving` is a
property of the COMPOSED STAGE, never of the block kind.**

> ### ⚠ Cautionary note: this document was wrong, and the wrongness was plausible
>
> An earlier revision of this section stated as a **hard rule**: *"a repeater
> normalizes its output to 15 and destroys the analog signal-strength value; a
> comparator preserves it,"* and mandated a test named
> `hexanalog_bus_refuses_repeater_carrier`.
>
> The premise is **true per block** — feed a repeater 8, read 15
> (`probe_hex_vs_comparator` C3b.3, re-measured) — and **false per stage**.
> `TRANSMIT002` carries an analog 0–15 value through a comb of **15 delay-1
> repeaters, losslessly** (H2.2, H3.2): a comparator drives a decaying dust lane;
> the comb fires on every rung where ss ≥ 1, so exactly `v` rungs fire and the
> lowest sits at `z = 17 − v`; that rung injects 15 into a second dust lane, which
> decays back down to a tap comparator reading exactly `v`. **Strength → unary rung
> POSITION → strength are exact inverses.** The decay spent is
> `(v−1) + (15−v) = 14` cells for a 14-block traverse *for every* `v` — the refresh
> lands exactly where the budget runs out.
>
> So `normalizes(repeater)` does **not** imply `normalizes(stage containing
> repeaters)`. As originally written, our planner would have **refused the user's
> own working, fastest analog carrier, by name.**
>
> This is exactly the plausible-but-wrong axiom the whole model exists to
> prevent — the same failure mode as "pitch 2 in y" (§4.3) and "the instant
> crossing always works" (§5). The rule read as physics; it was a generalization
> from one block to an unexamined composition. **The lesson is structural, not
> anecdotal: no correctness property of a carrier may be inferred from its
> palette. Every one must be measured on the composed stage.**

The *intent* of the old rule stands — silent corruption is the failure class this
project keeps paying for — so the gate stays. Only its **basis** changes, from
inference to measurement:

```rust
pub enum ValuePreserving {
    /// Earned by passing the 16-level in-sim round trip. Carries the citation.
    Measured { probe: &'static str },
    /// Measured to NORMALIZE or otherwise alter the value. Carries the citation.
    MeasuredNot { probe: &'static str, observed: &'static str },
    /// Never characterized. Treated as NOT preserving for gating purposes, and
    /// reported as "unmeasured", which is a different sentence from "unsafe".
    Unmeasured,
}
```

> **Hard rule (restated).** A bus whose `Encoding` carries meaning in the signal
> *strength* — `Encoding::HexAnalog` and any future analog encoding — **must refuse
> any carrier whose `value_preserving` is not `Measured`.** The planner checks
> this **before** cost, as a precondition, and reports the refusal by name *and by
> reason* — distinguishing "measured to normalize" from "never measured", because
> those call for different actions from the user (pick another carrier vs. run the
> probe).

**The single source of truth for the flag is the in-sim round trip.** It is the
same test this document already mandated, now promoted from corroboration to
*definition*:

* `value_preserving_carriers_round_trip_all_16_levels` — for every carrier
  claiming `Measured`, drive 0..15 through it in-sim under the `mc-tick` feature
  and assert `v_out == v_in` for all 16. A carrier that cannot pass this cannot
  hold `Measured`. This is what makes the flag trustworthy rather than
  aspirational, and it is the *only* way to obtain it.
* `no_carrier_claims_measured_without_a_probe_citation` — a cheap structural test
  over the registry, so `Measured { probe: "" }` cannot slip in.
* **Deleted:** `hexanalog_bus_refuses_repeater_carrier`. It encoded the block-kind
  inference in its *name* and would fail on `HexCombStage` forever. Replaced by
  `hexanalog_bus_refuses_carriers_that_are_not_measured_preserving`, which
  `HexCombStage` **passes** (it is `Measured`) and `RepeaterLine` fails.
* The fuzz oracle (§6.5 item 6) is unchanged in intent and gains precision: no
  random combination may route an analog bus on a carrier that is not `Measured`.

Existing analog work to check against: `redstone-eda/notes-hex-transport.md`
(authoritative for `HexCombStage`), `compositor/hexanalog.py`,
`compositor/hexanalog_bus8.py`, `showcase/hexanalog_trunk.schem`,
`showcase/hexanalog_bus8.schem`, and the `Encoding::HexAnalog` variant already in
`crates/nucleation-routing/src/bus.rs` and `src/io_contract/bus.rs`. Note that
`hexanalog.py` *encodes and decodes* a nibble to and from a strength, whereas
`TRANSMIT002` is **pure transport** with no encoder or decoder — a distinction the
`Encoding` field must keep straight.

**(d) Carrier cost is NOT additive over cells — some stages are atomic tiles.**

Every cost model in the fabric charges per cell (`TRANSPORT_MODEL.md`, "What the
search state must become", point 3: dust 1 ss / 0 gt, a hop +1 ss, a repeater 0 ss
/ 2 gt). `HexCombStage` breaks that in two ways at once:

* it costs **0 ss over 16 blocks** — not 16 × something; and
* with a comb shorter than 15 rungs it costs **negative ss**:
  `out = min(15, v + (15 − comb_len))`, exact over 20 measured combinations
  (H6.3). A short comb is a **free level shifter**.

A negative edge weight destroys the monotonicity that A\* and Dijkstra require:
with negative edges, a settled node is no longer final, `f` is no longer
non-decreasing along a path, and any admissible heuristic over ss becomes
unusable. Guessing at a "reduced cost" transform here would be exactly the kind of
plausible-but-wrong move (c) just warned about. So:

> **Rule.** A carrier stage whose ss law is not a per-cell decay is modelled as an
> **`AtomicStage`: an indivisible tile with an entry/exit contract**
> (`entry_ss` requirement → `exit_ss` law, fixed `span`, fixed `delay_gt`, fixed
> `footprint`), priced as **one unit**. It is **never** decomposed into per-cell
> moves, and its internal cells are never individually routable.

How admissibility is preserved:

1. **Two disjoint pricing kinds.** `Pricing::PerCell` carriers may become A\*
   edges, because their costs genuinely accumulate and are non-negative.
   `Pricing::AtomicStage` carriers may not; they are **placed**, like the crossing
   and pivot tiles in §5, through `select_tile` with preconditions and a ranked
   fallback. The `Pricing` enum is what stops a stage leaking into the edge
   relaxation, and a debug assertion should enforce it.
2. **Stages are edges of fixed length in the coarse graph.** A stage becomes a
   single edge from its entry port to its exit port, `span` cells long, with
   weight taken from the **`delay_gt` / footprint / coherence** terms — all
   non-negative. The ss delta is *not* a cost term at all: it is a
   **precondition and a post-condition** carried in the `DecayLedger`, checked
   like `entry_ss_min` in §5.3 step 3. Signal strength is a *resource with a
   budget*, not a distance. That reframing is what removes the negative weight
   entirely, and it is the same treatment §5 already gives every tile.
3. **A gain is a capability, not a discount.** Because the ss delta never enters
   the cost function, a `+k` stage cannot be exploited to manufacture a shorter
   path. It shows up only as a *wider* set of legal successors — a stage that can
   *raise* a stale value opens routes a decaying carrier could not reach.
4. **Fixed span means fixed geometry.** A stage's `span` is a constant, so the
   planner must reserve exactly that room (the same worst-case-reservation
   discipline `shift_len_max` already uses for level shifts) and must handle
   spans that are not a whole multiple of the stage length — for `HexCombStage`
   the remainder surfaces as a deliberate level shift, not a graceful loss.

**Level shifting is therefore a first-class carrier capability.** `StageContract`
should expose it directly:

```rust
impl StageContract { pub fn ss_gain(&self) -> i8; }   // HexCombStage: 15 - comb_len
```

A carrier that can raise a value subsumes part of what a dedicated refresh station
or level-shift tile is for — `shift_plan` / `bus_levelshift.py` exist to move a
bundle in *y*, but the refresh-station machinery inside them (`REFRESH_AT`,
`SHIFT_DUST_CAP`, the `StationEntry`/`StationRepeater`/`StationExit` columns) is
solving the same ss-budget problem a `+k` stage solves for free. When the chosen
carrier already refreshes or gains, **the planner must not also insert a station**.
That is a concrete simplification to look for during §8 step 4b/5, and a scenario
family in §6.1: *a bus on a gaining carrier plans zero refresh stations.*

**(e) A route is a path; some carriers are FIELDS.** `HexCombStage` is ~60 cells
of a **single net**, and *which rung conducts depends on the transported value*:
`v = 3` conducts through rung `z = 14`, `v = 15` through rung `z = 2` (H2.4). The
geometry is value-independent; the electrical path inside it is not. No "sequence
of cells + delay" representation can express that.

Consequences, all of them requirements rather than observations:

* **Occupancy and interference are computed over the stage's whole footprint, for
  ALL values** — never for one static conduction pattern. A planner that sampled
  the build at `v = 0` would see 15 dark rungs and conclude the lane was free to
  neighbour at pitch 1; the measured truth is that the OUTPUT lane runs **hot at
  15 along most of its length**, so a foreign dust one cell away picks up ss 14
  (H8.6) and pitch 2 is mandatory (H8.7). The clearance predicate must be the
  **union over all 16 values**, i.e. the worst case, computed once per stage and
  cached in the profile. This is why `interferes()` is evaluated against the
  stage's declared envelope, not against a sampled snapshot.
* **DRC and LVS must treat the stage as ONE net.** Any check asserting "one driver
  per net cell" or "a net is a simple path" will flag a correct build: here 15
  repeaters legitimately drive one output lane, and the active driver changes with
  the value. So `drc` needs a stage-aware exemption keyed on the placed tile
  (which the planner knows, because it placed it), and `lvs` must collapse the
  stage to a single two-terminal element with the measured `exit_ss` law rather
  than tracing conduction through it. Cheap to state now, expensive to discover
  later from a false DRC failure on the user's own build.
* `value_dependent_conduction: bool` on the profile is what tells the rest of the
  system to take this path. It is also a **flag no carrier may set without a
  probe**, for the same reason as (c).
* Two hex buses side by side therefore need `3 + 1 = 4` u-cells each: the pitch-2
  rule applies to the **bundle**, not to a wire. Note also that this is a carrier
  whose middle column is **devices, not wire** — a lane shape the old
  one-claim-per-cell, dust-only model has no name for.

Two further measured facts worth carrying into the profile, because both are
planner-visible and neither is guessable: the stage is **pipelined to depth 4 gt**
(two values genuinely in flight), and values closer than **3 gt** are *swallowed*
by the delay-1 repeaters and never appear at the output (H5.8) — the comparator
chain filters identically (C4.2), so this is a property to *record*, not a defect
to hold against either carrier. And one clearance trap from the same probe run,
which belongs in the mechanism model rather than here: **a floor lever strongly
powers its attachment block, and dust reads a strong block at 15 on all six
faces**, so a lever attachment must be treated as a 6-directional 15-source. It
silently corrupted an analog lane during the probe build.

**Openness.** `CarrierId` is registry-addressed with a declared profile and a
`provenance` citation, exactly like `TileSpec`. Adding a piston-based or
observer-based transport must be *data plus a probe*, never a router change. Note
that pistons and observers are recorded in `TRANSPORT_MODEL.md` as **not
modelled**, so such a carrier cannot be added until its mechanism rows are probed
— the profile's `mechanisms` field is what makes that dependency explicit and
checkable. `notes-hex-transport.md` proposes `hex_comb_stage` as **row 12** of
that table, with full §A/§B entries; adopting it there is the natural home for the
mechanism-level half of this carrier.

### 4.7 `FormTransition`: changing form is a tile

If a form can be anything, converting between forms is a first-class operation
and must be a **template kind alongside the crossings**, carrying the same
preconditions + cost + ranked-fallback discipline (§5):

```rust
pub enum TileRole {
    Crossing, Via, LevelShift, Station,
    FormTransition(TransitionKind),
}
pub enum TransitionKind {
    Gather, Scatter,   // narrow <-> wide bundle (the vec3 split / merge)
    Rotate,            // change travel axis, cross-section preserved
    Transpose,         // swap u/v — vertical stack <-> horizontal row
    RePitch,           // change lane pitch (dust pitch 2 -> strong-block pitch 1)
    Shift,             // translate the bundle within its plane
    Recarrier,         // change `Carrier` mid-route
}
```

`Recarrier` deserves one note now that carriers can be atomic (§4.6d): a
transition into an `AtomicStage` carrier must satisfy that stage's `entry_ss`
requirement and reserve its full fixed `span`, and a transition *out of* one starts
the next segment's `DecayLedger` from the stage's measured `exit_ss` — for
`HexCombStage` the exact value `min(15, v + (15 − comb_len))`, not an assumed 15
and not a decayed remainder. A `Recarrier` is therefore the one transition whose
pre- and post-conditions come from the carrier registry rather than from the tile's
own geometry.

The existing verified pivot tiles (`PivotKind` v2h / h2v / flat90,
`pivot_tiles.md`, `pivot_fragment`) are **one instance** of this — specifically
`Rotate` and `Transpose` for the 8-bit dust cross-section — and they migrate into
the registry as `TileSpec`s with `role = FormTransition(..)`, keeping their
existing provenance strings. The level-shift tile (`shift_plan`) is another
instance.

This is what makes *"changing form is the bus's responsibility, not the
component's"* actually true: a component declares a `BusType` and a `Form`, and
the **bus** inserts whatever transitions are needed, ranked by cost, with a
guaranteed fallback — instead of every component being obliged to present the one
blessed shape. It is also exactly what a **horizontal realizer** needs: a
horizontal cross-section plus a `Transpose` at each end, and no new concepts.

### 4.8 How `coherence` and `footprint` select the form

`BusCostVector` already carries the two terms that make form selection
measurable, and its doc comment already says why they are separate:

* **`coherence`** — "summed cross-section area ABOVE the canonical form's, per
  slice along the route's principal axis, plus a fixed charge per form
  conversion. Zero means the bits travelled together the whole way in their
  canonical arrangement — a bus that reads as ONE object." With layer 3 explicit,
  this stops being a heuristic and becomes computable:
  `Σ_slices (area(actual_xs) − area(canonical_xs)) + Σ transition charges`. That
  one number is what prefers `Nest` over `Linear` for `vec3<f32>` (a 96-tall
  column must fan out and re-gather; three side-by-side columns need not), and
  what penalizes a route that dissolves a bundle into single wires and rebuilds
  it later.
* **`footprint`** — "occupied volume including a one-cell clearance shell." This
  is what prefers a compact 4×4 square over a 16-tall stack in a low-ceiling
  build, and it is the term `Tile { aspect }` is optimising.
* **`skew_rt`** enters too: a cross-section whose lanes take unequal path lengths
  through a transition has skew. So `Gather`/`Scatter` templates must publish
  **per-lane** delay, not one number.

Form selection therefore needs no separate mechanism: enumerate candidate
`LayoutPolicy × Carrier × Roll` combinations, **drop the ones that fail a
precondition** (a carrier that is not `Measured` value-preserving under an analog
encoding, §4.6c; a stage whose `entry_ss` requirement the ledger cannot meet; a
cross-section that fails `validate` for that carrier, §4.3), plan or estimate the
survivors, and rank by the same weighted `BusCostVector` the tile selector uses
(§5.3). Same cost vector, same precondition-then-cost ordering, same
ranked-fallback discipline, same reporting — *"chose `HexCombStage` +
`Nest{horizontal-in-vertical}`: 4.00× faster, 3× width; rejected
`ComparatorChain`: delay budget; rejected `RepeaterLine`: value_preserving =
MeasuredNot (arrives normalised to 15); rejected `Linear`: coherence 1840"*.

### 4.9 Consequences for the rest of this document

* `BusSpec` / `BusPort` (§1.3) are re-expressed as
  `{ ty: BusType, order: LaneOrder, form: Form, face: Face, dir: InOut }` — where
  `Form` already carries the carrier — and `BusPort::mates` becomes "the types
  adapt (layer 1) **and** the forms are congruent, or a `FormTransition` exists
  that bridges them (layer 4, including a `Recarrier` when the carriers differ)".
  `Encoding` stays on the port and is what triggers the §4.6c analog check.
* `PortReq` in tile preconditions (§5.1) constrains a **`Form`** — cross-section
  *and* carrier — not a `BusForm` variant. So the three crossing tiles declare
  *which cross-sections and which carriers they accept*, and a 4×4 square bus, or
  a repeater-carried hex bus, honestly **fails** `xw_updown`'s `PortReq` instead
  of being silently mis-stamped. Another instance of the §5 thesis.
* `discover_ports` (§3.5) must infer all four layers **and the carrier** from raw
  geometry: leaf count from the number of parallel runs, `LaneOrder` from spatial
  order (with significance flagged `Ambiguous`, since geometry cannot reveal
  endianness), `CrossSection` from the *measured* offsets — irregular ones
  included, which is why `from_offsets` must be first-class — `Carrier` from the
  mechanisms found along the run (`mech_of`), and `Form` from the travel axis. The
  plugin is exactly the case where no contract exists, so `Confidence` matters
  most here. Two hard-won cautions:
  * **Discovery must not infer `value_preserving` from what it sees.** Finding
    repeaters says nothing (§4.6c) — the flag comes only from a registry entry that
    already passed the round trip. If discovery matches a *known* carrier, it
    inherits that carrier's measured flag; if it matches nothing, the flag is
    `Unmeasured` and an analog route is refused pending a probe. Never guess from
    the palette; that is the exact error this document made.
  * **A recognised `AtomicStage` must be discovered whole.** Walking
    `TRANSMIT002`'s 60-odd cells as a path would produce nonsense: it is one net,
    3 wide, with a device column in the middle and a value-dependent conduction
    path (§4.6e). Discovery therefore needs stage-shaped template matching against
    the carrier registry *before* it falls back to per-cell run tracing, and a port
    on a stage carrier is the stage's entry/exit contract, not a set of dust cells.
* The harness (§6) gains cross-section and carrier scenario families, plus
  property tests that `min_pitch` is derived rather than assumed and that
  irregular cross-sections are not second-class.

---

## 5. Tile selection must be conditional

> *"The instant crossing might not always work."* Correct, and today the code
> cannot express that. `pivot::pivot_for(from, to) -> Option<PivotKind>` picks a
> tile from the two ports' **forms alone** — it never looks at the world, the
> arriving signal strength, the y-parity, or whether the intersection cell is
> occupied. That is a latent wrong-answer generator.

### 5.1 A tile is a candidate with preconditions

```rust
pub struct TileSpec {
    pub id: TileId,                       // "xw_updown" | "xw_hop" | "xw_buffered" | …
    pub role: TileRole,                   // Crossing | Pivot | Via | LevelShift | Station
    pub template: CellTemplate,           // nucleation_routing::cell — geometry, keepout,
                                          //   edge_contract, delay_rt
    pub pre: Preconditions,
    pub post: Postconditions,
    pub cost: TileCost,                   // contribution to BusCostVector
    pub provenance: &'static str,         // "crosswire_tiles.md xw_updown;
                                          //  verify_crosswire.py instant_B, 285 checks"
}

pub struct Preconditions {
    /// Tile-local occupied extent (from `template.keepout`).
    pub footprint: Aabb,
    /// Cells that must satisfy a material predicate, tile-local.
    /// `Clearance` ∈ { Air, Sturdy, Conducting, Insulating, NoForeignLive, NoSupport }
    pub clearance: Vec<(Aabb, Clearance)>,
    /// Port requirements: which `Form`s (cross-section shapes AND carriers,
    /// §4.4/§4.6) this tile accepts, plus axis, face and encoding. NOT a
    /// `BusForm` enum variant — a 4x4 square bus or a repeater-carried hex bus
    /// must be able to FAIL this honestly.
    pub entry: PortReq,
    pub exit:  PortReq,
    /// Refuse the tile if the arriving signal is weaker than this.
    pub entry_ss_min: u8,
    /// Relative y of the two lines' ports.
    pub level: LevelReq,      // SameLevel | Pitch(1) | Pitch(2)
    /// Hard y-parity constraint (xw_hop's one real cost).
    pub parity: ParityReq,    // Any | AxisByYParity { x_on: &[1,3], z_on: &[0,2], modulo: 4 }
}

pub struct Postconditions {
    pub exit_ss: ExitSs,      // Decays(u8) | Refreshed(15)
    pub exit_level: LevelDelta,
    pub exit_form: BusForm,
    pub delay_gt: u32,
    /// Cells the tile leaves claimed, so the next leg plans against truth.
    pub claims: Vec<Pos>,
}
```

`provenance` is mandatory and load-bearing: it names the probe run that earned
the numbers, so a tile can never be edited without someone noticing the
citation went stale.

### 5.2 The three crossings as data

Straight from `crosswire_tiles.md` and `TRANSPORT_MODEL.md` (measured;
`verify_crosswire.py`, 881 checks, 0 crosstalk):

| | `xw_updown` | `xw_hop` | `xw_buffered` |
|---|---|---|---|
| **envelope** | 7×4×7 per 4-level unit (3-level envelope, tiling pitch 2) | 7×4×9 per 4-level unit | **5×2×5** (5×5 plan — smallest) |
| **lines/unit** | 2 X + 2 Z | 2 X + 2 Z | 1 X + 1 Z |
| **delay** | **0 gt** | **0 gt** | 2 gt (4/6/8 at `delay=2..4`) |
| **ss cost** | +2 both axes | +1 (X) / +2 (Z, incl. lane jog) | **0 — refreshed to 15** |
| **`level`** | `SameLevel` — both ports at the same y | `Pitch(1)` | `Pitch(1)` (X at `y+1`, Z at `y`) |
| **`parity`** | `Any` | **`AxisByYParity { x_on: [1,3], z_on: [0,2], modulo: 4 }`** — the axes live on opposite y-parities | `Any` |
| **`entry_ss_min`** | 3 | 2 (X) / 3 (Z) | **0** |
| **`exit_ss`** | `Decays(2)` (measured 5 after a 2-dust stub) | `Decays(1)`/`Decays(2)` (measured X 6, Z 3) | **`Refreshed(15)`**, out-port 14 |
| **key `clearance`** | intersection `(3,y,13)` **must be AIR**; dip lane `y−1` and bump lane `y+1` free; bump supports **must be solid** (they are simultaneously the dip's lids and the CUT cells) | hop cell above the foreign line free; solid support under **every** dust; cut cells air wherever a diagonal must survive | intersection cell solid **and `(2,y−1,2)` must be AIR** (the strong block has no support beneath it); repeater backs entered **in line** (the pointing law) |
| **role** | default for coplanar bus crossings | for already-interleaved buses | **the guaranteed fallback** |

Note how many of these are *world* conditions (`AIR`, `NoSupport`, `solid`) and
how many are *arithmetic* conditions (level, parity, ss headroom). Both kinds
are currently invisible to the selector.

### 5.3 The selection algorithm

```rust
pub fn select_tile(
    reg:  &TileRegistry,
    role: TileRole,
    site: &Site,            // the two ports, the arriving DecayLedger, the y's
    w:    &dyn BusWorld,
    ctx:  &PlanCtx,         // cost weights, seed, budget
) -> Result<Chosen, Vec<Rejection>> {
    let mut ranked = reg.candidates(role);          // declared, deterministic order
    ranked.sort_by_key(|t| ctx.weights.key(&t.cost)); // stable sort; ties by TileId
    let mut rejected = Vec::new();
    for t in ranked {
        match check(t, site, w, ctx) {
            Ok(placement) => return Ok(Chosen {
                tile: t.id, placement,
                why: ctx.weights.explain(&t.cost),  // "0 gt, +2 ss, 7x7 — cheapest legal"
                rejected,                            // and WHY each better one lost
            }),
            Err(reason) => rejected.push(Rejection { tile: t.id, reason }),
        }
    }
    Err(rejected)   // caller surfaces the whole list, never a bare "no tile"
}
```

`check` runs preconditions **cheapest-first**, so the recorded rejection reason
is the most informative one available and no world reads happen for a candidate
that arithmetic already excludes:

1. **Port/role match** — forms, widths, encoding, axis pair. `BusPort::mates`
   is the existing predicate; `PortReq` generalizes it.
2. **Level & parity** — pure arithmetic on the two ports' y. This is what
   rejects `xw_hop` for coplanar buses and `xw_updown` for 1-y-pitch buses.
   Zero world reads.
3. **Signal-strength headroom** — `site.entry_ss` (from the leg's
   `DecayLedger`) vs `pre.entry_ss_min`. **This is the step that makes
   `xw_buffered` the fallback**: at the far end of a long run both 0-tick tiles
   fail here, and only the tile with `entry_ss_min = 0` /
   `exit_ss = Refreshed(15)` survives.
4. **Footprint** — `template.keepout` vs occupancy, via `column_free` /
   `occupant`. Cheap, bulk.
5. **Cell-wise legality via the mechanism model** — for each tile cell,
   `can_occupy(mech, cell, view)`; for each (tile cell, foreign neighbour) pair,
   `interferes(&Placement::new(mech, cell, net), &foreign, view)`. **This is
   where "the instant crossing might not always work" is decided honestly.**
   `xw_updown` needs `(3,y,13)` to be air and its bump supports to be solid; a
   player's block in the intersection, or a foreign dust two cells away, is
   discovered by `interferes` returning `Some(reason)` — not by a comment, not
   by a hardcoded assumption.
6. **Post-conditions recorded** — the returned `Chosen` carries exit ss, exit
   level, exit form and delay, which feed the next leg's `DecayLedger`. Assert
   them against stamped geometry in the golden tests (§6), so a tile cannot
   promise a refresh it does not deliver.

**Preference and the fallback invariant.** The default weights put `delay_rt`
above `length`/`footprint`, so 0-tick tiles rank above `xw_buffered` — but the
weights are caller-supplied (a latency-critical bus and a footprint-critical
bus want different orders). What must hold *regardless of weights*:

> **Registry invariant.** For every `TileRole` there exists at least one
> candidate whose preconditions are weight-independent and world-independent
> apart from footprint. For `Crossing` that candidate is `xw_buffered`
> (`entry_ss_min = 0`, `parity = Any`, smallest plan-view footprint, and it is
> the only one that refreshes).

Test it: `registry_has_an_unconditional_fallback_for_every_role`. Without that
invariant, a weight vector could leave a site with no legal tile for reasons
that have nothing to do with the world.

**Reporting.** Reuse the exclusion-reason discipline already established by
`design_corridor::diagnose` / `blocking_layers` / `first_blocker_on_line`: the
answer to "why is my bus slow?" must be a list — *"`xw_updown` rejected: cell
(14,7,22) is stone, needs AIR; `xw_hop` rejected: ports are coplanar, needs
opposite y-parity; chose `xw_buffered`: +2 gt, refreshes to 15"* — not a
shrug.

### 5.4 One precondition model, two uses

`TRANSPORT_MODEL.md` ranked unlock #4 wants `xw_hop`'s crossing primitive
promoted from a *stamped tile* to a router **move**, so the router can cross a
foreign line anywhere for +1 ss and 0 ticks rather than only where a tile was
pre-placed. Design `Preconditions` so it can be evaluated over **one cell** (a
move's edge predicate, feeding `Fabric::legal`) or **N cells** (a tile stamp) —
`check_cellwise(&self, cells: &[Pos], w) -> Result<(), Reason>` with the tile
case being `cells = footprint`. Then the same verified data serves both, and
promoting a tile to a move is a registry change, not a rewrite.

---

## 6. Iterating and optimising in isolation

The point of the extraction: **bus quality becomes measurable outside the
studio.** Today the only way to know whether routing got better is
`tests/design_routability.rs` in the main crate — which requires building
`nucleation`, `UniversalSchematic`, contracts and the corpus. The 52% → 84%
routability number in `IMPROVEMENTS.md` is exactly the metric that ought to be a
crate-level gate.

### 6.1 Scenario suite

A scenario is a small declarative file — no schematics, no NBT, no corpus:

```
world:  Empty(64,32,64) | Blocks[...] | Gen { seed, walls: 12, litter: "redstone" }
ports:  [ { at, width, pitch, form, axis, face }, … ]
buses:  [ { driver: 0, sinks: [1,2], style, weights } ]
expect: { routable: true, drc_clean: true, max_delay_rt: 8 }
```

Precedent: `pnr-core/tests/grid_fabric.rs` tests the `Fabric` trait with a toy
fabric and no Minecraft at all. Same idea one layer up.

Families: empty plane; random keepout walls; the dense 6-cell grid from
`IMPROVEMENTS.md`; crossing-heavy (two orthogonal 8-bit buses at *k*
crossings); level-shift ladders at `k ∈ {1,2,3,5,8} × {down,up} × {fresh,stale}`
mirroring `bus_levelshift.py`; congested corridors where earlier buses took the
lanes; and **foreign-redstone worlds** (random dust/repeater/piston litter) for
the plugin case.

Plus, from the §4 data model — these are the families that did not exist before
and are the whole reason the model was generalized:

* **topology sweep** — the same `Scalar{8}` bus routed as vertical, horizontal,
  diagonal, 4×4 lattice, multi-trunk, and three hand-authored *irregular* offset
  sets. Every one must route in an empty world; the irregular ones must not be
  measurably worse-served than the lattices.
* **composition sweep** — `vec3<f32>` under `Linear`, `Nest` with all four
  inner/outer direction pairs (vertical-in-horizontal, horizontal-in-vertical,
  and both same-direction cases), `Tile`, and `Interleave`. Tracks which policy
  the cost vector actually picks per scenario — that ranking *is* the deliverable.
* **carrier sweep** — the same bus under every registered carrier, at spans
  16/32/48 so the measured head-to-head is reproduced as a regression: a wide-open
  world (where `HexCombStage` must win on `delay_rt`, 4.00× and exactly constant
  across the three spans) and a 1-wide corridor (where its 3 × 2 bundle must not
  fit and `ComparatorChain` must win). The scenario proving **per-segment** carrier
  choice beats either carrier alone — open trunk plus a narrow pinch — belongs
  here, and the baseline numbers come straight from
  `notes-hex-transport.md` §3.
* **atomic-stage arithmetic** — spans that are and are not whole multiples of a
  stage's fixed `span`. For `HexCombStage` the remainder must surface as a
  *reported level shift* (`out = min(15, v + (15 − comb_len))`), never as a
  silent loss and never as a routing failure. Include the deliberate short comb as
  a **+k level shifter** used on purpose.
* **gaining carriers plan no stations** — a bus on a carrier that refreshes or
  gains must plan **zero** refresh stations. Guards the double-spend §4.6d warns
  about, where `REFRESH_AT` / `SHIFT_DUST_CAP` machinery fires needlessly on top
  of a carrier that already solved the ss budget.
* **analog safety** — a `HexAnalog` bus offered each carrier in turn. Expected:
  `RepeaterLine` refused (`MeasuredNot`, arrives normalised to 15), an uncited
  carrier refused (`Unmeasured`, with a "run the probe" reason), and
  **`HexCombStage` ACCEPTED** — it is a repeater carrier and it is
  `value_preserving = Measured`. That last case is the regression test for the
  error in §4.6c, and it must be present precisely because the intuitive
  expectation is wrong. Seeded from `notes-hex-transport.md`,
  `showcase/hexanalog_trunk.schem` and `compositor/hexanalog.py`.

### 6.2 Tracked metrics

Per suite run, emitted as JSON and diffed against a committed baseline:

* **routability rate** — the headline number.
* **cost vector** mean and p95: `BusCostVector { length, delay_rt, skew_rt,
  coherence, footprint }`.
* **tile mix histogram** — how often we fell back to `xw_buffered`. An
  excellent regression signal: a change that quietly stops satisfying
  `xw_updown`'s preconditions shows up here long before anyone notices the
  extra ticks.
* **form / policy / carrier mix** — which `LayoutPolicy`, `Carrier` and `Roll`
  the ranked search actually chose, per scenario family. This is the metric that
  tells you whether the generalized model is *being used* or whether everything
  is quietly collapsing back to the old vertical-dust default.
* **transition count** — `FormTransition`s inserted per bus, by kind. A rise
  here with no cost improvement means a layout policy is fighting the geometry.
* **rejection-reason histogram** — which precondition fails most. Tells you
  what to build next, from data.
* **planner effort** — nodes expanded, ladder rung reached, tile checks.

Gate policy, given the standing warning that the bench gate is noisy: gate
hard on **routability rate ≥ floor** and **DRC-clean = 100%**; only *report*
cost deltas. Wire it into `tools/prepush.py` and `tools/check_routing.sh`.

### 6.3 Golden-file tile geometry tests

Every `TileSpec` stamps into a canonical `BTreeMap<Pos, String>` written to a
committed `tests/golden/<tile_id>.txt`. A template edit becomes a reviewable
diff. Seed the goldens from the cell listings in `crosswire_tiles.md` and
`pivot_tiles.md` so the docs and the code cannot drift apart. Existing
precedent to carry over: `pivot_integration.rs` (geometry) and `pivot_sim.rs`
(in-sim, `required-features = ["mc-tick"]`).

Also golden the **post-conditions**: assert that the `exit_ss` a tile promises
equals what `nucleation_routing::drc::decay_check` computes on the stamped
geometry. That is the test that catches a lying tile.

### 6.4 Property tests

`proptest` as a **dev**-dependency only (the crate's runtime dep list stays
`pnr-core` + `nucleation-routing`), or a hand-rolled seeded generator if the
zero-dep aesthetic wins.

* **Congruent ports ⇒ coherence ≈ 0.** Two identical, aligned, same-form ports
  need no adapter and no pivot; `BusCostVector.coherence` must be 0 and the
  tile list empty. **This property is what retires the congruent fast path**
  (§4.4): once the general search satisfies it for every cross-section — vertical,
  horizontal, square, irregular — the hack has no remaining job and must be
  deleted.
* **`min_pitch` is derived, not assumed.** For `DustOnSolid`, `min_pitch` in the
  vertical direction must come out as 2 *from the interference search*; for
  `StrongBlockChain` it must come out as 1. If either constant appears in the
  source, the test is a lie — assert it against `interferes()` directly.
* **Irregular cross-sections are not second-class.** For a random legal offset
  set and the lattice with the same lane count and extent, routability must be
  equal and the cost vectors within a stated tolerance. Anything that special-cases
  regularity trips this.
* **Sub-bundle orientation is free.** For every inner/outer direction pair,
  `Nest` must produce a routable form in an empty world. No pair may be
  privileged, and none may fail for a reason other than a reported precondition.
* **A carrier that is not `Measured` value-preserving is refused under an analog
  encoding.** `HexAnalog` × (`MeasuredNot` | `Unmeasured`) ⇒ `Err`, with the
  carrier named *and the reason distinguished*. Fuzz it: no random combination may
  ever produce a *routed* analog bus on such a carrier. Silent corruption must be
  structurally impossible. Conversely — and this is the important half —
  `HexAnalog` × `HexCombStage` must **route**, because a repeater-based carrier can
  be lossless (§4.6c).
* **`Measured` is unforgeable.** Every carrier claiming `Measured` round-trips all
  16 levels in-sim under the `mc-tick` feature, and every `Measured` carries a
  non-empty probe citation. This test *is* the definition of the flag, not a
  corroboration of it.
* **No `AtomicStage` cost ever enters an A\* edge.** Assert structurally: a stage
  is placed via `select_tile`, never relaxed as an edge, and no edge weight in the
  search is negative. This is the property that keeps the heuristic admissible in
  the presence of a `+k` gaining stage (§4.6d).
* **A gain cannot shorten a route.** Adding a gaining carrier to the registry must
  never reduce a plan's `length` below what a non-gaining carrier achieves on the
  same scenario — signal strength is a budget, not a distance. If this fails, ss
  has leaked into the cost function.
* **Value-dependent conduction is handled as a worst case.** For a carrier with
  `value_dependent_conduction`, the clearance/occupancy footprint computed by the
  planner must equal the **union over all 16 values**, and must therefore forbid a
  pitch-1 neighbour on `HexCombStage`'s output lane even though at `v = 0` every
  rung is dark (§4.6e, H8.6). Sampling one value must not be able to pass.
* **A stage is one net to DRC and LVS.** A correctly placed `HexCombStage` must
  produce zero DRC violations, despite 15 repeaters driving one output lane. A
  false positive here is the failure mode §4.6e predicts.
* **Removing a gate never lengthens the plan.** Monotonicity in the request.
* **Enlarging the free region never worsens the cost vector.** Monotonicity in
  the world. This catches ladder/effort bugs where a later rung "wins" with a
  worse plan than an earlier rung could have found — a real hazard given
  `LADDER`'s retry structure.
* **Permitted adjacencies never break DRC.** For each legal adjacency in
  `TRANSPORT_MODEL.md §B` (strong↔strong at d=1, weak↔dust at d=0,
  strong-block-under-repeater), a plan that uses it must pass
  `nucleation_routing::drc`. This is the test that lets us *stop* being
  over-conservative — ranked unlock #1 — without fear.
* **Determinism.** Same scenario + same seed ⇒ byte-identical `BusPlan`.
* **Budget honesty.** `plan_step` never expands more than `budget.nodes`.
* **Rollback totality.** After a cancelled or failed plan+apply, the world is
  bit-identical to before.

### 6.5 Fuzzing

Random `BusType`s (nested arrays and structs, leaf counts 1..128) × random
`LaneOrder`s × random cross-sections (lattices **and** arbitrary offset sets) ×
random carriers × random rolls × random worlds. The oracle is **not** "must
route" — unroutable is a legitimate answer. It is:

1. never panic;
2. never return `Ok` with a plan that fails `drc`;
3. never return `Ok` with an unsatisfied post-condition;
4. never exceed the node budget;
5. never write outside `bounds()` or into an `Immovable` cell;
6. never return `Ok` for an analog encoding on a carrier whose `value_preserving`
   is not `Measured` (§4.6c) — the one oracle whose violation is invisible in the
   geometry;
7. never return `Ok` for a cross-section that `validate` rejects for the chosen
   carrier;
8. never subdivide, overlap or partially place an `AtomicStage`, and never emit a
   negative edge weight (§4.6d).

Every DRC-failing case found gets minimized and committed as a regression
scenario. This is the mechanism by which "the instant crossing might not always
work" stops being an intuition and becomes a test.

---

## 7. What this buys, concretely

* **Topologies stop being hardcoded.** Vertical, horizontal, diagonal, 4×4
  square, multi-trunk and irregular bundles are all the same code path
  (§4.3–§4.5). `vec3<f32>` composes, with each sub-bundle free to have its own
  orientation.
* **Transports stop being hardcoded.** The user's repeater-carried hex bus
  (**measured** 4.00× faster, 3× wider, analog-lossless) and a comparator/dust/block
  trunk (compact, 1.0 gt/block) are two `Carrier` values ranked by the existing cost
  vector, choosable per segment (§4.6) — and an analog bus still cannot be silently
  normalized to 15, because the gate is now the measured round trip rather than a
  guess about repeaters.
* **Carrier cost stops being assumed additive.** Stages with a fixed span and a
  zero-or-negative ss delta are priced as atomic tiles, which keeps A\* admissible
  and turns a short comb into a free level shifter instead of an unroutable
  negative edge (§4.6d).
* The congruent fast path gets deleted rather than maintained (§4.4).
* The `xw_updown` / `xw_hop` / `xw_buffered` trio becomes selectable *by
  condition* instead of by assumption — ranked unlock #2, and the answer to
  "Still open P1: vertical level adapters", since `xw_updown` removes the
  "two buses must occupy disjoint y-bands" constraint.
* Negotiated congestion (`pnr_core::congestion::route_all`) has a place to live
  and a metric to justify it — "Still open P2".
* Bus quality gets a number that does not require the studio.
* A plugin becomes possible without forking anything.

---

## 8. Migration plan

Each step is independently green. Steps 2–4 are pure moves whose behaviour is
asserted identical by the existing tests; step 5 is the only behaviour change
and it lands with its own tests.

**Step 0 — BLOCKING: let the in-flight work land.** `src/design.rs`,
`src/design_corridor.rs` and `crates/nucleation-routing/**` are being edited
right now. Nothing here starts until that is merged and green.

**Step 1 — create the crate, empty but wired.** `crates/nucleation-bus/` with
`[dependencies] pnr-core, nucleation-routing`; `crates/*` is already a workspace
member glob; add an optional dep in the root `Cargo.toml` under the existing
`routing` feature (`Cargo.toml:69`). Green by construction — nothing depends on
it yet.

**Step 2 — move the pure data.** `bus.rs` → `nucleation_bus::form`; `pivot.rs`
→ `nucleation_bus::tiles::pivot`. **Delete** them from `nucleation-routing`
(re-exporting would invert the dependency) and shrink its `pub use` list
accordingly. `via.rs` stays (`fabric.rs:10`, `router.rs:14` need it).
*Breaks:* every `nucleation_routing::{BusSpec, BusPort, Axis, Face, InOut,
Encoding, Pitch, PivotKind, BusForm, route_bus, …}` import. All are in-repo:
`src/routing.rs` (which does `pub use nucleation_routing::*`, so its glob needs
a second glob or explicit re-exports), `src/bridge/routing.rs`, `src/design.rs`,
and the two pivot tests, which move with the code.

**Step 3 — define `BusWorld`, make the corridor generic, DO NOT MOVE IT YET.**
In place, change `BusFabric<'a>` to borrow `&'a dyn BusWorld` instead of
`&'a OccupancyIndex`, and add a `DesignWorld<'a>` wrapper implementing
`BusWorld` over `Design` (a wrapper rather than
`impl BusWorld for OccupancyIndex`, because `OccupancyIndex` has no write side
and `BusWorld` does). Add the blanket `BlockView` impl. Prove it: `tests/design_routability.rs`,
`design_reroute_stress.rs`, `design_bus_topology.rs`, `design_level_shift.rs`
must stay green **and produce identical plans**. This is the risky step and it
is deliberately taken before any file moves, so a regression bisects to one
commit.

**Step 4 — move the corridor and the planner.** `src/design_corridor.rs` →
`nucleation_bus::corridor`. Out of `src/design.rs`: `ShiftCell`, `shift_plan`,
`shift_len`, `shift_len_max`, `SHIFT_DUST_CAP`, `REFRESH_AT` →
`nucleation_bus::levelshift`; `BusCost`, `BusCostVector` →
`nucleation_bus::cost`; `RunInfo`, `Segment`, `SegmentKind`, `BusStyle` →
`nucleation_bus::plan`; `WidthMap` → `nucleation_bus::ty`. `Design::route_bus`
shrinks to: build `DesignWorld`, call `nucleation_bus::plan`, apply the
`BusPlan` into the `bus:<name>` layer, map failure to `BusState::Failed`.

Tests that **move** (they need no `UniversalSchematic`, only geometry):
`design_level_shift.rs`, `design_routability.rs`, `design_reroute_stress.rs`,
`design_width_adapt.rs` — these become the seed of the §6.1 scenario suite.

Tests that **stay** (they are about `Design` semantics, not bus quality):
`design_bus_cross.rs`, `design_bus_topology.rs`, `design_drag.rs`,
`design_typed_drag.rs`, `design_layer_precedence.rs`, `design_promotion.rs`,
`design_export_after_promotion.rs`, `design_instance_transform.rs`,
`design_wire_states.rs`. Note that `design_layer_precedence.rs` carries two
`#[ignore]`d executable specifications of the `Region::get_bounding_box`
issue — they must remain ignored, not accidentally "fixed", or that open item
silently changes meaning.

**Step 4b — land the data model (§4), behind a compatibility shim.** This is a
core-data-model change, so it gets its own step and its own review. Order within
the step, each sub-step green:

1. `BusType` + `LaneOrder` + type-level adaptation (`WidthMap` generalized). Pure
   data, no geometry, no world — testable in isolation, and
   `design_width_adapt.rs`'s semantics are the acceptance tests.
2. `CrossSection` with `from_offsets` as the primitive and `line`/`lattice` as
   constructors; `validate` / `min_pitch` derived from `interferes()`. Acceptance:
   `min_pitch(DustOnSolid, vertical) == 2` **falls out** and the existing `bus8`
   v2 geometry is reproduced byte-for-byte.
3. `Carrier` + `CarrierProfile` registry, seeded with `DustOnSolid` only, so
   behaviour is unchanged. Then the `Pricing` split (`PerCell` vs `AtomicStage`)
   **before** any non-additive carrier is added — otherwise the first stage to land
   will be silently decomposed into edges. Then `ComparatorChain` and `RepeaterLine`
   (both `PerCell`, both already measured in `probe_hex_vs_comparator`), then
   `HexCombStage` as the first `AtomicStage`, transcribing the profile table from
   `notes-hex-transport.md` §4 verbatim with its probe citations. Finally the
   `ValuePreserving` gate and its tests — including the *acceptance* case
   (`HexAnalog` × `HexCombStage` routes), which is the regression test for the
   error recorded in §4.6c.
   Also in this sub-step: the stage-aware `drc`/`lvs` exemption (§4.6e), since a
   placed `HexCombStage` will otherwise trip "one driver per net cell" the moment
   it lands. That touches `crates/nucleation-routing`, so it must be sequenced with
   whoever owns that crate rather than bundled blindly.
4. `Form` into the search state; `LayoutPolicy` registry; `BusForm` demoted to a
   classifier.
5. **Delete the congruent fast path** and prove the general search still satisfies
   the congruent-ports property (§6.4). Do not skip this — an unretired hack will
   silently shadow the new model and hide its bugs.
6. `FormTransition` roles registered, with the existing pivot and level-shift
   tiles migrated in under their existing provenance.

Compatibility shim: keep `BusStyle`'s current fields mapping onto a default
`(Linear vertical, DustOnSolid)` form so every existing caller and every stored
design keeps its exact behaviour until it opts in. Removing the shim is a later,
separate decision.

**Step 5 — conditional tile selection.** Introduce `TileSpec` / `TileRegistry`
/ `Preconditions` / `Postconditions` / `select_tile`, and populate the registry
with the three crossings, the pivots, the vias and the level-shift tile. Keep
`pivot_for` as a thin wrapper during the transition so old tests compile;
delete it once goldens cover the new path. This is the only behaviour change in
the plan — land it with §6's harness, in the same PR or immediately after.

**Step 6 — the harness.** Scenario suite, golden tiles, property tests, fuzz
target, metrics JSON, `prepush.py` + `check_routing.sh` integration.

**Step 7 — resumability and the plugin** (a separate project). `Planner::plan_step`,
the read-set/write-set commit protocol, `discover_ports`,
`pnr_core::astar::route_resumable`.

Then the JVM surface. `nucleation-bus` is **not** bridged directly; the plugin
consumes it through a new Diplomat opaque in `src/bridge/` (e.g.
`src/bridge/bus.rs`, alongside the existing `routing.rs` and `design.rs`), so
`tools/gen-bindings.sh` generates the Kotlin/Java binding into
`bindings/kotlin` (patched by `tools/patch-kotlin-bindings.py`, smoke-tested by
`bindings/kotlin/smoke` and `smoke-java`). **A design consequence worth stating
now:** Diplomat opaques are JSON-in / JSON-out (see `Routing.route_net`), so
`BusWorld` cannot be implemented *in Kotlin* and passed down as a trait object
without a callback ABI. Two options:

* **(a) snapshot** — the plugin uploads a region as a block list, gets back a
  write set. One round trip, no callbacks, works with today's bridge.
  **Recommended for v1.**
* **(b) callback `BusWorld`** — needs Diplomat callback support; the `store`
  module's `callback` backend is the local precedent. *Speculative*, and
  strictly better if it lands.

---

## 9. Open questions

1. **Crate name and scope.** `nucleation-bus` says "buses", but the tile
   registry and `BusWorld` would serve any stamped-tile physical planning
   (vias, pivots, stations, future clock trees). Is the intended scope buses, or
   physical planning generally (`nucleation-plan`)?
2. **Where does the mechanism model live?** Keep `transport.rs` in
   `nucleation-routing` (proposed here — cheapest), or split it into a tiny
   `nucleation-mech` crate so `bus` and `routing` become siblings (cleaner if
   the layering ever binds)?
3. **Who chooses the cost weights?** A per-bus `BusStyle` field the user sets in
   the studio, a design-level default, or named presets ("latency", "compact",
   "reach")? This now also decides `LayoutPolicy` and `Carrier`, so the answer
   matters more than it did.
4. **How much of form selection is automatic?** Does the user pick a topology and
   a carrier (with the planner only validating), does the planner search the whole
   `LayoutPolicy × Carrier × Roll` space, or is it "the user pins what they care
   about, the planner searches the rest"? The third is most useful and needs a
   pinning syntax.
5. **Does `BusType` come from the HDL/contract layer, or is it declared at the
   bus?** `vec3<f32>` is a *type*, and `CellContract` / `IoLayout` already carry
   typed ports. If contracts can express `Array` / `Struct`, layer 1 should be read
   from them rather than re-declared — but that is a contract-schema change
   outside this crate.
6. **Per-segment carrier changes: on by default, or opt-in?** A `Recarrier`
   mid-route is strictly better on cost, but it makes a build visually
   heterogeneous and a user may reasonably want one uniform transport.
7. **Where does the carrier registry live if the plugin must extend it?**
   In-crate `&'static` profiles are cheapest but are not extensible from Kotlin
   across the Diplomat boundary.
8. **Analog beyond lossless/not.** `ValuePreserving` is now three-valued
   (`Measured` / `MeasuredNot` / `Unmeasured`), which fixed the block-kind error —
   but are there carriers that preserve only *part* of the range, or preserve it
   with a known offset (a comparator subtract chain, a container read, a short comb
   used deliberately as `+k`)? If so the flag wants to become the same `SsLaw` the
   `StageContract` already carries, and "value-preserving" is just `SsLaw::Identity`.
   Deciding this before the registry is populated is much cheaper than after.
9. **How does a carrier get characterized in the first place?** `HexCombStage`
   exists as a profile only because a user handed us a build and someone wrote two
   probe scripts. Is there a repeatable harness — "point it at a schematic, get a
   `CarrierProfile`" — or does every carrier stay a bespoke probe? The `mc-tick`
   feature plus the round-trip test are most of the machinery already.
10. **Does `TRANSPORT_MODEL.md` adopt `hex_comb_stage` as row 12?**
    `notes-hex-transport.md` §4 proposes it with full §A/§B entries. The mechanism
    table is currently all *primitives*; a composed stage is a different kind of
    row, so this may want its own table rather than row 12.
11. **Fallback semantics on a latency-critical bus.** If only `xw_buffered` fits
    and the bus is declared latency-critical, do we (a) use it and report the
    +2 gt, (b) fail and ask the user, or (c) reroute to avoid the crossing
    entirely? (c) is the best answer and much the most work.
12. **Foreign-redstone policy.** Is "unmodelled mechanism ⇒ keepout + halo"
    absolute, or may a user opt into routing adjacent to unknown redstone? What
    halo radius is defensible without quasi-connectivity or observer modelling?
13. **Live-world consent.** Does the plugin require an explicit permitted region
    (WorldGuard-style)? May a plan ever *break* existing redstone with consent?
14. **Plugin platform** — Paper/Spigot, Fabric, or both? Decides the
    JNI/Panama story and whether physics-suppressed batch writes exist.
15. **Snapshot vs callback `BusWorld` across the FFI** (§8 step 7). Is
    snapshot-only acceptable for v1, and what region cap (128³?) is reasonable?
16. **Does negotiated congestion become the default bus router** as part of this
    extraction, or stay a follow-up? It changes plans, so it changes goldens —
    better to decide before the goldens are written than after.
17. **Python's role.** Do `redstone-eda/*.py` remain the authoritative tile
    spec, or does the golden-file registry become authoritative with the Python
    probes demoted to verification?
18. **Publishing.** `pnr-core` and `nucleation-routing` already carry
    `license = "MIT"` and descriptions. Does `nucleation-bus` publish to
    crates.io, or stay path-only?
19. **`no_std + alloc`** — worth requiring, given the crate already cannot use
    fs, threads or clocks? (*Speculative*: may cost nothing.)
