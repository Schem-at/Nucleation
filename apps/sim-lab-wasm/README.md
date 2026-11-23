# sim lab

A browser playground for the tick engine: drop in any schematic, fly around
it, click its levers, and watch mc-tick run — no backend, no install.

```sh
npm install
npm run build && npx vite preview --port 8455
```

## What it does

- **Any format the library reads** — drag a file onto the window or use
  *open…*; detection is by content, not extension.
- **Chunked rendering.** The whole build meshes once through
  `ChunkMeshResult` at 16³, and a tick that changes six blocks re-meshes
  only the one or two chunks those blocks touch. That is what makes a
  54k-block ship viewable *and* live.
- **Flight.** WASD to move, mouse to look, space/shift for up and down,
  ctrl to sprint, esc to release the pointer.
- **Interaction**, with the game's own mouse verbs. The crosshair marches a
  voxel ray to the first solid cell — the same traversal the engine uses for
  line of sight, offset half a block because block cells are *centred on
  integers*. **Right-click** calls `useBlock`: levers flip, buttons press,
  note blocks step their note. **Left-click** breaks, via
  `placeBlock(air)` — the engine's own removal, so supports drop and
  observers fire exactly as they would for a player mining it. A flash
  marks every click: green when the state changed, amber when nothing
  happened, red for a break.
- **Settle mode.** How the build starts, because it matters: *as it stood*
  (default — at rest, nothing fires), *as if pasted* (vanilla's full
  placement pass, so observers pulse), or *quiet* (`onPlace` only, the
  gametest framework's knownShape placement).
- **Run / step / rate.** Use a fractional viewing rate, fast-forward or uncapped
  simulation, or step an exact number of game ticks.
- **Pistons slide.** The engine supplies each carried block, stationary base,
  short/long arm state, source, destination and stroke timing. The viewer
  interpolates the last completed tick, so a new stroke starts at its source
  instead of skipping its first half-step. Pausing holds the fractional pose;
  explicit steps show the completed state. Fast-forward samples the same clock
  without changing physics or adding artificial easing. Landing meshes remain
  until their chunk replacement is ready. Shared prototypes, including both
  piston arm lengths, are prepared before play; clones retain their shared GPU
  geometry when an arm switches length.
- **Saved player-head skins.** Saved `SkullOwner`/`profile` texture URLs are
  downloaded and registered before meshing. Sim Lab fetches saved textures over HTTPS from Mojang's texture CDN. The load
  status reports skins loaded and any defaults used after a failed request.
  A reload retries failures. Name-only/UUID-only profiles without saved texture
  URLs use the engine's default skin; no player-account lookup is performed.

## Rebuilding the engine

`public/engine` holds a compiled wasm core. It does **not** rebuild when the
app does. Run any Rust build on **harrison-work**, then copy the generated
engine assets back before building the JavaScript app. The `engine` script
below invokes Rust and belongs on that build host:

```sh
npm run engine && npm run build
```

Forgetting it is the classic trap: a Rust fix lands, the app rebuilds
happily, and the browser keeps running the old engine — which looks exactly
like the fix not working.

The feature set matters: the tick engine is `mc-tick`, *not* `simulation`
(that is the separate MCHPRS redstone world). Building without it yields an
engine that loads and meshes and silently has no `TickSimulation`.

## Testing it

`verify.mjs` drives the real page headlessly — load a file, count meshed
chunks, use the first interactive block, report console errors:

```sh
node verify.mjs ../../crates/mc-tick/tests/corpus/litematics/6x6_sliding_door.litematic door
```

`window.simlab` exposes `{ world, player, scene }` for the console.

## Coordinate frames

Sim Lab translates its private schematic copy by the saved bounding-box minimum
before meshing. Rendering, picking, simulation changes and live entities then
share the engine's local coordinate system. Every region and its attached
entities move together; archived files are unchanged. The original minimum is
kept as `world.origin` and passed to the engine for Java redstone hash ordering.
Use complete schematic bounds here, matching `TickSimulation.fromSchematic`,
rather than tight content bounds.

With the local Atlas serving its retained plot entries, run:

```sh
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  node verify-coordinates.mjs http://127.0.0.1:8767
```

This compares the render mirror, picking grid, mesh coordinates and live state
against the original-source simulation at its saved origin, including piston
movement, positive/negative origins, multiple regions, a container and a cart.
It checks coordinate consistency, not a new vanilla reference capture.


## Mesh invalidation

Block meshes are rebuilt on visible changes, independently of simulation ticks
and camera draws. `applyChanges` keeps the last state per cell in a presentation
batch, updates the render schematic once, and dirties only chunks whose geometry
may change. A pending A→B→A reversal cancels its rebuild. Each chunk captures its
revision when its own meshing starts; later changes remain pending, and concurrent
flush requests share one job. Seam neighbours (including diagonals for AO) and
negative-coordinate growth are retained. Piston stand-ins retire when the matching
chunk is installed, or immediately when a cancelled rebuild leaves it current.

Scaffolding distance, leaf distance/persistence, and note-block pitch/instrument/
power can be visually inert. They are ignored **only** when the loaded resource
pack's blockstate conditions do not reference them. Missing or unrecognised pack
definitions fall back to complete state identity. Other fields, including dust
power/tint, waterlogging and piston poses, remain significant. The pack's
blockstate definitions must stay fixed during a World; load a new World after
replacing them. Cached flight models share these same visual identities.

Entities still synchronise on batches with no block changes. Camera drawing and
piston interpolation remain per animation frame. This changes presentation work,
not simulation events, recording, inventories or physics.

```sh
node verify-mesh-invalidation.mjs
node verify-presentation-clock.mjs
```

`world.renderStats` exposes record, mirror-write, visible-change and chunk-rebuild
counts for diagnostics.
