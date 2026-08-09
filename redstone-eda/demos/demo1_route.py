"""demo1: auto-route a net past an obstacle, DRC it, then PROVE conduction.

A lever sits on the west side, a target rail on the east, and a stone wall
in between.  Routing.route_net (the new bridge API) finds a path, emits the
dust + supports + any repeaters straight into the schematic, and returns the
path as JSON.  Routing.drc audits the result, and mc-tick's TickSimulation
proves the routed wire actually conducts when the lever flips.
"""
import json
import os

import _common
from _common import n, rs

b = rs.Build("demo1_route")

# Source: a lever on stone, output cell east of it (left as air for the
# router to claim).
b.stone(0, 0, 0)
b.force(0, 1, 0, rs.LEVER_OFF)
LEVER = (0, 1, 0)

# Obstacle: a stone wall across the direct path, wide and tall enough that
# the router must detour around its edge.
for z in range(-4, 5):
    for y in range(0, 4):
        b.stone(4, y, z)

# Destination: an unpowered rail cell at (9, 1, 0), east of the wall.
DST = (9, 1, 0)

print("== demo1: route past a wall, DRC, then simulate ==")
print(f"lever {LEVER} -> dst {DST}, wall at x=4 (z -4..4, y 0..3)")

path = json.loads(n.Routing.route_net(b.s, 1, 1, 0, *DST, "net0"))
print(f"routed: {len(path)} path cells: {path[0]} .. {path[-1]}")
under = [p for p in path if p[1] < 1]
print(f"the path dives under the wall through {len(under)} below-grade cells")
assert under, "expected a detour, the wall was ignored"

violations = json.loads(n.Routing.drc(b.s, False))
print(f"DRC: {len(violations)} violations")
assert not violations, violations

# Simulate: flip the lever, settle, and read power at the far end.
sim, (ox, oy, oz) = _common.simulate(b.s)
before = _common.power_at(sim, DST[0] - ox, DST[1] - oy, DST[2] - oz)
sim.use_block(LEVER[0] - ox, LEVER[1] - oy, LEVER[2] - oz)
sim.run_until_quiescent(400)
after = _common.power_at(sim, DST[0] - ox, DST[1] - oy, DST[2] - oz)
print(f"conduction: dst wire power {before} -> {after} after the lever flip")
assert before == 0 and after > 0, (before, after)

out = os.path.join(_common.HERE, "routed.schem")
b.s.save_to_file(out)
print(f"saved {out}")
print("demo1 PASS: the routed net conducts end to end")
