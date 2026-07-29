# Tick simulation

Load a schematic and run it the way the game would — tick by tick, in update
order, with pistons that push what vanilla pushes and comparators that read what
vanilla reads. This is `TickSimulation`, backed by the `mc-tick` engine.

It is not the same tool as [redstone simulation](redstone-simulation.md). That
page describes the MCHPRS redpiler: a compiled circuit executor, very fast, for
logic. This one is a faithful reimplementation of the game's tick loop, for when
*order* is the answer you need — a piston door lives or dies on which of two
opposed pistons is notified first, and a plausible-looking simulation that is off
by one tick is worse than none.

Everything here is headless. No rendering feature, no window, no game.

## Why you can trust it

The engine is checked against real Minecraft, not against a reading of its
source. Structures are run inside a headless 26.2 server via the gametest harness
in `tools/gametest`, the resulting traces are captured, and the engine is diffed
against them tick by tick. Sixty of those captures are pinned as tests. When the
engine and the game disagree, the capture wins and the engine is fixed — several
behaviours in it exist only because a capture overturned a confident reading of
the bytecode.

Where a behaviour could not be captured or verified, it is left unimplemented and
fails loudly rather than guessing. A block the engine cannot model refuses to
load, by name. That strictness is the point: a quietly wrong simulation is the
one failure mode this tool cannot tolerate.

## Quick start

```python
import json
from nucleation import Schematic, TickSimulation, TickSettleMode

schem = Schematic.load_from_file("door.litematic")
sim = TickSimulation.from_schematic(schem, TickSettleMode.InWorld, 0, 0, 0, "")
sim.set_rng_seed(12345)

# Find the lever and pull it.
world = json.loads(sim.world_snapshot_json())
lever = next(b["pos"] for b in world if b["state"].startswith("minecraft:lever"))
sim.run_until_quiescent(200)
sim.use_block(*lever)
sim.run_until_quiescent(300)

print(sim.tick_count(), "ticks")
for change in json.loads(sim.changes_json())[:5]:
    print(change["tick"], change["pos"], change["from"], "->", change["to"])
```

```javascript
const { TickSimulation, TickSettleMode } = await import("/engine/index.mjs");

const sim = TickSimulation.fromSnbt(snbt, TickSettleMode.InWorld, 0, 0, 0, "");
sim.setRngSeed(12345n);          // seeds are BigInt in JS
sim.useBlock(9, 3, 0);
sim.runUntilQuiescent(300);
```

## Loading a build

| constructor | takes |
|---|---|
| `from_schematic(schematic, settle, ox, oy, oz, extra_states)` | any format nucleation reads |
| `from_snbt(text, settle, ox, oy, oz, extra_states)` | gametest-flavor structure SNBT |
| `from_blocks(...)` | a palette plus a flat index array — no text, for tight loops |

`gametest_snbt(schematic)` converts a schematic to the SNBT flavor the engine and
the gametest oracle both read, which is also what the video renderer consumes.

### Settle mode is the most consequential argument

A schematic is not automatically a world. How you bring it to life changes what
you are measuring, and picking wrong produces a confidently wrong answer.

- **`InWorld`** — the build *is* the world. Nothing is placed, nothing settles.
  Use this for a build saved at rest: it preserves derived state the author saved,
  including repeater `locked` flags and comparator outputs.
- **`Placement`** — run vanilla's placement pass, exactly as pasting the build
  would. This is a *destructive* operation and that is faithful: `placeInWorld`
  re-derives repeater `locked` and wire connections, and loads block-entity NBT
  *after* the block writes. A door whose memory cell depends on a comparator
  reading a container will come up unlatched, because the container's contents
  do not exist yet at the moment the lock is derived. Use it when you want to
  know what happens when someone pastes the build.
- **`Quiet`** — `onPlace` only, no settle. Matches a `knownShape` capture.

If a build ticks to quiescence in zero ticks under `InWorld`, it was genuinely at
rest as saved, and that is the mode you want.

**`Quiet` is not "the gentle one".** Both `Quiet` and `Placement` run the
placement pass, which blanks the region and re-writes every block one at a time
so that each landing block's already-placed neighbours get a shape update. Every
observer in the build therefore watches the block it faces *appear*, and pulses.
On real doors that is not a rounding error — the certified reference set changes
50 to 896 blocks before anyone touches it:

| door | `InWorld` | `Quiet` | `Placement` |
|---|---|---|---|
| 4x4 sliding | at rest | 73 changes | 78 changes |
| 6x6 sliding | at rest | 836 changes | 896 changes |
| fast 4x4 vault | at rest | 50 changes | 121 changes |

If you are timing a saved build, use `InWorld` and check `changes_count() == 0`
before you start the clock. A door that is already moving when you actuate it
gives an open time that is confidently wrong rather than obviously wrong.
`examples/door_batch_load.rs` does exactly this check over a list of files.

### `extra_states`, and why your redstone block does nothing

Behaviours bind to *interned* block states when the simulation is constructed. A
state that first appears later — because you `place_block` it — has no behaviour
and sits inert. Name such states up front, semicolon-separated:

```python
sim = TickSimulation.from_snbt(snbt, mode, 0, 0, 0,
                               "minecraft:redstone_block;minecraft:lever[face=floor,facing=north,powered=false]")
```

`minecraft:redstone_block` and every facing of any shulker box held as an item
are always pre-interned for you. Everything else is your responsibility, and the
symptom of forgetting is silence rather than an error.

### Origin matters more than you would think

`updatePowerStrength` iterates a `HashSet<BlockPos>` whose order follows from
*absolute* position, so a build recorded away from the origin hands out its
neighbour updates in an order a zero-based replay cannot guess. If you are
reproducing a capture, pass the origin the capture recorded. For most work,
`0, 0, 0` is fine — origin affects tick-exact ordering in wire cascades, not
whether a machine functions.

## Running

```python
sim.step()                       # one game tick
sim.run(80)                      # eighty
sim.run_until_quiescent(300)     # until nothing is pending, or the budget runs out
sim.is_quiescent()               # nothing scheduled, nothing queued
sim.tick_count()
```

`run_until_quiescent` returns whether it actually settled. A machine that never
settles — a clock, a piston tape — will exhaust the budget, which is information
rather than an error.

## Interacting

```python
sim.use_block(x, y, z)                       # right-click, empty hand
sim.place_block(x, y, z, "minecraft:air")    # write a state (air breaks a block)
sim.get_block(x, y, z)                       # the state descriptor
```

Levers, buttons and note blocks respond to `use_block`. To pulse a signal, place
`minecraft:redstone_block` and then place `minecraft:air` over it.

## Checkpoints

```python
saved = sim.checkpoint()
... # try something
sim.restore(saved)
```

Cheap enough to sit inside a search loop. Measuring a door's reset time means
trying "toggle, wait N ticks, toggle" for increasing N until the world comes back
to where it started — a checkpoint per trial makes that nearly free, and the same
trick makes batch evaluation fast (wire one empty world, checkpoint it, and
restore-and-place per candidate rather than rebuilding).

## Reading results

Structured data crosses as JSON strings.

| call | gives you |
|---|---|
| `world_snapshot_json()` | every non-air block: position and state |
| `changes_json()` | every block change: tick, position, from, to |
| `changes_count()` | how many, without materialising them |
| `events_summary_json()` | per tick: block changes, piston events, redstone events |
| `item_entities_json()` | live item entities and minecarts, with container contents |

Snapshots omit air. Absence means air — compare over the union of two snapshots'
keys rather than assuming a missing entry is a missing block.

### Scalar queries, for loops that cannot afford JSON

```python
sim.non_air_count()
sim.non_air_center_x()   # centre of mass along x
sim.non_air_min_x()
sim.non_air_max_x()
```

A genetic algorithm evaluating thousands of machines a second should never parse
a snapshot. These exist so it does not have to.

### Update recording — the sub-tick view

The engine can record every neighbour and shape update it delivers, which is what
makes intra-tick propagation legible: you can watch a signal cross a build one
dispatch at a time, including updates that land on blocks which do nothing.

```python
sim.record_updates(True)
sim.run(40)
heat = json.loads(sim.updates_heat_json(0, 40))   # per tick, per cell: counts
wave = json.loads(sim.updates_wave_json(12))      # one tick, in dispatch order
```

Each raw record carries `tick`, `seq` (intra-tick order), `pos`, `from`, `kind`
(`neighbor` or `shape`), `phase`, and the block state **at dispatch time** —
which block sat there mid-tick decides whether an update did anything, and it is
invisible in a snapshot.

`phase` names where in the game's tick the update was delivered — the compact
views carry the legend in their payload (`phases`), so read it from there rather
than hard-coding the list. For a piston door the traffic lands almost entirely in
`block_events`, `block_entities` and `block_ticks`, with `boundary` covering
dispatches outside the phase walk.

Record *before* the stimulus you care about. Recording after a build has already
settled captures nothing, correctly — a quiet world delivers no updates.

Volume is the catch. A 6×6 piston door's open-close cycle produces about 119,000
updates, 15.8 MB as raw JSON. Prefer the compact views:

- `updates_heat_json(from, to)` — per (tick, cell) counts split by kind and
  phase. Around 0.9 MB for that same cycle. This is what you want for playback.
- `updates_wave_json(tick)` — one tick as parallel arrays with integer codes and
  a deduplicated state table. About 0.3 MB for the busiest tick, against 2.7 MB
  raw, because a tick touching 19,834 cells still only touches 181 distinct
  states.
- `updates_json()` / `updates_json_between(from, to)` — the raw log. Correctness
  work only.

Read before you disable: `record_updates(false)` drops the log.

## Determinism and randomness

The engine is deterministic. Behaviours that jitter in vanilla — dispenser
trajectories, dispenser slot choice, the drops from a block a piston destroys —
use each distribution's mean unless you seed it:

```python
sim.set_rng_seed(12345)
```

Seeded, they draw from a bit-exact reimplementation of `java.util.Random`'s LCG
in vanilla's own draw order, so a seeded run is exactly reproducible. It is *not*
a claim to match a live server draw-for-draw: a real `ServerLevel.random` is
shared with everything else happening in the world.

## Performance

Measured on an M-series laptop, a small flying machine over 80 ticks:

| path | evals/sec |
|---|---|
| single simulation, browser wasm | ~4,700 per worker |
| `eval_flight_batch`, node wasm | ~6,500 |
| Python, single process | ~700–740 |
| Python, `Pool(8)` | ~2,800 |

Construction was about 35% of a short evaluation until batching moved it: wire an
empty world once, checkpoint it, then restore-and-place per candidate. What
remains is stepping, which is the honest cost.

Dead machines are nearly free — the engine fast-forwards quiescent ticks, so a
genome that never moves costs roughly 30,000 evaluations a second.

## What is not modelled

Stated plainly, because the alternative is a user discovering it as a wrong
answer:

- **Mobs and players.** Weighted pressure plates, tripwire and similar are
  registered only in the state that is a fixed point with nobody present.
- **Boats.** Minecarts are modelled and oracle-verified; boats are not.
- **Item stack sizes.** Everything is treated as stacking to 64, so comparator
  container reads are correct only for 64-stackable items.
- **Some components.** Anything not implemented refuses to load and names itself,
  rather than being silently treated as air or stone.
- **Item drops.** A capture pins block-level outcomes; the items a destroyed
  block produces are not pinned unless the capture was taken with entities.

## Gotchas worth knowing before you hit them

**A schematic is not always a valid world state.** Formats lose things. A `.schem`
export may carry no block entities at all, so comparators lose their stored
`OutputSignal` and machines that depend on them degrade — with no error, because
every block is present and correct. If a build behaves differently from how its
author describes it, compare block-entity counts before suspecting the engine.

**Derived properties are real state.** Repeater `locked`, note-block
`instrument`, wire connections: the game recomputes these on placement. Under
`InWorld` the engine trusts what the file says, which is usually right for a
saved build and always right for a build the file recorded at rest.

**A comparator emits what it stored, not what its state claims.** A comparator
whose block state says `powered=true` but whose block entity holds
`OutputSignal: 0` emits nothing — that is vanilla, verified by capture, and it is
how a schematic with empty containers quietly stops working.

## Where to look next

- `crates/mc-tick/PROJECT.md` — the engine's own design notes and the
  verification discipline
- `crates/mc-tick/tests/cases/README.md` — the folder-driven scenario harness:
  one JSON file per test, no recompilation to add a case
- `tools/gametest/README.md` — the vanilla oracle: how captures are produced
- `docs/features/redstone-simulation.md` — the redpiler, for when you want logic
  throughput rather than tick fidelity
