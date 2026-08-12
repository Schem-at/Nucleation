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

`dev`, `build` and `verify` all re-sync the engine and then **verify** that every
copy of it is the one on disk (`npm run check-engine`); the header badge turns
red and says `STALE ENGINE` if the page ever loads a different one. See
[The engine you are measuring](#the-engine-you-are-measuring) for why that guard
exists — it is the difference between a red check and a red herring.

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
| click a port | start (or finish) a bus — a port press **never** moves anything |
| click a body | select an instance; click empty ground to deselect |
| drag a body | move it once the pointer has travelled 4 px; buses reroute live |
| `R` / `⇧R` | rotate 90° / −90° in place; buses reroute, the gizmo shows the new axes |
| `G` | grab-move; click to drop, `Esc` puts it back |
| `F` | frame the selection (or the whole design when nothing is selected) |
| `A` | frame the whole design |
| `Del` / `⌫` | delete the instance — **confirms first** if buses die with it, with the count |
| `⌘/Ctrl-Z` / `⇧⌘/Ctrl-Z` | undo / redo |
| `?` | keyboard + colour legend |
| `Esc` | cancel placing/connecting/grabbing, close an overlay, or deselect |
| `⇧`+click | add/remove an instance from an area selection (the last click is primary) |
| `⌘/Ctrl-C` `V` `X` `D` | copy / paste at the cursor / cut / duplicate with an offset |
| `C` / hold `Alt` | connect mode — component bodies stop being pickable entirely |
| right-click | context menu on whatever is under the cursor |

Four kinds of thing are selectable, and each answers `Del` differently, because
they are different things: an **instance** (deleting it takes its buses, so it
asks), a **bus** (deleting it drops the declaration; `Rip` keeps it), a **gate**
(deleting it only lets the route straighten), and a **port** (not deletable at
all — it belongs to a cell).

### Picking: the port always wins, and a click is not a drag

> *"it's awkward to select an IO and route it, I always accidentally move the
> component"*

Two separate bugs wearing one symptom, and both of them were in how a press was
resolved.

**1. Ports are picked in screen space, and they outrank everything.** A port
marker sits one block *outside* its column, which puts it inside or behind the
neighbouring instance's invisible pick box more often than not. Ray order then
handed the press to the box — the port was drawn on top and was not what you
hit. So ports are no longer ray-picked at all: every marker is projected to the
screen and the nearest one within **15 CSS pixels** (or 2.5× the cone's own
on-screen radius, whichever is larger) takes the press. Three consequences, all
of them the point:

* depth cannot steal a press — a port buried inside a cell mesh is still the
  target, which is what `portOverBody` in `__eda.pickStats()` counts;
* the target is a **constant size on screen**, so a port stays clickable at the
  zoom where its label has already been decluttered away (the verify sweeps four
  orbit radii and asserts every on-canvas port resolves to itself);
* it costs nothing to draw: the port cones left the ray-pick list, so this adds
  zero geometry and zero draw calls.

Ties go to the port nearest the camera. Buses are still consulted last, for the
original reason: a port cone sits *on* a bus's first cell.

**2. A press on a body is a selection until it travels 4 pixels.** A press used
to become a drag on the first `pointermove`, so a jittery click nudged the
component and cost an undo — the "I always accidentally move it" half of the
report. A press now parks in a `pending` slot: under 4 px it stays a selection
and the instance does not move one block; past 4 px it promotes to a drag, which
is still **one undo step**. A press on a *port* never parks there at all, so no
amount of subsequent movement can turn it into a move.

**Affordances**, so none of this has to be discovered by experiment: the cursor
is a **crosshair** over a port and **move** over a body; a hovered port scales
up and gets its label back even when labels are decluttered; and the hint bar
says what the click will do — *"click to start a bus from u0.sum (uint8[8]) · the
component will NOT move"*. See `docs/18-hover-port-affordance.png`.

**Connect mode** (`C`, or hold `Alt`) is the escape hatch for a dense scene:
bodies are not in the pick list at all, so a press can only ever be a port. The
canvas says so with a badge, and the verify asserts that every body pixel on the
canvas stops resolving to an instance while every port keeps resolving to itself
(`docs/19-connect-mode-bodies-unpickable.png`).

### Different widths are a question, not a refusal

Connecting a 1-bit carry to an 8-bit word used to print `width mismatch` and
stop. Worse, it usually printed `type mismatch` first — because `ty` is a
*display* string with the width baked into it (`uint8`), so comparing two of them
answered "same type **and** same width?" and every width difference came back as
a type error. The studio looked broken over something the engine has an adapter
for.

Now the connect **routes**, LSB-aligned by default (bit 0 to bit 0, so the
magnitude survives), and the toast states the mapping rather than leaving it to
be discovered by poking values through it:

```
bus3: u0.cout[1] → u1.a[8] aligned LSB · driver bit 0 → sink bit 0
      · 1 bit(s) carried · 7 sink bit(s) read 0
```

Destination bits nothing drives read 0 with no hardware at all (the engine
reports them as `tied_zero`). The other alignments live on the bus's own
right-click menu — **MSB** (top bit to top bit), **Shift ±1** — because the
choice is worth changing without redoing the connection; re-aligning is one undo
step and touches the endpoints not at all. Alignment is part of the bus's
*intent*, so it survives a re-route, a gate edit, an undo and a restore.

Type checking now compares FAMILIES: a boolean is a 1-bit unsigned word, exactly
as it is in Verilog. **Signedness** is the difference that really does change
what the bits mean, and it is still refused, and now says why.

The one case that stays a question is **loss**. Bits of the source that fall
outside the destination are dropped, which changes what the design computes, so
it goes through the same confirm every destructive action does — naming the count
— and `truncate` is only ever opted into, never assumed.

### Copy / paste

An instance is a *reference to a library cell plus a transform* — that is why
ten placements cost one mesh — so a paste places **another reference**, never a
copy of the blocks. `⌘V` costs zero new cell meshes, which the verify asserts.

What travels with a copy: the transform, and the per-port Exec/Bus promotions
(those are per-instance patches, so they genuinely are part of the instance).
What does not: geometry, and buses with an endpoint outside the copied set —
half a bus is nothing.

For an **area** copy (`⇧`-click a group), a bus whose driver *and* every sink are
inside the set has its **intent** copied — driver, sinks, gates — and the paste
re-declares it for the new group, with every endpoint remapped to the copies. A
bus is an intent the router realizes; blocks are its output, not its identity.
If the new copy cannot route, it is left FAILED with the router's own reason,
like every other failed route here.

A paste is **one undo step** (`Studio.transaction`) covering the placements, the
promotions and the recreated buses — and one refresh, so the renderer never
meshes the half-promoted intermediate states nobody sees. Names are derived from
the source (`u1` → `u2`, `add0` → `add1`, otherwise `_copy`), and the group is
nudged clear of existing keepouts rather than being refused.

### Gates are checkpoints, not endpoints

A gate says *the route must pass through here*; an endpoint says *this bus
drives that port*. Conflating them is the mistake that makes routing feel
arbitrary, so nothing in the UI does: removing a gate leaves the netlist alone
and lets the router take a **straighter** path (the verify asserts the cell
count does not grow), while removing an endpoint changes what the design means.

Full lifecycle on canvas: right-click a bus → **Add gate here** at the clicked
point; drag the ◆ handle to steer (only the two adjacent spans re-route); select
it and `Del`, or use its ✕ in the Buses panel, to remove it. Each is its own
undo step. A selected bus lists its checkpoints in order with their coordinates,
and prints `N trunk span(s)` so the model is on screen rather than implied.

Two engine notes the UI works around rather than hides:

- a click lands on whatever *bit* of the bus's 2y-pitch stack is under the
  cursor, so the gate's **level snaps to the bus's own** (a gate off the trunk
  level is a level change, which this router refuses — and says so);
- `Design::add_gate` resolves a bus's endpoints through the *declared* port
  table, so it refuses any bus running between placed cells. Those buses fall
  back to re-declaring the bus with the new gate list, which routes the whole run
  instead of two spans — same result, more work. `remove_gate` is used when the
  loaded engine has it and falls back the same way.

### Bus drawing: a bus IS redstone

A bus fragment is not an annotation near redstone, it is dust, repeaters and the
blocks carrying them. Painting it as an opaque coloured slab therefore *deletes*
information — and what the colour is for is identity, which survives a tint or an
outline just as well. Three presets (header dial, remembered in `localStorage`,
overridable per bus in the Buses panel):

| preset | what it draws | for |
| --- | --- | --- |
| `solid` | the fragment in the bus colour | tracing a route at maximum contrast |
| `translucent` | tinted, the blocks reading through | the default |
| `outline` | silhouette only, in the bus colour | seeing the actual redstone |

Hovering or selecting a bus raises its emphasis temporarily, so `outline` stays
traceable: the fragment under the cursor comes forward without the other seven.
Compare `docs/15-textured-bus-solid.png` with
`docs/16-textured-bus-outline-shows-redstone.png`.

### Right-click

Context-sensitive on what is under the cursor: an **instance** offers rotate,
duplicate, copy/cut, a per-port Exec/Bus submenu, promote-all-inputs, frame, rip
its buses, delete; a **port** offers start-a-bus and its mode switch; a **bus**
offers add-gate-here, remove-a-gate submenu, re-route, rip, its drawing style,
focus-the-blockage when FAILED, delete; **empty space** offers paste, add a
component from the library, frame all, and the view toggles. `Esc` closes it, and
right-clicking mid-gesture cancels the gesture instead of opening a menu.

The highest-value entry is **Add gate here**, because it is the only affordance
that can exist: gate steering needs a world position, and no panel has one.

### Undo/redo

The design layer offers `rip` / `reroute` / `remove_*` / `set_port_mode` but no
document history, so the studio keeps an **operation journal**: every mutator
records its inverse. Place, move, rotate, delete, route, rip, delete-bus and
port-mode toggles are all reversible, and a **drag is one undo step** — any
adaptive pause commits and the final drop coalesce into "where the gesture
started" → "where it ended". Undoing the delete of a bus-carrying instance re-places
the body, re-applies its promotions and **re-routes its buses**; the verify
script asserts exactly that. Gate add/move/remove and pastes are journalled too,
a paste as **one** step via `Studio.transaction` (which also defers the change
notification to the commit, so a dozen edits cost one refresh). `declarePort` has
no engine inverse and is deliberately *not* journalled rather than faked. Loading
a demo clears the history — a starting point is not an edit.

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

### ...but the renderer does not TRUST the changed set

"Sometimes when I move a component the bus doesn't update right" is what an
incremental renderer costs you if the changed set is the only thing deciding
what to redraw. A changed set is a performance hint; it can be incomplete, it
can be spent by a failed frame, and it can be stale by a frame. So three
mechanisms sit under it, each aimed at one of those:

1. **Geometry revisions.** Every layer (each bus, the loose base, each cell
   variant) carries a `rev` that moves only when its blocks actually change
   content — a signature comparison, not a re-read. The renderer keys its mesh
   cache on `rev`, so a layer whose geometry moved can never keep a stale mesh
   even if nobody named it. Revisions come from a process-wide counter, because
   a per-layer one hands a NEW document's first `loose` layer revision 1 — the
   number a renderer that outlives the document swap is already holding. (That
   is not hypothetical: the consistency check below caught exactly that, and the
   symptom was the design's loose hardware silently not drawn.)
2. **The dirty set is consumed last.** `scene()` clears the flags only after
   every read has succeeded, and a caller that fails to apply a scene calls
   `Studio.invalidateAll()`. A throw mid-scene used to mean permanent staleness:
   the work was never done and nothing remembered it was owed.
3. **Commit sequence numbers.** A live re-route commit is deferred to the next
   animation frame, so a drop could land the final position and then have the
   queued frame land the position from *before* it — re-routing every affected
   bus for a component that had already moved. Every intent takes a number when
   it is formed and a commit is refused if a higher-numbered one has landed.
   Coalescing may drop work; it can never apply older work over newer.

And one assertion, which is the actual regression guard:
`window.__eda.consistency()` reads back **what is in the scene graph** — the
vertex buffers and the instance matrices, not the arrays the renderer was handed
— and compares it cell-for-cell with a fresh, cache-bypassing read of the
engine. `npm run verify` runs it after a 12-frame drag, a rotate, a rip, a
re-route, a promotion, a delete, an undo, every paste and every gate edit.

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

The pointer path is free: a drag is a GPU matrix preview and does no wasm work.
**Adaptive reroute** learns the measured cost per object: a route known to fit a
32 ms frame may refresh after a 160 ms pointer pause; an unknown or slow route is
computed once on drop. This prevents a synchronous router from turning every
small pointer movement into a multi-frame stall while preserving live feedback
where the measured cost genuinely permits it.

`Design::bus_blocks_json(name)` already keeps the redraw local. The remaining
long-pole is the synchronous router/acceptance pass itself; moving the engine
into a **Web Worker** would take that final drop computation
off the main thread entirely. Not done here: every `Studio`/`window.__eda` call
plus the whole verify script is synchronous against the engine, so the port is
a larger change than this pass.

The current routing pass is recorded in `docs/profile-adaptive-before.json` and
`docs/profile-pin-context.json`. On that same 12-instance scene, bus rerouting
fell from **1821 ms to 630 ms** by validating only the current bus intent and the
electrical neighbourhood of its endpoint pins. A 30-frame drag fell from **31
routes to one route on drop**; pointer handling itself averages **0.03 ms** per
move and performs no wasm reads or cell remeshing.

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

### The engine you are measuring

The **first** check of the run asserts that the page is running the engine that
is on disk, because everything after it is a statement about a router and is
worth nothing if that router is two builds old. This is not hypothetical: `npm
run build` used to skip `sync-engine`, so `dist/engine/` (what the harness
serves) drifted from `dist/npm-eda/`, and three gate checks went red against a
reason string — "cannot ramp between levels" — that had already been deleted
from the engine. The failure was then blamed on the wrong lane.

Three defences now, because there are three ways to serve the wrong engine:

| defence | catches |
| --- | --- |
| `build` runs `sync-engine`, and `npm run check-engine` compares `dist/npm-eda` / `public/engine` / `dist/engine` by sha256 | a build or a verify against a copy that drifted, on the filesystem, before a browser is involved |
| `/engine/*` is served `no-store` in dev **and** preview | a browser cache answering with an engine the server no longer has |
| `__ENGINE_SHA__` (baked at build) vs `engine/BUILD.json` (shipped beside the wasm), surfaced as `__eda.engineStamp()` and a red **STALE ENGINE** badge | the page having loaded something other than what this build was made against — asserted first thing by the verify |

**129 checks**, all through the same code paths the mouse and keyboard drive
(`window.__eda`) — and the picking suite goes further and drives the **real
pointer** (`page.mouse`), because the whole claim there is that the browser's own
events resolve the way we say they do. Green means the UI works and not just the
engine: library
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

...the **editing model** added in this pass:

| assertion | why it is the right thing to assert |
| --- | --- |
| every rendered layer matches the engine's blocks after a drag, rotate, rip, re-route, promotion, delete and undo | the regression guard for "sometimes the bus doesn't update": it reads the scene graph's own buffers, not the arrays the renderer was handed |
| a live frame queued before a drop is refused when it fires, and the instance stays where the drop put it | the drag race, asserted by the count of commits actually dropped |
| ⌘C/⌘V places one new instance of the same cell, uniquely named, for **0** new cell meshes | a paste is a reference, which is the whole model |
| ...with its Exec/Bus port modes carried over | the per-instance patches are part of the instance |
| ...as **one** undo step, and consistent both after the paste and after the undo | one action, one ⌘Z |
| an area copy takes the bus with both ends inside, and the paste RECREATES it with endpoints remapped to the copies | a bus is an intent, not blocks |
| ...and one ⌘Z removes the pasted instances *and* the recreated bus | the transaction, end to end |
| ⌘D lands the copy clear of the original; ⌘X then ⌘V round-trips | the two shortcuts people actually use to array a cell |
| clicking a bus selects it, the hint bar names Del/R/F, its outliner row highlights, `Del` deletes it behind the same confirm | a bus is a first-class object, not an outliner row |
| bus drawing has three presets, one bus can override the global, and the choice survives a reload | "the bus lines hide the redstone" is a preference, so it is stored |
| the right-click menu is context-sensitive per instance / port / bus / ground, `Esc` closes it, its Delete uses the same confirm | a menu that acts on something the canvas does not show as selected is how you delete the wrong cell |
| "Add gate here" carries the clicked (x, z) and snaps only the LEVEL to the bus's | the entry only means anything with a position |
| a gate lands, the bus stays routed, and the geometry passes THROUGH that cell | otherwise it is a decoration |
| dragging the handle moves it for **0** cell re-meshes | the 2-adjacent-span fast path |
| removing it leaves the endpoints untouched and the route no longer | gate = route, endpoint = netlist, asserted rather than described |
| add / drag / remove are each their own undo entry | each is a thing a user did |

...the **picking model** (driven with real pointer input, on its own page):

| assertion | why it is the right thing to assert |
| --- | --- |
| every on-canvas port resolves to itself at four orbit radii | a hit target that shrinks with zoom is the bug, so the zoom is swept, not assumed |
| ...including at the radii where labels have decluttered away | that is exactly where the old ray-vs-cone pick had already stopped working |
| N of the ports sit behind or inside a body's pick box and still win | ray order would have given those presses to the body — the priority rule, measured rather than described |
| a real mousedown on a port + 40 px of movement starts a **bus**, with **0** instance transform change and no new undo entry | the reported bug, as a gesture: a port press never touches the drag state |
| a real 3 px press on a body **selects** it and moves it not one block | the other half of the report: a click is not a drag |
| a 140 px press does move it, in **one** undo step | the threshold must not cost the gesture it is protecting |
| ...for 0 cell re-meshes, 0 texture builds and an unchanged draw-call count over 20 frames | the new pick path is screen-space arithmetic; it may not cost geometry |
| the cursor is crosshair over a port and move over a body | the affordance, read off the DOM |
| a hovered port's label comes back at a zoom where others are hidden | hovering is how you know *which* port you are about to route |
| the hint bar says "click to start a bus from `<port>`" | the bar's whole job is the next click |
| connect mode: every body pixel stops resolving to an instance, every port still resolves | the dense-scene escape hatch, as a measurement over the canvas |

...and the **width adapters**:

| assertion | why it is the right thing to assert |
| --- | --- |
| a narrow driver into a wider port CONNECTS, LSB-aligned, asking nothing | nothing is lost, so there is nothing to ask — the old refusal was the bug |
| ...and the UI states the mapping (bits carried, sink bits reading 0) | a width-adapted bus is the one routed bus that does not mean what a reader assumes |
| ...with the engine's own resolved bit map behind it | the sentence must be the engine's answer, not the app's guess |
| the bus's menu offers MSB and Shift ± only where the ends differ | a menu full of inapplicable entries stops being read |
| MSB re-aligns the SAME endpoints, and it is one undo step | alignment is route-level intent, not netlist |
| a lossy connection asks first, naming the dropped count, and Cancel leaves no bus | dropping a word's high bits is not the router's call |
| ...and confirming it opts `truncate` in explicitly | the permission has to be recorded, not inferred |

...and the **performance contract**, re-run unchanged:

| assertion | why it is the right thing to assert |
| --- | --- |
| 20 drag frames ⇒ 0 cell re-meshes, 0 block dumps out of wasm | a drag is a transform; if it re-reads anything, the design has slipped |
| ...at ≤ 33 ms of main-thread work per frame | the ≥ 30 fps target, stated as a budget, not as a headless frame rate |
| K placements of one cell ⇒ 1 mesh build, 1 instanced group | the instancing claim itself |
| ...read out of wasm once, from the CELL | no per-instance region dumps |
| port-mode toggle ⇒ exactly 1 cell variant re-meshed | the only edit that changes a placed cell's blocks |
| one bus re-route ⇒ exactly 1 bus layer RE-READ, 0 cells, and re-meshed only if its blocks moved | re-reading is the contract; re-meshing is a decision, and an identical route already has a correct mesh |
| live re-route commits ≤ 1 engine move per animation frame | there is no fixed throttle left to drift out of date |
| 10 placements / 3 cells ⇒ the draw-call count | the number a reader can check |

The counters behind these are `viewer.meshBuilds` (`cells`, `instancedGroups`,
`matrixWrites`, `buses`, `loose`, `texture`) and `studio.sceneReads`
(`flatten`, `cellDump`, `instDump`, `busDump`, `looseDump`); the UX ones are
`__eda.labels()`, `__eda.history()`, `__eda.coach()`, `__eda.toasts()`,
`__eda.pendingConfirm()`, `__eda.lastFailure()`, `__eda.focus()`,
`__eda.consistency()`, `__eda.staleCommits()`, `__eda.clipboard()`,
`__eda.gates()` and `__eda.contextMenu()`; the picking ones are
`__eda.pickStats()` (`portPicks`, `bodyPicks`, `portOverBody`, `dragsStarted`,
`clicksBelowThreshold`), `__eda.pickThresholds()`, `__eda.probeAt(x, y)`,
`__eda.portScreen(name)`, `__eda.onCanvas(x, y, inset)`, `__eda.hoverPort()` and
`__eda.connectMode()`; the width ones are `__eda.connect(a, b)`,
`__eda.busAlign(bus)`, `__eda.setBusAlign(bus, adapt)`, `__eda.busWidthMap(bus)`
and `__eda.alignmentLine(bus)`; and `__eda.engineStamp()` says which engine
answered any of it. All are on
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
| `11-copy-paste-group.png` | ⌘C/⌘V — a second placement of the same cell, framed with its original |
| `12-context-menu-instance.png` | the right-click menu on an instance (rotate, duplicate, port-mode submenu, rip its buses) |
| `13-bus-gates-and-context-menu.png` | a bus with two checkpoints, its menu open, its gate list in the panel |
| `14-textured-resource-pack.png` | the textured view (needs `pack.zip`) |
| `15-textured-bus-solid.png` | the same scene with `solid` buses — the coloured slab hides the redstone it is made of |
| `16-textured-bus-outline-shows-redstone.png` | ...and with `outline` buses: dust, repeaters and blocks visible, identity kept |
| `17-instanced-10-placements.png` | 10 placements over 3 cells, 7 draw calls |
| `18-hover-port-affordance.png` | hovering a port: crosshair cursor, the label back, and the hint bar saying *"click to start a bus from u0.sum · the component will NOT move"* |
| `19-connect-mode-bodies-unpickable.png` | connect mode (`C` / hold `Alt`) — the badge, and bodies that no longer take a press |

## Known rough edges

- **A width-adapted bus in the two-adder demo often routes FAILED.** The
  adaptation itself lands (the engine returns the bit map, `tied_zero` and all);
  the demo is simply a tight scene, and a 1-bit carry into an 8-bit word has
  nowhere to run. The verify tries eight pairs and reports the state it got, so
  the number is never quietly a pass.
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
- **`Design::add_gate` cannot resolve instance ports.** It looks its bus's
  driver and first sink up in the *declared* port table, so any bus between two
  placed cells — which is most of them — is refused with
  `NucleationError.InvalidArgument`, even though `move_gate` handles a gate on
  the same bus fine. The app falls back to re-declaring the bus with the new gate
  list (whole run re-routed instead of two spans), so gates work everywhere; the
  fast path returns as soon as the engine resolves endpoints there. Same for
  `remove_gate`, used when present.
- Buses realize a single-level 2y-pitch stack: both endpoints must share a y.
  Moving an instance off that level reports FAILED with that explanation
  (which is what the verify script's failure case uses).
- The textured view re-meshes the WHOLE design on a port-mode toggle, not just
  the cell that changed — `meshGlb()` goes through `flattenComposite()`, which
  has no per-layer entry point. The abstract view does do it per cell.
