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
against them tick by tick. There are 103 captures in
`crates/mc-tick/tests/traces/`, 88 of them named by a test. `cargo test -p
mc-tick` runs 332 tests — 326 pass, 6 are ignored — and the conformance binary
that replays those captures is 81 of them. When the
engine and the game disagree, the capture wins and the engine is fixed — several
behaviours in it exist only because a capture overturned a confident reading of
the bytecode, and this page flags the places where that happened.

Where a behaviour could not be captured or verified, it is left unimplemented and
fails loudly rather than guessing. A block the engine cannot model refuses to
load, by name; so does an entity, and so does an entity that *is* modelled but
carries state needing behaviour nobody has implemented. That strictness is the
point: a quietly wrong simulation is the one failure mode this tool cannot
tolerate.

## Quick start

```python
import json
from nucleation import Schematic, TickSimulation, TickSettleMode

schem = Schematic.load_from_file("tests/scenarios/55_3x3.litematic")
sim = TickSimulation.from_schematic(schem, TickSettleMode.InWorld, 0, 0, 0, "")

# A saved build should be at rest as loaded. Check before starting the clock.
assert sim.run_until_quiescent(400) and sim.changes_count() == 0

sim.use_block(9, 2, 19)              # the door's button
sim.run_until_quiescent(400)

print(sim.tick_count(), "ticks,", sim.changes_count(), "block changes")
for change in json.loads(sim.changes_json())[:3]:
    print(change["tick"], change["pos"], change["from"], "->", change["to"])
```

```
36 ticks, 227 block changes
0 [9, 2, 19] minecraft:oak_button[...powered=false] -> ...powered=true
0 [9, 2, 20] minecraft:note_block[...powered=false] -> ...powered=true
0 [9, 3, 20] minecraft:dispenser[facing=east,triggered=false] -> ...triggered=true
```

```javascript
import { readFileSync } from "node:fs";
const { Schematic, TickSimulation, TickSettleMode } = await import("./engine/index.mjs");

const schem = Schematic.fromLitematic(readFileSync("55_3x3.litematic"));
const sim = TickSimulation.fromSchematic(schem, TickSettleMode.InWorld, 0, 0, 0, "");
sim.runUntilQuiescent(400);
sim.useBlock(9, 2, 19);
sim.runUntilQuiescent(400);
console.log(sim.tickCount(), sim.changesCount(), sim.motionSemantics());
```

There is no `loadFromFile` in the wasm build — there is no filesystem. Read the
bytes yourself and hand them to `Schematic.fromLitematic` (or `fromData`,
`fromSchematic`, …). Method names are camelCase in JS and seeds are `BigInt`.

## Loading a build

| constructor | takes |
|---|---|
| `from_schematic(schematic, settle, ox, oy, oz, extra_states)` | any format nucleation reads |
| `from_snbt(text, settle, ox, oy, oz, extra_states)` | gametest-flavor structure SNBT |
| `from_blocks(...)` | a palette plus a flat index array — no text, for tight loops |

`gametest_snbt(schematic)` converts a schematic to the SNBT flavor the engine and
the gametest oracle both read, which is also what the video renderer consumes.
It carries the schematic's `DataVersion` through, so `gametest_snbt` → `from_snbt`
loads a build under the same `Entity.load` rules `from_schematic` would — read
[the version gate](#nan-velocities-and-the-version-gate) anyway, because *which*
rules those are is the single most consequential thing on this page.

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
On real doors that is not a rounding error — the reference set changes 50 to 896
blocks before anyone touches it:

| door | `InWorld` | `Quiet` | `Placement` |
|---|---|---|---|
| 4x4 sliding | at rest | 73 changes | 78 changes |
| 6x6 sliding | at rest | 836 changes | 896 changes |
| fast 4x4 vault | at rest | 50 changes | 121 changes |
| record 3x3 (`tests/scenarios/55_3x3.litematic`) | at rest | 68 changes | 83 changes |

If you are timing a saved build, use `InWorld` and check `changes_count() == 0`
before you start the clock. A door that is already moving when you actuate it
gives an open time that is confidently wrong rather than obviously wrong.
`examples/scenario_inspect.rs` does exactly this check for a path, and the
`{"expect": "changes", "count": 0}` assertion in a scenario descriptor pins it —
see `tests/scenarios/README.md`.

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

## NaN velocities and the version gate

This is the sharpest edge in the engine, and it is not a curiosity. The record
3x3 piston door — the smallest that exists, and a conformance target here — is
**glued together by minecarts whose velocity is NaN**. Its builders wired as much
of the piston layout as conventional redstone allowed, ran out of space, and then
used entities to cram redstone circuits inside other redstone circuits. Nothing
in that build is decorative. A NaN velocity makes an entity's own physics
dead: every comparison against it is false, so nothing accelerates it, nothing
drags it, and nothing but a piston arm can move it. That is what holds those
builds in shape. Sanitise one velocity and the machine silently un-glues: no
error, no warning, just a door that no longer works and a plausible-looking
trace explaining why.

The engine therefore preserves IEEE-754 exactly and clamps nothing. `f64` NaN
and ±Infinity survive NBT read, SNBT write, SNBT parse and the physics itself.

### `Entity.load` is version-dependent, and the boundary is real

Vanilla's own loader changed. Read a nan-cart build at the wrong version and it
loads as ordinary carts:

| DataVersion | rule | a NaN `Motion` |
|---|---|---|
| ≤ 4556 (≤ 1.21.10) | per-component: `\|v\| > 10 → 0`, ±Infinity → 0 | **kept** |
| ≥ 4671 (≥ 1.21.11) | `isFinite` guard on the *whole vector* | **dropped**, previous velocity kept |

Note what each rule does *not* do. The old rule keeps NaN precisely because
`NaN > 10` is false — the check that kills a large finite velocity cannot see a
NaN at all. The new rule discards the entire vector, not just the offending
component. The bisect left a gap between 4556 and 4671 with no released version
in it; `MotionSemantics::for_data_version` puts the boundary at 4671. Bytecode
and the bisect are in `tools/gametest/NAN-MOTION-VERSIONS.md`.

Which rule a run is using is readable, and worth reading before you trust a
result:

```python
sim.motion_semantics()      # "clamp_abs_ten" | "drop_non_finite"
```

`from_schematic` derives it from the file's own DataVersion. The record door is
DataVersion 4082, so it loads under the NaN-keeping rule and its six NaN velocity
components survive:

```python
import json, math
schem = Schematic.load_from_file("tests/scenarios/55_3x3.litematic")
sim = TickSimulation.from_schematic(schem, TickSettleMode.InWorld, 0, 0, 0, "")
print(sim.motion_semantics())                                  # clamp_abs_ten
carts = json.loads(sim.item_entities_json())["minecarts"]
print(sum(1 for c in carts
          if any(isinstance(v, float) and math.isnan(v) for v in c["vel"])))  # 6
```

**`from_snbt` does this too**, from the text's own `DataVersion`. `gametest_snbt`
stamps the schematic's version rather than a fixed one, and mc-tick's parser
reads it back, so the obvious round trip is faithful:

```python
snbt = TickSimulation.gametest_snbt(schem)   # DataVersion: 4082, NaN token intact
b = TickSimulation.from_snbt(snbt, TickSettleMode.InWorld, 0, 0, 0, "")
print(b.motion_semantics())                  # clamp_abs_ten — same machine
```

This used to be lossy in a way that was invisible: the emitted text hardcoded the
canonical oracle version (4903) and the parser skipped `DataVersion` entirely, so
`from_snbt` always chose `drop_non_finite` and the door's six NaN velocities came
back as zero. The NaN *token* survived the write the whole time — it was the load
rule that was thrown away. Both halves are pinned by
`the_snbt_round_trip_keeps_the_record_doors_nan_carts` and its negative control
in `src/bridge/mc_tick.rs`.

Two things still to know. A schematic that states no version at all is stamped
with the canonical one, which selects the modern NaN-dropping rule — a default,
not a reading of the file. And SNBT text written by hand may carry no
`DataVersion`, in which case `from_snbt` falls back to the engine default, also
`drop_non_finite`. Either way `motion_semantics()` tells you which rule you got,
and on a nan-cart build it is worth reading.

### SNBT can hold NaN; JSON cannot

Binary NBT stores `TAG_Double` as eight raw IEEE-754 bytes, so NaN and ±Infinity
are natively representable and survive a faithful reader untouched.

SNBT is where the format itself breaks down. Vanilla's number grammar requires a
digit, so there is no production for either value — yet vanilla's *writer* emits
`NaN`, `Infinity` and `-Infinity` via Java's `Double.toString`. **Vanilla can
write a structure file it cannot read back.** This parser accepts exactly those
three spellings and invents no notation of its own.

JSON cannot represent either value at all, which makes any JSON hop a place the
mechanism dies:

- `Schematic.get_entities_json()` renders the record door's six NaN components as
  `null`. A converter routed through it loses the machine before the engine ever
  sees it; the bridge reads the typed NBT directly for exactly this reason.
- `TickSimulation.item_entities_json()` goes the other way and emits the bare
  token `NaN`, which is **not valid JSON**. Python's `json.loads` accepts it by
  default and hands you `float('nan')`, which is usually what you want; a strict
  parser rejects it, and `JSON.parse` in JS throws. If you need strict JSON,
  substitute before parsing and accept that you are erasing the mechanism.

One more measured detail: on the record door the count is 6 NaN components and 0
Infinity. The ±Infinity values are intermediate states that have already
collided into NaN by the time a world is saved, so NaN is what a loader actually
meets — but handle Infinity anyway, because it is what NaN is made of.

## Entities

Entities are a registry, the same shape as the block side: one row per type in
`vanilla::entity_table()`, and a type with no row is refused **by name** rather
than loaded inert. Adding a frozen type is one row.

A row says four things — dimensions, whether the body stops a minecart, its
motion class, and which riders sit where on it.

### Dimensions are mechanism, not trivia

Read out of the game's own registry (`tools/gametest/src/EntityDims.java` prints
`EntityType.getDimensions()` after `Bootstrap.bootStrap`), so they cannot drift
the way a remembered number can:

| kind | width | height | stops a cart | motion |
|---|---|---|---|---|
| `minecart`, `furnace_minecart`, `chest_minecart`, `hopper_minecart`, `tnt_minecart` | 0.98f | 0.7f | yes | minecart |
| `item` | 0.25 | 0.25 | no | item |
| `fireball`, `dragon_fireball` | 1.0 | 1.0 | no | frozen |
| `small_fireball` | 0.3125 | 0.3125 | no | frozen |
| `villager` | 0.6f | 1.95f | yes | frozen |
| `blaze` | 0.6f | 1.8f | yes | frozen |
| `oak_boat` | 1.375 | 0.5625 | yes | frozen |
| `armor_stand` | 0.5 | 1.975f | **no** | frozen |

The `f` suffixes are load-bearing. `EntityType.VILLAGER` is `sized(0.6F, 1.95F)`
and both literals are floats, so a cart resting on a villager's head settles at
`2.950000047683716` — which is `1.0 + 1.95f`, not `1.0 + 1.95`. The eighth
decimal became observable the moment a cart could stand on one.

The widths decide designs. The record 3x3 door uses a **dragon** fireball where a
small one will not do: a dragon fireball is exactly one block tall, so resting at
the bottom of a cell it spans the whole cell and reaches a pressure plate at the
floor *and* the piston above. A small fireball is 5/16 and reaches neither.

**Motion classes.** `Item` is `ItemEntity.tick` — gravity, drag, collision,
merging. `Minecart` is `AbstractMinecart.tick` — rails, slopes, pushes,
cart-on-cart collision. `Frozen` is no physics at all: a hitbox that holds its
position. That is not a simplification for its own sake — the record doors'
fireballs are caught mid-flight by a piston-and-cobweb trick and genuinely have
zero motion, and so does a `noai` blaze standing on a plate. What the engine has
no answer for is one of these carrying real velocity, and it refuses that rather
than freezing something that ought to be moving. A frozen body is still displaced
by a piston arm; that is the one force in the engine that does not go through an
entity's own physics.

### The vehicle predicate: "living is solid" is refuted

The obvious rule for "does a minecart driving into this body stop" is
`Entity.canBeCollidedWith` read as *living entities are solid, projectiles are
transparent*. Ten bodies were measured, each twice — dropped on from above and
driven into sideways — and the rule fails at **both** edges at once:

- An **armor stand** is a `LivingEntity` and a cart falls straight through it. It
  reproduces the empty-floor control exactly.
- An **oak boat** is not living and holds a cart up, at 1.5625.

What fits all ten is vanilla's *vehicle* predicate: a cart's collision set is
`canBeCollidedWith() || isPushable()`, and `ArmorStand.isPushable` is overridden
to `false`. That reading is why the table is not ten coincidences — but the
measurement is still what the engine returns, not the derivation.

Two asymmetries are measured and deliberately absent, because this column only
answers "what stops a cart":

- A living body's *own* movement is not stopped by any of it. A blaze dropped onto
  a minecart lands on the floor at y = 1.0 on the same tick as the empty control.
  Carts are transparent to a falling mob. Nothing here moves a mob, so there is
  nothing to implement — but a future mob-physics pass must not reuse this table.
- A **rideable** cart coming within `inflate(0.2, 0, 0.2)` of a free living entity
  *mounts* it rather than being stopped by it, and carries on with the ridden
  `0.997` slowdown instead of the empty `0.96`. Not modelled. It cannot fire on
  the record door, whose rolling stock is furnace carts (not rideable) and whose
  blazes are already passengers (which vanilla's gate excludes).

A `zombie` measures as solid and is **not registered** — it is a row nobody has
needed yet, and until it exists a zombie refuses by name.

### Passengers

Riders are nested in the vehicle's `Passengers` list, not listed at the top
level, so **a top-level count under-reports what loads**. The record door's save
holds 22 top-level entities; the world it loads into holds 24, the difference
being two blazes riding two of the plain minecarts:

```python
ents = json.loads(sim.item_entities_json())
len(ents["minecarts"]) + len(ents["frozen"]) + len(ents["items"])   # 24
sorted(b["pos"][1] for b in ents["frozen"] if b["kind"] == "minecraft:blaze")
# [2.1875, 2.25]  — each its cart's y plus the measured 0.1875 seat
```

The whole cast, for a sense of what an entity-abuse build is made of: 15 furnace
minecarts, 4 plain minecarts, 2 dragon fireballs, 2 blazes and 1 small fireball.
Six of the 19 carts carry a NaN velocity component.

A rider has **no position of its own**. `Entity.rideTick` zeroes its velocity and
ticks it, and `positionRider` then hard-sets it to `vehicle.position() + seat`
with no collision check — every tick, unconditionally. So the engine stores
`(vehicle, seat)` and re-derives the box whenever anything moves. A cart rolling
east drags its rider's x to the last digit; a cart falling through air lands with
its rider; **a NaN vehicle pins its rider forever.**

The seat is a property of the **pair**. It is not a constant and it is not
derivable from the hitboxes — on one and the same minecart:

| rider | seat offset |
|---|---|
| `blaze` | +0.1875 |
| `small_fireball` | +0.1875 |
| `villager` | **0.0** — lower, despite being taller |

`entity::passenger_attachment` is that measured table and refuses every pair it
has not seen. Only the plain minecart is measured: the container and furnace
variants share its *hitbox*, but an attachment point is not a hitbox, so each
carries an empty seat list and a `Passengers` tag on one refuses in the parser.

**Not modelled: the rider's velocity.** The engine exposes a rider as a box with
no velocity where vanilla reports `(0, -0.0784000015258789, 0)` — one step of
living-entity gravity, overwritten before it can do anything. It is inert;
nothing reads it and it never moves the rider. But it is a difference, and it is
also exactly the number the door's saved riders carry, which is why that number
is *not* evidence they are falling.

### What refuses

Refusal is on **capability**, not on the type existing. Furnace carts, fireballs
and villagers load fine as the mass and hitboxes the record doors use them for.
What refuses is any of them carrying state that would need behaviour nobody has
implemented — a cart with `Fuel` or `PushX` to drive itself, a fireball with
`Motion` to fly, a villager with `Motion` to walk — because running one of those
as a stationary box is a confident wrong answer whose wrongness is invisible.

```python
# each of these raises, and the message names the type or the tag
'{... nbt: {id: "minecraft:creeper"}}'                                  # no row
'{... nbt: {id: "minecraft:dragon_fireball", Motion: [0.5d,0,0]}}'      # would fly
'{... nbt: {id: "minecraft:furnace_minecart", Passengers: [...]}}'      # unmeasured seat
```

## Pistons and entities

**Extension displaces without imparting velocity.** The entity is moved by the
slab the leading face sweeps and its own velocity is untouched — which is why a
frozen fireball stays frozen after being shoved half a block.

**Retraction is three geometries**, and which one applies is decided by the move,
not by the entity:

1. **A pulled block sweeps entities on the ordinary slab** — exactly as a pushed
   one does. Worth almost a whole block: a dragon fireball goes from 4.45 to 3.50
   and the plate it lands on powers.
2. **A head with nothing to pull clears its own square.** Not a slab at all: it
   clears the block the head is leaving and reaches nowhere else, and it can push
   an entity *backwards*, so the displacement is signed. The gate is a *point* in
   the vacated block, and which point differs by axis: along the piston it is the
   box centre, but **across** the piston the vertical coordinate is the entity's
   **feet**, not its centre. That asymmetry is the shape of `Entity.position()` —
   `(centre x, min y, centre z)` — showing through, and it is not cosmetic: the
   record door's fireball with feet at y = 0.875 and centre at 1.03125 was refused
   outright by a gate that used the centre everywhere. Eleven lanes separate the
   two readings on `min y` alone.
3. **An entity in the piston's own square is driven to the outermost line it can
   reach.** Gated across the axis on the piston *arm's* narrow column rather than
   on the whole block — measured to the thousandth, twelve lanes, and the edge
   flips exactly at 6/16. That is why a fireball resting on the floor *below* the
   arm is left alone while the same fireball lifted into the arm's band is thrown
   clear. Along the axis it tries the outermost target first, so an entity that
   can clear the whole square in one step does rather than stopping at the arm;
   one that cannot reach anything is shoved back a whole step and the next step
   finishes the job. This is the geometry the record door's downward-facing
   pistons actually use.

Cases 3 and 2 are tried in that order — an entity in the piston's square is
resolved there, and only one that is not falls through to case 1.

**Sweeps clip against the block arriving in the destination cell.** A cell a
piston is moving a block into holds a `moving_piston` placeholder, which is not a
full cube — so naive block collision sees the whole stroke as empty air. It is
not: `MovingPistonBlock` delegates its collision shape to the block entity, which
answers with the moved block's shape. Without this clip the record 3x3 door's
fireball ends 0.3225 east of where it started instead of flush against the
piston, a plate latches, and the west pair never releases.

**And only on the second half-step.** A moving block is transparent to the entity
it is shoving on the first of the two half-block steps and solid on the second.
That is measured, not assumed: widen the entity until its leading face already
sits on the line and the two rules give opposite answers — a dragon fireball
flush against the arriving cell would be pinned in place by a solid-all-along
rule, and vanilla moves it the full 0.51 anyway, then clips its *second* step to
0.49. This is `PistonMovingBlockEntity.getCollisionShape`'s `progress < 1.0 &&
NOCLIP == getMovementDirection()` showing through, with `progress` read after the
step's increment.

An entity moving under its **own** power matches neither NOCLIP clause, so it
gets the shape on *both* steps. That is not symmetry for its own sake: three of
the record door's fifteen furnace carts stand over sticky pistons that fire when
the button is pressed, and without this they took gravity for the two ticks the
head needed to come home, ended up inside the cell the retracting base was about
to reoccupy, and fell out of the world.

**A plate can be tripped by the *intermediate* box.** A piston shove is not a
jump from one settled box to another. Vanilla calls `entity.move(MoverType.PISTON,
…)` with the whole step, then pushes the entity back out of the piston's block —
and `entityInside` fires at the far position, *before* that correction. The
intermediate box is invisible in an entity log, which only prints settled
positions, but it is perfectly visible in a pressure plate's block state, and the
record 3x3 door is built on exactly that: a light weighted plate powers on the
retraction tick while the fireball's settled box never comes within 0.08 of the
plate's touch box, and the same fireball parked there never presses it.

`piston_retract_contacts()` is a tripwire from when retraction was unmodelled: it
counts entities the engine saw in a retracting sweep it could not reproduce. All
three geometries are implemented now, so it reports **0** — including on the
record door, which used to name six. It is kept because the next geometry that
turns out not to be covered should be reported rather than guessed at. A non-zero
value means a result is not trustworthy.

## Components that need entities to exist

**Detector rails.** `checkPressed` selects on `AbstractMinecart` alone, so no
other entity powers one, and it searches the cell inset by 0.2 on every side
except the bottom: `AABB(x+0.2, y, z+0.2, x+0.8, y+0.8, z+0.8)`. It powers 15 in
every direction, and strongly powers only the block *below* it — so dust touching
only that block still reads 15, and dust beside the rail's floor learns about the
change by no other route than the rail updating its own neighbours *and* the
block under it. Release is on a timer, not on departure: the capture powers on
tick 13 as the cart arrives and releases on tick 33, twenty ticks later,
regardless of when the cart left.

**Weighted pressure plates.** Fully simulated, and they used to be one of the
things this page said were not. The reasoning was that nothing could stand on one
with no player in the world; the record 3x3 door disproved it, because its plates
are pressed by *entities the pistons move*. The signal is
`Mth.ceil(min(count, maxWeight) * 15 / maxWeight)` in `f32`, over
`getEntitiesOfClass(Entity.class, …)` — **every** entity type counts, items
included. `maxWeight` is 15 for the light plate and 150 for the heavy one, which
is the entire difference between them:

| items on the plate | 0 | 1 | 3 | 5 | 11 |
|---|---|---|---|---|---|
| light (`maxWeight` 15) | 0 | 1 | 3 | 5 | 11 |
| heavy (`maxWeight` 150) | 0 | 1 | 1 | 1 | 2 |

A plate carries `power`, not `powered`, and emits that level rather than a flat
15 — read as omnidirectional 15 it would drive a full-strength line off a single
item. Rechecks run every **10** ticks, not the 20 an ordinary plate uses
(`WeightedPressurePlateBlock.getPressedTime` returns 10, overriding
`BasePressurePlateBlock`). Each recheck is scheduled as a relative delay from the
last one, so the cadence is anchored on the tick the entity arrived rather than on
any global phase. A plate also strongly powers the block it stands on, for
`Direction.UP` alone — so dust touching only that support block reads the plate's
level, and dust beside the support learns about a change by no other route.

**Bucket dispensing.** Vanilla splits the bucket family in two, so this is a
table rather than an `if`. A *filled* bucket (`water`, `lava`, `powder_snow`)
empties its contents into the cell in front and the dispenser is left holding
`minecraft:bucket`. The write gates on strictly-air and goes out with flags 3, so
the landing block hands out ordinary neighbour updates and an observer watching
that cell pulses two ticks later. A non-air front cell is **not** a refusal —
`emptyContents` returns false, the behaviour falls through to the default eject,
and the filled bucket leaves as an item entity while no block changes.

The *empty* bucket runs `BucketPickup.pickupBlock` on the front cell and takes the
filled bucket back. Where that lands depends on what is left in the slot: with the
stack down to one the slot is replaced in place; with items still in it the result
goes to the first slot holding the same item, else slot 0, else it is ejected.
Both directions are measured end to end on the same geometry, five and six lanes.

`minecraft:milk_bucket` is the one that looks like it should have a behaviour and
does not — it appears nowhere in `DispenseItemBehavior`. The seven **mob** buckets
do have one, and it is refused by name at dispense time rather than approximated:
`MobBucketItem.emptyContents` places the fluid *and* spawns the mob. The block
half would be easy; the entity half is a whole subsystem, and falling through to
the default eject would be a plausible wrong answer.

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
| `item_entities_json()` | item entities, minecarts and frozen bodies |
| `motion_semantics()` | which `Entity.load` rule this run is using |
| `piston_retract_contacts()` | entities in an unmodelled retracting sweep (0 today) |

Snapshots omit air. Absence means air — compare over the union of two snapshots'
keys rather than assuming a missing entry is a missing block.

`item_entities_json()` has three lists: `items` (with `count` and container
`contents`), `minecarts` (with `kind` and `vel`), and `frozen` (kind and
position, no velocity, riders included). It can emit the bare token `NaN`; see
[above](#snbt-can-hold-nan-json-cannot).

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

`phase` names where in the game's tick the update was delivered. The compact
views carry the legend in their payload (`phases`), so read it from there rather
than hard-coding it — it is eleven entries, vanilla's ten tick phases plus
`boundary` for dispatches outside the phase walk:

```python
json.loads(sim.updates_heat_json(0, 30))["phases"]
# ['boundary', 'world_border', 'weather', 'block_ticks', 'fluid_ticks', 'raids',
#  'chunk_manager', 'block_events', 'entities', 'block_entities', 'player_inputs']
```

Three of those matter for a piston door. A piston *decides* to move in
`block_ticks`, actually *starts* moving in `block_events` (which chains within
the same tick), and *completes* two phases later in `block_entities`. Player
input is last in the tick, which is why an input applied "now" is only observed by
the world on the following tick.

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

## Auditing a file before you trust it

Some exporters drop block entities. Every block is present and correct, the build
loads clean, and it behaves differently — because a comparator that lost its
stored `OutputSignal` reads 0, and a container that lost its contents reads
empty. There is no error to catch, so check instead:

```python
TickSimulation.block_entity_audit_json(schem)
# {"present":1,"missing_total":0,"missing":[],"summary":""}
```

`missing` counts blocks that *need* a block entity and have none, by name, and
`summary` is a sentence you can put in front of a user when it is non-empty. Run
it before certifying or benchmarking a build you did not save yourself. A `.schem`
export carrying no block entities at all is the common case.

## Self-testing builds

A `.litematic` can carry its own test. A root-level `NucleationTest` compound
holds a JSON descriptor beside `Metadata`; `tests/scenarios/` is scanned at
runtime, so dropping a file in adds a test with no recompilation and no Rust
anywhere that names the build.

```text
cargo test --test litematic_cases                        # all of them
MC_TICK_CASE=55_3x3 cargo test --test litematic_cases    # one
```

Assertions are **end-states only** — entity counts, rider seats, fills over named
cells, change counts, quiescence, minimum entity y. Nothing asserts a tick-by-tick
path, so a faster backend that reaches the same world still passes. The
assertion vocabulary and the `scenario_inspect` workflow for measuring a build and
embedding the numbers are in
[`tests/scenarios/README.md`](../../tests/scenarios/README.md) — this page does
not duplicate them.

The record door lives there, with two scenarios in one file: untouched it changes
no block in 400 ticks and stays quiescent with all 24 entities on their exact
seats; pressed, it closes 6 of its 9 doorway cells, settles at 4, records 227
block changes and drops nothing below y = 0. Its DataVersion of 4082 is not
decoration — it selects the load rule that keeps the NaN velocities the door is
made of, and a round trip that loses it produces a build that quietly does not
work.

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
answer.

**Retraction of a body wider than the piston arm is a known disagreement, and it
fails loudly.** Vanilla shoves such a body `+0.25` out and then `-0.25` back — a
round trip ending exactly where it began, because the two half-steps clip against
two different surfaces: the extended base's own collision box (the block minus
the 4/16 arm slot) on the way in, and the retracted full cube on the way out. Two
body widths 9.5e-9 apart both land on the same line, which no constant
displacement explains. The engine instead shoves `+0.49` and does not bring the
body back, because a 1.0-wide box straddles the arm column rather than lying
inside it, so the third geometry's cross-axis gate declines and the pulled
block's own sweep answers. The geometry is understood; the wiring does not exist.
It is asserted **as a disagreement**, with both sets of numbers, so that fixing
retraction's law fails that test rather than passing silently. Narrow bodies hide
it — a 0.3125 fireball fits inside the arm column and takes the other branch.

**The record door does not close.** Pressed, it closes 6 of its 9 doorway cells
and settles at 4. Nine is the goal; 6 is today's truth, pinned so that a
regression reads as a different number instead of as nothing at all.

**Mobs and players.** No AI, no pathing, no player physics. Mobs exist as frozen
hitboxes and are useful as scaffolding — a cart resting on a villager's head is a
real mechanism — but nothing moves them, and a mob carrying velocity refuses.
Cauldrons, campfires, unbooked lecterns and unpressed tripwire are registered
only in the state that is a fixed point with nobody present; any other state is
left unregistered so the build fails loudly.

**A rider's velocity.** Exposed as absent where vanilla reports −0.0784. Inert
today; a future behaviour that reads passenger velocity would find zero.

**Item stack sizes.** Everything is treated as stacking to 64, so comparator
container reads are correct only for 64-stackable items.

**Item drops.** A capture pins block-level outcomes; the items a destroyed block
produces are not pinned unless the capture was taken with entities.

**Boats and armor stands are obstacles, not vehicles.** Both are registered with
measured dimensions and a measured answer to "does this stop a cart" — a boat
does, an armor stand does not. Neither has physics of its own, and nothing rides
either.

**Some components.** Anything not implemented refuses to load and names itself,
rather than being silently treated as air or stone. That includes types the engine
knows about but carrying state it cannot run.

## Gotchas worth knowing before you hit them

**A schematic is not always a valid world state.** Formats lose things — run
`block_entity_audit_json` rather than trusting a clean load. If a build behaves
differently from how its author describes it, compare block-entity counts before
suspecting the engine.

**A JSON hop can destroy the mechanism.** Not a hypothetical: see
[the version gate](#nan-velocities-and-the-version-gate). NaN does not survive
JSON. The SNBT round trip does — `gametest_snbt` writes both the NaN token and the
DataVersion that decides whether loading keeps it, and `from_snbt` reads both —
but any detour through JSON still flattens the velocity to `null` or `0`.

**Derived properties are real state.** Repeater `locked`, note-block
`instrument`, wire connections: the game recomputes these on placement. Under
`InWorld` the engine trusts what the file says, which is usually right for a
saved build and always right for a build the file recorded at rest.

**A comparator emits what it stored, not what its state claims.** A comparator
whose block state says `powered=true` but whose block entity holds
`OutputSignal: 0` emits nothing — that is vanilla, verified by capture, and it is
how a schematic with empty containers quietly stops working.

**Counting entities at the top level under-reports.** Riders are nested. Count
what the simulation loaded, not what the file lists.

## Where to look next

- `crates/mc-tick/PROJECT.md` — the engine's own design notes and the
  verification discipline
- [`crates/mc-tick/docs/entity-abuse-in-record-doors.md`](../../crates/mc-tick/docs/entity-abuse-in-record-doors.md)
  — what the record doors do with entities, with a checklist marking what is
  settled, what is measured but unmodelled, and what was refuted
- [`tools/gametest/NAN-MOTION-VERSIONS.md`](../../tools/gametest/NAN-MOTION-VERSIONS.md)
  — the `Entity.load` bisect, with bytecode
- [`tests/scenarios/README.md`](../../tests/scenarios/README.md) — self-testing
  litematics: the descriptor, the assertion vocabulary, how to add one
- `crates/mc-tick/tests/cases/README.md` — the folder-driven scenario harness:
  one file per test, no recompilation to add a case
- `tools/gametest/README.md` — the vanilla oracle: how captures are produced
- [`docs/features/redstone-simulation.md`](redstone-simulation.md) — the redpiler,
  for when you want logic throughput rather than tick fidelity
