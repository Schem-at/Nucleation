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
      0.666. **Retraction is now measured too, and it is two mechanisms, not
      one.** `piston_pull.entities.log` read as inconclusive only because it was
      read as displacements — the carts move `+0.01` and the small fireball
      `-0.32375`, which look unrelated. As a box edge they are one number: all
      four finish with their trailing face at exactly `3.02`.
      - **A pulled block sweeps.** A sticky piston retracting *with a block to
        pull* drags entities by the ordinary `sweep_displacement` slab, the same
        one extension uses. This is the real "pull", and it is worth almost a
        whole block: in `piston_pull_plate` a dragon fireball goes 4.45 → 3.94 →
        3.50 and **the pressure plate it lands on powers** — a block-state
        reading, so no entity filter can distort it. Mirrored on the opposite
        axis, with negative controls for "never retracts" and "clear of the
        sweep". `piston_pull.snbt` had **nothing to pull**, which is why it saw
        none of this.
      - **A retracting head ejects its own square, and reaches nowhere else.**
        `head_eject_displacement`: gated on the entity's *centre* being in the
        block the head is leaving — a box-overlap gate is refuted by a dragon
        fireball overlapping that block by 0.05 which vanilla never touches —
        and it drives the trailing face to `blockMin + 0.02`, in **either**
        direction, at most `PISTON_STEP + PISTON_OVERSHOOT` per step. Fitted
        bit-exactly to fifteen lanes across four captures, sticky and
        non-sticky, at two heights, both sides of the block.
      - **An entity in the piston's own square is ejected by the arm.** This is
        what the record door's downward-facing pistons do, and it was the last
        of the three left open, because `piston_pull_inside` moved a vertical
        entity where `piston_pull_law` lane 1 left a horizontal one alone.
        **Neither the axis nor the floor was the difference, and both captures
        were right.** Lane 1's fireball sits at y = 1.0, so its box tops out at
        1.3125 — *below the piston's arm*, which starts at 1.375. Lift the same
        fireball by 0.34375, change nothing else, and vanilla throws it
        `0.41625`, which is exactly what the vertical law demanded.
        `piston_pull_float` is that capture: lane 1 unchanged and still
        motionless, lane 2 the same fireball lifted and moving, and a minecart,
        a furnace cart and a **NaN furnace cart** all moving from the floor
        because at 0.7 tall they reach the arm without being lifted.
        `crate::piston::inside_eject_displacement` is the law:
        - the gate across the piston is the **arm's own 4/16 column**, strictly
          — `piston_square_yband` walks a box across both edges in thousandths
          and the answer flips exactly at 6/16 and 10/16, twelve lanes;
        - along the piston, the entity is driven to the outermost of three
          lines it can reach in one `PISTON_MAX_STEP`: trailing face `1.01` of
          the way through the square, or `0.76`, or leading face back to `0.24`;
          failing all three it retreats a whole step. The second step's lines
          are `1.02` — the same `blockMin + 0.02` the head-only law lands on —
          and `-0.01`;
        - the hand-over is at *exactly* `0.51`: `piston_square_threshold` shows
          a target costing 0.51 taken and one costing 0.5101 refused, at both
          of them.
        Fitted bit-exactly to fifty-five lanes across `piston_pull_inside`,
        `piston_pull_float`, `piston_pull_xsweep`, `piston_square_cart` and
        `piston_square_threshold`, on both axes and both sides of the block,
        for a small fireball, a dragon fireball, a minecart, a furnace cart and
        a NaN furnace cart. **One lane in fifty-five disagrees**, `x = 2.65635`
        in the threshold rig, where vanilla moves `0.51` and the law says
        `0.5099`; every neighbour on both sides agrees, so it is a seam 1e-4 of
        a block wide, and it is asserted as a disagreement rather than hidden.
        Nothing produces `PistonPush::Unmodelled` any more, so
        `Simulation::piston_retract_contacts` is empty by construction. On the
        3x3 door it reads **0** — but it read 0 before this change too, in every
        run reachable from `examples/door55_sim`: under `InWorld` the door does
        not retract a piston at all, and under `Quiet` the contacts had already
        been consumed. What the new law does change on that door is that a
        **third** furnace cart moves — id 20, sitting in a
        `sticky_piston[extended=false,facing=down]` square, which is precisely
        this geometry. The "six remaining contacts" this section used to quote
        are not reproducible from the example as it stands, and that is itself
        unverified.
      - **The rig was lying, and it was not the game's fault.**
        `piston_pull_law` is not uniform: lanes z = 15, 17, 19 and 21 carry a
        stone block at `(4,1,z)` for the sticky head to **pull** and the other
        eight have nothing. The pulled block's sweep lands on top of everything
        else, so the identical fireball finishes at 3.02 in a pull-free lane and
        3.00 in a pulling one — which read as run-to-run non-determinism for an
        hour. `piston_pull_uniform` is the proof (twelve lanes, one start
        position, the answer changes at exactly z = 15), and
        `piston_pull_square` is the same rig rebuilt with twelve pull-free
        lanes. Every constant above comes from `piston_pull_square`.
        **A retraction that both pulls a block and closes over an entity in its
        own square is still not modelled as a combination**: the engine applies
        the two laws independently and vanilla does not, by 0.02 in the lanes
        measured.
- [x] Per-entity hitbox dimensions. Read out of the game's own registry and
      cross-checked against plate-edge probes: minecart and every container
      variant 0.98 x 0.7, dragon fireball 1.0 x 1.0, small fireball 0.3125,
      villager adult 0.6 x 1.95 and baby 0.49 x 0.98, **blaze 0.6 x 1.8**,
      item 0.25. An unmeasured entity gets **no** box and the simulation refuses
      it by name. **The registry's literals are `float`**, and the eighth decimal
      is observable: the villager's height was written as the decimal `1.95`
      until a cart could stand on one, at which point `blaze_ride_ai` read
      `2.950000047683716` where the engine gave `2.95`. It is `1.95F` —
      1.9500000476837158 — like the cart's `0.98F` and the blaze's `1.8F` before
      it. The blaze is `blaze_reach.entities.log`: nine floor plates
      straddling the width edges — clear at 1.76 and 11.24, touching at 5.77 and
      15.23, which bounds the half-width in (0.2925, 0.3025) — plus the four
      baby-villager offsets, where a 0.49-wide body reads clear and a blaze
      reads *touching*, so it cannot be the baby's width either. Height comes
      from a plate two blocks up that a blaze with its feet at 1.205 reaches and
      one at 1.195 does not, bounding it in (1.795, 1.805);
      `blaze_reach_villager_control.entities.log` is the same rig with a
      1.95-tall villager, which reaches both.
- [x] **Passengers.** Entities nested in a vehicle's `Passengers` list rather
      than listed at the top level, which is what the record door's two blazes
      are and why a top-level count of the save reads 22 where vanilla reads 24.
      Read by the parser, emitted by the bridge's SNBT writer, seated by
      `Simulation::spawn_authored_rider`. A rider has **no position of its own**:
      `Entity.rideTick` zeroes its velocity, ticks it, and `positionRider` then
      hard-sets it to `vehicle.position() + seat` with no collision check — so
      the engine stores `(vehicle, seat)` and re-derives the box whenever
      anything moves. `blaze_ride.entities.log` measures all of it over twenty
      ticks: a cart at rest, a cart rolling east (the rider's x tracks to the
      last digit), a cart falling through air (the rider lands with it), and a
      **NaN** cart, whose rider is pinned forever. `blaze_ride_ai.entities.log`
      repeats the last two with AI on and gets the same positions with the
      rider's velocity reading `(0, -0.0784000015258789, 0)` every tick and
      never moving it — one step of living-entity gravity, overwritten before it
      can do anything. That number is exactly what the door's saved riders
      carry, which is why it is *not* evidence they are falling.

      The seat is a property of the **pair**, not a constant and not derivable
      from the hitboxes: on one and the same minecart a blaze sits 0.1875 above
      it, a small fireball 0.1875, and a **villager 0.0** — lower, despite being
      taller. `entity::passenger_attachment` is that measured table and refuses
      every pair it has not seen; `Passengers` on any vehicle but a plain
      minecart refuses in the parser.

      **Not modelled: the rider's velocity.** The engine exposes a rider as a
      box with no velocity, where vanilla reports −0.0784. It is inert — nothing
      reads it and it never moves the rider — but it is a difference, and a
      future behaviour that reads passenger velocity would find zero here.
- [x] Entities resting on other entities' hitboxes. **This line has now been
      wrong twice, and both times the oracle said which half.**
      `blaze_ride_ai.entities.log` drops a minecart from y = 3.0 onto four
      different bodies at y = 1.0, with a fifth lane over bare floor as the
      control:

      | body under it | cart settles at | reading |
      |---|---|---|
      | blaze (1.8 tall) | 2.799999952316284 | rests on it, exactly its top |
      | villager (1.95) | 2.950000047683716 | rests on it, exactly its top |
      | small fireball | 1.0 | falls straight through |
      | dragon fireball | 1.0 | falls straight through |
      | nothing (control) | 1.0 | falls |

      This page then read that as `Entity.canBeCollidedWith` — "true for a living
      entity, false for a fireball". **`cart_body` refutes it at both edges.**
      Six more bodies, met twice each (dropped on from above and driven into
      sideways), and two of them fall the wrong side of "living":

      | body | cart | rest height / stop face |
      |---|---|---|
      | zombie | **solid** | 2.950000047683716 |
      | oak boat | **solid** | 1.5625 (0.5625 tall, 1.375 wide) |
      | minecart | **solid** | 1.699999988079071 |
      | **armor stand** | transparent | reproduces the control exactly |
      | ghast fireball | transparent | ditto |
      | item entity | transparent | ditto |

      An **armor stand is a `LivingEntity` and a cart falls straight through
      it**; a **boat is not living and holds one up**. What fits all ten is
      vanilla's *vehicle* predicate — a cart's collision set is
      `canBeCollidedWith() || isPushable()`, and an armor stand answers false to
      both. `entity::blocks_a_cart` is that table, kind by kind, refusing any
      kind it has not seen, and `sim::cart_obstacle_bodies` feeds the qualifying
      boxes into the **same** obstacle list the other carts go through.

      Four more things the same captures settle:

      - **It is a full box, not a ledge.** `cart_body2` drives a cart east into
        a blaze and it stops with its east face at `6.199999988079071` — the
        blaze's west face to the last bit. A support-only model drives through.
      - **`onGround` is set by it.** `cart_body`'s z=37.5 and z=40.5 lanes are
        the same cart with vx = 0.1, one over a blaze and one over stone; the
        blaze lane takes `comeOffTrack`'s *grounded* ×0.5 branch the tick it
        lands rather than the airborne ×0.95f, exactly as the stone lane does.
      - **The body feels nothing back.** A cart pressed against an AI-enabled
        blaze for twenty ticks (`cart_body3`) leaves it at (2.5, 1.0, 1.5) to
        the last digit, and a cart sitting on its head does not press it down.
      - **The solidity is one-way.** A blaze dropped from y = 3 onto a NaN cart,
        an ordinary cart and a furnace cart lands on the *floor* at 1.0 on tick
        19 in all three lanes — the same tick as the empty control. A mob's own
        movement collides with no cart. Nothing in this engine moves a mob, so
        there is nothing to implement, but a future mob-physics pass must not
        reuse the table above.

      A **passenger's** box counts too: `cart_body2` drops a furnace cart onto a
      blaze seated on a NaN cart and it rests at `2.987499952316284`, the
      vehicle's y plus the 0.1875 seat plus 1.8f.

      Pinned by `tests/cart_body.rs` (six cases). Five of the six fail with the
      body list emptied, which is how they were checked.

      **Two things these captures measured and the engine does not model.**
      Neither can fire on the record door, and both are named here so that stays
      a fact rather than an assumption:

      - **A rideable cart mounts a mob rather than hitting it.** In `cart_body` a
        plain minecart rolling east picks a blaze up at t18 — from 0.2 away, the
        `inflate(0.2, 0, 0.2)` push search, not the collision box — and carries
        on with the ridden 0.997 slowdown instead of the empty 0.96. The door's
        rolling stock is furnace carts, which are not rideable, and both of its
        blazes are already passengers, which vanilla's gate excludes.
      - **A furnace cart is slower than a plain one, two ways.** Its velocity
        decays by **0.9408 = 0.96 × 0.98** a tick, not 0.96, and its stride is
        capped at **0.2** a block rather than 0.4 — both visible for nineteen
        ticks in `cart_body2`'s control lane, where the displacement is a flat
        0.2 while the velocity falls past it. `minecart.rs` applies 0.96 and 0.4
        to every variant. All fifteen of the door's furnace carts are motionless
        (`cart_furnace_yaw`, and `door55_in_world` records them holding position),
        so nothing depends on it today; a build that rolls one would be
        mis-timed.
- [x] A cart with **no block under its own column** is held up by a block under
      any column its box overlaps. This is what actually holds the record door's
      end cart: it stands in an `observer` over air, with 0.245 of its 0.98
      width above the dispenser in the column before it. `cart_ledge.json` is
      the capture — the overhanging cart never moves for forty ticks, and its
      control, the same cart clear of the ledge, falls at once and is removed.
      **The "carts resting on carts" hypothesis for that cart is refuted.**
      Nothing rests on anything; the engine's block sweep already had it right,
      and the cart only fell because it was being shoved off the ledge first.
- [x] `Rotation` on a **furnace** cart. `AbstractMinecart`'s push gate reads yaw
      as a polar angle and demands a dot of 0.8 against the line between the
      pair, so a cart facing ±Z is inert to a neighbour offset along ±X.
      `cart_furnace_yaw.json` is two identical x-separated furnace pairs that
      differ in nothing but this number, and only the yaw-0 one moves.
      Every one of the door's fifteen furnace carts carries `Rotation: [±90, 0]`
      and its top row is strung out along x — so vanilla never touches that row,
      and `door55_in_world.entities.log` shows it perfectly motionless.
      This was a **silent drop in three places at once**: the bridge's SNBT
      writer emitted no `Rotation` tag, `SpawnedFurnaceMinecart` had no `yaw`
      field to put it in, and `spawn_authored_furnace_minecart` hard-coded 0.
      Loaded facing +X the row shoved itself apart on tick 2 and walked its end
      cart out of the world by tick 200. The same writer also dropped `Fuel` and
      `PushX`/`PushZ`, which made the engine's refusal to run a self-propelled
      cart unreachable from any loaded world; both are emitted now.
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

Two things that capture *can* still say, and does:

**The two blazes are passengers, not missing entities — and they are in now.**
The save holds 22 top-level entities and our extraction keeps all 22; the
capture's 24 counts the two `minecraft:blaze` riders nested in the `Passengers`
list of two of the four plain minecarts — both of them nan carts, riders sitting
0.1875 above their vehicle with ordinary finite gravity of their own. Extraction
was never cropping anything; the engine was dropping the tag.

It no longer does. Loaded under `InWorld`, `55_3x3.zip` now yields **24 bodies**,
with the two blazes at y = 2.2500 and 2.1875 — each its cart's y plus 0.1875,
which is the seat measured independently in `blaze_ride.entities.log` — and the
door is still at rest: zero block changes over 400 ticks, quiescent, no entity
moved, zero `piston_retract_contacts`. Pinned by
`the_record_doors_two_blazes_are_seated_passengers` in `src/bridge/mc_tick.rs`,
which asserts the top level is 22 *and* the loaded world is 24, so the second
number cannot be a recount of the first.

**Vanilla does not move that door's top row, and does not change a block.**
Over the four ticks of `door55_in_world.entities.log` the seven furnace carts at
y = 6 hold their positions to the last digit and the capture records *zero* tick
with a block change. That is a hard target for any run of ours, and the engine
now meets the first half of it: with `Rotation` plumbed through, no cart in the
row moves and none falls out of the world. It does **not** meet the second half.

## Settled: the door does not actuate itself — the diagnostic asked for a paste

This section used to read "the open problem: the door actuates itself on load",
and reported that the engine changed **68 blocks** over the first ten ticks —
a bank of observers flipping `powered=false → true` at tick 1 and the cascade
after it — where vanilla's `--in-world` capture of the same save changes
nothing. It concluded that `TickSettleMode` "is meant to absorb exactly that and
does not absorb this."

**That conclusion was wrong, and it was wrong in the way this project keeps
getting caught by: the instrument was lying.** `examples/door55_sim.rs`
constructed its simulation with `TickSettleMode::Quiet`, not `InWorld`. The 68
changes were real engine behaviour, faithfully reported — they were simply the
behaviour of a mode that *places* the build, measured while we were asking a
question about *loading* it.

Run through the same path with the mode the comparison actually calls for, the
door is exactly where vanilla leaves it:

| mode | block changes, nothing triggered | entities moved | `piston_retract_contacts` |
|---|---|---|---|
| `Quiet` | 68 by tick 10 | 1 (a furnace cart, tick 6) | 11 |
| `InWorld` | **0 over 400 ticks** | **none** | **0** |

Zero changes, zero entity motion, quiescent, and it stays that way for as long
as it is run. That matches `door55_in_world.entities.log` on both halves of the
target this document sets — the top row holds its positions *and* not a block
changes — and it removes the second of the two reasons given here not to believe
a door: the ten `piston_retract_contacts` were ten entities standing in a
retraction that only happened because the build had been pasted.

### What happens when somebody does touch it

`examples/door55_sim.rs` takes `--press T` now (and `--button x,y,z`; the button
is otherwise searched for, because the extraction's origin is not the save's).
Pressed on tick 5 under `InWorld`, the door **starts and then stalls**, and it
does so identically with and without the body-collision work above:

```text
  block changes: 23, over ticks 5-10 and 34-40
  quiescent: true          entities in a retracting piston's sweep: 6
  no entity moved
```

Everything that happens is the first stage and only the first stage. The oak
button powers the note block beside it, the observer at `(72, 2, 20)` pulses,
and the down-facing sticky pair at `(71, 2, 20)`/`(71, 1, 20)` runs one
extend-and-retract of the double piston extender at `(71, 0, 20)`. Thirty game
ticks later the button releases, the observer pulses again on the falling edge,
and the same pair runs the same cycle backwards. Nothing downstream of it moves,
no entity moves, and the world is quiescent again by tick 41.

The six `piston_retract_contacts` are the reason to expect exactly this: they
are entities standing in a *down*-facing piston's own square as its head
retracts into it, which is the one piston case this page records as **measured
but not determined** (see the `piston_pull_inside` entry above). The door's
pistons face down, and the stage that would follow this one is behind that gap.
So a door that starts, does its first extender, and stops is the predicted
outcome of the retraction blocker, not of anything to do with entity hitboxes.

### Why placement arms every observer

`Simulation::place_on_place` — which `Quiet` and `Placement` both run, and
`InWorld` does not — blanks the region to air and re-writes every block one at a
time, handing each landing block's already-placed neighbours a shape update.
This is a faithful model of `StructureTemplate.placeInWorld`. It is also, for a
build cut out of a running world, a complete fiction: **every** observer in the
build watches the block it faces *appear*, and an observer that sees its facing
neighbour change pulses. Nine of them do here, at tick 1, and the pistons follow.

The first divergent cell is `(72, 1, 20)`,
`observer[facing=east,powered=false] → powered=true`, at tick 1. It is not a
boundary artefact — the cut face has nothing to do with it, and neither does any
re-derived `locked` flag, rail shape or wire connection. The perturbation is
global because the re-write is global.

`InWorld` writes the blocks and stops. That is all it was ever supposed to do,
and it does it.

### What this cost, and where it may still be costing

Pinned by `the_record_door_is_at_rest_under_in_world_and_disturbed_under_quiet`
in `src/bridge/mc_tick.rs`, which asserts both halves — the door at rest under
`InWorld`, *and* that `Quiet` still disturbs it, so the first assertion cannot
pass by accident.

The ordinary corpus was checked too, and the same measurement is now built into
`examples/door_batch_load.rs` (`SETTLE=in-world|quiet|placement`), which reports
per door whether it holds still when nobody touches it. The three certified
reference doors are **at rest under `InWorld`** and badly disturbed under
anything else:

| door | `InWorld` | `Quiet` | `Placement` |
|---|---|---|---|
| `4x4 sliding door` | at rest | 73 changes | 78 changes |
| `6x6 sliding door` | at rest | 836 changes | 896 changes |
| `fast 4x4 vault door (barrels filled)` | at rest | 50 changes | 121 changes |

So a door measured under a placement mode is measured against a machine that has
already moved several hundred blocks before the clock starts. The browser
certificate path (`apps/door-cert-wasm`) already uses `InWorld` and is clean.
**`apps/door-cert/backend/main.py` uses `Placement`, and `apps/flying-ga` uses
`Quiet`** — the latter is arguably right, since a GA genome genuinely is pasted,
but the former is not, and every verdict it has produced is downstream of the
disturbance in the table above. That is an `apps/` change and is left to whoever
owns it.
