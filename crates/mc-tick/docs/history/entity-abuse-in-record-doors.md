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

> **Where the instruments went.** This document is a narrative and names the
> diagnostics that were current while it was being written —
> `examples/door55_sim.rs`, `door55_doorway.rs`, `door55_xray.rs`,
> `door55_render.rs`, `world_entity_audit.rs`, `door_batch_load.rs`. All six are
> gone. Everything they measured that mattered now lives in
> `tests/scenarios/55_3x3.litematic`, which is this door carrying its own test
> (24 entities, blaze seats at y 2.1875 and 2.25, 0 block changes untouched, 6 of
> 9 doorway cells when pressed, 227 changes, quiescent, nothing below y=0), and
> the one general-purpose replacement is `examples/scenario_inspect.rs`, which
> takes a path and prints those same quantities. Read the references below as
> history, not as commands.

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
- [x] Pistons move entities — including entities whose own physics are dead.
      **Superseded: the fitted laws below are history.** The engine now runs
      vanilla's actual `PistonMovingBlockEntity.moveCollidedEntities`, read out
      of the 26.2 server jar (unobfuscated) and reproduced shape for shape:
      the real piston-head boxes (`piston::head_shape_boxes`, `SHORT` once
      `progress > 0.25`), `getMovementArea`'s swept slab per box
      (`piston::moved_shape_displacement`), the drag toward a retracting base
      clipped by the base's own 12/16 slab (`piston::retracting_base_box`),
      `fixEntityWithinPistonBase` pushing back out (`base_fix_displacement` —
      both finally wired in), `Entity.limitPistonMovement`'s ±0.51-per-axis
      running total, per-move `applyEffectsFromBlocks` (`entityInside` fires
      the tick the drag lands, which is how the door's plate presses at t13
      with `power=1`), and in-flight cells turning solid one by one in
      block-entity order rather than all at once. Every capture lane the fitted
      laws were tuned to still passes, and the two wide-body lanes that never
      fitted — `piston_plate_clip`'s `retracting_a_body_wider_than_the_arm` —
      now agree bit-exactly: the +0.25 out is the drag clipped on the base
      slab, the −0.25 back is `fixEntityWithinPistonBase`, and head-eject's
      mysterious `+0.02` is the two calls' `+0.01` overshoots stacked. The
      fitted functions (`head_eject_displacement`,
      `inside_eject_displacement`, `sweep_displacement`) remain as measured
      documentation with their unit tests, but nothing in the simulation calls
      the first two any more; their 1e-4 seam at `x = 2.65635` was an artifact
      of the fit, not of the game. The history below is how the shape of the
      real algorithm was cornered one capture at a time.
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

### Settled: the dispenser places its powder snow, and the machine runs

The section below records the state before bucket dispensing existed: 23–25
block changes, the first extender and nothing else. **That stall was the
dispenser.** The build's only block entity, the dispenser at `(73,3,20)`, holds
one `minecraft:powder_snow_bucket`, and the engine had no bucket behaviour — so
the bucket was ejected as an item entity and fell to y ≈ −30, leaving the cell
two observers are watching permanently empty.

Both directions are now measured (`bucket_dispense.json`, `bucket_pickup.json`,
five and six lanes, negative-controlled) and implemented. Pressed on tick 5
under `InWorld`, the door now runs:

| | before | after |
|---|---|---|
| block changes | 25 | **186**, over ticks 5–39 |
| components receiving updates | 12 of 32 | **32 of 32** |
| powder snow at `(74,3,20)` | never | **tick 7** |
| observers `(74,2,20)` / `(74,4,20)` | never | **both fire, tick 9**, clear at 11 |
| `piston_retract_contacts` | 0 | 0 |
| quiescent | true | true, from tick 40 |

The chain is: button (t4) → note block → dispenser `triggered` (t4) → the
down-facing sticky pair at `(71,2,20)`/`(71,1,20)` extends (t5) → **powder snow
lands at `(74,3,20)` (t7)** → both watching observers pulse (t9) → the bottom
double extender at `(73,0,20)`/`(74,0,20)` runs (t9–11) → the west end of the
build finally moves (t11 onward), and at **t13 the light weighted pressure plate
at `(64,3,20)` reads `power=2`** — the two dragon fireballs, which have arrived
at x = 65.5, being counted as two entities on a `maxWeight` 15 plate. That plate
is the first trigger in this build the engine has ever made fire.

Three things it does **not** do, all named rather than smoothed over:

- ~~**Fireball id=11 never reaches its plate.**~~ **Settled: vanilla throws it
  EAST, the plate does fire, and two things about our retraction law were
  wrong.** See "Settled: the fireball does clip its plate" below. The plate at
  `(74,1,20)` now reads `power=1` from tick 12, all three of the pistons that
  never moved now move, block changes go 186 → 220, and the passage fills
  **6 of 9** instead of 3.
- **Two quartz blocks end at y = −1.** The down-facing sticky pistons at
  `(70,3,20)` (t15) and `(71,2,20)` (t35) push their quartz columns one cell
  down and land a block at `(70,-1,20)` and `(71,-1,20)` — one cell below the
  build's own floor, in cells the loaded region holds no block for. Either the
  extraction cropped the machine's floor or the machine stands on air, and which
  it is decides whether that push is legitimate. **Everything after t15 is
  running on a world that may be one cell too shallow**, and this is the next
  thing to settle.
- **There is no passage in this sample to open.** The extracted region declares
  `size: [103, 7, 66]` and contains **53 blocks**: the mechanism, an 11×5 slice
  one block thick at `z=20` (plus the button at `z=19`), and a detached 3×3
  quartz pad with a sea lantern at `(67..69, 0, 0..2)`, nineteen cells away in
  z. No 3×3 panel, no doorway. Net of the whole cycle only **6 cells** differ
  from the start state — two emptied (`(70,1,20)`, `(73,0,20)`), the two quartz
  blocks at y = −1, a piston that ended one cell along at `(72,0,20)`, and the
  powder snow. So "does it open" cannot be answered from `55_3x3.zip` as it
  stands; what can be said is that the mechanism cycles end to end and returns
  almost exactly home.

`examples/door55_render.rs` output of this run:
`/Users/harrison/Desktop/nucleation-handoff/door55/door55_bucket_dispense.mp4`
(60 ticks, press on 5).

### Settled: the fireball does clip its plate, and vanilla throws it east

The builders' account of this gadget — *"when this piston that has the pressure
plate on it pulls back, the fireball will barely clip the pressure plate"* — is
correct, and the engine was wrong twice. Four new captures settle it, and the
oracle was sanity-checked first by re-capturing `piston_square_yband` and
diffing: 171 identical entity lines, only the known warmup count differing.

`piston_plate_clip` is the door's own geometry rebuilt as sixteen lanes with a
**light weighted pressure plate** to read out — a block-state channel, so no
entity filter can distort it. In the replica lane the extension throws the
fireball west 0.6875 (0.51 then 0.5, clipped by the moving block behind it,
which the engine did **not** do either — see "Settled: the sweep is clipped by
the block in flight" below) and then **the retraction
brings it all the way back east**, +0.51 then +0.1775, settling with its east
face on 5.0 — exactly where it started — and **the plate powers**. Negative
controls in the same rig: extend-only never powers it, no-fireball never powers
it, and a lane with nothing for the piston to push (so the fireball ends a block
west, out of reach) never powers it.

`plate_reach_flush` then rules out the obvious explanation. Twelve static lanes,
no pistons, nothing moving: a fireball whose east face is **5.0625 exactly does
not** press the plate and one at 5.0626 does. `TOUCH_AABB` is the cell inset a
pixel and `AABB.intersects` is strict, exactly as modelled — and a fireball
parked with its east face flush on 5.0, which is where the replica lane
*settles*, never presses it in 24 ticks, on a piston or on stone. **So the
trigger is inside the tick.**

Two law corrections came out of that, both implemented:

- **A piston shove has an intermediate position, and `entityInside` fires at
  it.** `piston_head_transient`, sixteen lanes sweeping the start x with and
  without a block to pull: every lane settles with its east face on 4.98, and
  the plate fires on the tick the *step* would have carried the box past
  5.0625 — x = 4.45 on the first step, x = 4.35 not until the second. The
  threshold is one `PISTON_MAX_STEP` from the box's leading face, so a
  retraction drags the entity a whole step inward and *then* corrects it back to
  the line. That is `entity.move(MoverType.PISTON, …)` followed by the push out
  of the piston base, and it explains where head-eject's unexplained `+0.02`
  comes from. `Simulation::piston_probes` is the intermediate box and
  `notify_piston_probes` fires `entityInside` at it, with the body view pointed
  at the probe so a plate counts the entity once (the capture reads `power=1`,
  never 2). For a pulled-block sweep the probe is the *requested* displacement,
  before collision clips it — which is the door's own case, where the fireball is
  aimed 0.3425 east and stops at 5.0 against the piston base.
- **The gate across the piston axis is the entity's FEET, not its centre.**
  `piston_head_yband`, eleven lanes with x fixed at the door's own 4.84375 and
  only y varying: a fireball at `[1.99, 2.3025]` — two thirds of its box in the
  block *above* the vacated one, centre y 2.14 — **is** ejected, and one at
  `[0.95, 1.2625]` — overlapping the vacated block by 0.2625 from below, centre
  y 1.11 — is **not**. Neither a centre gate nor a box-overlap gate produces
  that pair; `BlockPos.containing(position())` does, because an entity's
  `position()` is (centre x, **min** y, centre z). Along the piston the gate
  stays the centre — `piston_pull_plate`'s `headonly` lane and
  `piston_pull_inside`'s `inside 2.9` lane both demand it — so the probe is now
  per axis. This is precisely why id=11 was refused: feet 0.875 are in the
  piston's row, its centre 1.03125 is not.

**The door still does not close.** *(Superseded — it closes now; see "Settled:
the door closes, tick-exact against a full-fidelity replica" at the end of this
page.)* With both fixes the machine fills 6 of the 9
passage cells and then falls back, and it ends well away from home with the plate
at `(74,1,20)` reading `power=1`. The three cells that never fill are `(67,2)`,
`(68,0)` and `(69,2)`.

### Settled: the sweep is clipped by the block in flight

The trailing `power=1` above was the last symptom, and its cause was that
**nothing clipped a piston's shove**. `Simulation::shove` has always run
`entity.move`'s block collision, but the cells a piston is moving a block into
hold a `moving_piston` placeholder, and `moving_piston` is not a full cube — so
the whole stroke read as empty air and the fireball was carried the full distance
the sweep asked for. It ended `0.3225` **east** of its start, inside the plate's
touch box, where the plate latched and held the west closing pair out.

`piston_plate_clip` had the answer in it all along; it had only ever been read
for the plate. Read for the *positions*, all fifteen lanes say the same thing,
and they say it about extension as well as retraction:

- **extension.** Lane `z=1` step two asks for `0.5` west and vanilla gives
  `0.1775` — the room from `4.1775` to `x = 4.0`, the east face of the cell the
  pushed sticky piston is arriving in. The engine gave the full `0.5` and ended
  at `3.83375`, a whole block west, which is why the retraction could not reach
  it: the replica lane was silently behaving like the `replica_nopush` control.
  So extension was **not** bit-exact here, and the four captures it was verified
  against never posed the question — `piston_entity` shoves everything into open
  air.
- **it is a distance to a surface.** Lane `z=21` starts the same fireball
  `0.25625` further east and its second step is `0.43375`, not `0.1775`. Both
  land the west face on exactly `4.0`.
- **retraction.** Lane `z=5` retracts twice from two different places — `0.02`
  out of the head's own eject and `0.1775` out of a full stroke — and both land
  the **east** face on `x = 5.0`, the west face of the piston's own square. That
  square holds a `moving_piston` for the two ticks the head takes to come home,
  and its pending write is the retracted base, which *is* a full cube.
- **the negative controls are in the same rig.** `z=17` has nothing to push, so
  its second step is the full `0.5` and it is right to be unclipped; `z=19` has
  nothing to *pull*, so its retraction is the head's eject alone and stops at
  `4.82375`.

A fifth capture, `piston_clip_sizes`, settles **when** the block in flight starts
colliding, which fifteen lanes of 0.3125 fireball cannot: every one of them is
still a whole block clear of the arriving block when its first step is taken, so
"solid at its destination all along" fits as well as "solid only on the second
step". Widen the body until its leading face already sits on the line and the two
rules give opposite answers. A **dragon fireball** in the replica lane, box
`[4.0, 5.0]`, starts flush against the cell the pushed sticky piston is arriving
in — and vanilla moves it the full `0.51` anyway, then clips its *second* step to
`0.49` against the cell the pushed **quartz** is arriving in, `x = 3.0`. A 0.98
furnace minecart, `0.02` clear of the same line, gets `0.51` and then a full
`0.5`. So a moving block is transparent on the first step and solid at its
destination on the second, which is `PistonMovingBlockEntity`'s `progress < 1.0`
read *after* the step's increment — half a block, then a whole one.

`Simulation::blocks_in_flight` supplies those cells to `shove` as unit cubes at
their destinations, for moves on their second step (`resolve_on == tick + 1`)
whose **landed** state is a full cube — which is why an extension's own head slot
is not among them: it lands `piston_head`, and a cube there would have stopped
lane `z=1` at `4.25` instead of `4.15625`. `tests/piston_plate_clip.rs` pins all
fifteen small-fireball lanes, every position and all thirty-seven plate
transitions, plus the three wide-body lanes. Two negative controls, both observed
failing: with the clip disabled five of its six tests fail and the "nothing in the
way" control still passes; with the clip present on *both* steps the narrow lanes
still pass and the dragon fireball is pinned at `4.5` and never moves at all.

The wide-body capture also found a hole that is **not** this clip, recorded as a
disagreement in `retracting_a_body_wider_than_the_arm_still_disagrees`: retracting
a body wider than the piston arm. Lanes `z=5` (dragon fireball) and `z=9` (furnace
minecart) are shoved `+0.25` east by vanilla and then `-0.25` back, a round trip;
the engine shoves `+0.49`, and the cart never returns. A 1.0-wide box straddles
the arm's 4/16 column instead of lying inside it, so
`inside_eject_displacement`'s cross-axis gate declines and the pulled block's own
sweep answers instead. Every lane of `piston_plate_clip` proper takes the other
branch, which is why fifteen lanes never saw it.

On the door: fireball id=11 now ends at `73.84375`, its exact start, flush on
`x = 74.0`; the plate returns to `power=0` at t33; `(73,0,20)` returns to
`extended=false`; block changes go 219 → 227; the run reaches **quiescence**
where it did not before; and the world ends 10 cells from home instead of 12.

**The passage count is measuring nothing.** A census of the whole declared
region — `CENSUS=1 examples/door55_doorway` — finds **53 blocks, at z ∈ {0,1,2},
19 and 20 only**: the 43-block mechanism slab at `z=20`, the button at `z=19`,
and the 3×3 quartz pad with its sea lantern lying **flat at y=0, z=0..2**,
eighteen cells away. There is no panel at `(67..69, 0..2, z=20)`; those are
interior voids of the mechanism's own slab, and "9 of 9" is unreachable in this
save no matter what the physics does. The four cells that stay filled —
`(67,1)`, `(68,1)`, `(69,0)`, `(69,1)` — are quartz the mechanism pushed into
that void and had nothing to push back out.

**The next thing to measure is three furnace carts.** ids 14, 21 and 23, the
east column's scaffolding, end at y ≈ −72 with `vel = -0.757` and are still
falling at t120. They fall in the same tick with the clip and without it, so it
is not this fix, and they do not move at all until the button is pressed. Each
was last over a column whose `y=0` cell holds `sticky_piston[extended=false]` —
a full cube by `is_full_cube` — where carts 13 and 22, over quartz, both land at
`y = 1.0`. Whether the cycle legitimately empties everything beneath them at the
moment they let go, or the support test misses a retracted piston, cannot be read
off this save: `door55_in_world.entities.log`'s entity lines are not
reproducible. It needs a rig of its own — a furnace cart at `y = 1.7` embedded in
a down-facing sticky piston's head slot, the piston cycled under it, and the
cart's settled `y` read off the entity log.

### Retracted: the plate's recheck was never being lost

An earlier revision of this section called that trailing `power=1` "a concrete
defect rather than a mystery" — the plate powers at t12, releases at t22,
re-powers at t23 "and then never reschedules", so the `WEIGHTED_PLATE_RECHECK`
scheduled from the block-entities phase "is being lost". **That was wrong, and it
was wrong because the run it was read off ended between two rechecks.** Traced at
the plate, with a press on tick 5, the cadence is intact: power at t12, release at
t22, re-power at t23, and then a recheck at t33, t43, t53, t63 … each one finding
the fireball still in the touch box and re-booking the next. Nothing is lost;
`TickQueue::schedule`'s dedup never fires for that position, and the schedule
made from the block-entities phase lands exactly where it should. The plate holds
because **the fireball is genuinely parked inside its touch box** — id=11 settles
at x = 74.16625, east face 74.3225, and `piston_plate_clip` says vanilla returns
it to its start with its east face flush on 74.0, where `plate_reach_flush` proves
it does not press. The plate is reporting its input correctly; the input is wrong.

That is the fourteenth time in this effort an instrument reported an absence that
was really a blind spot, and it cost a full investigation. The lesson is the same
one as before: a cadence claim needs a run long enough to contain two of them.

What the investigation *did* turn up is a real deviation, in the same block and
found by reading `BasePressurePlateBlock.checkPressed` rather than by guessing:
its write is `Level.setBlock(pos, state, 2)` followed by `updateNeighbours`,
which is `updateNeighborsAt(pos)` **and `updateNeighborsAt(pos.below())`**. Both
plates in this engine did a plain flag-3 `set`, so the second call was missing
entirely — and because `getDirectSignal` answers for `Direction.UP` alone, a
plate strongly powers the block it stands on, and anything touching only that
block learns of the change by no other route. `plate_recheck.json` (section 13 of
`capture-entity-evidence.sh`, eleven lanes) measures both halves: the ten-tick
interval and its release, anchored on the press across five staggered
departures — the first capture ever to see a weighted plate release, which needs
a gravityless entity because an item pressing a plate is an item resting on it —
and dust beside a plate's support reading 1 under a light weighted plate and 15
under an oak plate, with a plain-stone control lane that stays dark. The engine
left all three at zero.

Fixing it changes the door's numbers and does not close it: 220 block changes →
219, cells differing from the start state 9 → 12, the doorway peak still 6 of 9
(t27 rather than t26) falling back to 4 rather than 3, and the same three cells
never fill. **The nearest gap is now the fireball's resting place, not the
plate.** Our engine grants id=11 a net +0.3225 east where the replica lane says
vanilla nets zero, so the collision clip that should stop it against the piston
base at 74.0 is not being applied to the pulled-block sweep. That is what to
measure next.

Two things this work leaves unverified. The intermediate-box law is measured for
`head_eject_displacement` and for the pulled-block sweep; it is *applied* to
`inside_eject_displacement` on the assumption that the same
`entity.move(MoverType.PISTON, …)` produces it, and no capture reads a plate on
that geometry. And nothing here was captured against the door itself, which
remains impossible: the only oracle is 26.2 and the door is DataVersion 4082.

### What happened before that — the first extender and nothing else

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

## Settled: the door closes, tick-exact against a full-fidelity replica

One button press now closes all nine doorway cells — `t38 fill 9/9`, 266 block
changes, quiescent, all 24 entities intact — and the engine's block timeline is
**tick-for-tick identical to real Minecraft** across the whole cycle. The only
diffs in a per-tick set comparison are write-coalescing artifacts: six cells
where vanilla's capture logs `A -> B` and the engine logs `A -> air -> B`
within the same tick, identical states before and after.

### The oracle that finally existed

"There cannot be a capture of this door working without a 1.21.3 oracle" turned
out to be wrong by one idea: the 26.2 `Entity.load` NaN-drop only bites at
**load**. `tools/gametest/capture.sh`'s world mode ticks the save's blocks in
place (no placement disturbance), and TraceCapture's `--spawn` sets motion
directly, bypassing `setDeltaMovement` — so deleting `entities/*.mca` from the
save and respawning all 24 entities with their exact positions, `Motion` (NaN
components and the 4.28e-59 denormal included), `Rotation` (a new `yaw=` spawn
flag; a furnace cart's yaw is its push gate) and the two blaze riders (`ride`)
reproduces the door entirely. `captures/door55_replica.entities.log` is that
run: nan carts frozen, blaze wall standing, pressed at t5, **closed 9/9 and
quiescent by t42**. `captures/door55_pressed.entities.log` is the same press
over the loaded (NaN-stripped) entities — the run that first showed the block
choreography, at the cost of the freed carts drifting.

### What the engine was missing, in causal order

Every one of these was found by diffing the engine against the two captures and
confirmed in the 26.2 source (the jar is unobfuscated; 1.21.3's Mojang
mappings were pulled to check nothing here is version-gated — it is not):

1. **The piston-entity laws were fits; the algorithm is now vanilla's.**
   `moveCollidedEntities` reproduced shape for shape — see the superseded note
   on the pistons checklist entry above. This is what returns every fireball
   and cart to its exact rest position and makes the machine re-triggerable.
2. **A cleanly-shoved entity fires `entityInside` the tick it moves.**
   `moveEntityByPiston` ends in `applyEffectsFromBlocks`, so the plate at
   (0,3) presses at t13 — the drag's own tick — reading `power=1`, because the
   second stacked fireball has not moved yet when the first is counted (an
   already-pressed plate's `entityInside` is a no-op; the 10-tick recheck
   reads 2 at t23). The engine notified shoved entities a tick late, and that
   one tick let its (0,4) row-return event run after the row had landed and
   succeed — vanilla's runs at t14, mid-flight, fails, and **is dropped**,
   which is the window the whole close fits through.
3. **An observer's pulse is a flag-2 write.** `ObserverBlock.tick` dispatches
   no generic neighbour updates; only `updateNeighborsInFront`'s pair hears
   the pulse. The engine's flag-3 write poked the row-shifter piston behind
   the row's west observer, re-shifting the row a phase early. With flag 2 the
   land-pulses of the two moved observers fire the down-pistons at t16 over
   columns 3 and 4 — the row is still west — and the top corners close.
4. **`finalTick` lands through `setBlock`, so `onPlace` runs.** Both engine
   finalTick sites (the interrupted extension and the short-pulse drop) now
   run the landed block's `on_placed`. The repositioned west piston lands at
   (7,0) during t38's events phase, re-checks its power inside that phase —
   QC through the observer's emission into the air above — queues, chains,
   and fires the same tick, closing the bottom-middle cell. That is the last
   stroke of the close.

### What one press does, measured

Button t5 → dispenser plants powder snow t8 → east bottom battery stages t10-12
→ row shifts west t12-14, dragging the 1e-7-straddling dragon fireballs onto
the west plate (t13, power=1) and pinning the small fireball's plate (t13) →
west DPEs push the middle and bottom west cells t14-18 → the moved observers'
land-pulses fire the down-pistons over columns 3/4 at t16, short-pulse-dropping
quartz into the top cells t18 → east side fills (5,1)/(5,0) t17-19 → the
plate recheck reads 2 at t23, the row returns east t23-25, carrying the
fireballs home → the down-pistons fire again over columns 4/5 t27-31 (the
middle-top quartz cycles down one) → button releases t34 → the release chain
re-stages the west piston down to (7,0) t36-38, which fires on landing and
pushes the last two quartz west, **(4,0) closes t38-40** → quiescent. The
detector rail never powers during the close: the top-row cart the row drags
west stops flush against the seated blaze at x = 2.6 (a cart's collision set
includes a pushable mob, and carts have no step-up), exactly as vanilla does.
The rail is the *opening* trigger — the builders' "powers during the opening
sequence and not the closing one" — and the reopen, a second press, is the
next thing to build a scenario for.

## Settled: the reopen too — the full cycle is a bit-identical round trip

The second press works. Pressed again at t60, the row shifts west, the critical
cart — parked flush against the seated blaze since the close — is dragged the
last 0.28 west, falls onto the **detector rail** (t67), and the rail powers the
deep pistons that pull the whole panel back out. The rail releases at t87 as
the cart rides the returning row home, and by t96 the world is **bit-identical
to its initial state**: every block, and every one of the 24 entities, back
where the save put them. `tests/scenarios/55_3x3.litematic`'s third scenario
pins the round trip with `expect: initial`, and the whole 220-tick two-press
timeline has **zero divergent ticks** against the replica capture
(`captures/door55_cycle.entities.log`).

Two more vanilla laws fell out of the reopen, both measured on
`captures/piston_drop_lift.entities.log` (a two-lane rig replicating the
door's (3,4)-drop geometry) and then confirmed on the door:

- **The arm's drag is gated on the entity's `position()` point, not its box.**
  A furnace cart whose box genuinely overlaps a retracting arm's 4/16 column
  by 0.205 is never touched; the same cart centred on the column is. What
  separates every measured lane — that rig, `piston_square_yband`'s 6/16 and
  10/16 min-y flips, `piston_head_yband`'s feet gate, and the door's critical
  cart — is (centre x, **min** y, centre z) lying inside the arm's
  cross-section. The plate (and any carried block) still sweeps by plain
  strict `AABB.intersects`. This is `piston::SweptBox`'s `arm` flag, and it is
  the one place the "real algorithm" rewrite had to re-admit a fitted gate:
  the oracle refused the pure box-intersect reading of the source, and the
  oracle wins.
- **A sticky piston only drops a block that is still travelling away from
  it.** Vanilla's short-pulse gate is `entity.getDirection() == direction &&
  entity.isExtending()`; finalising *any* in-flight move at the target — a
  move retracting through that cell included — landed the reopen's (2,1)
  piston one tick early, the single seam in an otherwise exact cycle.
