# Entities as redstone: what the record doors actually do

Source: the door team's own explanation of the smallest 3x3 (Ganymede, doormaker
and Sacred, interviewed on video). Written down here because the sample world
`55_3x3.zip` is a conformance target, and every one of these behaviours is load
bearing — **nothing in that build is decorative.** If an entity is there, it is
holding something in place or triggering something.

The one-line reason all of it exists: they wired as much of the piston layout as
conventional redstone allowed, ran out of space, and then used entities to *cram
redstone circuits inside other redstone circuits*. Entities occupy the space
that blocks cannot.

## Why the sample is a world and not a schematic

The mechanism is entity state — positions to sub-block precision, hitboxes, and
above all **velocities that are not finite numbers**. Pasting the build breaks
it, because placement re-derives what the entities were carefully glitched into.
A schematic that round-trips the blocks perfectly still produces a door that
does not work.

## NaN minecarts ("nan carts") — the load-bearing glitch

Minecarts colliding on the same rail amplify each other's velocity. On sloped
rails the door team drive that to overflow, so the velocity saturates to
**±Infinity**. Collide a `+Infinity` cart with a `-Infinity` cart and the game
computes `+Inf + -Inf`, which is **NaN**. That cart's physics are then dead:

- it does not fall when the block under it is broken,
- it cannot be pushed by players or other entities,
- **only a piston can move it**,
- and it is *contagious* — a normal cart that collides with it multiplies by NaN
  and becomes a NaN cart too ("a zombie minecart").

Four nan carts in this door, in two pairs. One pair holds two villagers at exact
Y values and stops the villagers shoving them around; the other pair sits inside
a block beside the villagers, where an ordinary cart would immediately be pushed
out of the door. They are used as **glue**: if any of these were free to move,
the door would break.

### What this demands of the simulator

**IEEE-754 semantics must be preserved exactly. Do not sanitise.** Any
`if !v.is_finite() { v = 0.0 }`, any clamp, any "reasonable bounds" guard on
entity velocity will convert a nan cart into an ordinary cart, which then moves,
which un-glues the machine. The failure is total and it is silent.

This includes serialisation. Measured on this world: **6 NaN components, 0
Infinity, 60 finite** — the ±Infinity values are intermediate states that have
already collided into NaN by the time the world is saved, so NaN is what a
loader actually meets. Handle Infinity anyway; it is what NaN is made of.

Binary NBT stores TAG_Double as eight raw IEEE-754 bytes, so NaN and ±Infinity
are natively representable and survive a faithful reader untouched. **SNBT is
where it breaks**: the text grammar has no production for either — vanilla's
number pattern requires a digit — yet vanilla's *writer* emits `NaN`,
`Infinity` and `-Infinity` via Java's `Double.toString`. Vanilla can therefore
write a structure file it cannot read back. Our parser accepts exactly those
three spellings and invents no notation of its own.

JSON is the other trap: it cannot represent either value, so any JSON hop
renders them `null`. That is not hypothetical — `get_entities_json` does
exactly this, and a converter routed through it would lose the mechanism before
the engine ever saw it. The bridge reads the typed NBT directly for this reason.

Note also that surviving finite components are values like
`4.27987680632209e-59`. A formatter that drops to exponent notation and a parser
that does not accept exponents combine into a silent corruption: `4.27e-59`
reads as *two* numbers, a three-element `Motion` becomes four elements, and the
whole vector is then discarded as malformed. The two fixes only work together.

## Frozen fireballs — one-block-tall pressure-plate triggers

Fireballs are frozen in place (piston + cobweb timing catches them mid-flight;
small fireballs come from a dispenser, dragon fireballs from a real dragon).
Once frozen they sit still and **a piston arm can push and pull them**. They
exist to hit a pressure plate, so that finishing one piston movement
automatically starts the next, while costing only the single block the plate
occupies.

Hitbox size is the whole reason both kinds appear:

- **small fireball** — small; used for the double piston extender, where it
  barely clips the plate as the piston retracts.
- **dragon fireball** — much larger, and specifically **one block tall**, needed
  where the trigger must reach both a pressure plate at the bottom of a block
  *and* the piston above it. A small fireball cannot span that.
- ghast fireballs would work but are punchable, with a large interaction radius,
  so they are not viable.

So fireball **hitbox dimensions are mechanism, not cosmetics**, and the plate
must respond to a fireball entity resting on it.

## Villagers — hitboxes used as scaffolding

The villagers power nothing. One acts as a **wall** stopping a minecart
travelling too far right; the other acts as a **floor** the minecart lands on
during closing. Their combined effect is that the detector rail inside them
is activated by the cart on the *opening* sequence only, and not on closing.

They stay put because they are held by a pair of nan carts. What is needed from
a villager here is its **hitbox and its solidity to entity collision** — not AI,
not pathfinding, not trading.

## Minecart chains — carts supporting carts

The remaining carts (a top row of five/seven, plus a stack) do exactly one job
each: hold another cart in position. A cart outside the piston's push range is
chained to the others so they all collide and hold each other in place, which
stops the critical cart drifting left and falling out of the door. The lower
stack props up a cart that has no block beneath it when the door is closed.

Remove the supports and the critical cart falls through, the top row dislodges,
and the door is destroyed. This is why **entity-to-entity collision is not
optional** for this build.

## The point of all of it

Every one of these tricks — nan carts, frozen fireballs, villager scaffolding,
cart chains — exists so that **one detector rail powers during the opening
sequence and not the closing one**, in a space with no room left for redstone.

## Checklist for the engine

Every line below now says where it stands and what settled it. `[x]` means a
capture agrees with the engine; `[~]` means measured but only partly modelled;
`[ ]` means still unverified. **Contagion is struck through because the oracle
refuted it** — the builders' account was wrong, and this page was wrong with it.

- [x] IEEE-754 velocities preserved end to end: NaN, +Inf, -Inf and denormals,
      through NBT read, SNBT write, SNBT parse, and the physics itself.
      *Version-dependent, and the version is the whole story* — see
      `tools/gametest/NAN-MOTION-VERSIONS.md` and `crates/mc-tick/src/motion.rs`.
      Under DataVersion <= 4556 `Entity.load` keeps NaN and zeroes infinities;
      from 4671 it drops the whole vector. The record door is 4082.
- [ ] ~~NaN contagion through cart-cart collision (`normal * NaN = NaN`).~~
      **REFUTED by `nan_contagion.entities.log`.** `setDeltaMovement` refuses a
      non-finite vector, so a NaN velocity can never be written *into* a cart
      that does not already have one, and never written *out of* one that does.
      A NaN cart is a fixed point: inert, unmovable, never lost, never spread.
      Across 61 samples a normal cart struck a NaN cart at t36 and stayed
      finite for the remaining 24 ticks; the non-finite count never changed.
      The "zombie minecart" does not exist.
- [x] Entity-to-entity collision and the resulting velocity amplification.
      Bit-exact across five captures, including the chain law — carts block each
      other through the same sweep as blocks.
- [~] Pistons move entities — including entities whose own physics are dead.
      **Extension is measured and exact** (`piston_entity.json`): the shove is
      the depth of the entity's own hitbox in the arm's half-block sweep, capped
      at the step, plus 0.01, and it is positional only — no velocity is
      imparted, so a NaN cart arrives still NaN. A normal cart ends a full
      extension 1.0 blocks along, a dragon fireball 1.01, a small fireball
      0.666. **Retraction is not.** `piston_pull.entities.log` shows a solid
      head does not eject an entity standing inside it, and that a *retracting*
      `moving_piston` displaces entities only fractionally and not uniformly
      backwards — nothing here reproduces a "pull" of a whole block, and no
      model tried so far predicts those numbers. The engine therefore does not
      displace on retraction, and *counts* every time an entity stands in a
      retracting sweep instead
      (`Simulation::piston_retract_contacts`), so a build that depends on the
      unmodelled path cannot look like one that does not.
- [x] Per-entity hitbox dimensions. Read out of the game's own registry and
      cross-checked against plate-edge probes: minecart and every container
      variant 0.98 x 0.7, dragon fireball 1.0 x 1.0, small fireball 0.3125,
      villager adult 0.6 x 1.95 and baby 0.49 x 0.98, item 0.25. An unmeasured
      entity gets **no** box and the simulation refuses it by name.
- [~] Entities resting on other entities' hitboxes. Carts collide with carts
      exactly; a *frozen* body (fireball, villager) is an obstacle to a cart but
      has no support physics of its own, because it has no physics at all.
- [x] Pressure plates triggered by entity presence, fireballs included —
      `fireball_reach.json` finds the plate's edge exactly where the measured
      widths put it.
- [x] Detector rail powered by a cart, and *only* by a cart:
      `DetectorRailBlock.checkPressed` selects on `AbstractMinecart`, so a
      fireball or villager standing on one must not power it.
- [x] `furnace_minecart` as a cart variant. Dimensionally identical to a plain
      cart, which is all the record door needs — all fifteen of its furnace
      carts carry `Fuel: 0` with `PushX`/`PushZ` zero. **Self-propulsion is not
      implemented**, and a fuelled cart is refused rather than run as a
      passenger.

Where the oracle disagrees with this page, the oracle wins and this page is
wrong — which has already happened once, to the contagion line above.

## What is still not known about this door

The door's mechanism is DataVersion 4082 and the only oracle available is 26.2,
where `Entity.load` destroys that mechanism before the first tick. So there is
**no capture of this door working**, and there cannot be one without a 1.21.3
oracle. `door55_in_world.entities.log` is a capture of it *failing*: 24
entities, zero non-finite, and eight of them — including both former nan carts
and the minecarts two blazes are riding — moving on tick 0.

That is the boundary of what verification can currently say. Anything below the
line "the engine keeps the nan carts frozen where 26.2 frees them" is unchecked
against the game.
