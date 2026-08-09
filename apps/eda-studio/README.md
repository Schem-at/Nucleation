# EDA Studio

A browser front end for nucleation's redstone EDA tier (`src/design.rs`): a
Photoshop-style layer stack of **cell instances**, **loose hardware** and
**routed buses**, edited on a 3-D canvas, checked (DRC/LVS/STA), baked, poked
through its typed contract, and exported.

Everything runs in the browser: the engine is the nucleation wasm build
(`bridge,simulation,mc-tick,routing,hdl,meshing`) plus the hand-written
`Design` veneer; Verilog synthesis is YoWASP yosys; textured rendering is
nucleation's own meshing pipeline.

## Run

```sh
# 1. build the engine (repo root) — routing + hdl + tick engine + meshing
NUCLEATION_WASM_FEATURES=bridge,simulation,mc-tick,routing,hdl,meshing \
  ./tools/package-npm.sh dist/npm-eda

# 2. run the app (this directory)
npm install
npm run dev          # syncs public/engine + public/cells, serves :8455
```

Open <http://localhost:8455/> and press **Load demo** (or `/?demo=1`).

## The interaction model

One selection, one mode, and a hint bar that always says what the next click
does. `Esc` always cancels back to idle.

| key | action |
| --- | --- |
| click | select an instance (raycast); click empty ground to deselect |
| drag | move the selected instance; buses reroute live |
| `R` / `⇧R` | rotate 90° / −90° in place; buses reroute, the gizmo shows the new axes |
| `G` | grab-move; click to drop, `Esc` puts it back |
| `Del` / `⌫` | delete the instance — buses that terminated on it are ripped and named |
| `Esc` | cancel placing/connecting/grabbing, or deselect |

The selected instance gets a highlighted bounding box, a translucent shell and
an **axis indicator** (red = local +X, blue = local +Z after `rot`).

### IO and wiring

Ports render as cones at their bit column, coloured by what they do:

- **green ▲ drives a bus** (a design input, or a *cell output*),
- **blue ▼ receives a bus** (a design output, or a *cell input*),
- **grey ✗ executor-only** — real IO that no bus can land on.

Each carries a floating label (`u0.sum : uint8`). Hovering highlights the port
and prints its geometry; **click a green ▲, then a blue ▼** to route a bus — a
dashed ghost line follows the cursor in between. Width/type mismatches are
refused with a readable toast, never a silent no-op. `show IO` toggles the whole
layer.

A grey ✗ target does **not** refuse: the click **auto-promotes** it and then
routes (see below). Only ports that genuinely cannot be promoted refuse, with
the engine's reason.

### Port modes: the thing that used to surprise everyone

Community cells name **executor hardware**: inputs are *levers*, outputs are
*lamps*. Nothing in redstone drives a lever, so `ADD007.a` was grey ✗ and
`add.sum → bcd.bin` was impossible however good the router got.

Every instance port now has a **mode**, shown as a two-state switch beside its
chip in the outliner:

- **Exec** — the shipped hardware. Hand-drivable through the typed executor,
  not routable.
- **Bus** — *promoted*: the lever is swapped for a driver stub ending in dust.
  Routable, no longer hand-drivable.

**You never have to press the switch to make a connection.** Clicking an
executor-only port as a bus target promotes it *as part of the connect gesture*
and names what it changed in one toast — `promoted u2.a: 8 lever → 16 stone + 8
redstone_wire + 8 repeater; bus2 routed: u1.sum → u2.a`. The switch is there for
manual control and, above all, for going **back**: Exec mode restores the
original blocks byte-exactly, which is why it is a switch and not a one-shot
button, and it is remembered in `.nucm`. A bus that terminated on the port is
ripped when you flip it, and named in the toast.

The verify script drives exactly that gesture (`u1.sum` → a fresh adder's lever
bank `a`) and asserts the four things that make it trustworthy: the mode flips
to Bus, the bus lands `routed`, the report names the conversion, and flipping
back restores the hardware.

Promotion also fixes **form**: a bus realizes a vertical 2y-pitch stack, and a
lot of community IO is a horizontal ROW (`BINTOBCD001.bin`'s levers march along
x at pitch 2). Promotion grows a staircase form adapter so the port presents the
canonical column. See `redstone-eda/DESIGN_SPEC.md` for the geometry and the
strong-vs-weak-power reason the strategy depends on the lever's face.

Verified end to end in the tick engine (`tests/design_promotion.rs`): the
canonical `add → binary-to-BCD → 7-segment` pipeline, both buses routed, 8/8
BCD values and 8/8 segment patterns exact.

## Instance ports (the engine feature this app needed)

`Design::place` used to register nothing connectable: `route_bus` only knew
*declared* design ports, so a placed cell's contract ports were unreachable.
Now instances expose them as first-class endpoints named `{instance}.{port}`:

```js
d.place("add0", "ADD007", [0, -1, 4], 0);
d.instancePorts();  // [{name: "add0.sum", role: "output", routable: true,
                    //   wires: [[15,2,5], ...], step: [0,2,0]}, ...]
d.routeBus("sum_bus", { driver: "add0.sum", sinks: ["sum_out"] });
```

Three things make that work (`src/design.rs`):

- **transform** — the cell's contract, mapped through the instance transform.
- **dust taps** — a contract names executor hardware (levers/lamps) while a bus
  lands on *dust*, so each bit's connection cell is the cell itself if it holds
  dust, else the first dust neighbour. No tap on every bit ⇒ `routable: false`
  with the reason, rather than a mis-route.
- **pin access** — a bus may enter the influence halo of the instances it
  terminates on (routing *into* what you are connecting to is what a pin is
  for); every other instance's halo still blocks, and hard body cells always do.

`resolve_port` returns DESIGN-facing direction, so a cell **output** resolves
to a driver. Directions are checked both ways: driving *out of* a cell input is
refused by name.

## Renderer

Two views, one scene (toggle: **textured**):

- **abstract** (default) — flat-shaded voxels coloured by layer and block kind,
  per-bus colours, FAILED layers red. This is what you drag against.
- **textured** — nucleation's `meshing` feature turns the composited design
  into a GLB against a resource-pack ZIP you supply (file input, left panel);
  three.js loads it with `GLTFLoader`. Measured on a 1.4k-block design: pack
  load ~260 ms, mesh ~200 ms, ~900 KB GLB. Cached per document version, so
  orbiting is free and only a real edit re-meshes.

Bus layers stay abstract *on top* of the textured mesh — routing is what the
colours encode, and a resource pack cannot show it. Markers, gizmos and labels
draw in both views. `rendering` (wgpu) stays out of the wasm build; `meshing`
is enough and is already the package default.

### One mesh per cell (GPU instancing)

An instance is a cell **reference** plus a transform, and the renderer is built
so nothing in the pipeline forgets that.

- A **cell variant** — a library cell plus whatever port-mode patches are in
  effect — is meshed exactly **once**, into a single buffer geometry with each
  block's colour as a **vertex attribute**. Colour-as-attribute, not
  colour-as-material, is what gets a cell to one draw call: an ADD007 body uses
  seven block colours, and a material per colour meant seven meshes per cell.
- Every placement of that variant is a row in **one `THREE.InstancedMesh`**.
  A drag or a rotate is therefore **one `Matrix4` write** — the engine's
  placement (`at + R_y(rot)` about the cell's min corner) *is* an affine matrix,
  so the four rotations are four matrices over the same mesh, not four meshes.
- Selection never extracts an instance from its batch: the highlight is a
  separate gizmo overlay (translucent shell, axis arrows, bbox outline).
- **Buses and the loose base stay unique meshes** — no two buses are the same
  shape, so there is nothing to instance.
- Port cones are two `InstancedMesh`es (opaque routable, translucent blocked)
  with per-instance colour; instance outlines are one merged `LineSegments` plus
  a separate object for the selected box. Hover and selection write a colour and
  a matrix — they no longer rebuild the marker layer or its DOM labels.

**10 placements over 3 distinct cells: 3 meshed cells, 3 instanced groups,
7 draw calls** (0.7 per instance). Asserted, with the mesh-build and wasm-read
counters, by `npm run verify`.

### The scene only re-reads what changed

`Studio.scene()` returns the document model plus an explicit statement of what
changed, and re-reads only that out of wasm:

| edit | what leaves wasm |
| --- | --- |
| drag / rotate an instance | nothing (matrix writes only) |
| a bus re-routes | that bus's region |
| a port-mode toggle | that one instance's region (a new variant) |
| place a cell | that cell's own schematic, once, ever |
| `setBlock` on the base | the loose layer |

The loose layer is read from flatten's own non-`inst:`/`bus:` regions rather
than by subtracting instance blocks from a full-document dump — that dump was
36 ms at 2.5k blocks and grew with every placed cell.

### Numbers

`npm run profile` writes `docs/profile-<tag>.json`; `docs/profile-before.json`
and `docs/profile-after.json` are this pass, on the two-adder demo grown to 12
instances (headless chromium, software GL):

| | before | after |
| --- | --- | --- |
| drag: main-thread work per frame | 431 ms | **0.05 ms** |
| ...of which scene extraction | 339 ms | 0.45 ms renderer + 22 ms engine |
| cell re-meshes over 30 drag frames | 31 | **0** |
| re-route latency | 436 ms | **104 ms** |
| port-mode toggle | 1462 ms | **140 ms** |
| demo load to first render | 2107 ms | 1683 ms |
| JS heap | 87.5 MB | **33.5 MB** |
| draw calls, 10 instances / 3 cells | 26 | **7** |

The pointer path is now free: with live re-route off, a drag is 0.05 ms a frame
and bounded only by the display. The fixed **250 ms live-reroute throttle is
gone** — the gesture previews on the GPU immediately and the document commit is
coalesced to at most one per animation frame, so the engine runs as often as it
can finish instead of on a timer.

**What is still slow, and why.** With live re-route ON the drag is bounded by the
engine, not the renderer: ~88 ms per committed frame, of which ~22 ms is
`Design::flatten()` (it materializes every instance's blocks into regions just so
the app can read one re-routed bus back) and the rest is the router. Two things
would fix it, both outside this app:

1. a `Design::bus_blocks_json(name)` (or `flatten_buses()`) so a live drag reads
   one bus instead of flattening the document — would take that 22 ms to ~1 ms;
2. moving the engine into a **Web Worker**, which would take the router's ~65 ms
   off the main thread entirely. Not done here: the measured bottleneck was
   marshalling volume, not thread contention (the same 88 ms would still be 88 ms
   in a worker, just not blocking paint), and every `Studio`/`window.__eda` call
   plus the whole verify script is synchronous against the engine, so the port is
   a larger change than this pass.

## Export tiers

| button | tier | what it contains |
| --- | --- | --- |
| `.schem` | artifact | every layer composited into one region + the merged contract |
| `.litematic` | interchange | same blocks, `inst:*` / `bus:*` regions preserved |
| `.nucm` | project | the full document: cell references, transforms, bus states, gates |

**Fixed here:** `.schem` export silently dropped whole layers. `flatten()`
keeps named regions, but the region merge mirrors
`UniversalSchematic::get_block`, which answers from the *default* region first
whenever a coordinate lies inside its (dense) bounding box — so a bus fragment
threading the endpoint hardware's own bbox read back as air. A 1091-block
design exported as 739 blocks. `Design::flatten_composite()` now composites the
stack explicitly (topmost non-air wins) before writing, and the verify script
asserts `.schem` block count == flattened count. The underlying
`get_block`/`get_merged_region` precedence is still inverted for layered
documents — a core issue beyond this app.

## Verification

```sh
npm run smoke                   # node-only: veneer + engine, no browser
npm run build && npm run verify # drives the REAL app in headless chromium
npm run profile                 # -> docs/profile-current.json
EDA_PROFILE_TAG=before npm run profile   # tag a baseline to diff against
```

`npm run verify` exercises the same code paths the mouse and keyboard drive
(`window.__eda`), so green means the UI works, not just the engine: library
auto-load, instance ports (routable + refused-with-reason), selection and the
hint bar, `R` rotate, `Del` delete ripping its bus, click-to-connect, a bus
FAILED then healed, a typed poke through the routed chain (`u1.a=99 + u1.b=28
→ sum_out=127`), all three export tiers, **auto-promotion** on connect (and the
reverse), and the **performance contract**:

| assertion | why it is the right thing to assert |
| --- | --- |
| 20 drag frames ⇒ 0 cell re-meshes, 0 block dumps out of wasm | a drag is a transform; if it re-reads anything, the design has slipped |
| ...at ≤ 33 ms of main-thread work per frame | the ≥ 30 fps target, stated as a budget, not as a headless frame rate |
| K placements of one cell ⇒ 1 mesh build, 1 instanced group | the instancing claim itself |
| ...read out of wasm once, from the CELL | no per-instance region dumps |
| port-mode toggle ⇒ exactly 1 cell variant re-meshed | the only edit that changes a placed cell's blocks |
| one bus re-route ⇒ exactly 1 bus re-meshed, 0 cells | bus dirtiness is per bus, from the engine's own report |
| live re-route commits ≤ 1 engine move per animation frame | there is no fixed throttle left to drift out of date |
| 10 placements / 3 cells ⇒ the draw-call count | the number a reader can check |

The counters behind these are `viewer.meshBuilds` (`cells`, `instancedGroups`,
`matrixWrites`, `buses`, `loose`, `texture`) and `studio.sceneReads`
(`flatten`, `cellDump`, `instDump`, `busDump`, `looseDump`), both exposed on
`window.__eda`. Results land in `docs/verify-out.json`; screenshots in `docs/`.

The textured check needs a resource pack at the repo root (`pack.zip`, **not**
committed); it is skipped, not failed, when absent.

## Screenshots

`docs/01-demo-adder-instance-ports.png` … `09-instanced-10-placements.png`,
regenerated by `npm run verify`.

## Known rough edges

- **`Check` is never clean with community cells placed.** An ADD007 alone,
  with zero buses, produces 253 DRC violations (`floating`,
  `unattached_wall_torch`, `repeater_cycle`) inside its own body. That is a
  DRC-vs-community-hardware gap in the routing crate, not something this app
  causes or can fix; the demo therefore does not claim a clean check.
- Out of the box only `ADD007.sum` is routable across the enhanced library —
  every other port is bare levers/lamps/buttons with no adjacent dust. Connecting
  to one promotes it automatically; the ones that genuinely cannot be promoted (a
  ceiling lever, no room for a form adapter) hide the switch and keep their
  reason.
- **Promoting a port breaks `.litematic` export for the rest of the document's
  life.** `Design::to_litematic` then fails with `NucleationError.Serialize`,
  and flipping the port back to Executor does *not* un-break it. `.schem` and
  `.nucm` are unaffected. Reproduced standalone against the engine (nothing this
  app does); the verify script therefore runs its export checks before its
  promotion checks. Engine bug, needs a fix in `src/design.rs`.
- The engine runs on the main thread. That no longer costs the drag gesture
  anything (see *Renderer → Numbers*), but a live-re-route drag and the
  200 ms textured re-mesh are still main-thread stalls a Web Worker would hide.
- Buses realize a single-level 2y-pitch stack: both endpoints must share a y.
  Moving an instance off that level reports FAILED with that explanation
  (which is what the verify script's failure case uses).
- The textured view re-meshes the WHOLE design on a port-mode toggle, not just
  the cell that changed — `meshGlb()` goes through `flattenComposite()`, which
  has no per-layer entry point. The abstract view does do it per cell.
