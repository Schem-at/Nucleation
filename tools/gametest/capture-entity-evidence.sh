#!/usr/bin/env bash
# Regenerate every entity-behaviour capture, exactly as it was first taken.
#
# These answer three questions the record-door document
# (crates/mc-tick/docs/entity-abuse-in-record-doors.md) asserts but never
# measured: whether a NaN velocity is contagious, what a piston does to an
# entity, and what a villager's hitbox actually is.
#
# The trace JSONs land in crates/mc-tick/tests/traces/. The per-tick entity
# state lands in tools/gametest/captures/*.entities.log, because JSON has no
# spelling for NaN and the trace writer refuses to invent one — see
# TraceCapture --entity-log.
#
#     tools/gametest/run.sh                     # once, to fetch the server jar
#     tools/gametest/capture-entity-evidence.sh
set -euo pipefail
cd "$(dirname "$0")/../.."

CAP=tools/gametest/captures
TRACES=crates/mc-tick/tests/traces
mkdir -p "$CAP"

# ---------------------------------------------------------------------------
# 1. NaN contagion.
#
# Five carts on one flat rail:
#   id1  normal, rolling east at 0.3 — the striker
#   id2  NaN in Motion.z, at rest on the rail — the target
#   id3  NaN in Motion.x, at rest on the rail
#   id4  NaN in Motion.z, floating two blocks above the rail — the "does it
#        fall?" probe
#   id5  finite zero, floating likewise — the control that proves falling is
#        observable at all
#
# Result: id5 falls in two ticks, id4 never moves; id1 collides with id2 at
# t36 and stays finite for the remaining 24 ticks; non-finite count is 3 at
# every one of the 61 samples. NaN is inert and NOT contagious in 26.2.
#
# NB the velocities here cannot come from the structure file: SNBT has no
# production for NaN. --spawn writes them, and in 26.2 it has to go behind
# Entity.setDeltaMovement to do it — see the note that capture prints.
tools/gametest/capture.sh --structure nucleation:nan_contagion --max-ticks 60 --entity-log \
  --spawn 'minecraft:minecart@5.5,1.0625,1.5:0.3,0,0' \
  --spawn 'minecraft:minecart@12.5,1.0625,1.5:0,0,NaN' \
  --spawn 'minecraft:minecart@18.5,1.0625,1.5:NaN,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,1.5:0,0,NaN' \
  --spawn 'minecraft:minecart@21.5,3.0,1.5:0,0,0' \
  --out work/nan_contagion.json | tee "$CAP/nan_contagion.entities.log"

# ---------------------------------------------------------------------------
# 2. The record door itself, ticked in place from the real save.
#
# 55_3x3.zip is a pre-26.2 save; capture.sh relocates region/, entities/ and
# poi/ into dimensions/minecraft/overworld/ first, because the server reads
# only the new layout and silently generates fresh terrain over the old one.
#
# Result: 22 entities load, six of which carry a NaN in Motion.z on disk, and
# the non-finite count is ZERO. 26.2's Entity.load hands Motion to
# setDeltaMovement, which drops any non-finite vector whole, so a nan cart
# cannot survive being loaded. The door comes apart during warmup.
#
#   unzip 55_3x3.zip -d /tmp/w55
WORLD="${WORLD55:-/tmp/w55/55 3x3}" tools/gametest/capture.sh \
  --structure nucleation:nan_contagion --in-world -8,-2,12,12,10,28 \
  --max-ticks 4 --entity-log \
  --out work/door55_entities.json | tee "$CAP/door55_in_world.entities.log"

# ---------------------------------------------------------------------------
# 2b. The record door, PRESSED — twice.
#
# door55_pressed: the save's own entities, loaded (so 26.2 strips the NaN and
# the glue carts drift). The block choreography of the close is still readable
# for the early ticks and this run is what first showed it.
#
#   WORLD="${WORLD55:-/tmp/w55/55 3x3}" tools/gametest/capture.sh \
#     --structure nucleation:nan_contagion --in-world -8,-2,12,12,10,28 \
#     --use 13,4,6 --use-tick 5 --max-ticks 140 --entity-log \
#     --out work/door55_pressed.json | tee "$CAP/door55_pressed.entities.log"
#
# door55_replica: the capture that "could not exist". Entity.load's NaN-drop
# only bites at *load* — so delete entities/*.mca from a copy of the save and
# respawn all 24 entities exactly: positions, Motion (NaN components and the
# 4.28e-59 denormal), Rotation via the `yaw=` spawn flag (a furnace cart's yaw
# is its push gate), and the two blaze riders via `ride`. --spawn sets motion
# directly, bypassing setDeltaMovement, so the nan carts stay frozen. The spawn
# list is generated from the litematic (region-local entity coords + region
# offset (4,0,1) + world offset (4,2,-13) = +(8,2,-12)). Result: the door
# closes 9/9 from one press and is quiescent by t42, and the engine matches it
# tick for tick.
#
#   rm -f "$W55COPY/entities/"*.mca
#   WORLD="$W55COPY" tools/gametest/capture.sh \
#     --structure nucleation:nan_contagion --in-world -8,-2,12,12,10,28 \
#     --use 13,4,6 --use-tick 5 --max-ticks 140 --entity-log \
#     <24 x --spawn, see captures/door55_replica.entities.log's spawn echo> \
#     --out work/door55_replica.json | tee "$CAP/door55_replica.entities.log"

# ---------------------------------------------------------------------------
# 3. Pistons moving entities — the extension half.
#
# Four lanes, one sticky piston each facing east, one entity per lane standing
# in the block the head is about to occupy. Powered at t2, unpowered at t14.
#
# Result at t2-t3: every entity is displaced east, including the NaN cart,
# which keeps its NaN throughout. Displacement is positional only — no
# velocity is imparted to anything. Retraction moves nothing, because by then
# the entities are clear of the arm.
tools/gametest/capture.sh --structure nucleation:piston_entity --max-ticks 30 --entity-log \
  --spawn 'minecraft:minecart@3.5,1.0,1.5:0,0,0' \
  --spawn 'minecraft:minecart@3.5,1.0,3.5:0,0,NaN' \
  --spawn 'minecraft:small_fireball@3.5,1.0,5.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@3.5,1.0,7.5:0,0,0' \
  --at 2:1,1,1:redstone_block --at 2:1,1,3:redstone_block \
  --at 2:1,1,5:redstone_block --at 2:1,1,7:redstone_block \
  --at 14:1,1,1:air --at 14:1,1,3:air --at 14:1,1,5:air --at 14:1,1,7:air \
  --out work/piston_entity.json | tee "$CAP/piston_entity.entities.log"
cp work/piston_entity.json "$TRACES/piston_entity.json"

# ---------------------------------------------------------------------------
# 4. Pistons moving entities — the retraction half.
#
# Same lanes, but the pistons start extended and held by a redstone block, and
# the entities start inside the head's own block. Power is cut at t6.
#
# Result: nothing moves for six ticks, so a solid piston head does not eject an
# entity that is inside it. When the head becomes a retracting moving_piston
# the entities are displaced, but only fractionally and not uniformly westward.
# This does not reproduce a "pull" of a whole block.
tools/gametest/capture.sh --structure nucleation:piston_pull --max-ticks 20 --entity-log \
  --spawn 'minecraft:minecart@3.5,1.0,1.5:0,0,0' \
  --spawn 'minecraft:minecart@3.5,1.0,3.5:0,0,NaN' \
  --spawn 'minecraft:small_fireball@3.5,1.0,5.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@3.5,1.0,7.5:0,0,0' \
  --at 6:1,1,1:air --at 6:1,1,3:air --at 6:1,1,5:air --at 6:1,1,7:air \
  --out work/piston_pull.json | tee "$CAP/piston_pull.entities.log"
cp work/piston_pull.json "$TRACES/piston_pull.json"

# ---------------------------------------------------------------------------
# 5. Villager hitbox, by the fireball_reach method.
#
# A pressure plate's touch box is [1/16, 15/16] in x and z, so a plate at block
# x = P is touched by anything overlapping [P + 0.0625, P + 0.9375]. Each
# villager below sits a few thousandths to one side of a predicted edge; the
# plate's `power` in the trace is the answer, and it is a block-state read, so
# no entity-type filter can distort it.
#
#   plate  offset   who            edge tested                 expected
#   2      1.76     adult          west, centre - 0.3          clear
#   6      5.77     adult          west, centre - 0.3          touching
#   10     11.24    adult          east, centre + 0.3          clear
#   14     15.23    adult          east, centre + 0.3          touching
#   18     17.81    baby           west, centre - 0.245        clear
#   22     21.83    baby           west, centre - 0.245        touching
#   26     27.19    baby           east, centre + 0.245        clear
#   30     31.17    baby           east, centre + 0.245        touching
#   34/38  centred  adult/baby     control: both must touch
#
# The last two are the height rig: a cobblestone wall carries a plate one block
# up, beside a villager that overlaps it horizontally but cannot collide with
# the wall's narrow post. Only a villager taller than one block reaches it.
#
#   42     41.9     adult          height > 1.0                touching
#   46     46.0     baby           height < 1.0                clear
#
# Result: all twelve agree. Adult 0.6 x 1.95, baby 0.49 x 0.98.
tools/gametest/capture.sh --structure nucleation:villager_reach --max-ticks 8 --entity-log \
  --spawn 'minecraft:villager@1.76,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@5.77,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@11.24,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@15.23,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@17.81,1.0,1.5:0,0,0:noai,baby' \
  --spawn 'minecraft:villager@21.83,1.0,1.5:0,0,0:noai,baby' \
  --spawn 'minecraft:villager@27.19,1.0,1.5:0,0,0:noai,baby' \
  --spawn 'minecraft:villager@31.17,1.0,1.5:0,0,0:noai,baby' \
  --spawn 'minecraft:villager@34.5,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@38.5,1.0,1.5:0,0,0:noai,baby' \
  --spawn 'minecraft:villager@41.9,1.0,2.5:0,0,0:noai' \
  --spawn 'minecraft:villager@46.0,1.0,2.5:0,0,0:noai,baby' \
  --out work/villager_reach.json | tee "$CAP/villager_reach.entities.log"
cp work/villager_reach.json "$TRACES/villager_reach.json"

# ---------------------------------------------------------------------------
# 6. What actually holds the record 3x3 door's top row up.
#
# Two structures rather than one, because the conformance harness numbers
# entities by the order they *start moving* — see `normalize_entity_ids` — and
# lanes whose movers interleave would compare a different cart against a
# different cart.
#
# cart_furnace_yaw: two pairs of furnace carts, each 0.98 apart along +X on
# flat stone, differing in nothing but Rotation.
#
#   z=2   Rotation 90    facing +-Z against an +-X separation, dot 0
#   z=7   Rotation 0     facing +-X, dot 1
#
# Result: the z=7 pair shoves itself apart on tick 0 and the z=2 pair never
# moves at all, for forty ticks. `AbstractMinecart`'s push gate reads yaw as a
# polar angle, and a furnace cart is subject to it exactly like a plain one.
# Every one of the door's fifteen furnace carts carries Rotation [+-90, 0] and
# its top row runs along x, which is why vanilla leaves that row alone.
tools/gametest/capture.sh --structure nucleation:cart_furnace_yaw --max-ticks 40 \
  --entities --entity-log \
  --out work/cart_furnace_yaw.json | tee "$CAP/cart_furnace_yaw.entities.log"
cp work/cart_furnace_yaw.json "$TRACES/cart_furnace_yaw.json"

# cart_ledge: a ledge of stone that stops at x=3, and two carts.
#
#   z=2   x=4.245   air under its own column, 0.245 of its box over x=3
#   z=6   x=5.5     clear of the ledge entirely — the control
#
# Result: the overhanging cart never moves; the control falls at once and is
# removed on tick 18. A cart is held by a block under *any* column its box
# overlaps, so the door's end cart — an observer at its own x, air below, and a
# quarter of its width over the dispenser before it — needs nothing beneath it.
# This is what refuted "carts resting on carts" as the reason it fell.
tools/gametest/capture.sh --structure nucleation:cart_ledge --max-ticks 40 \
  --entities --entity-log \
  --out work/cart_ledge.json | tee "$CAP/cart_ledge.entities.log"
cp work/cart_ledge.json "$TRACES/cart_ledge.json"

# ---------------------------------------------------------------------------
# 7. Blaze hitbox, by the fireball_reach / villager_reach method.
#
# The record 3x3 door's two riders are blazes, and an unmeasured entity is
# refused by name — so the box has to come from the game. `blaze_reach` is
# `villager_reach`'s rig with the same ten floor plates, plus a second height
# rig two blocks up (a cobblestone-wall post carrying a plate at y=3) that a
# 1.8-tall body straddles and a 1.95-tall one clears.
#
#   plate  offset   edge tested                          expected
#   2      1.76     west, centre - 0.3                   clear
#   6      5.77     west, centre - 0.3                   touching
#   10     11.24    east, centre + 0.3                   clear
#   14     15.23    east, centre + 0.3                   touching
#   18     17.81    baby-villager west edge (0.245)      touching   <- discriminates
#   22     21.83                                         touching
#   26     27.19    baby-villager east edge (0.245)      touching   <- discriminates
#   30     31.17                                         touching
#   34     34.5     centred control                      touching
#   42     41.9     height > 1.0                         touching
#   44     43.9 @ feet 1.205, nogravity: top 3.005 > 3.0 touching
#   48     47.9 @ feet 1.195, nogravity: top 2.995 < 3.0 clear
#
# The two discriminating rows are why this is not just villager_reach again: a
# 0.49-wide body reads *clear* at 17.81 and 27.19. Width lands in (0.585, 0.605)
# and height in (1.795, 1.805) — 0.6 x 1.8, which is what the registry says.
tools/gametest/capture.sh --structure nucleation:blaze_reach --max-ticks 8 --entity-log \
  --spawn 'minecraft:blaze@1.76,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@5.77,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@11.24,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@15.23,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@17.81,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@21.83,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@27.19,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@31.17,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@34.5,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@41.9,1.0,2.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@43.9,1.205,2.5:0,0,0:noai,nogravity' \
  --spawn 'minecraft:blaze@47.9,1.195,2.5:0,0,0:noai,nogravity' \
  --out work/blaze_reach.json | tee "$CAP/blaze_reach.entities.log"
cp work/blaze_reach.json "$TRACES/blaze_reach.json"

# The control for the height rig: the same twelve offsets with villagers. Both
# fine plates fire for a 1.95-tall body, so a rig that reported "clear" for
# everything could not produce the blaze result above.
tools/gametest/capture.sh --structure nucleation:blaze_reach --max-ticks 8 --entity-log \
  --spawn 'minecraft:villager@1.76,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@5.77,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@11.24,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@15.23,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@17.81,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@21.83,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@27.19,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@31.17,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@34.5,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:villager@41.9,1.0,2.5:0,0,0:noai' \
  --spawn 'minecraft:villager@43.9,1.205,2.5:0,0,0:noai,nogravity' \
  --spawn 'minecraft:villager@47.9,1.195,2.5:0,0,0:noai,nogravity' \
  --out work/blaze_reach_villager_control.json \
  | tee "$CAP/blaze_reach_villager_control.entities.log"

# ---------------------------------------------------------------------------
# 8. What a mount does to a rider.
#
# `Passengers` cannot be authored in a structure file any more than a NaN
# velocity can, so `--spawn ...:ride` seats an entity on the one spawned before
# it. Nine lanes:
#
#   z=1.5   cart at rest on rail + blaze     -> the seat offset
#   z=4.5   NaN cart + blaze                 -> is the rider pinned?
#   z=7.5   cart rolling east + blaze        -> does the rider track x?
#   z=10.5  cart falling through air + blaze -> does the rider fall with it?
#   z=13.5  blaze alone in the air (control)
#   z=16.5  NaN cart + blaze + a cart dropped on the blaze's head
#   z=19.5  the same cart with nothing under it (control)
#   z=22.5  cart + villager                  -> a *different* seat
#   z=25.5  cart + small fireball
#
# Result: every rider sits at vehicle + seat on every tick of twenty, x and z
# included, and the seat is a property of the pair — blaze and small fireball
# 0.1875, villager 0.0. The NaN cart's rider never moves. The cart dropped on
# the blaze settles at 2.9875, the blaze's exact top, while its control falls to
# the floor.
tools/gametest/capture.sh --structure nucleation:blaze_ride --max-ticks 20 --entity-log \
  --spawn 'minecraft:minecart@2.5,1.0625,1.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,1.0625,1.5:0,0,0:noai,ride' \
  --spawn 'minecraft:minecart@2.5,1.0,4.5:0,0,NaN' \
  --spawn 'minecraft:blaze@2.5,1.0,4.5:0,0,0:noai,ride' \
  --spawn 'minecraft:minecart@0.5,1.0625,7.5:0.3,0,0' \
  --spawn 'minecraft:blaze@0.5,1.0625,7.5:0,0,0:noai,ride' \
  --spawn 'minecraft:minecart@2.5,4.0,10.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,4.0,10.5:0,0,0:noai,ride' \
  --spawn 'minecraft:blaze@2.5,4.0,13.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@2.5,1.0,16.5:0,0,NaN' \
  --spawn 'minecraft:blaze@2.5,1.0,16.5:0,0,0:noai,ride' \
  --spawn 'minecraft:minecart@2.5,3.0,16.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,19.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,1.0625,22.5:0,0,0' \
  --spawn 'minecraft:villager@2.5,1.0625,22.5:0,0,0:noai,ride' \
  --spawn 'minecraft:minecart@2.5,1.0625,25.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.0625,25.5:0,0,0:ride' \
  --out work/blaze_ride.json | tee "$CAP/blaze_ride.entities.log"

# The same questions with AI *on*, plus what a cart will and will not rest on.
#
# A riding blaze with AI reads vel=(0, -0.0784000015258789, 0) on every one of
# thirty ticks and never moves a millimetre: `Entity.rideTick` zeroes the delta,
# the tick applies one step of gravity, and `positionRider` overwrites the
# position anyway. The door's saved riders carry exactly that number, so it is
# not evidence that they fall.
#
# The four support lanes settle it for carts too: a cart dropped from y=3 rests
# on a blaze (2.799999952316284) and on a villager (2.950000047683716), each the
# body's exact top, and falls straight through a small fireball and a dragon
# fireball to the floor. `Entity.canBeCollidedWith` is true for a living entity
# and false for a projectile. The engine models neither yet.
tools/gametest/capture.sh --structure nucleation:blaze_ride --max-ticks 30 --entity-log \
  --spawn 'minecraft:minecart@2.5,1.0625,1.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,1.0625,1.5:0,0,0:ride' \
  --spawn 'minecraft:minecart@2.5,1.0,4.5:0,0,NaN' \
  --spawn 'minecraft:blaze@2.5,1.0,4.5:0,0,0:ride' \
  --spawn 'minecraft:blaze@2.5,1.0,10.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@2.5,3.0,10.5:0,0,0' \
  --spawn 'minecraft:villager@2.5,1.0,13.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@2.5,3.0,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.0,16.5:0,0,0:nogravity' \
  --spawn 'minecraft:minecart@2.5,3.0,16.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@2.5,1.0,19.5:0,0,0:nogravity' \
  --spawn 'minecraft:minecart@2.5,3.0,19.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,27.5:0,0,0' \
  --out work/blaze_ride_ai.json | tee "$CAP/blaze_ride_ai.entities.log"

# ---------------------------------------------------------------------------
# 9. Piston RETRACTION moving entities — the half that piston_pull left open.
#
# `piston_pull` put every entity inside the head's own block and read the
# answers as displacements, where they look unrelated: the carts move +0.01 and
# the small fireball -0.32375. Read as a box edge they are one number — all four
# finish with their trailing face at exactly 3.02 — and that is the law.
#
# Three rigs, because there turned out to be two mechanisms and they had been
# run together:
#
#   piston_pull_law    a sticky piston with NOTHING to pull, entity started all
#                      over the head's block and outside it, at two heights,
#                      plus a non-sticky control. Every lane whose entity CENTRE
#                      is in the head's block finishes at 3.02 no matter which
#                      side it started or how tall it sits; every lane whose
#                      centre is outside is untouched. The gate is the centre,
#                      not the box.
#
#   piston_pull_plate  the record door's own trick, read off a PRESSURE PLATE
#                      rather than off the entity. A dragon fireball at y=1.02
#                      dips into the plate's touch box and pokes into the
#                      piston's block at once — which is what the doc says the
#                      dragon fireball is *for*. With a stone block for the
#                      sticky head to pull it is dragged 4.45 -> 3.94 -> 3.50,
#                      nearly a full block, and the plate it lands on POWERS at
#                      t8. With nothing to pull, the same entity in the same
#                      place never moves. So the "pull" is the pulled block's
#                      sweep, not the arm's: a retracting head reaches nowhere
#                      outside the square it is leaving.
#
#                      The plates are OAK. A stone plate is Sensitivity.MOBS and
#                      cannot see a fireball at all; the first cut of this rig
#                      used stone and recorded a flat nothing that read as "it
#                      never reached the plate" when it meant "this plate is
#                      blind to fireballs".
#
#   piston_pull_fit    start-position sweep with a non-cube hitbox, to separate
#                      "flush against the arriving block" from the real answer.
#                      Every lane stops at 3.00 — the piston body — and the
#                      per-step distances are the ordinary sweep capped at
#                      PISTON_STEP + PISTON_OVERSHOOT.
#
#   piston_pull_inside a VERTICAL piston whose head retracts UP into its own
#                      square, with entities standing in THAT square rather than
#                      in the one the head leaves. This is the geometry the
#                      record door uses — its pistons face down — and none of
#                      the rigs above covers it.
#
#                      Vanilla moves them, and within this capture the law is
#                      exact, 8 predictions out of 8: on the first step the
#                      entity is pushed out of the middle half of the piston's
#                      square, [3.25, 3.75], to whichever side it is nearer, to
#                      3.24 or 3.76; on the second it is pushed out of the whole
#                      square, to 2.98 or 4.01. It reproduces the lane that does
#                      not move on the first step (its box clears [3.25,3.75])
#                      and the one that is thrown upward instead of down.
#
#                      **It is still not implemented, because it contradicts a
#                      lane we already have.** `piston_pull_law`'s first lane is
#                      the same situation on the x axis — a small fireball at
#                      2.5, inside the horizontal piston's own square — and
#                      vanilla leaves it exactly where it is for twenty ticks,
#                      where this law demands a shove of 0.41625. Two captures,
#                      one answer each, and no single rule gives both. So the
#                      engine reports this case (`PistonPush::Unmodelled` ->
#                      `Simulation::piston_retract_contacts`) rather than
#                      guessing, and the next experiment is the one that
#                      separates them: the horizontal rig rebuilt with the
#                      entity floating clear of the floor, to find out whether
#                      the difference is the axis or the support.
tools/gametest/capture.sh --structure nucleation:piston_pull_law --max-ticks 16 --entity-log \
  --spawn 'minecraft:small_fireball@2.5,1.0,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.1,1.0,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.5,1.0,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.9,1.0,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.3,1.0,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.5,1.5,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.17625,1.0,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.5,1.0,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.1,1.0,17.5:0,0,0' \
  --spawn 'minecraft:minecart@3.5,1.0,19.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@3.5,1.0,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.5,1.0,23.5:0,0,0' \
  --at 6:1,1,1:air --at 6:1,1,3:air --at 6:1,1,5:air --at 6:1,1,7:air \
  --at 6:1,1,9:air --at 6:1,1,11:air --at 6:1,1,13:air --at 6:1,1,15:air \
  --at 6:1,1,17:air --at 6:1,1,19:air --at 6:1,1,21:air --at 6:1,1,23:air \
  --out work/piston_pull_law.json | tee "$CAP/piston_pull_law.entities.log"

tools/gametest/capture.sh --structure nucleation:piston_pull_plate --max-ticks 18 --entity-log \
  --spawn 'minecraft:dragon_fireball@4.45,1.02,1.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@4.6,1.02,3.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@4.45,1.02,5.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@4.45,1.02,7.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@3.55,1.02,9.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@4.2,2.0,11.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@4.8,2.0,13.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@5.2,2.0,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.45,2.0,17.5:0,0,0' \
  --spawn 'minecraft:minecart@4.45,2.0,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.9,2.0,21.5:0,0,0' \
  --at 8:1,2,1:air --at 8:1,2,3:air --at 8:1,2,7:air --at 8:6,2,9:air \
  --at 8:1,2,11:air --at 8:1,2,13:air --at 8:1,2,15:air --at 8:1,2,17:air \
  --at 8:1,2,19:air --at 8:1,2,21:air \
  --out work/piston_pull_plate.json | tee "$CAP/piston_pull_plate.entities.log"

tools/gametest/capture.sh --structure nucleation:piston_pull_fit --max-ticks 16 --entity-log \
  --spawn 'minecraft:small_fireball@4.15,2.0,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.1,2.0,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.9,2.0,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.6,2.0,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.2,2.0,9.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@4.05,2.0,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.5,2.0,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@3.9,2.0,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.14,2.0,17.5:0,0,0' \
  --at 8:1,2,1:air --at 8:1,2,3:air --at 8:1,2,5:air --at 8:1,2,7:air \
  --at 8:1,2,9:air --at 8:1,2,11:air --at 8:1,2,13:air --at 8:1,2,15:air --at 8:1,2,17:air \
  --out work/piston_pull_fit.json | tee "$CAP/piston_pull_fit.entities.log"

tools/gametest/capture.sh --structure nucleation:piston_pull_inside --max-ticks 16 --entity-log \
  --spawn 'minecraft:small_fireball@2.5,3.2,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,2.9,3.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@2.5,2.6,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,3.2,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,3.6,9.5:0,0,0' \
  --at 6:2,4,1:air --at 6:2,4,3:air --at 6:2,4,5:air --at 6:2,4,9:air \
  --out work/piston_pull_inside.json | tee "$CAP/piston_pull_inside.entities.log"

# ---------------------------------------------------------------------------
# 10. What a cart stands on, and what it walks through.
#
# `blaze_ride_ai` had already shown a cart resting on a blaze and falling
# through a fireball, and the obvious conclusion — "a LivingEntity is solid, a
# projectile is not" — is **wrong**, which is why these four rigs exist. Ten
# bodies, each met twice: dropped on from above, and driven into sideways.
#
#   cart_body   drop lanes and the two probes the drop lanes cannot answer.
#
#               Resting height is the body's exact float top: blaze
#               2.799999952316284, villager 2.950000047683716, zombie
#               2.950000047683716, boat 1.5625, cart 1.699999988079071.
#               Armor stand, small fireball and dragon fireball reproduce the
#               empty control's fall to 1.0 on tick 10, digit for digit. So an
#               **armor stand is a LivingEntity a cart falls through** and a
#               **boat is not living and holds one up**: "living" is refuted at
#               both of its edges, and what fits all ten is vanilla's vehicle
#               predicate, canBeCollidedWith() || isPushable().
#
#               z=37.5 and z=40.5 are the onGround probe — the same cart with
#               vx = 0.1, one over a blaze and one over stone. Off-rail,
#               comeOffTrack halves the horizontal velocity when grounded and
#               multiplies by 0.95f when airborne, and the blaze lane takes the
#               *grounded* branch the tick it lands: 0.09025 -> 0.045125 ->
#               0.0225625. Resting on a mob is being on the ground.
#
#               z=43.5 is the negative control for the support being positional:
#               the same blaze, the cart 1.5 blocks to the side, and it falls.
#
#               The rail lanes here measure something else entirely and are the
#               reason cart_body2 exists: a **plain** cart rolling at a free
#               blaze does not stop, it *picks the blaze up* — AbstractMinecart
#               mounts a pushable LivingEntity it finds inside inflate(0.2,0,0.2)
#               — and then coasts on at the ridden 0.997 instead of the empty
#               0.96. A small fireball in the same lane is bit-identical to the
#               empty control at every tick.
tools/gametest/capture.sh --structure nucleation:cart_body --max-ticks 40 --entity-log \
  --spawn 'minecraft:minecart@1.5,1.0625,1.5:0.3,0,0' \
  --spawn 'minecraft:blaze@6.5,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@1.5,1.0625,4.5:0.3,0,0' \
  --spawn 'minecraft:villager@6.5,1.0,4.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@1.5,1.0625,7.5:0.3,0,0' \
  --spawn 'minecraft:small_fireball@6.5,1.0,7.5:0,0,0:nogravity' \
  --spawn 'minecraft:minecart@1.5,1.0625,10.5:0.3,0,0' \
  --spawn 'minecraft:armor_stand@2.5,1.0,13.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@2.5,3.0,13.5:0,0,0' \
  --spawn 'minecraft:oak_boat@2.5,1.0,16.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,16.5:0,0,0' \
  --spawn 'minecraft:zombie@2.5,1.0,19.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@2.5,3.0,19.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,1.0,22.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,22.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,1.0,25.5:0,0,NaN' \
  --spawn 'minecraft:blaze@2.5,3.0,25.5:0,0,0:noai' \
  --spawn 'minecraft:blaze@2.5,1.0,28.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@2.5,3.0,28.5:0,0,NaN' \
  --spawn 'minecraft:blaze@2.5,1.0,31.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,31.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,34.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,1.0,37.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@2.5,3.0,37.5:0.1,0,0' \
  --spawn 'minecraft:minecart@2.5,3.0,40.5:0.1,0,0' \
  --spawn 'minecraft:blaze@2.5,1.0,43.5:0,0,0:noai' \
  --spawn 'minecraft:minecart@4.0,3.0,43.5:0,0,0' \
  --out work/cart_body.json | tee "$CAP/cart_body.entities.log"

# NB two lanes of that rig are **void and must not be read as answers**: the
# noai blazes at z=25.5 and z=28.5 were meant to ask whether a mob rests on a
# NaN cart, and a `noai` mob turns out not to fall at all — it holds y = 3.0 for
# forty ticks. cart_body2 asks the same question with AI on.
#
#   cart_body2  the sideways half, with **furnace** carts, because a plain one
#               mounts the mob instead of hitting it. Every lane stops with its
#               east face on the body's west face, exactly: blaze and villager
#               at x = 5.709999978542328 (face 6.199999988079071), boat at
#               5.322499990463257 (face 5.8125, so width 1.375). Armor stand,
#               dragon fireball and the empty control all reach the same
#               5.764574172160477.
#
#               Its drop lanes answer the two the first rig could not:
#               a blaze **with AI** dropped from y=3 onto a NaN cart, an
#               ordinary cart and a furnace cart lands on the *floor* at 1.0 on
#               tick 19 in all three lanes, the same tick as the empty control.
#               A mob's own movement collides with no cart — the solidity is
#               one-way. And a furnace cart dropped on a **seated** blaze rests
#               at 2.987499952316284, the vehicle's y plus the 0.1875 seat plus
#               1.8f, so a passenger's box holds a cart up like any other.
tools/gametest/capture.sh --structure nucleation:cart_body2 --max-ticks 40 --entity-log \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,1.5:0.3,0,0' \
  --spawn 'minecraft:blaze@6.5,1.0,1.5:0,0,0:noai' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,4.5:0.3,0,0' \
  --spawn 'minecraft:villager@6.5,1.0,4.5:0,0,0:noai' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,7.5:0.3,0,0' \
  --spawn 'minecraft:oak_boat@6.5,1.0,7.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,10.5:0.3,0,0' \
  --spawn 'minecraft:armor_stand@6.5,1.0,10.5:0,0,0:noai' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,13.5:0.3,0,0' \
  --spawn 'minecraft:dragon_fireball@6.5,1.0,13.5:0,0,0:nogravity' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,16.5:0.3,0,0' \
  --spawn 'minecraft:minecart@2.5,1.0,22.5:0,0,NaN' \
  --spawn 'minecraft:blaze@2.5,3.0,22.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,1.0,25.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,3.0,25.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,3.0,28.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@2.5,1.0,31.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,3.0,31.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@2.5,1.0,34.5:0,0,0:nogravity' \
  --spawn 'minecraft:furnace_minecart@2.5,3.0,34.5:0,0,0' \
  --spawn 'minecraft:oak_boat@2.5,1.0,37.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@2.5,3.0,37.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,1.0,40.5:0,0,NaN' \
  --spawn 'minecraft:blaze@2.5,1.0,40.5:0,0,0:ride' \
  --spawn 'minecraft:furnace_minecart@2.5,4.0,40.5:0,0,0' \
  --out work/cart_body2.json | tee "$CAP/cart_body2.entities.log"

#   cart_body3  does the body feel anything back? A furnace cart presses against
#               a blaze **with AI** for twenty ticks and the blaze holds
#               (2.5, 1.0, 1.5) to the last digit; a cart resting on an AI blaze
#               reproduces the noai lane's x bit for bit. Nothing is pushed and
#               nothing takes the weight. The villager and zombie lanes are
#               **not** evidence either way — both wander off under their own AI,
#               changing z as well as x, which is pathing and not a shove.
#
#               Its `minecraft:item` lane is void: `--spawn` builds an ItemEntity
#               with an empty stack and the game discards it before tick 0. The
#               item answer comes from cart_body4, which authors it in the
#               structure file instead.
tools/gametest/capture.sh --structure nucleation:cart_body2 --max-ticks 40 --entity-log \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,1.5:0.3,0,0' \
  --spawn 'minecraft:blaze@6.5,1.0,1.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,4.5:0.3,0,0' \
  --spawn 'minecraft:villager@6.5,1.0,4.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,7.5:0.3,0,0' \
  --spawn 'minecraft:zombie@6.5,1.0,7.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,10.5:0.3,0,0' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,13.5:0.3,0,0' \
  --spawn 'minecraft:item@6.5,1.0,13.5:0,0,0' \
  --spawn 'minecraft:blaze@2.5,1.0,22.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@2.5,3.0,22.5:0.1,0,0' \
  --spawn 'minecraft:villager@2.5,1.0,25.5:0,0,0:noai' \
  --spawn 'minecraft:furnace_minecart@2.5,3.0,25.5:0.1,0,0' \
  --out work/cart_body3.json | tee "$CAP/cart_body3.entities.log"

#   cart_body4  the two kinds `--spawn` cannot ask about: a **ghast** fireball
#               (`minecraft:fireball`, which shares the dragon fireball's 1x1
#               box and had never been measured on its own) and a real **item
#               entity**, authored into the structure with a stack so the game
#               keeps it. Both are transparent in both axes — the two rail lanes
#               and the two drop lanes all reproduce their controls exactly.
tools/gametest/capture.sh --structure nucleation:cart_body4 --max-ticks 40 --entity-log \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,1.5:0.3,0,0' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,4.5:0.3,0,0' \
  --spawn 'minecraft:fireball@6.5,1.0,4.5:0,0,0:nogravity' \
  --spawn 'minecraft:furnace_minecart@1.5,1.0625,7.5:0.3,0,0' \
  --spawn 'minecraft:fireball@2.5,1.0,10.5:0,0,0:nogravity' \
  --spawn 'minecraft:furnace_minecart@2.5,3.0,10.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@2.5,3.0,13.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@2.5,3.0,16.5:0,0,0' \
  --out work/cart_body4.json | tee "$CAP/cart_body4.entities.log"

# Every lane of every rig below cuts its own power at t6.
LANES_AIR="--at 6:1,1,1:air --at 6:1,1,3:air --at 6:1,1,5:air --at 6:1,1,7:air
  --at 6:1,1,9:air --at 6:1,1,11:air --at 6:1,1,13:air --at 6:1,1,15:air
  --at 6:1,1,17:air --at 6:1,1,19:air --at 6:1,1,21:air --at 6:1,1,23:air"

# ---------------------------------------------------------------------------
# The third geometry, settled: an entity inside the piston's OWN square.
#
# Section 9 said the vertical rig and `piston_pull_law` lane 1 could not both be
# right. They can. **Neither the axis nor the floor was the variable.**
#
# piston_pull_float is `piston_pull_law` again with the entities LIFTED. Lane 1
# is the contradicting lane exactly as it was, on the floor at y=1.0, and it
# still does not move. Lane 2 is the same fireball raised by 0.34375 and nothing
# else, and vanilla throws it 0.41625 — the number the vertical law demanded all
# along. Lanes 9-11 are a minecart, a furnace cart and a NaN furnace cart, which
# reach the same band from the floor because they are 0.7 tall; all three move.
#
# What lane 1 was measuring is the piston ARM: a 4/16 column through the middle
# of the block, which a box must overlap in BOTH cross-axes to be touched. The
# floor fireball tops out at 1.3125 and the arm starts at 1.375.
tools/gametest/capture.sh --structure nucleation:piston_pull_law --max-ticks 16 --entity-log \
  --spawn 'minecraft:small_fireball@2.5,1.0,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.34375,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.64375,1.34375,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.94375,1.34375,7.5:0,0,0' \
  --spawn 'minecraft:dragon_fireball@2.9,1.0,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.24375,1.34375,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.34375,13.7:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.7,15.5:0,0,0' \
  --spawn 'minecraft:minecart@2.5,1.0,17.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@2.5,1.0,19.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@2.5,1.0,21.5:0,0,NaN' \
  --spawn 'minecraft:small_fireball@2.5,1.34375,23.5:0,0,0' \
  $LANES_AIR --out work/piston_pull_float.json \
  | tee "$CAP/piston_pull_float.entities.log"

# `piston_pull_law` IS NOT A UNIFORM RIG, and reading it as one costs a day.
# Lanes z = 15, 17, 19 and 21 carry a stone block at (4,1,z) for the sticky head
# to PULL; the other eight have nothing. The pulled block's sweep lands on top
# of everything else, so the identical fireball finishes at 3.02 in a pull-free
# lane and at 3.00 in a pulling one, and two captures of "the same" lane read as
# non-determinism. `piston_pull_uniform` is the proof: twelve lanes, one x, and
# the answer changes at exactly z = 15.
tools/gametest/capture.sh --structure nucleation:piston_pull_law --max-ticks 12 --entity-log \
  --spawn 'minecraft:small_fireball@2.45,1.34375,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,17.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,23.5:0,0,0' \
  $LANES_AIR --out work/piston_pull_uniform.json \
  | tee "$CAP/piston_pull_uniform.entities.log"

# `piston_pull_square` is that rig with twelve pull-free sticky lanes and
# nothing else. Every constant in `piston::inside_eject_displacement` comes
# from the four captures below.
#
# The law: the entity is driven to the outermost of three lines it can reach in
# one 0.51 step — trailing face 1.01 of the way through the square, or 0.76, or
# leading face back to 0.24 — and if it can reach none it retreats a whole step.
# On the second step the lines are 1.02 and -0.01. Watch lane 7 (x=2.70) jump to
# the outer line where lane 6 (x=2.65) stops at the inner one: that is the
# hand-over, measured to a ten-thousandth by the threshold capture below.
tools/gametest/capture.sh --structure nucleation:piston_pull_square --max-ticks 16 --entity-log \
  --spawn 'minecraft:small_fireball@2.40,1.34375,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.45,1.34375,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.50,1.34375,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.55,1.34375,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.60,1.34375,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.65,1.34375,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.70,1.34375,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.75,1.34375,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.80,1.34375,17.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.85,1.34375,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.90,1.34375,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.95,1.34375,23.5:0,0,0' \
  $LANES_AIR --out work/piston_pull_xsweep.json \
  | tee "$CAP/piston_pull_xsweep.entities.log"

# The same sweep with the record door's own hitbox — a minecart, 0.98 x 0.7 —
# which needs no lifting, being tall enough to reach the arm from the floor.
# The last two lanes clear the middle half and are handled by
# `head_eject_displacement` instead: that is the seam between the two laws.
tools/gametest/capture.sh --structure nucleation:piston_pull_square --max-ticks 12 --entity-log \
  --spawn 'minecraft:minecart@2.70,1.0,1.5:0,0,0' \
  --spawn 'minecraft:minecart@2.75,1.0,3.5:0,0,0' \
  --spawn 'minecraft:minecart@2.80,1.0,5.5:0,0,0' \
  --spawn 'minecraft:minecart@2.85,1.0,7.5:0,0,0' \
  --spawn 'minecraft:minecart@2.90,1.0,9.5:0,0,0' \
  --spawn 'minecraft:minecart@2.95,1.0,11.5:0,0,0' \
  --spawn 'minecraft:minecart@3.00,1.0,13.5:0,0,0' \
  --spawn 'minecraft:minecart@3.05,1.0,15.5:0,0,0' \
  --spawn 'minecraft:minecart@3.10,1.0,17.5:0,0,0' \
  --spawn 'minecraft:minecart@3.20,1.0,19.5:0,0,0' \
  --spawn 'minecraft:minecart@3.25,1.0,21.5:0,0,0' \
  --spawn 'minecraft:minecart@3.30,1.0,23.5:0,0,0' \
  $LANES_AIR --out work/piston_square_cart.json \
  | tee "$CAP/piston_square_cart.entities.log"

# The gate across the piston, to a thousandth. The arm's column is [6/16, 10/16]
# and the intersection is strict: a box whose top is exactly 1.375, or whose
# bottom is exactly 1.625, is NOT moved, and one 0.0025 inside is thrown the
# full 0.41625. Twelve lanes, both edges, and the answer flips exactly there.
tools/gametest/capture.sh --structure nucleation:piston_pull_square --max-ticks 12 --entity-log \
  --spawn 'minecraft:small_fireball@2.5,1.05,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.06,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.0625,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.065,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.08,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.20,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.34375,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.60,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.62,17.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.625,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.63,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.5,1.70,23.5:0,0,0' \
  $LANES_AIR --out work/piston_square_yband.json \
  | tee "$CAP/piston_square_yband.entities.log"

# Where one target gives way to the next. A target costing exactly 0.51 is taken
# and one costing 0.5101 is not, at BOTH hand-overs — which is why the limit is
# 0.51 and not the 0.5 step it is built out of. Lane 3 (x=2.65635) is the one
# lane in fifty-five that the fitted law misses, by 1e-4, and it is asserted as
# a disagreement in `the_hand_over_between_targets_is_exactly_the_step_limit`.
tools/gametest/capture.sh --structure nucleation:piston_pull_square --max-ticks 12 --entity-log \
  --spawn 'minecraft:small_fireball@2.66625,1.34375,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.65875,1.34375,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.65635,1.34375,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.65625,1.34375,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.65615,1.34375,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.65375,1.34375,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.65125,1.34375,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.64625,1.34375,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.40725,1.34375,17.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.40625,1.34375,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.40615,1.34375,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@2.40525,1.34375,23.5:0,0,0' \
  $LANES_AIR --out work/piston_square_threshold.json \
  | tee "$CAP/piston_square_threshold.entities.log"

# ---------------------------------------------------------------------------
# 12. The record 3x3 door's fireball-onto-a-plate trick, read off the PLATE.
#
# Every capture above reads a retraction through the *entity log*, which prints
# settled positions only. The record door's own gadget cannot be answered that
# way: a small fireball sits inside the sticky piston at (74,0,20) with its east
# face flush on the block boundary at x = 74.0, exactly 0.0625 short of the
# light weighted plate at (74,1,20), and the builders say that when "the piston
# that has the pressure plate on it pulls back, the fireball will barely clip
# the pressure plate". A plate's `power` is a block state, so it survives every
# entity filter — and it is the only channel that can see *inside* a tick.
#
# Rig frame, mapped from the door: piston (74,0,20) -> (5,1,z), the plate
# (74,1,20) -> (5,2,z), the pushed sticky piston (73,0,20) -> (4,1,z), the
# pushed quartz (72,0,20) -> (3,1,z), the observer the fireball is also embedded
# in (73,1,20) -> quartz at (4,2,z). The fireball's box x [73.6875, 74.0],
# y [0.875, 1.1875] becomes a spawn at (4.84375, 1.875).
#
# What the three captures settle, in order:
#
#   piston_plate_clip  the door's own sequence and its variants. Lane z=1 is the
#                      replica: the extension throws the fireball WEST 0.6875
#                      (0.51 then 0.5, clipped by the moving block at x=3), and
#                      then **the retraction brings it all the way back east**,
#                      +0.51 then +0.1775, clipped with its east face on 5.0 —
#                      its exact start — and the PLATE POWERS at t9. So vanilla
#                      throws it east, and the door's plate does fire.
#                      Negative controls in the same rig: `extend_only` never
#                      powers the plate, `negcontrol_noball` never powers it,
#                      `replica_nopush` (nothing to push, so the fireball ends a
#                      block west and the retraction cannot reach it) never
#                      powers it. Positive control: `poscontrol_onplate` powers
#                      it on tick 0.
#   plate_reach_flush  and yet the plate's sensing edge is exactly where it was
#                      always modelled. Twelve lanes, no pistons, nothing moves:
#                      a fireball whose east face is 5.0625 does NOT press it and
#                      one at 5.0626 does, so `TOUCH_AABB` really is the cell
#                      inset a pixel and `AABB.intersects` really is strict. A
#                      fireball parked with its east face flush at 5.0 — the
#                      settled position lane z=1 above ends in — never presses
#                      it, on a piston or on stone, for 24 ticks.
#                      **So the trigger is intra-tick.**
#   piston_head_transient  how far intra-tick. Sixteen lanes sweep the start x
#                      with and without a block to pull. Every lane ends with its
#                      east face on 4.98, and the plate fires on the tick the
#                      *step* would have carried it past 5.0625 — x = 4.45 fires
#                      on the first step, x = 4.35 does not and fires on the
#                      second. The threshold is one PISTON_MAX_STEP (0.51) from
#                      the box's east face, so a retraction drags the entity a
#                      whole step inward and only then corrects it back to the
#                      line. That intermediate box is what presses the plate.
#   piston_head_yband  and the gate across the axis is the entity's FEET. Eleven
#                      lanes, x fixed at the door's own 4.84375, y swept: a
#                      fireball at [1.99, 2.3025] — two thirds of it in the block
#                      above the vacated one, centre y 2.14 — is ejected, and one
#                      at [0.95, 1.2625] — overlapping the vacated block by
#                      0.2625, centre y 1.11 — is not. Only `min y` separates
#                      them, which is `BlockPos.containing(position())` showing
#                      through, and it is why `head_eject_displacement`'s
#                      all-three-axes centre gate refused the door's fireball
#                      id=11 (feet 0.875, centre 1.03125).
#
# Plate `power` is in the trace JSON at (5,2,z); the entity log gives the
# settled positions. Read both.
tools/gametest/capture.sh --structure nucleation:piston_plate_clip --max-ticks 32 --entity-log \
  --spawn 'minecraft:small_fireball@4.84375,1.875,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.15625,1.875,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.0,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.9,1.875,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.95,1.875,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,17.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@5.1,1.875,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.8,1.875,23.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.875,1.875,25.5:0,0,0' \
  --spawn 'minecraft:small_fireball@5.0,1.875,27.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,29.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.0,31.5:0,0,0' \
  --at 2:6,1,1:redstone_block --at 8:6,1,1:air --at 2:6,1,3:redstone_block --at 6:6,1,5:air \
  --at 6:6,1,7:air --at 6:6,1,9:air --at 6:6,1,11:air --at 6:6,1,13:air --at 6:6,1,15:air \
  --at 2:6,1,17:redstone_block --at 8:6,1,17:air --at 6:6,1,19:air --at 2:6,1,21:redstone_block \
  --at 8:6,1,21:air --at 6:6,1,23:air --at 6:6,1,25:air --at 6:6,1,27:air \
  --at 2:6,1,29:redstone_block --at 20:6,1,29:air --at 2:6,1,31:redstone_block --at 8:6,1,31:air \
  --out work/piston_plate_clip.json | tee "$CAP/piston_plate_clip.entities.log"

# The same rig with WIDE bodies, which is what settles *when* the block in flight
# starts colliding. Every lane above is a 0.3125 fireball that is still a whole
# block clear of the arriving block when its first step is taken, so the first
# step never binds and both "solid at its destination all along" and "solid only
# on the second step" fit. A dragon fireball is 1.0 wide, so in the replica lane
# its box is [4.0, 5.0] — its leading face already flush on the line the small
# fireball is clipped to — and the two rules give opposite answers.
#
# Result: vanilla moves it the full 0.51 on the first step and only then clips,
# to 0.49, against the cell the pushed *quartz* is arriving in (x = 3.0). So a
# moving block is transparent on the first step and solid at its destination on
# the second, which is `progress < 1.0` read after the step's increment. The
# 0.98 furnace minecart, 0.02 clear of the same line, gets 0.51 then a full 0.5.
#
#   z=1   dragon fireball,  extend then retract,  a block to push  4.5 -> 3.5
#   z=5   dragon fireball,  starts extended                       round trip +-0.25
#   z=17  dragon fireball,  nothing to push (control)              4.5 -> 3.49
#   z=9   furnace minecart, starts extended                        round trip +-0.25
#   z=29  furnace minecart, extend then retract                    4.51 -> 3.4999999905
#
# The two `starts extended` lanes are *not* clip evidence and are recorded as a
# disagreement instead: a 1.0-wide body straddles the piston arm's 4/16 column
# rather than sitting inside it, so `inside_eject_displacement` declines and
# `head_eject_displacement` answers -0.02 where vanilla moves it +0.25 and back.
# That is a hole in retraction's law for wide bodies, not in this clip.
tools/gametest/capture.sh --structure nucleation:piston_plate_clip --max-ticks 24 --entity-log \
  --spawn 'minecraft:dragon_fireball@4.5,1.875,1.5:0,0,0:nogravity' \
  --spawn 'minecraft:dragon_fireball@4.5,1.875,5.5:0,0,0:nogravity' \
  --spawn 'minecraft:dragon_fireball@4.5,1.875,17.5:0,0,0:nogravity' \
  --spawn 'minecraft:furnace_minecart@4.51,1.0,9.5:0,0,0' \
  --spawn 'minecraft:furnace_minecart@4.51,1.0,29.5:0,0,0' \
  --at 2:6,1,1:redstone_block --at 8:6,1,1:air \
  --at 6:6,1,5:air --at 6:6,1,9:air \
  --at 2:6,1,17:redstone_block --at 8:6,1,17:air \
  --at 2:6,1,29:redstone_block --at 14:6,1,29:air \
  --out work/piston_clip_sizes.json | tee "$CAP/piston_clip_sizes.entities.log"

tools/gametest/capture.sh --structure nucleation:plate_reach_flush --max-ticks 24 --entity-log \
  --spawn 'minecraft:small_fireball@4.84375,1.875,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.83375,1.875,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.8,1.875,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.90625,1.875,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.90635,1.875,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.9,1.875,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.6875,1.875,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.9375,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,2.25,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,23.5:0,0,0' \
  --out work/plate_reach_flush.json | tee "$CAP/plate_reach_flush.entities.log"

tools/gametest/capture.sh --structure nucleation:piston_head_transient --max-ticks 16 --entity-log \
  --spawn 'minecraft:small_fireball@4.2,1.875,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.35,1.875,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.45,1.875,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.5,1.875,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.55,1.875,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.65,1.875,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.75,1.875,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.9,1.875,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.2,1.875,17.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.35,1.875,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.45,1.875,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.5,1.875,23.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.55,1.875,25.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.65,1.875,27.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.75,1.875,29.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.9,1.875,31.5:0,0,0' \
  --at 6:6,1,1:air --at 6:6,1,3:air --at 6:6,1,5:air --at 6:6,1,7:air --at 6:6,1,9:air \
  --at 6:6,1,11:air --at 6:6,1,13:air --at 6:6,1,15:air --at 6:6,1,17:air --at 6:6,1,19:air \
  --at 6:6,1,21:air --at 6:6,1,23:air --at 6:6,1,25:air --at 6:6,1,27:air --at 6:6,1,29:air \
  --at 6:6,1,31:air \
  --out work/piston_head_transient.json | tee "$CAP/piston_head_transient.entities.log"

tools/gametest/capture.sh --structure nucleation:piston_head_yband --max-ticks 14 --entity-log \
  --spawn 'minecraft:small_fireball@4.84375,0.5,1.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,0.7,3.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,0.95,5.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.0,7.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.34375,9.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.6875,11.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.7,13.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,15.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.99,17.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,2.0,19.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,2.5,21.5:0,0,0' \
  --spawn 'minecraft:small_fireball@4.84375,1.875,23.5:0,0,0' \
  --at 6:6,1,1:air --at 6:6,1,3:air --at 6:6,1,5:air --at 6:6,1,7:air --at 6:6,1,9:air \
  --at 6:6,1,11:air --at 6:6,1,13:air --at 6:6,1,15:air --at 6:6,1,17:air --at 6:6,1,19:air \
  --at 6:6,1,21:air --at 6:6,1,23:air --at 6:6,1,25:air \
  --out work/piston_head_yband.json | tee "$CAP/piston_head_yband.entities.log"

# ---------------------------------------------------------------------------
# 13. A weighted plate's recheck cadence, and what a plate powers underneath it.
#
# Two things the engine had never measured, in one rig of eleven lanes.
#
# The cadence first. `WeightedPressurePlateBlock.getPressedTime()` is `bipush
# 10`, but no capture had ever *seen* a weighted plate release: every one of them
# kept an item on the plate, and an item pressing a plate is an item **resting**
# on it. `TOUCH_AABB` and the plate's collision shape share the same 14/16
# footprint, so anything inside the touch box is standing on the plate and can
# never fall out of it — which is why the constant stayed bytecode-only. Only a
# gravityless entity moved by something else releases one.
#
#   lanes z=1..13   a small fireball straddles the plate's touch box from the
#                   cell east of it, and a +z piston shoves it out at a
#                   staggered tick. The plates all press at t0; the fireball
#                   leaves at roughly t3, t7, t11, t15 and t21; the plates
#                   release at t10, t10, t20, t20 and t30. That is the ten-tick
#                   interval, measured, and it is anchored on the PRESS — never
#                   on the departure, though `entityInside` fires every tick the
#                   fireball is there and could have re-booked it.
#   lane z=16       negative control: nothing shoves it, so it stays powered for
#                   all 44 ticks — four consecutive rechecks, none lost.
#   lane z=19       negative control: no entity at all, so it never powers.
#   lane z=22       MEANT to test a second entity arriving while the plate is
#                   already pressed, and does NOT: the head-eject shoves the
#                   second fireball only 0.16625 west (just clear of the cell it
#                   vacates), so it never reaches the touch box. Recorded as a
#                   null result rather than dropped.
#
# Then the strong power. `BasePressurePlateBlock.checkPressed` writes with flags
# **2** and then calls `updateNeighbours`, which is `updateNeighborsAt(pos)` AND
# `updateNeighborsAt(pos.below())` — the same pair the detector rail needs, for
# the same reason: `getDirectSignal` answers for `Direction.UP` alone, so a plate
# strongly powers the block it stands on and a component touching only that
# block hears about the change by no other route.
#
#   lanes z=25/28/31  dust laid beside the plate's stone support and never
#                     adjacent to the plate reads 1 under a light weighted plate
#                     and 15 under an oak plate, while the control lane whose
#                     (5,2,z) is plain stone stays dark for all 44 ticks. The
#                     engine left all three at zero until this landed.
#
# The plate's `power` and the dust's `power` are both block states, so the trace
# JSON carries the whole result; the entity log gives the shove distances.
tools/gametest/capture.sh --structure nucleation:plate_recheck --max-ticks 44 --entity-log \
  --at 1:7,2,0:redstone_block \
  --at 1:7,2,18:redstone_block \
  --at 3:9,2,22:redstone_block \
  --at 5:7,2,3:redstone_block \
  --at 9:7,2,6:redstone_block \
  --at 13:7,2,9:redstone_block \
  --at 19:7,2,12:redstone_block \
  --out "$TRACES/plate_recheck.json" | tee "$CAP/plate_recheck.entities.log"

echo "captures written to $CAP and $TRACES"
