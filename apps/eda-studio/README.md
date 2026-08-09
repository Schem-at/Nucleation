# EDA Studio

A browser front end for nucleation's redstone EDA tier (`src/design.rs`): a
Photoshop-style layer stack of **cell instances**, **loose hardware** and
**routed buses**, edited on a 3-D canvas, checked (DRC/LVS/STA), baked, poked
through its typed contract, and exported.

Everything runs in the browser. The engine is the nucleation wasm build plus
the hand-written `Design` veneer; Verilog synthesis is YoWASP yosys; textured
rendering is nucleation's own meshing pipeline.

**Feature set** — `bridge,simulation,mc-tick,routing,hdl,meshing`:

| feature | what the app needs it for |
| --- | --- |
| `bridge` | the generated `src/bridge/` → `bindings/` surface (everything) |
| `routing` | `Design`, `route_bus`, the corridor router and its diagnostics |
| `simulation` + `mc-tick` | `bake()` → the typed executor behind the poke panel |
| `hdl` | `Hdl.compileBlif` for the Verilog → cell path |
| `meshing` | the textured view's GLB, against a resource-pack ZIP |

`rendering` (wgpu) stays **out**: `meshing` is enough, and it is the package
default.

## Run

```sh
# 1. build the engine (repo root) — routing + hdl + tick engine + meshing
NUCLEATION_WASM_FEATURES=bridge,simulation,mc-tick,routing,hdl,meshing \
  ./tools/package-npm.sh dist/npm-eda

# 2. run the app (this directory)
npm install
npm run dev          # syncs public/engine + public/cells, serves :8455
```

Open <http://localhost:8455/>.

### What you land on

The default page is **not an empty grid**: it loads the chain
`tests/design_promotion.rs` verifies end to end — **ADD007 → BINTOBCD001 →
NUMDISPLAY001**, both buses routed — frames it, and runs a dismissible 4-step
coach over it (*this is a routed design* → *click a port to start a bus* →
*R rotates, Del deletes* → *Exec/Bus, then Bake and Export*). Dismissal is
remembered in `localStorage`. With no design at all, the canvas prints the four
things to do first, in order.

| URL | state |
| --- | --- |
| `/` | the verified chain, framed, coach on first visit |
| `/?chain=1` | the same, explicitly |
| `/?demo=1` | two ADD007 adders + an 8-bit lamp readout |
| `/?crossing=1` | the DESIGN_SPEC sketch: two crossing 8-bit buses |
| `/?empty=1` | nothing placed — the empty state, and what the benchmarks use |
| `/?coach=1` | force the coach open again |

## The interaction model

One selection, one mode, and a hint bar that always says what the next click
does. `Esc` always cancels back to idle — unwinding one layer at a time (modal
→ overlay → gesture → selection), never a dead key. Press <kbd>?</kbd> for the
full legend, in the app.

| key | action |
| --- | --- |
| click | select an instance (raycast); click empty ground to deselect |
| drag | move the selected instance; buses reroute live |
| `R` / `⇧R` | rotate 90° / −90° in place; buses reroute, the gizmo shows the new axes |
| `G` | grab-move; click to drop, `Esc` puts it back |
| `F` | frame the selection (or the whole design when nothing is selected) |
| `A` | frame the whole design |
| `Del` / `⌫` | delete the instance — **confirms first** if buses die with it, with the count |
| `⌘/Ctrl-Z` / `⇧⌘/Ctrl-Z` | undo / redo |
| `?` | keyboard + colour legend |
| `Esc` | cancel placing/connecting/grabbing, close an overlay, or deselect |

### Undo/redo

The design layer offers `rip` / `reroute` / `remove_*` / `set_port_mode` but no
document history, so the studio keeps an **operation journal**: every mutator
records its inverse. Place, move, rotate, delete, route, rip, delete-bus and
port-mode toggles are all reversible, and a **drag is one undo step** — the
sixty committed live-reroute frames coalesce into "where the gesture started"
→ "where it ended". Undoing the delete of a bus-carrying instance re-places
the body, re-applies its promotions and **re-routes its buses**; the verify
script asserts exactly that. `declarePort` and `addGate` have no engine inverse
and are deliberately *not* journalled rather than faked. Loading a demo clears
the history — a starting point is not an edit.

### Colour semantics, documented in the UI

One colour, one meaning, defined once as CSS variables and printed as a
swatch legend in the left panel *and* in the `?` overlay, so the canvas and the
panels cannot drift apart:

| colour | meaning |
| --- | --- |
| green ▲ | drives a bus — start one here |
| blue ▼ | receives a bus — finish one here |
| grey ✗ | executor-only — promotes itself when you click it as a target |
| yellow | selected (shell + axis arrows + outline) |
| white | hovered |
| red | a FAILED bus, and the reason next to it |
| orange ◆ | a gate — drag it to re-route two legs |

Bus layers carry a dim **emissive** term that cell bodies do not: half the
palette (the teal, the mid-green) otherwise sat right on the stone/quartz greys
it threads through, and brightness separates *wiring* from *structure* in the
abstract view and over a resource-pack mesh alike. Selection gizmos draw
depth-test-free with a dark backing outline for the same reason — a gizmo that
vanishes inside a bright block is not a gizmo.

### Labels declutter themselves

A 12-instance design carries ~50 port labels; zoomed out they are a grey mat
over the geometry they name. Three rules, all keyed off one measurement — how
many screen pixels a block spans **at that label's own depth**:

- below **3.2 px per block** the label is dropped and its 3-D cone marker stays:
  you lose the name, never the affordance;
- between there and 9 px it fades, so distance reads as distance;
- labels landing in the same 86×15 px screen cell collide, and the one nearest
  the camera wins.

Whatever is **selected or hovered** is exempt and never dropped. The counts
(`shown`, `hiddenSmall`, `hiddenOverlap`, `hiddenBehind`) are on
`window.__eda.labels()` and asserted by `npm run verify`.

### Errors you can act on

`design_corridor::diagnose` already produces excellent diagnostics — the
blocking layer, the coordinate, the bounded search, a cross-level probe — but
one of them is 300–500 characters of prose, and a panel that prints it verbatim
reads as noise. `src/reasons.ts` splits each into **headline** (what failed,
where), **fix** (what to move) and **at** (where to look), keeping the engine's
own words behind a disclosure triangle. Nothing is invented: every clause is
lifted from the string.

```
engine   segment (15, 6, 25) -> (40, 2, 4): this bus form is a SINGLE-LEVEL 2y-pitch
         stack, but the two anchors' bit-0 dust sits at y=6 and y=2 (a 4-block level
         change). Move one endpoint's instance by -4 in y so both ports share a level,
         or split the run with a gate placed at the target level — vertical level
         adapters are not implemented yet
studio   Bus bus1 failed: the two ends are on different levels — driver bit 0 at y=6,
         sink bit 0 at y=2 between (15, 6, 25) and (40, 2, 4)
         ↳ move one endpoint's instance by -4 in y so both ports share a level, or
           split the run with a gate at the target level          [→ (15, 6, 25)]
```

Recognised shapes: level mismatch, a walled-in endpoint, no corridor (including
"a clear corridor DOES exist at y=…", which is a completely different fix),
executor-only ports, width mismatches, and `driver port \`x\`: …` /
`sink port \`x\`: …` wrappers, which recurse. Anything unrecognised degrades to
its own first sentence — never to "failed".

Toasts **stack** (four at a time, newest last), each has an ×, an optional
second line naming the fix, and an optional `→ (x,y,z)` button that flies the
camera there. The column is anchored top-centre so it can never cover the hint
bar. A **FAILED bus row in the outliner is click-to-focus**: it flies to the
coordinate the router named.

### The outliner

Instances are **grouped by cell type with counts** — the same model the
renderer uses (one mesh per cell, N placements), so the panel and the machine
agree. Section headers carry counts (`3 in 3 type(s)`, `2 · 2 routed`,
`1 · 0 routed · 1 failed`). Port chips read `▲ sum : uint8` with the
reversible **Exec/Bus** badge attached. Bus rows show driver → sinks, width,
gates, state, and **latency + skew** from the engine's STA machinery
(`bus_skew_json`, e.g. `7t · skew 0t`). Click a bus row (or its **Focus**
button) to fly there; double-click an instance card, or press its **Frame**
button, to frame it — a single click selects and deliberately does *not* move
the camera.

Destructive actions confirm **in-app** (not `window.confirm`, which blocks the
engine, cannot carry a count and is auto-dismissed by every headless driver)
with the count in the prompt: *"Delete u1 and rip 2 buses?"*, listing them, and
saying that undo puts everything back.

### IO and wiring

The selected instance gets a highlighted bounding box (over a dark backing
outline, drawn depth-test-free), a translucent shell and an **axis indicator**
(red = local +X, blue = local +Z after `rot`).


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

**71 checks**, all through the same code paths the mouse and keyboard drive
(`window.__eda`), so green means the UI works and not just the engine: library
auto-load, instance ports (routable + refused-with-reason), selection and the
hint bar, `R` rotate, `Del` delete ripping its bus, click-to-connect, a bus
FAILED then healed, a typed poke through the routed chain (`u1.a=99 + u1.b=28
→ sum_out=127`), all three export tiers, **auto-promotion** on connect (and the
reverse), plus the **UX contract**:

| assertion | why it is the right thing to assert |
| --- | --- |
| the default page lands on the chain, buses already routed | onboarding is a property of the *default*, not of a button someone has to find |
| a 4-step coach opens on first visit, walks, and stays dismissed on reload | teaching once is the feature; nagging is the bug |
| `?` opens the legend (18 rows) and `Esc` closes it | discoverability, asserted rather than hoped for |
| with no design, the canvas prints the first four steps in order | the empty state is a screen users *will* see |
| labels: 16/17 shown framed, **0** past the 3.2 px-per-block threshold, markers stay | declutter is a measured rule, not a taste |
| ...and the selected label is exempt at any zoom | the exemption is the part that would silently rot |
| undo/redo a placement; a 20-frame drag is **one** undo step | the journal's coalescing is the only reason undo is usable during a drag |
| undoing a bus-carrying delete restores the instance **and** re-routes its bus | the one undo that rebuilds more than a transform |
| `Del` on a bus-carrying instance confirms first, with the count, and Cancel keeps it | destructive-by-accident is the failure mode |
| a 335-char engine reason becomes headline + fix + coordinate | the whole point of `reasons.ts`, checked against a real router string |
| ...the panel shows both and keeps the raw text behind a disclosure | the summary must never be the *only* copy |
| the FAILED row is click-to-focus: the camera flies to the named coordinate | "somewhere at (15,6,25)" is only useful if you can get there |
| toasts stack, each × dismisses one, the column never overlaps the hint bar | the old single toast erased its own story |
| `Esc` cancels placing **and** grabbing (put back where it started), camera freed | one key, no dead ends |
| clicking empty ground deselects | the escape hatch people try before they find `Esc` |

...and the **performance contract**, re-run unchanged:

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
(`flatten`, `cellDump`, `instDump`, `busDump`, `looseDump`); the UX ones are
`__eda.labels()`, `__eda.history()`, `__eda.coach()`, `__eda.toasts()`,
`__eda.pendingConfirm()`, `__eda.lastFailure()` and `__eda.focus()`. All are on
`window.__eda`. Results land in `docs/verify-out.json`; screenshots in `docs/`.

The textured check needs a resource pack at the repo root (`pack.zip`, **not**
committed); it is skipped, not failed, when absent.

## Screenshots

Regenerated by `npm run verify` into `docs/`:

| file | what it shows |
| --- | --- |
| `01-onboarding-coach.png` | the default landing state: the chain framed, coach step 1/4 |
| `02-chain-routed.png` | the same design after the coach is dismissed — two routed buses, timing in the panel |
| `03-outliner-grouped.png` | instances grouped by type with counts, port chips + Exec/Bus badges, a selection |
| `04-demo-adder-instance-ports.png` | the two-adder demo and its instance ports |
| `05-selection-gizmo-io-labels.png` | selection shell, axis arrows, IO labels |
| `06-rotated-90.png` | `R` — the gizmo's axes follow the rotation |
| `07-connected-instance-port-to-readout.png` | a bus routed by clicking two ports |
| `08-bus-failed-reason.png` | a FAILED bus as a **sentence** — headline, fix, `→ (15, 6, 25)`, engine words behind a disclosure |
| `09-baked-typed-poke.png` | the poke panel after `Bake` |
| `10-auto-promoted-cell-to-cell.png` | auto-promotion on connect, cell to cell |
| `11-textured-resource-pack.png` | the textured view (needs `pack.zip`) |
| `12-instanced-10-placements.png` | 10 placements over 3 cells, 7 draw calls |

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
