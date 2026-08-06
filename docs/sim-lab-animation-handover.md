# Sim Lab — animation correctness and renderer performance

Handover for a fresh context. §3 (animation correctness) is **done** — see the
"What was done" subsection there for what landed and what deliberately did not.
§4's remaining work (a pack-wide atlas) is still open.

---

## 0. Nucleation, and what the lab is for

**Nucleation is the product. The lab is a wrapper around it.**

Nucleation is a Rust library for Minecraft builds — read and write every
schematic format, mesh them, diff and fingerprint them, segment them out of
worlds, and (via `crates/mc-tick`) simulate them tick-for-tick against vanilla.
It ships to Python, JS/wasm, JVM, PHP and C through generated bindings
(`src/bridge/` → `bindings/`), and vanilla parity is the thing it sells.

`apps/sim-lab-wasm/` is a **demo and a test harness**: the most convincing
proof that the library works, and the fastest way to see a machine misbehave.
It is not where behaviour should live.

### The rule this implies

**Any capability worth having belongs in the library, not the app.** If the lab
needs something, the question is not "how do I do this in TypeScript" but
"what is missing from Nucleation, and why has no other consumer needed it yet".

Concretely, a change lands in the library when:

- it is knowledge about *what the simulation did* (the app must not re-derive
  it — see §3, where exactly this went wrong),
- another binding would plausibly want it (Python, JVM, CLI),
- or it is logic that could silently drift from the engine's truth.

A change may stay in the app only when it is genuinely presentation: camera,
input, colours, DOM.

Library work carries library obligations, in this order:

1. a Rust test that fails without it,
2. exposure through `src/bridge/` so *every* binding gets it, not just JS,
3. regenerated bindings,
4. the app rewritten to consume it rather than keeping its own copy.

The app-side workaround in §4 (`atlasCache`) is deliberately *not* an exception
to this — see the note there. It is a stopgap with a known proper home.

---

## 1. What the sim lab is

`apps/sim-lab-wasm/` — a browser app: drop a schematic, fly through it, click
its levers, watch `mc-tick` run. No backend. React + three.js over the
Diplomat-generated wasm bindings.

Files that matter:

| Path | What |
|---|---|
| `src/world.ts` | schematic + simulation + chunked three.js scene; meshing, flights, entities |
| `src/App.tsx` | render loop, tick pacing, HUD, controls |
| `src/mesher.ts` | `loadMeshEnv()` — engine, resource pack, mesh config |
| `src/player.ts` | camera, pointer lock, raycast |

### Running it

```sh
cd apps/sim-lab-wasm
npx vite preview --port 8455 --strictPort      # serves the built dist/
```

**Use `http://localhost:8455`, not `http://127.0.0.1:8455`.** Vite binds IPv6
localhost only; the IPv4 address is refused and reads exactly like a broken
build. Two probes wasted a cycle on this before being fixed.

`vite preview` serves the *built* output, so `npm run build` after any `src/`
edit. `npm run dev` if you want hot reload instead.

### Rebuilding the wasm engine

Only needed when Rust changes (`crates/mc-tick`, `src/bridge`):

```sh
NUCLEATION_WASM_FEATURES=bridge,simulation,meshing,mc-tick ./tools/package-npm.sh dist/npm-mctick
cd apps/sim-lab-wasm && npm run sync-engine && npm run build
```

The feature list is not optional — `package-npm.sh` defaults to
`bridge,simulation,meshing` and **omits `mc-tick`**, producing an engine with
no simulation. `sync-engine` checks for `machineGraphJson` and fails loudly if
you got it wrong.

### Fixtures

| Build | Path | Why |
|---|---|---|
| BB (flying machine) | `~/Library/Application Support/PrismLauncher/instances/26.2/minecraft/schematics/BB.litematic` | the animation and perf workhorse; matches vanilla exactly over 400 ticks |
| Elevator Decorated | `~/Downloads/Elevator Decorated.litematic` | 384 entities, 168 leashed boats |
| doors | `crates/mc-tick/tests/corpus/litematics/*.litematic` | **all have 0 entities** — checked all seven |

BB does nothing until kicked: `sim.placeBlock(31, 7, 13, "minecraft:air")`.
A build left quiescent retires empty ticks and any number measured on it is
meaningless — see §4.

---

## 2. Probes

All in `apps/sim-lab-wasm/`, all playwright, all expect the server on 8455.
Run them from that directory — `node` resolves playwright out of its
`node_modules`, and cwd drift bit me repeatedly.

| Probe | Answers |
|---|---|
| `verify-anim.mjs <build>` | do piston strokes slide, or teleport |
| `verify-entities.mjs <build>` | are entities drawn, sized right, in the right place |
| `verify-tps.mjs <build> [--break x,y,z]` | target vs achieved tps across the slider; where it throttles |
| `profile-remesh.mjs <build>` | ms/tick split across step / applyChanges / flush |
| `verify-handover.mjs <build>` | **is every cell drawn by something** — flight→chunk hand-off, and flights with no mesh |
| `atlascheck.mjs <build>` | **are chunk atlases identical** — the guard on §4's cache |
| `chunkcmp.mjs <build>` | meshing cost at 16³ / 32³ / whole-build |
| `strokedump.mjs <build>` | per-tick changes vs flights created — the §3 evidence |

`verify-tps.mjs` reloads the build between rows on purpose: a flying machine
left running across a sweep travels thousands of blocks and grows its region,
so the last row would measure a world the earlier rows built.

---

## 3. Animation correctness — DONE

Three reported symptoms, one root cause plus one contributing bug. The
diagnosis below is kept because it is the evidence; what was actually built is
at the end of the section.

### Symptoms

- blocks disappear mid-animation
- the piston head detaches from the block it is pushing
- blocks phase through one another

### Evidence

`strokedump.mjs` on BB, one stroke:

```
tick 1: FLIGHT piston_head  33,6,13->32,6,13  start=3427 dur=2000
        FLIGHT slime_block  31,8,13->30,8,13  start=3440 dur=2000
        >> 8 in air; distinct starts=6 durs=1 DESYNCED

tick 3: change 32,6,13 -> piston_head      <- the block LANDS here
        change 31,9,13 -> redstone_block
        >> 8 in air                         <- ...while still flying toward it
```

### Root cause

**Flight lifetimes are wall-clock; landings are decided by the simulation;
nothing ties the two together.**

`applyChanges(changes, moveSeconds)` launches a flight per `moving_piston`
change with `dur = moveSeconds * 1000` and `start = performance.now()`.
The simulation writes the real block into the destination on its own schedule
(tick 3 above). Whichever finishes first produces the artefact:

- **sim lands first** → the chunk re-meshes with the real block while a copy is
  still animating into the same cell → two of them → *phasing*
- **flight expires first** → source already cleared, destination not yet
  written → *disappearing*

### Contributing bug

Eight flights of one rigid stroke got **six different start stamps** spanning
16 ms, because `performance.now()` is sampled inside each `launch()` call. A
body vanilla moves as one unit is animated as N independent objects, so it
shears — that is the detaching head.

### Fourth, separate

Whenever more than one tick is stepped in a frame, `App.tsx` passes `move = 0`
and strokes teleport outright. This is why it looks worse the faster you run.
See the `steps === 1` condition in the frame loop.

### The deeper problem — and why the fix is library work

Read §0 first. The lab is **re-deriving the simulation's behaviour from its
output**: `applyChanges` regex-matches `moving_piston[facing=…]` out of the
block-change stream, guesses the source cell as `pos - facing`, and reads what
was there out of a schematic mirror that lags a batch behind. That is a
reimplementation of piston mechanics in TypeScript, sitting downstream of an
engine that models them exactly.

And the engine already knows. `crates/mc-tick/src/timeline.rs`:

```rust
/// One successfully dispatched piston stroke.
pub struct PistonEvent {
    pub tick: u64,
    ...
}
pub enum PistonAction { /* extend, retract (pulling when sticky) */ }
```

Authoritative, from the simulator itself, with the tick it was dispatched on —
so it knows when a stroke starts *and* when it lands. **It is not exposed
through the bridge**: `grep -n 'Timeline' src/bridge/mc_tick.rs` returns
nothing. The lab guesses because nobody gave it the answer.

Every symptom in this section follows from that. Wall-clock durations exist
because the app has no tick to land on. Six start stamps exist because the app
sees N independent block changes where the engine dispatched one stroke. The
teleport at speed exists because a batch of changes carries no stroke structure
at all.

### What was done

The plan said "expose the timeline's `PistonEvent`s". What landed instead is
**`Simulation::moving_blocks()`** — the engine's list of blocks it currently
has in flight. It is the same principle applied one step closer to the
question: a stroke still leaves the consumer to work out *which cells* it
carries, and `PendingMove` — the engine's own in-flight record — already knows
each cell, the state that will land there, its sweep direction and the tick it
resolves on. Nothing is re-derived, and there is no stroke-reassembly step in
between.

**A retracting piston does not move** — its arm comes home and its body stays
put. This was got wrong first, so the sources are recorded here.

- The **server** jar (`deof-mc`, `PistonMovingBlockEntity`) only has the offset
  math: `getExtendedProgress = extending ? f - 1 : 1 - f`, applied along the
  BE's `direction`, which `PistonBaseBlock.triggerEvent` sets to the piston's
  **facing** for both extension and retraction. Read alone, that says the
  `movedState` — the retracted base — slides from the head slot into the base
  cell. It is not enough to answer the question.
- The **client** jar (26.2, `net.minecraft.client.renderer.blockentity`
  `.PistonHeadRenderer`) settles it. `extractRenderState` has a branch for
  `isSourcePiston() && !isExtending()`: it puts a **`piston_head`** (TYPE from
  the base, FACING from the base, SHORT from `progress >= 0.5`) in the moving
  slot, and the base with **`EXTENDED=true`** in a separate `base` slot. Then
  `submit` does `pushPose → translate(offset) → submit(block) → popPose` and
  submits `base` **outside** that translate. Only the arm is interpolated.
- G4mespeed's `GSPistonHeadRendererMixin` corroborates the shape of that method
  (it `@ModifyConstant`s the `4.0f` short-arm cutoff to `0.5f`); its real
  contribution is elsewhere — sub-tick interpolation and
  `cPistonRenderDistance` for block entities beyond normal BE render range,
  which is a problem the lab does not have.

So the engine has to report what *travels* separately from what *lands*, or
every consumer slides the whole piston body a block backwards.

**Library** (`crates/mc-tick/`):

- `PendingMove` gained `started_on`, so a move carries its whole window and not
  just its end. Nothing in the engine reads it; a consumer interpolating one
  cannot do without it.
- `Simulation::moving_blocks() -> Vec<MovingBlock>`: `to`, `from`, `state`,
  `carried`, `remains`, `travel`, `extending`, `started_on`, `lands_on`,
  `source_piston`. Deferred writes with no `Sweep` are excluded — those are
  delayed writes, not motion.
- `carried` is what to **draw travelling** and `remains` what to **park at
  `to`** for the whole move. They equal `state` / `None` for everything except
  a retracting source piston, where they are the `piston_head` and the
  `EXTENDED=true` body — vanilla's two render slots. `PendingMove` carries
  them from `defer_source_retracting`, so the piston states the answer rather
  than a consumer reassembling a head state out of a base's.
- `carried_short` is the arm with `short=true`. The game draws a moving head
  **shortened while it is within half a block of its body** and full length
  beyond that; without it the shaft is visibly drawn through the back of the
  piston as the arm comes home. `PistonHeadRenderer` writes that as two
  *opposite* comparisons — `SHORT` at `progress <= 0.5` on the extension branch
  (`fcmpg; ifgt`) and at `progress >= 0.5` on the retraction branch
  (`fcmpl; iflt`) — which is one rule read from each end, since an extension
  travels away from the base and a retraction back to it. Choosing between the
  two forms is the mover's job (it holds the sub-tick progress); naming the
  states is the engine's. `piston_head[…,short=true,…]` is interned as a
  companion purely so it can be handed over — the simulation never places it.
- `Dir::name()`, because the CLI already had a private copy of it.
- `crates/mc-tick/tests/moving_blocks.rs` — 5 tests, including
  `a_retracting_piston_walks_its_head_home_and_leaves_its_body_where_it_is`.
  The one that matters most is
  `a_flight_lives_until_its_block_lands_and_not_past_it`: it walks the whole
  window asserting the destination is unwritten for exactly as long as the
  flight is listed. That is the double-draw and the gap, pinned from both
  sides.
- **Tick frame of reference**, which bit twice while writing those tests:
  `tick_count()` counts *completed* ticks, while `started_on`/`lands_on` name
  the tick that was executing. A flight dispatched during the step that takes
  `tick_count` 0 → 1 has `started_on == 0`. It is listed while
  `tick_count() <= lands_on` and gone after — so its last listed tick is the
  one where it is visually home but the world still holds the placeholder,
  which is vanilla's `progressO` reaching 1.0 on the landing tick.

**Bridge**: `TickSimulation::moving_blocks_json()`, bindings regenerated — all
five. Binding-level tests added to `tests/node_mc_tick_sim_test.mjs` (19/19)
and `tests/python_mc_tick_sim_test.py`, including the invariant that
`remains != null` exactly when a flight is a retracting source piston.

**App**: `world.ts` lost the `moving_piston` regex, the `FACING` table and the
schematic-mirror source lookup. `flights` is now a `Map` keyed by destination,
reconciled against `movingBlocksJson()` in `syncFlights()`; a flight draws
`carried` on the interpolated path and `remains` parked at `to`, mirroring
vanilla's two slots. `animate()` takes a **fractional tick**
(`tickCount + carry`) instead of `performance.now()`.
`applyChanges` lost its `moveSeconds` parameter and `App.tsx` lost the
`steps === 1` guard — a stroke is now drawn correctly whether one tick or fifty
were stepped, so there is nothing left to guard.

**A cell is not a flight's identity.** The first version of `syncFlights` keyed
the flight map on the destination cell and skipped any cell it already held —
`if (this.flights.has(k)) continue`. Consecutive strokes reuse a destination: a
sticky piston extends its head into a slot and, two ticks later, retracts and
pulls a block back into that same slot. The reconciler kept the head flight, so
the pulled block was **never drawn while in flight** and the head stayed on
screen **past its landing**, beside the real one. That is precisely the
"blocks disappear on retractions / piston heads get duplicated" report, and on
BB it happens dozens of times in 60 ticks. Reconciliation now compares
`started` + `from` + `carried` and replaces the flight when any of them change.

**A cell must be drawn by something at every painted frame.** Two separate
ways it was not, both reported as "the meshes blink a bit":

- **The hand-off.** A flight retires on the tick its block lands — exact, and
  the simulation's decision. The chunk that must then draw that block is
  re-meshed *asynchronously*, one `MeshResult` at a time. Disposing the flight
  at retirement left the cell drawn by nothing in between: **1446 of 4965
  retirements on BB, up to 8 painted frames each.** `World.retire` now parks
  the meshes at the destination instead, and `flush` releases them in the same
  turn as the chunk swap — so no frame shows both and none shows neither. A
  flight whose cell is genuinely empty (a stroke cancelled mid-flight;
  `finalTick` lands air for a source piston) is still dropped at once.
- **A flight with no mesh yet.** A one-block mesh is a GLB parse, and until it
  finished the flight was not drawn *at all* — the cell blank for the whole
  stroke, since the source has already been cleared. **601 of 5484 flights.**
  `warmBlockMeshes` pre-parses every state in the world snapshot at load.
  Note it is **not** `paletteJson()`, which was tried first and is wrong: that
  returns bare block *names* with no properties, so all 155 of its entries were
  cache keys no flight could ever match — 155 parses for zero hits. Flights key
  on the full state string. `warmState`, called on every change, then catches
  what the simulation invents later (`piston_head`, `observer[powered=true]` —
  in no initial state anywhere). Residual: **26 of 5346 (0.5%)**, the first
  flight of each such derived state, bounded by state count not by runtime.

`strokedump.mjs` guards the reconciler: for every flight the engine reports,
the app must be drawing *that* flight in that cell. `verify-handover.mjs`
guards both of the above. It also counts overlaps from each cell's
**last** write in a batch rather than every write — `applyChanges` replays a
batch in order and re-meshes once, so an intermediate state (a head landing in
a slot the same tick's retraction turns straight back into a placeholder) is
never on screen and must not be counted as a double-draw.

Measured on BB (`strokedump.mjs`), against the "8 in air; distinct starts=6
DESYNCED" in the evidence above:

```
0 overlaps, 0 ticks with desynced flights, 0 stale/missing flights
>> 38 launched together; distinct starts=1
```

(also clean over a 250-tick sweep, which is where the stale-flight bug showed
up — 14 ticks was too short to catch it.)

`verify-anim.mjs`: `piston_head[facing=west] [33,6,13] -> [32,6,13] ticks 1..3`,
sliding, and the arm sampled at 0/¼/½/¾/1 of its stroke reads `SSSll`
extending and `llSSS` retracting — vanilla's flip, from both ends. Note the app
no longer special-cases the head slot; the engine names `piston_head` itself.

### What was deliberately not done

The CLI's animated-GLB path still projects `PistonEvent`s into
`schematic_mesher::Timeline`, and the mesher works out the carried cells at its
end. So the browser and the CLI *still* reach strokes by two routes — the
handover called that out and it remains true. Closing it means changing the
Timeline format in `../Schematic-Mesher` to carry cells rather than infer them,
which is a push to another repo (see §4's note) and a bigger change than the
symptoms justified. `moving_blocks` is the right source for it when someone
does.

### Verifying it again

`strokedump.mjs <build>` — must print `0 overlaps, 0 ticks with desynced
flights`; both probes were rewritten for the new flight shape. Then
`verify-anim.mjs <build>` for the slide. The Rust and binding tests are what
actually guard it; the probes need a browser and a human.

---

## 4. Renderer performance — done, and what remains

### Done

Re-meshing one chunk took **293 ms**. Measured, not guessed:

```
parse WITH embedded PNG    367.1 ms
parse WITHOUT the PNG        0.5 ms
-> image decode = 366.6 ms of every chunk re-mesh
```

Every chunk's GLB embeds a texture atlas and the browser re-decoded it on
every re-mesh. Fixed by caching decoded materials **keyed on the atlas bytes**
(`atlasCache` in `world.ts`), parsing cache-hit chunks with the image stripped
out of the GLB container and re-attaching the cached materials.

**293 ms -> 40 ms per chunk. 1 tps -> 9 tps of fully-drawn simulation.**
89.6% hit rate over 30 distinct atlases. Geometry byte-identical.

> The obvious version of this — decode once, share the materials globally — is
> **wrong**. The atlas is built per chunk from the block types that chunk
> contains, so six chunks of one flying machine embed six different atlases.
> `atlascheck.mjs` is that guard; run it if you touch this.

**This is a stopgap living in the wrong place, by §0's rule.** `atlasCache` and
`stripEmbeddedImages` in `world.ts` are the app compensating for the mesher
emitting a redundant atlas per chunk — every consumer that meshes incrementally
would need the same trick, and only this one has it. It earns its place for now
because it is a 7× win available today against a change in another repo, but it
should be deleted, not maintained: a pack-wide atlas removes the reason it
exists. Do not build on it.

Chunk size is now a setting (16³ / 32³ / 64³ / whole build), because it trades
geometry re-meshed against number of distinct atlases and the balance depends
on the build. On BB, per tick of meshing: **183 ms @16³, 110 ms @32³, 75 ms
whole-build**. A large static build will likely invert this.

The chunk grid is **already unbounded** — coordinates are `floor(x / size)`
with no clamp, so it follows a machine that travels. "Whole build" is a
starting size, not a boundary: BB flew past its own build and grew a second
box. Do not drop the chunk abstraction to force a single mesh; it would
re-mesh a 54k-block ship in full every tick to solve a problem that belongs to
the atlas.

### Remaining: a pack-wide atlas

The real fix for the residual 40 ms (which is almost entirely the ~10% of
parses that still miss).

- Lives in **`../Schematic-Mesher`** (local clone present), pinned by git rev
  in `Cargo.toml:177`.
- Relevant: `src/atlas.rs` (`AtlasBuilder`, `TextureAtlas`), `src/mesh_output.rs`,
  and `set_atlas_max_size` already exists in `src/wasm.rs`.
- Build the atlas from the **pack**, not the chunk's block set; emit it once
  and have every chunk reference it.
- Takes hit rate to 100% and makes chunk size a pure geometry tradeoff.
  Re-mesh would drop to ~21 ms (mesh + base64 round-trip).
- Workflow: edit clone -> push -> bump rev in `Cargo.toml` -> rebuild wasm.
  **Ask before pushing to that repo.**

Beyond that, the next tier is avoiding the base64/GLB round-trip entirely by
exposing raw vertex buffers across the bridge and building `BufferGeometry`
directly. Geometry parse is only 0.5 ms, so this is worth maybe 2-4 ms/chunk —
do the atlas first.

---

## 5. Also live in the lab

- **Entities are drawn** (`syncEntities` in `world.ts`): boats, carts, riders,
  items, as translucent boxes at their **measured hitboxes** — the engine knows
  an entity's box and kind and nothing else, and the box is the volume that
  decides whether a piston can shove it.
- **Leashed entities get a brighter outline and no rope.** The leash *target*
  is discarded at parse time on purpose: a litematic keeps a fence knot's
  source-world coordinates while storing the entity relative to its region, so
  the anchor cannot be trusted. Reversing that is a parser decision
  (`crates/mc-tick/src/structure.rs`, the `"leash" | "Leash"` arm), not a
  renderer one.
- **Rate slider is log**: linear 0.1-20 tps over the bottom 60% of travel,
  exponential to 20k over the top 40%, last notch uncapped. Ticks run against
  a 10 ms frame budget. Target and achieved are separate readouts because the
  gap between them is the answer.

Current tps on BB: exact to 112, degrading by 632, throttling from 3.6k,
sustaining ~590 — against an engine ceiling of 11,100. Note that ~590 is
*simulation* rate; only ~9 tps of it is fully drawn. Closing that gap is what
§3 and §4 are for.

---

## 6. Timeline and GLB export

Exists and works — **CLI only, not in the browser**.

- `crates/mc-tick/src/timeline.rs` — data model, cycle detection
- `crates/nucleation-cli/src/commands/animate.rs` -> `src/meshing/mod.rs`
  `to_animated_glb`

Gated behind `--features mesh`, which is why `animate --help` prints nothing
on a default build. Proven end to end:

```sh
cargo run --release -p nucleation-cli --features mesh -- \
  animate crates/mc-tick/tests/corpus/structures/flying_machine_east.snbt \
  --ticks 60 --place 2:2,1,1=minecraft:redstone_block \
  --place 4:2,1,1=minecraft:air --timeline-json tl.json -o fly.glb
# -> fly.glb (260,752 bytes)
#    cycles: exact 0..7 period 7 drift (0,0,0)
#            translated 5..15 period 10 drift (-1,0,0)
```

Range selection: `--range`, `--between-actions N`, `--cycle`. Note the action
syntax is `--place TICK:X,Y,Z=STATE` (equals, not colon).

**Not wired into the lab.** The bridge already carries what it would need
(`updates_json_between`, `updates_heat_json`).

---

## 7. Gates and rules

```sh
cargo test --workspace --features mc-tick --tests --lib   # 1546 passing
cargo build --features mc-tick --examples                 # NOT covered by the above
cd apps/sim-lab-wasm && npm run build
```

`--tests --lib` skips examples, which is how `examples/scenario_inspect.rs`
rotted unnoticed after its `#[path]` target became the `mc-test` crate. Run
both.

- **Vanilla parity is the product.** Never modify a test, fixture, expectation
  or benchmark assertion to make a change pass.
- **No `Co-Authored-By` or generated-by trailers on commits.**
- `tools/gametest/pack/data/nucleation/structure/cca32.snbt` and `cca64.snbt`
  are pinned deliberately: they record that *vanilla itself* gives the "wrong"
  answer on the 8-bit adder's upper bits. Not bugs to fix.

---

## 8. Session commits

| | |
|---|---|
| `9a68c52b` | chunk size a setting |
| `771151ac` | atlas cache — 293 ms -> 40 ms per chunk |
| `902425d4` | log rate slider + achieved-rate readout |
| `d78a572e` | entities drawn in the lab |
| `fcb71eef` | `scenario_inspect` fixed onto `mc-test` |
| `20daf415` | piston slide animation |
| `debc5959` | CLI `animate` / `io` / `mesh` / `pack` / `man` |
| `83db3087` | entity NBT into gametest SNBT |
| `a4241fdf` | bridge tick-window queries |
| `ad1e7576` | world_segment scoring/tiers |
| `b6f1f0a5` | mc-tick timeline + boats and leashes |

---

## 9. Measurement traps hit this session

Recorded because each cost real time and each looks like a result.

- **A quiescent build reports ~10M tps.** That is 100 ns/tick — not throughput,
  but the cost of asking an idle sim ten million times whether it has anything
  to do. Kick the machine; check what fraction of ticks had work pending.
- **A profiling loop that leaks wasm objects** reported 326 ms/chunk and a
  1 tps ceiling while the app was measurably doing 600 tps. Measure the real
  loop in situ. If a probe disagrees with the app, trust the app.
- **A 250 ms measurement window** sees one tick or none at 2 tps, so the
  readout quantises to multiples of four and correctly-paced 2 tps displays as
  4. Wait for enough *ticks*, not enough time.
- **A sweep that leaves a flying machine running between rows** measures a
  world the earlier rows built.
- **`2>&1 >file`** sends stderr to the terminal. Correct is `>file 2>&1`.
- Bash calls do not share cwd reliably — use absolute paths or `cd` inside the
  same command.
