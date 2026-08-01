# mc-tick

A vanilla-accurate Minecraft tick engine. Load a build, run it the way the game
would — tick by tick, in the game's own phase order, with pistons that push what
vanilla pushes, comparators that read what vanilla reads, and entities that move
the way vanilla moves them — then read the results as data.

The engine is headless and self-contained: no window, no game, no rendering.
Rendering a run is a separate nucleation feature that consumes this engine's
output; see [the user manual](../../docs/features/tick-simulation.md).

## What it covers

- **The tick loop**: vanilla's ten phases in order (scheduled block ticks,
  fluid ticks, block events, entities, block entities, …), with block events
  that chain within a tick and piston moves that complete two phases after the
  event that started them.
- **Redstone**: dust (locational, like the game), torches with burnout,
  repeaters with locking, comparators with stored output, observers,
  quasi-connectivity, buttons, levers, plates (including weighted plates and
  their analog levels), detector rails, note blocks, dispensers and droppers
  with inventories, hoppers.
- **Pistons**: push limits and immovables, sticky retraction and the
  short-pulse block drop, `moving_piston` collision semantics, and vanilla's
  actual `moveCollidedEntities` for how strokes displace entities.
- **Entities**: items, minecarts (rail physics, cart-cart collision, furnace
  cart rules), and a measured registry of frozen bodies used as machinery —
  fireballs, blazes, villagers, boats. IEEE-754 velocities (NaN, ±Inf,
  denormals) are preserved exactly, with `Entity.load`'s version-dependent
  rules selected by the build's DataVersion.
- **Determinism**: seeded runs reproduce bit-for-bit via a faithful
  `java.util.Random`; unseeded runs use each distribution's mean.
- **Control**: step, run, run-until-quiescent, right-click, place blocks,
  checkpoint and restore.
- **Observability**: every block change with its tick; every neighbour/shape
  update with its intra-tick dispatch order and phase; entity positions;
  per-tick event summaries.

Anything the engine cannot model **refuses to load, by name** — a block, an
entity kind, or a modelled entity carrying state nobody has implemented. A
quietly wrong simulation is the one failure mode this tool does not permit.

## How it is verified

Behaviour is derived from the unobfuscated Minecraft server jar and validated
against traces captured from the real game running headless
(`tools/gametest/`). `cargo test -p mc-tick` replays those captures and diffs
the engine tick by tick; when the engine and a capture disagree, the capture
wins. Builds can also carry their own tests — a `.litematic` with an embedded
scenario descriptor is picked up by `cargo test --test litematic_cases` with no
Rust naming it (see [`tests/scenarios/`](../../tests/scenarios/README.md)).

The flagship conformance target is the world-record 55-block 3x3 piston door —
a machine made of NaN minecarts, frozen fireballs and a blaze used as an entity
wall. The engine runs its full close-and-reopen cycle with zero divergent ticks
against a capture of real Minecraft doing the same.

## Using it

**From Python, JavaScript, PHP, C or Kotlin**: use nucleation's bindings — the
engine is exposed as `TickSimulation`. The
[user manual](../../docs/features/tick-simulation.md) is written against the
Python API and covers building scenes, running, reading state, and composing
with the renderer.

**From Rust**: the crate is deliberately low-ceremony inside nucleation's
workspace, but constructing a fully-wired `Simulation` takes several steps
(state interning, behaviour registration, physics/fluid/rail tables, entity
spawning, settle mode). The canonical setup is
[`tests/support/scenario.rs`](tests/support/scenario.rs)'s `build_sim`, and the
bridge (`src/bridge/mc_tick.rs` in the workspace root) shows the same wiring
behind the `TickSimulation` API. Small self-contained tools live in
[`examples/`](examples/):

```text
cargo run -p mc-tick --example sim_summary -- path/to/build.snbt
```

## Documentation map

- [User manual](../../docs/features/tick-simulation.md) — the API, with
  runnable Python examples.
- [Mechanics notes](../../docs/features/tick-simulation-mechanics.md) — the
  measured behaviour underneath: version-gated NaN loading, entity hitboxes as
  mechanism, piston-entity displacement, plates and rails.
- [`docs/history/`](docs/history/) — the engineering record: the project
  charter, the roadmap as it evolved, and the record-door investigation that
  drove the entity work. Valuable, but it is history, not documentation.
