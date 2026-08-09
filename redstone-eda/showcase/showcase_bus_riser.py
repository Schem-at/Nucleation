"""bus_riser8.schem: an 8-bit bus routed up two levels via a torch-ladder bank.

Eight bit lanes at 2-block pitch, each a bare (lever, elevated destination)
pair; the NEW bridge router (`Routing.route_net`) independently emits each
lane's run-in, torch-ladder via and landing dust.  Identical lane geometry
gives identical ladder depth, so the bus is skew-matched by construction --
and this script MEASURES it: each bit's lever is flipped and the simulation
single-stepped until the top-of-riser wire first powers, giving a per-bit
arrival tick that must match across all eight bits.

Verification (all in-sim, before saving):
  * per-bit conduction: top wire power 0 -> >0 when that bit's lever flips;
  * per-bit isolation: while bit i is up, both z-neighbour bits read 0;
  * skew: identical first-arrival tick for all 8 bits;
  * Routing.drc over the finished bank: 0 violations;
  * all levers returned to rest, settled, and the state BAKED via bake_to.
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, ROOT)
sys.path.insert(0, os.path.join(ROOT, "demos"))

import _common  # noqa: E402
from _common import n, rs  # noqa: E402

BITS = 8
PITCH = 2
RISE_PLATFORM_Y = 6      # landing platform height; dst dust sits on top at y=7

b = rs.Build("bus_riser8")
levers, dsts = [], []
for i in range(BITS):
    z = i * PITCH
    b.stone(0, 0, z)
    b.force(0, 1, z, rs.LEVER_OFF)
    levers.append((0, 1, z))
    for x in range(6, 9):
        b.stone(x, RISE_PLATFORM_Y, z)
    dsts.append((7, RISE_PLATFORM_Y + 1, z))

paths = []
for i in range(BITS):
    z = i * PITCH
    p = json.loads(n.Routing.route_net(b.s, 1, 1, z, *dsts[i], "bit%d" % i))
    paths.append(p)
lens = sorted(set(len(p) for p in paths))
print("routed %d lanes; path cells per lane: %s (identical geometry: %s)"
      % (BITS, lens, len(lens) == 1))
assert len(lens) == 1, lens

violations = json.loads(n.Routing.drc(b.s, False))
print("DRC over the riser bank: %d violations" % len(violations))
assert not violations, violations[:3]

sim, (ox, oy, oz) = _common.simulate(b.s)


def s(p):
    return (p[0] - ox, p[1] - oy, p[2] - oz)


def pw(p):
    return _common.power_at(sim, *s(p))


arrivals = []
for i in range(BITS):
    assert pw(dsts[i]) == 0, (i, pw(dsts[i]))
    sim.use_block(*s(levers[i]))
    t = 0
    while pw(dsts[i]) <= 0 and t < 100:
        sim.step()
        t += 1
    top = pw(dsts[i])
    nz = [pw(dsts[j]) for j in (i - 1, i + 1) if 0 <= j < BITS]
    print("bit %d: top power %d after %d ticks; neighbours %s" % (i, top, t, nz))
    assert top > 0 and t < 100, (i, top, t)
    assert all(x == 0 for x in nz), (i, nz)
    arrivals.append(t)
    sim.use_block(*s(levers[i]))     # back to rest
    sim.run_until_quiescent(400)

print("arrival ticks per bit: %s" % arrivals)
assert len(set(arrivals)) == 1, "bus is skewed: %s" % arrivals

rest = [pw(d) for d in dsts]
assert all(x == 0 for x in rest), rest
tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_riser_tight.schem")
b.s.save_to_file(tmp)
tight = n.Schematic.open(tmp)
changed = sim.bake_to(tight)
out = os.path.join(HERE, "bus_riser8.schem")
tight.save_to_file(out)
print("baked %d settled states; saved %s" % (changed, out))
print("showcase_bus_riser PASS: 8/8 bits conduct, isolated, skew-matched at %d ticks"
      % arrivals[0])
