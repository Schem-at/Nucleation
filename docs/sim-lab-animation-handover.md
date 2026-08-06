# Sim Lab — animation correctness and renderer performance

Handover for a fresh context. Everything described here is committed and green;
the animation work in §3 is diagnosed but **not started**.

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
| `atlascheck.mjs <build>` | **are chunk atlases identical** — the guard on §4's cache |
| `chunkcmp.mjs <build>` | meshing cost at 16³ / 32³ / whole-build |
| `strokedump.mjs <build>` | per-tick changes vs flights created — the §3 evidence |

`verify-tps.mjs` reloads the build between rows on purpose: a flying machine
left running across a sweep travels thousands of blocks and grows its region,
so the last row would measure a world the earlier rows built.

---

## 3. THE TASK — animation correctness

Three reported symptoms, one root cause plus one contributing bug.

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

### Fix plan

1. **Retire a flight when its landing change arrives**, not when a timer
   expires. Single highest-value change: it makes a flight's lifetime exactly
   the interval the simulation says it is, killing both the double-draw and the
   gap.
2. **Launch a stroke as one group** — one start stamp and one duration shared
   by every block in it, so a rigid body stays rigid.
3. **Drive progress from tick count, not milliseconds**, so frame jitter and
   throttling cannot desync the picture from the simulation. This is the
   structural one and what makes it correct at any rate.
4. **Keep the destination cell empty until its flight retires**, rather than
   meshing the landed block the moment the change arrives.

(1) and (2) are contained in `launch` / `animate` / `applyChanges` in
`world.ts`. (3) touches the frame loop in `App.tsx` too.

Verify with `strokedump.mjs`: flights must retire on the same tick their
landing change arrives, and concurrent flights of one stroke must report
`distinct starts=1`. Then `verify-anim.mjs` for the slide itself.

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
