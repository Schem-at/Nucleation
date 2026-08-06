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
- **Run / step / rate.** Free-run at 1–60 tps, or single-step and watch one
  tick at a time.
- **Pistons slide.** A stroke takes two game ticks, and the blocks it carries
  are drawn moving across them instead of jumping twice. The engine only
  says *a block is in flight here* — a `moving_piston` placeholder — so the
  carried block's identity comes from the mirror schematic, which is still a
  batch behind and therefore still remembers. Each distinct block state is
  meshed once on its own and cloned per moving copy, so a hundred sliding
  slime blocks cost one mesh. A piston's own head is drawn as a head rather
  than a second copy of its base.

  Only while a stroke outlasts a frame: past ~30 tps it is over before it
  could be drawn, and interpolating then smears the machine rather than
  clarifying it. Single-stepping uses a fixed quarter-second beat, because
  that is the case where you most want to see the stroke.

## Rebuilding the engine

`public/engine` holds a compiled wasm core. It does **not** rebuild when the
app does, so a Rust change needs:

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
