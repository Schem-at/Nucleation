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
dashed ghost line follows the cursor in between. Width/type mismatches and
non-routable ports are refused with a readable toast, never a silent no-op.
`show IO` toggles the whole layer.

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

Clicking a greyed ✗ port promotes it on the spot and toasts what changed
("removed lever at (19,5,5); bin[0] now lands on dust at (0,5,-20)"). The
switch is **reversible** — Exec mode restores the original blocks byte-exactly,
which is why it is a switch and not a one-shot button — and it is remembered in
`.nucm`. A bus that terminated on the port is ripped when you flip it, and
named in the toast.

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

- **abstract** (default) — flat-shaded instanced cubes coloured by layer and
  block kind, per-bus colours, FAILED layers red. Rebuilds in milliseconds, so
  it is what you drag against.
- **textured** — nucleation's `meshing` feature turns the composited design
  into a GLB against a resource-pack ZIP you supply (file input, left panel);
  three.js loads it with `GLTFLoader`. Measured on a 1.4k-block design: pack
  load ~260 ms, mesh ~380 ms, ~900 KB GLB. Cached per document version, so
  orbiting is free and only a real edit re-meshes.

Bus layers stay abstract *on top* of the textured mesh — routing is what the
colours encode, and a resource pack cannot show it. Markers, gizmos and labels
draw in both views. `rendering` (wgpu) stays out of the wasm build; `meshing`
is enough and is already the package default.

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
npm run smoke                  # node-only: veneer + engine, no browser
npm run build && npm run verify # drives the REAL app in headless chromium
```

`npm run verify` exercises the same code paths the mouse and keyboard drive
(`window.__eda`), so green means the UI works, not just the engine: library
auto-load, instance ports (routable + refused-with-reason), selection and the
hint bar, `R` rotate, `Del` delete ripping its bus, click-to-connect, a bus
FAILED then healed, a typed poke through the routed chain (`u1.a=99 + u1.b=28
→ sum_out=127`), and all three export tiers. Results land in
`docs/verify-out.json`; screenshots in `docs/`.

The textured check needs a resource pack at the repo root (`pack.zip`, **not**
committed); it is skipped, not failed, when absent.

## Screenshots

`docs/01-demo-adder-instance-ports.png` … `08-textured-resource-pack.png`,
regenerated by `npm run verify`.

## Known rough edges

- **`Check` is never clean with community cells placed.** An ADD007 alone,
  with zero buses, produces 253 DRC violations (`floating`,
  `unattached_wall_torch`, `repeater_cycle`) inside its own body. That is a
  DRC-vs-community-hardware gap in the routing crate, not something this app
  causes or can fix; the demo therefore does not claim a clean check.
- Out of the box only `ADD007.sum` is routable across the enhanced library —
  every other port is bare levers/lamps/buttons with no adjacent dust. They are
  one **Bus**-mode click away; the ones that genuinely cannot be promoted (a
  ceiling lever, no room for a form adapter) hide the switch and keep their
  reason.
- The engine runs on the main thread. Meshing a 1.4k-block design is ~380 ms,
  and edits are milliseconds, so a worker was not worth the transfer cost; a
  much larger design would want one.
- Buses realize a single-level 2y-pitch stack: both endpoints must share a y.
  Moving an instance off that level reports FAILED with that explanation
  (which is what the verify script's failure case uses).
- The renderer does not yet share ONE mesh per cell across its instances
  (`THREE.InstancedMesh`). Dragging no longer re-meshes anything — the verify
  asserts 0 texture builds over 8 drag frames, and `viewer.meshBuilds` is the
  dev counter — but a port-mode toggle still rebuilds the whole textured scene
  rather than just that cell. Per-cell instancing is the performance pass.
