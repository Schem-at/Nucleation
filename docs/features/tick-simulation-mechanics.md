# Tick simulation: the mechanics underneath

The [user manual](tick-simulation.md) covers the API. This page is the measured
behaviour behind it: the load rules, entity semantics and displacement laws
that make community machines run. Read it when a build misbehaves and you want
to know whether the engine or the file is wrong, or when you are about to rely
on an edge the manual only names.

Everything here is capture-backed: derived from the unobfuscated Minecraft
server jar and validated against traces of the real game (`tools/gametest/`).
The full investigation that produced the entity model is preserved in
[`crates/mc-tick/docs/history/entity-abuse-in-record-doors.md`](https://github.com/Schem-at/Nucleation/blob/master/crates/mc-tick/docs/history/entity-abuse-in-record-doors.md).

## Why you can trust it

The engine is checked against a headless Minecraft 26.2 server through the
gametest harness in `tools/gametest`. Each structure produces a trace, which is
compared with the engine tick by tick. `crates/mc-tick/tests/traces/` contains
103 captures; tests name 88 of them. `cargo test -p mc-tick` runs 332 tests, with
326 passing and 6 ignored. The conformance binary replays 81 captures.

A capture takes precedence when it contradicts the implementation. Several
behaviours exist because a capture overturned the initial bytecode reading.
This page identifies those cases.

Where a behaviour could not be captured or verified, it is left unimplemented and
fails loudly rather than guessing. A block the engine cannot model refuses to
load, by name; so does an entity, and so does an entity that *is* modelled but
carries state needing behaviour nobody has implemented. That strictness is the
point: a quietly wrong simulation is the one failure mode this tool cannot
tolerate.

## Loading a build

| constructor | takes |
|---|---|
| `from_schematic(schematic, settle, ox, oy, oz, extra_states)` | any format nucleation reads |
| `from_snbt(text, settle, ox, oy, oz, extra_states)` | gametest-flavor structure SNBT |
| `from_blocks(...)` | a palette plus a flat index array: no text, for tight loops |

`gametest_snbt(schematic)` converts a schematic to the SNBT flavor the engine and
the gametest oracle both read, which is also what the video renderer consumes.
It carries the schematic's `DataVersion` through, so `gametest_snbt` → `from_snbt`
loads a build under the same `Entity.load` rules `from_schematic` would: read
[the version gate](#nan-velocities-and-the-version-gate) anyway, because *which*
rules those are is the single most consequential thing on this page.

### Settle mode is the most consequential argument

A schematic is not automatically a world. How you bring it to life changes what
you are measuring, and picking wrong produces a confidently wrong answer.

- **`InWorld`**: the build *is* the world. Nothing is placed, nothing settles.
  Use this for a build saved at rest: it preserves derived state the author saved,
  including repeater `locked` flags and comparator outputs.
- **`Placement`**: run vanilla's placement pass, exactly as pasting the build
  would. This is a *destructive* operation and that is faithful: `placeInWorld`
  re-derives repeater `locked` and wire connections, and loads block-entity NBT
  *after* the block writes. A door whose memory cell depends on a comparator
  reading a container will come up unlatched, because the container's contents
  do not exist yet at the moment the lock is derived. Use it when you want to
  know what happens when someone pastes the build.
- **`Quiet`**: `onPlace` only, no settle. Matches a `knownShape` capture.

If a build ticks to quiescence in zero ticks under `InWorld`, it was genuinely at
rest as saved, and that is the mode you want.

**`Quiet` is not "the gentle one".** Both `Quiet` and `Placement` run the
placement pass, which blanks the region and re-writes every block one at a time
so that each landing block's already-placed neighbours get a shape update. Every
observer in the build therefore watches the block it faces *appear*, and pulses.
On real doors that is not a rounding error: the reference set changes 50 to 896
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
`{"expect": "changes", "count": 0}` assertion in a scenario descriptor pins it: see `tests/scenarios/README.md`.

### `extra_states`, and why your redstone block does nothing

Behaviours bind to *interned* block states when the simulation is constructed. A
state that first appears later because you `place_block` it has no behaviour
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
`0, 0, 0` is fine: origin affects tick-exact ordering in wire cascades, not
whether a machine functions.

## NaN velocities and the version gate

This is the sharpest edge in the engine, and it is not a curiosity. The record
3x3 piston door: the smallest that exists, and a conformance target here: is
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
`NaN > 10` is false: the check that kills a large finite velocity cannot see a
NaN at all. The new rule discards the entire vector when any component is
invalid. The bisect left a gap between 4556 and 4671 with no released version
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
print(b.motion_semantics())                  # clamp_abs_ten: same machine
```

This used to be lossy in a way that was invisible: the emitted text hardcoded the
canonical oracle version (4903) and the parser skipped `DataVersion` entirely, so
`from_snbt` always chose `drop_non_finite` and the door's six NaN velocities came
back as zero. The NaN *token* survived the write the whole time: it was the load
rule that was thrown away. Both halves are pinned by
`the_snbt_round_trip_keeps_the_record_doors_nan_carts` and its negative control
in `src/bridge/mc_tick.rs`.

Two things still to know. A schematic that states no version at all is stamped
with the canonical one, which selects the modern NaN-dropping rule: a default,
not a reading of the file. And SNBT text written by hand may carry no
`DataVersion`, in which case `from_snbt` falls back to the engine default, also
`drop_non_finite`. Either way `motion_semantics()` tells you which rule you got,
and on a nan-cart build it is worth reading.

### SNBT can hold NaN; JSON cannot

Binary NBT stores `TAG_Double` as eight raw IEEE-754 bytes, so NaN and ±Infinity
are natively representable and survive a faithful reader untouched.

SNBT is where the format itself breaks down. Vanilla's number grammar requires a
digit, so there is no production for either value: yet vanilla's *writer* emits
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
meets. Infinity is handled because it is one source of NaN.

## Entities

Entities are a registry, the same shape as the block side: one row per type in
`vanilla::entity_table()`, and a type with no row is refused **by name** rather
than loaded inert. Adding a frozen type is one row.

A row says four things: dimensions, whether the body stops a minecart, its
motion class, and which riders sit where on it.

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/tick-sim/entity-cast.png" width="880" alt="The registry's cast rendered: the five minecart variants, a blaze riding a cart, a blaze, a villager, an armor stand, an oak boat, and the three fireball sizes as sprites">

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
`2.950000047683716`. That value is `1.0 + 1.95f`, rather than `1.0 + 1.95`. The eighth
decimal became observable the moment a cart could stand on one.

The widths decide designs. The record 3x3 door uses a **dragon** fireball where a
small one will not do: a dragon fireball is exactly one block tall, so resting at
the bottom of a cell it spans the whole cell and reaches a pressure plate at the
floor *and* the piston above. A small fireball is 5/16 and reaches neither.

**Motion classes.** `Item` is `ItemEntity.tick`: gravity, drag, collision,
merging. `Minecart` is `AbstractMinecart.tick`: rails, slopes, pushes,
cart-on-cart collision. `Frozen` is no physics at all: a hitbox that holds its
position. That is not a simplification for its own sake: the record doors'
fireballs are caught mid-flight by a piston-and-cobweb trick and genuinely have
zero motion, and so does a `noai` blaze standing on a plate. What the engine has
no answer for is one of these carrying real velocity, and it refuses that rather
than freezing something that ought to be moving. A frozen body is still displaced
by a piston arm; that is the one force in the engine that does not go through an
entity's own physics.

### The vehicle predicate: "living is solid" is refuted

The obvious rule for "does a minecart driving into this body stop" is
`Entity.canBeCollidedWith` read as *living entities are solid, projectiles are
transparent*. Ten bodies were measured, each twice: dropped on from above and
driven into sideways. The rule fails at both edges at once:

- An **armor stand** is a `LivingEntity` and a cart falls straight through it. It
  reproduces the empty-floor control exactly.
- An **oak boat** is not living and holds a cart up, at 1.5625.

What fits all ten is vanilla's *vehicle* predicate: a cart's collision set is
`canBeCollidedWith() || isPushable()`, and `ArmorStand.isPushable` is overridden
to `false`. That reading explains the ten rows. The
measurement is still what the engine returns, not the derivation.

Two asymmetries are measured and deliberately absent, because this column only
answers "what stops a cart":

- A living body's *own* movement is not stopped by any of it. A blaze dropped onto
  a minecart lands on the floor at y = 1.0 on the same tick as the empty control.
  Carts are transparent to a falling mob. Nothing here moves a mob, so there is
  nothing to implement. A future mob-physics pass must not reuse this table.
- A **rideable** cart coming within `inflate(0.2, 0, 0.2)` of a free living entity
  *mounts* it rather than being stopped by it, and carries on with the ridden
  `0.997` slowdown instead of the empty `0.96`. Not modelled. It cannot fire on
  the record door, whose rolling stock is furnace carts (not rideable) and whose
  blazes are already passengers (which vanilla's gate excludes).

A `zombie` measures as solid and is **not registered**: it is a row nobody has
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
# [2.1875, 2.25]: each its cart's y plus the measured 0.1875 seat
```

The whole cast, for a sense of what an entity-abuse build is made of: 15 furnace
minecarts, 4 plain minecarts, 2 dragon fireballs, 2 blazes and 1 small fireball.
Six of the 19 carts carry a NaN velocity component.

A rider has **no position of its own**. `Entity.rideTick` zeroes its velocity and
ticks it, and `positionRider` then hard-sets it to `vehicle.position() + seat`
with no collision check: every tick, unconditionally. So the engine stores
`(vehicle, seat)` and re-derives the box whenever anything moves. A cart rolling
east drags its rider's x to the last digit; a cart falling through air lands with
its rider; **a NaN vehicle pins its rider forever.**

The seat is a property of the **pair**. It is not a constant and it is not
derivable from the hitboxes: on one and the same minecart:

| rider | seat offset |
|---|---|
| `blaze` | +0.1875 |
| `small_fireball` | +0.1875 |
| `villager` | **0.0**: lower, despite being taller |

`entity::passenger_attachment` is that measured table and refuses every pair it
has not seen. Only the plain minecart is measured: the container and furnace
variants share its *hitbox*, but an attachment point is not a hitbox, so each
carries an empty seat list and a `Passengers` tag on one refuses in the parser.

**Not modelled: the rider's velocity.** The engine exposes a rider as a box with
no velocity where vanilla reports `(0, -0.0784000015258789, 0)`: one step of
living-entity gravity, overwritten before it can do anything. It is inert;
nothing reads it and it never moves the rider. But it is a difference, and it is
also exactly the number the door's saved riders carry, which is why that number
is *not* evidence they are falling.

### What refuses

Refusal is on **capability**, not on the type existing. Furnace carts, fireballs
and villagers load fine as the mass and hitboxes the record doors use them for.
What refuses is any of them carrying state that would need behaviour nobody has
implemented: a cart with `Fuel` or `PushX` to drive itself, a fireball with
`Motion` to fly, or a villager with `Motion` to walk. Running one of those
as a stationary box is a confident wrong answer whose wrongness is invisible.

```python
# each of these raises, and the message names the type or the tag
'{... nbt: {id: "minecraft:creeper"}}'                                  # no row
'{... nbt: {id: "minecraft:dragon_fireball", Motion: [0.5d,0,0]}}'      # would fly
'{... nbt: {id: "minecraft:furnace_minecart", Passengers: [...]}}'      # unmeasured seat
```

## Pistons and entities

The engine runs vanilla's actual `PistonMovingBlockEntity.moveCollidedEntities`,
shape for shape, rather than a fitted approximation:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/tick-sim/piston-shove.gif" width="560" alt="A sticky piston extends into a frozen dragon fireball, displacing it by exactly its penetration depth plus 0.01, and leaves it exactly where it was shoved when the head retracts">

That clip is the engine: the stroke displaces the fireball by its penetration
depth plus vanilla's `0.01`, imparts no velocity, and the retraction: with the
fireball outside the head's drag shapes: leaves it exactly where it landed.

- **The swept shape is the real one.** A carried block sweeps its collision
  shape; a source piston sweeps the piston head's own plate-and-arm boxes, with
  the arm going `short` once retraction passes quarter progress.
- **Extension displaces without imparting velocity.** An entity is moved by the
  slab the leading face sweeps; its velocity is untouched, which is why a frozen
  fireball stays frozen after being shoved half a block.
- **Retraction is a drag and a correction.** The head drags entities toward the
  base. It is clipped against the retracting base's own 12/16 collision box and
  `fixEntityWithinPistonBase` then pushes anything still overlapping the
  piston's cell back out. The two `+0.01` overshoots of that pair are why every
  head-ejected entity settles exactly `0.02` inside the vacated block.
- **One measured exception to pure box math.** The arm's drag applies only when
  the entity's `position()` point: centre x, **min** y, centre z: lies inside
  the arm's 4/16 column. A cart whose box genuinely overlaps the column by 0.2
  is untouched; the same cart centred on it is dragged. The oracle refused the
  plain box-intersect reading of the source, and the oracle wins.
- **`limitPistonMovement` composes same-tick shoves.** A tick's piston
  displacements accumulate per axis and clamp to ±0.51: two strokes shoving one
  entity compose through the accumulator, not by addition.
- **In-flight cells turn solid one at a time.** A cell a piston is moving a
  block into is transparent to that stroke's own cargo while its block entity is
  still mid-move, and becomes solid: at its destination: the moment that block
  entity has ticked to completion, in block-entity order. An entity moving under
  its *own* power sees the shape on both half-steps.
- **`entityInside` fires the tick a shove lands.** A pressure plate presses on
  the very tick a piston drags a fireball onto it, and a plate can be tripped by
  the *intermediate* box of a drag that a correction then takes back: invisible
  in an entity log, perfectly visible in the plate's block state.
- **Sticky short pulses drop the block.** This applies only to a block still travelling
  *away* from the piston. A move retracting through the target cell is left to
  land on its own cadence.

`piston_retract_contacts()` is a tripwire from when retraction was unmodelled.
It reports **0** on everything the engine can run, and a non-zero value means a
result is not trustworthy.

All of this is pinned by the record 55-block 3x3 door, which runs its full
close-and-reopen cycle with zero divergent ticks against a capture of real
Minecraft (`tools/gametest/captures/door55_cycle.entities.log`).

## Components that need entities to exist

**Detector rails.** `checkPressed` selects on `AbstractMinecart` alone, so no
other entity powers one, and it searches the cell inset by 0.2 on every side
except the bottom: `AABB(x+0.2, y, z+0.2, x+0.8, y+0.8, z+0.8)`. It powers 15 in
every direction, and strongly powers only the block *below* it. Dust touching
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
`getEntitiesOfClass(Entity.class, …)`: **every** entity type counts, items
included. `maxWeight` is 15 for the light plate and 150 for the heavy one, which
is the entire difference between them:

| items on the plate | 0 | 1 | 3 | 5 | 11 |
|---|---|---|---|---|---|
| light (`maxWeight` 15) | 0 | 1 | 3 | 5 | 11 |
| heavy (`maxWeight` 150) | 0 | 1 | 1 | 1 | 2 |

A plate carries `power`, not `powered`, and emits that level rather than a flat
15: read as omnidirectional 15 it would drive a full-strength line off a single
item. Rechecks run every **10** ticks, not the 20 an ordinary plate uses
(`WeightedPressurePlateBlock.getPressedTime` returns 10, overriding
`BasePressurePlateBlock`). Each recheck is scheduled as a relative delay from the
last one, so the cadence is anchored on the tick the entity arrived rather than on
any global phase. A plate also strongly powers the block it stands on, for
`Direction.UP` alone. Dust touching only that support block reads the plate's
level, and dust beside the support learns about a change by no other route.

**Bucket dispensing.** Vanilla splits the bucket family in two, so this is a
table rather than an `if`. A *filled* bucket (`water`, `lava`, `powder_snow`)
empties its contents into the cell in front and the dispenser is left holding
`minecraft:bucket`. The write gates on strictly-air and goes out with flags 3, so
the landing block hands out ordinary neighbour updates and an observer watching
that cell pulses two ticks later. A non-air front cell is **not** a refusal: `emptyContents` returns false, the behaviour falls through to the default eject,
and the filled bucket leaves as an item entity while no block changes.

The *empty* bucket runs `BucketPickup.pickupBlock` on the front cell and takes the
filled bucket back. Where that lands depends on what is left in the slot: with the
stack down to one the slot is replaced in place; with items still in it the result
goes to the first slot holding the same item, else slot 0, else it is ejected.
Both directions are measured end to end on the same geometry, five and six lanes.

`minecraft:milk_bucket` is the one that looks like it should have a behaviour and
does not: it appears nowhere in `DispenseItemBehavior`. The seven **mob** buckets
do have one, and it is refused by name at dispense time rather than approximated:
`MobBucketItem.emptyContents` places the fluid *and* spawns the mob. The block
half would be easy; the entity half is a whole subsystem, and falling through to
the default eject would be a plausible wrong answer.
