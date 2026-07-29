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

echo "captures written to $CAP and $TRACES"
