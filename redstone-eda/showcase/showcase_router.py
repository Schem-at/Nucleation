"""router_gallery.schem: four routing feats of the bridge router, side by side.

Every feat is authored as bare endpoints (lever + destination cell) and the
NEW bridge API `Routing.route_net` does the rest -- emitting dust, supports
(magenta concrete), stairs, torch-ladder vias and repeaters directly into the
schematic.  Feats, west to east along z bands:

  A (z 0..8)   obstacle maze: two offset walls force an S-detour / dive-under.
  B (z 14..18) vertical via: a torch-ladder climb to a platform 6 blocks up.
  C (z 24..31) multi-net braid: three nets squeeze through one 2x5 wall
               window with electrical clearances held (whole-build DRC = 0).
  D (z 38..44) shared-trunk fork: one lever, one trunk, a branch forked off
               an existing mid-trunk cell -- both ends conduct.

Verification (all in-sim, before saving):
  * each net's destination reads power 0 before and >0 after its lever flips;
  * braid isolation: flipping ONLY the middle net's lever powers ONLY its
    destination (clearance is electrical, not just geometric);
  * Routing.drc over the finished build reports 0 violations;
  * the build is returned to rest (all levers off, quiescent) and the settled
    state is BAKED into the saved schematic via TickSimulation.bake_to.
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

b = rs.Build("router_gallery")
nets = {}    # name -> (lever, dst) in build coords
extra_dst = {}  # name -> additional destinations to check


def lever_at(x, y, z):
    b.stone(x, y - 1, z)
    b.force(x, y, z, rs.LEVER_OFF)
    return (x, y, z)


# --- A: obstacle maze --------------------------------------------------------
A_LEV = lever_at(0, 1, 4)
for z in range(0, 7):            # wall 1: x=4, z 0..6, y 0..3
    for y in range(0, 4):
        b.stone(4, y, z)
for z in range(2, 9):            # wall 2: x=8, z 2..8, y 0..3
    for y in range(0, 4):
        b.stone(8, y, z)
A_DST = (12, 1, 4)
path_a = json.loads(n.Routing.route_net(b.s, 1, 1, 4, *A_DST, "maze"))
detour = [p for p in path_a if p[1] < 1 or not (0 <= p[2] <= 8) or len(path_a) > 12]
print("A maze: %d path cells (straight line would be 12); below-grade cells: %d"
      % (len(path_a), sum(1 for p in path_a if p[1] < 1)))
assert len(path_a) > 12 or any(p[1] < 1 for p in path_a), "no detour?!"
nets["maze"] = (A_LEV, A_DST)

# --- B: torch-ladder climb ---------------------------------------------------
B_LEV = lever_at(0, 1, 16)
for x in range(6, 9):            # platform at y=6
    b.stone(x, 6, 16)
B_DST = (7, 7, 16)
path_b = json.loads(n.Routing.route_net(b.s, 1, 1, 16, *B_DST, "climb"))
ys = sorted(set(p[1] for p in path_b))
print("B climb: %d path cells, y levels %s" % (len(path_b), ys))
assert max(ys) >= 7, ys
nets["climb"] = (B_LEV, B_DST)

# --- C: multi-net braid through a slotted wall -------------------------------
# Three nets at 2-block pitch each thread their own 1-wide slot in a shared
# wall.  route_net is stateless per call (the negotiated multi-net Workspace
# is a native-crate API), so the 2-block clearance discipline is what keeps
# the nets electrically separate -- and the isolation sim below PROVES it.
CZ = 25                          # lanes at z 25, 27, 29
c_levs = [lever_at(0, 1, CZ + 2 * i) for i in range(3)]
SLOTS = {CZ, CZ + 2, CZ + 4}
for z in range(CZ - 2, CZ + 7):  # wall x=5; slots y 1..2 at each lane z
    for y in range(0, 5):
        if not (z in SLOTS and 1 <= y <= 2):
            b.stone(5, y, z)
c_dsts = [(10, 1, CZ + 2 * i) for i in range(3)]
for i in range(3):
    p = json.loads(n.Routing.route_net(
        b.s, 1, 1, CZ + 2 * i, *c_dsts[i], "braid%d" % i))
    print("C braid%d: %d path cells through its slot" % (i, len(p)))
    nets["braid%d" % i] = (c_levs[i], c_dsts[i])

# --- D: shared-trunk fork ----------------------------------------------------
D_LEV = lever_at(0, 1, 40)
D_DST = (12, 1, 40)
trunk = json.loads(n.Routing.route_net(b.s, 1, 1, 40, *D_DST, "fork"))
mid = trunk[len(trunk) // 2]
D_BRANCH = (mid[0], 1, 44)
branch = json.loads(n.Routing.route_net(
    b.s, mid[0], mid[1], mid[2], *D_BRANCH, "fork"))
print("D fork: trunk %d cells, branch %d cells forked at %s" %
      (len(trunk), len(branch), mid))
nets["fork"] = (D_LEV, D_DST)
extra_dst["fork"] = [D_BRANCH]

# --- whole-build DRC ---------------------------------------------------------
violations = json.loads(n.Routing.drc(b.s, False))
print("DRC over the finished gallery: %d violations" % len(violations))
assert not violations, violations[:3]

# --- simulate and prove every net conducts -----------------------------------
sim, (ox, oy, oz) = _common.simulate(b.s)


def s(p):
    return (p[0] - ox, p[1] - oy, p[2] - oz)


def pw(p):
    return _common.power_at(sim, *s(p))


checked = 0
for name, (lev, dst) in nets.items():
    targets = [dst] + extra_dst.get(name, [])
    before = [pw(t) for t in targets]
    sim.use_block(*s(lev))
    sim.run_until_quiescent(400)
    after = [pw(t) for t in targets]
    print("net %-7s: dst power %s -> %s" % (name, before, after))
    assert all(x == 0 for x in before) and all(x > 0 for x in after), (name, before, after)
    checked += len(targets)
    sim.use_block(*s(lev))       # back to rest
    sim.run_until_quiescent(400)

# braid isolation: middle lever only
sim.use_block(*s(nets["braid1"][0]))
sim.run_until_quiescent(400)
iso = [pw(nets["braid%d" % i][1]) for i in range(3)]
print("braid isolation (only braid1 on): dst powers %s" % iso)
assert iso[0] == 0 and iso[1] > 0 and iso[2] == 0, iso
sim.use_block(*s(nets["braid1"][0]))
sim.run_until_quiescent(400)

# --- bake at rest and save ---------------------------------------------------
rest = [pw(d) for _, d in nets.values()]
assert all(x == 0 for x in rest), rest
tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_gallery_tight.schem")
b.s.save_to_file(tmp)
tight = n.Schematic.open(tmp)
changed = sim.bake_to(tight)
out = os.path.join(HERE, "router_gallery.schem")
tight.save_to_file(out)
print("baked %d settled states; saved %s" % (changed, out))
print("showcase_router PASS: %d destinations conducted, braid isolated, DRC clean"
      % checked)
