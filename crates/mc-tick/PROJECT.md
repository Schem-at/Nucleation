# mc-tick: a vanilla-accurate Minecraft tick simulation

## The goal

Give nucleation a **tick function** — the ability to load a schematic and simulate
it the way the real game would, tick by tick. The immediate product driving this is
**auto-timing of piston doors** on schemat.io: hand it a door, get its open/close
timing. But the deliverable underneath that is general — a controllable, accurate
simulation (step, run, reset, checkpoint, restore, inspect) that new behaviour plugs
into cheaply.

Long-term this is a fully-integrated simulation engine inside nucleation, accurate
enough to trust for timing analysis and broad enough to eventually cover redstone,
fluids, items and entities.

Three constraints shape every decision:

1. **Iteration speed.** nucleation is 188k lines / 159 deps; building its test
   binaries takes ~28s. The engine lives in a separate workspace crate with ~2
   dependencies, so its edit→test loop is **~0.7s**. That gap is the substrate the
   whole project rests on — nothing is allowed to erode it.
2. **Accuracy must be *verifiable*.** Piston doors live or die on update *order*. A
   plausible-looking simulation that is off by one tick is worse than none.
3. **Runtime speed is a feature.** The product runs many simulations (timing
   variants), so throughput is a design input, not an afterthought.

---

## The core discipline: how we know it's right

**Every behaviour is derived from ground truth — a captured vanilla trace or the
game's own bytecode — never from memory or community folklore.** This is the single
most important rule in the project. It has already overturned things that "everyone
knows" but that are subtly wrong:

- dust settles **synchronously** in one tick, not one block per tick
- a piston's move takes **two ticks** and completes in the block-entities phase
- quasi-connectivity is real and reaches **exactly one block up**
- a repeater reads input on the side its `facing` names
- torch burnout counts **turn-offs only** (8), not all toggles
- comparator priming is a concrete `output != storedOutput` check, not folklore

Writing these from recall would have produced a simulator that looks right on small
examples and reports every real door wrong. The differ (below) catches the rest.

### Sources of truth, and how each may be used

| Source | Licence | How we use it |
|---|---|---|
| **Unobfuscated MC 26.2 server jar** | Mojang © | *Read* the real class bytecode for exact constants, delays, priorities, phase order. Never vendored. This is the primary reference — as of recently Mojang ships the server jar with real names (`ServerLevel.tick`, `runBlockEvents`), which is why no Fabric/Yarn/Mixins are needed. |
| **Captured vanilla traces** | (our data) | The oracle runs the *actual game* headless and records what it does. This is the ground truth the engine is diffed against. |
| **techmcdocs GameTick page** | docs | The tick phase order. Cross-checked against `ServerLevel.tick`'s bytecode and confirmed exact. <https://techmcdocs.github.io/pages/GameTick/> |
| **alternate-current** | **MIT** | Read for wire-algorithm structure. MIT means its *code* is usable, but we still reimplement rather than vendor. Note: it is deliberately *non-locational*; vanilla dust is locational — a conscious deviation to track. |
| **Lithium** | LGPL-3.0 | Read to *understand* mechanisms; a source of gametest *structures* to test against. Its code is **not** ported (LGPL vs nucleation's MIT). |
| Pumpkin | GPL-3.0 | **Excluded.** Wrong accuracy class (loose port) and licence-incompatible. |
| MCHPRS | none | Pre-existing nucleation dependency (accelerated redstone backend). Separate from this engine; not a source of truth. |

**Clean-room rule:** no code is copied from any reference regardless of licence.
References inform understanding; behaviour is written from that understanding and
validated against captured traces.

---

## Tooling

### `crates/mc-tick` — the engine

The simulation itself. Pure Rust, ~2 dependencies, no nucleation dependency. Owns
the world model, the phase scheduler, block behaviours, and the control surface
(step/run/reset/checkpoint/restore).

### `crates/mc-tick-trace` — the trace contract

The schema both the engine and the Java oracle speak: per-tick ordered events, each
tagged with the phase it happened in. Provides `diff` (exact, order-sensitive) with
a tolerance mode for float entity positions, and `canonicalize` for comparing
against snapshot-derived captures (which don't know intra-tick order).

### `tools/gametest` — the vanilla oracle

Runs **real Minecraft 26.2 headless** to produce ground truth. No Fabric, no Loom,
no Yarn, no Mixins, no EULA — the unobfuscated jar plus its own bundled classpath is
enough, and `GameTestServer` runs in-process for testing.

- **`RunGameTests.java`** — runs structure tests via Mojang's `GameTestServer`,
  emitting a JUnit report. `run.sh` wraps it and **verifies the declared tests
  actually ran** (a malformed pack once reported "all passed" while running only
  vanilla's `always_pass`).
- **`TraceCapture.java`** — drives `ServerLevel.tick()` manually, one tick at a
  time, and records block changes between ticks. Supports `--break` (remove a
  block), `--pulse`/`--pulse-ticks` (momentary power), `--pulse-period` (square
  wave, for rate limits like burnout) and `--use`/`--use-tick` (an empty-hand
  right-click, the exact `GameTestHelper.useBlock` sequence with an equivalent
  mock player), plus `--known-shape` for quiet placement (no update pass — the
  vanilla mechanism behind loading a contraption at rest). Deletes its multi-hundred-MB world on exit.
- **`capture.sh`** — wraps a capture end to end: compiles the drivers, stages
  the datapack (converting `.snbt`) into a fresh trace universe, and runs
  `TraceCapture` with the given flags. One command per golden.
- **`Snbt2Nbt.java`** — converts authored `.snbt` structures to the binary `.nbt`
  datapacks require, using the game's *own* NBT parser so the conversion cannot
  disagree with the reader.

Key facts baked into the tooling: `pack_format` 107 needs `min_format`/`max_format`;
DataVersion is 4903; `SharedConstants.tryDetectVersion()` must precede bootstrap;
`waitUntilNextTick` must be pumped or chunk entities never load (scheduled ticks
silently never fire).

### The verification loop

```
author .snbt  →  Snbt2Nbt  →  real Minecraft (TraceCapture)  →  golden .json
                                                                     │
crates/mc-tick engine  →  emit trace  →  diff against golden  ◄──────┘
```

`tests/conformance.rs` is this loop as a Rust test: load a structure, wire vanilla
behaviour to it via the registry, settle, actuate (place/break/click at chosen
ticks), run, and assert the engine's trace matches the captured golden **tick for
tick**. Eleven cases pass today — piston QC, slime adhesion, note-block power,
note-block click, the manual engine twice (its 21-tick placement cycle, and a
55-tick padded run whose second activation is started by a player click), a
broken flying machine that must break exactly as vanilla breaks it, and a
quietly placed engine that stays perfectly still until clicked, and a
comparator reading a barrel's fullness (on and off) — all
driven entirely by the descriptor→behaviour registry with no hand-wiring.

### `crates/mc-tick/tests/corpus/` — zero-compile test cases

A data-driven runner that discovers `.case` files at runtime — adding a test is
adding a file, no recompilation. `load <name>.snbt` runs a real structure.

---

## What's simulated today

**162 tests, all green.** Everything below is trace- or bytecode-verified.

### Engine core
- **Tick phases** — all ten, in verified order, as an explicit walked sequence.
  Unimplemented phases (raids, weather, mob spawning) are named no-ops holding their
  real position. `BlockTicks → BlockEvents → BlockEntities` is the spine a piston's
  motion spans.
- **Scheduler** — scheduled ticks ordered by `(target, priority, insertion)`; block
  events insertion-ordered and chainable within a tick. Seven tick priorities.
- **World** — interned `u16` block states, dense bounded-region storage, clone-based
  checkpoints. Out-of-bounds reads air (documented divergence; load with padding).
- **Neighbour propagation** — `set` queues notifications; `settle` replays
  placement (`onPlace`), which is how QC sources are noticed.
- **Control surface** — step, run, run_until_quiescent, reset, checkpoint,
  restore; `place_block` and `use_block` as boundary actuations, and
  `set_ticking_bounds` for chunk-edge freezing.
- **Boundary time** — actions between ticks (placement, breaks, clicks) schedule
  with "now" = the last completed tick, one tick sooner than in-phase schedules;
  their own changes are observed by the upcoming tick. Captured three ways
  (placement observer pulse, boundary repeater, clicked note block).
- **Player input** — `Simulation::use_block` + `BlockBehaviour::on_used`,
  executing at the tick boundary exactly where vanilla processes use-block
  packets (which is why `Phase::PlayerInputs` sits last in the phase list).
- **Loud unknown-block failure** — an unimplemented block is *named*, never silently
  simulated as inert.
- **Descriptor→behaviour registry** (`vanilla.rs`) — turns
  `minecraft:sticky_piston[facing=east,...]` into behaviour automatically, so an
  arbitrary schematic runs without hand-registration.

### Redstone components
| Component | Verified facts |
|---|---|
| **Dust** | Synchronous settling, 15-block attenuation, locational default |
| **Redstone torch** | 2-tick delay, inverts support, NORMAL priority, burnout (8 turn-offs / 60t) |
| **Repeater** | delay×2 game ticks, 3-way priority (repeater priority via `shouldPrioritize`), locking |
| **Comparator** | compare/subtract with side inputs, fixed 2t, **priming** (strength-change scheduling), **container analog reads** (direct and through one conductor) |
| **Observer** | 2-tick pulse, watches facing side, ignores other sides, **emits from its back face only** (strongly — a conductor like slime re-emits it), pulses once on placement and again when moved, self-clears if it lands mid-pulse, updates the block in front *and its neighbours* on both edges |
| **Note block** | synchronous `powered` follow, block-event play (air above required), click cycles pitch 0-24 |
| **Redstone block / lever** | constant / toggled power sources |

### Pistons (the accuracy crux)
- Extend/retract via block events (phase 7, *same tick* as the trigger), with
  vanilla's **dispatch-time re-validation** — a queued extend whose power died
  is dropped; a retract whose power returned silently re-marks extended
- **12-block push limit** (verified: 12 moves, 13 doesn't)
- **Moving-block entities** — placeholders now, real states 2 ticks later (phase
  9); move writes are **silent** (no neighbour updates until landing, matching
  vanilla's flags), and landed blocks re-examine their world (how moved
  observers re-pulse)
- **Retraction travels too** — the base is a 2-tick `moving_piston` placeholder;
  in-flight heads are `finalTick`ed to air; pushed/pulled placeholders are
  always `type=normal`
- **Quasi-connectivity** — power read from the block above too, with vanilla's
  direction skips (facing at the base, Down at the probe)
- **Slime & honey adhesion** — drags on every face, dragged blocks start their own
  push lines, slime≠honey don't stick, immovable-anywhere cancels the whole push
- **Sticky pull** with adhesion; **short-pulse block dropping**
- Piston excluded from its own structure; block events deduplicate like
  vanilla's `ObjectLinkedOpenHashSet`

---

## What's still needed

Ordered roughly by leverage for the door-timing goal. Each item means **capture
first, then implement** — the coding is small once the trace exists.

### Near-term (unblocks real schematics)
- [x] **Run the manual engine end-to-end** — done, twice: the 21-tick placement
      cycle and a 55-tick padded run whose second activation starts from a
      player click. See `ROADMAP.md` §4b for everything that fell out of it
      (boundary time, silent moves, travelling retraction, observer emission
      direction, chunk-edge freezing).
- [x] **Note blocks** — captured and implemented (synchronous powered follow,
      block-event play, click cycles pitch).
- [x] **Player interaction** — `Simulation::use_block` + `on_used`, executing at
      the tick boundary where vanilla processes packets; `TraceCapture --use`
      captures the real click.
- [ ] **Buttons & pressure plates** — pulse lengths differ per material.
      **NEXT AGENT STARTS HERE** — capture first, as always.

### Redstone completeness
- [ ] Redstone lamp (has an off-delay), target block (analogue), dispenser/dropper
      (need container contents), rails, tripwire, daylight sensor, sculk sensor.
- [ ] Dust **locationality** — currently defaulting to vanilla-locational; verify
      against captures where alternate-current's non-locational model would differ.

### Larger subsystems (do not start until the differ makes regressions visible)
- [ ] **Block entities, properly** — moving pistons are deferred writes (correct for
      timing, not a real block-entity model). Needed once containers arrive.
- [ ] **Fluids** — water/lava flow, levels, the **flow vector field**. Prerequisite
      for entities, not an aside: water-stream item alignment *is* that field acting
      on entity motion.
- [ ] **Entities** — items (gravity, drag, flow response, merging, despawn), then
      minecarts (rails, momentum, powered rails), then armor stands. Float-based, so
      the differ's tolerance mode finally earns its keep. The trace format already
      defines entity events; the capture tool doesn't emit them yet.

### Cross-cutting
- [ ] **Wire the differ into the corpus runner** so `trace <name>.json` is a
      first-class expectation with `UPDATE_GOLDEN=1` re-record (conformance is
      currently a hand-written test per structure).
- [ ] **Throughput** — only once correctness is pinned. `TickQueue` BTreeMap →
      bucketed ring; `has_pending_at` linear scan; `resolve_push` `Vec::contains`;
      parallelism *across* simulations, never within a tick.
- [ ] **Product surface** — expose step/run/checkpoint through `src/bridge/` to all
      six languages, then build door-timing on top (actuate → run to quiescence →
      report tick counts + per-component timeline).

---

## Integration status with nucleation

- The engine is a **workspace sibling** of nucleation, not a dependency of it — by
  design, to protect the fast loop.
- `structure.rs` reads Java structure SNBT independently (a deliberate small
  duplication rather than a dependency edge).
- The bridge to nucleation proper is **litematic/schematic → SNBT → engine**, using
  nucleation's existing `to_structure_snbt`. That is the integration seam; the
  product surface (bridge bindings + timing) is the last roadmap item.

## Where the work lives

- `crates/mc-tick/ROADMAP.md` — sequenced work list (this doc's TODO in more detail)
- `crates/mc-tick/src/redstone_components.md` — the captured/bytecode facts per
  component, with the exact numbers and where each came from
- `tools/gametest/README.md` — how the oracle works and its hard-won gotchas
- `tools/gametest/capture.sh` — one-command trace capture (stage pack, fresh
  universe, run TraceCapture)
- Branch: `feat/mc-tick`
