"""Build, verify and save the two 90-degree crossing tiles side by side.

  * crossing_parity   -- buses offset 1 in y; the crossing bus's repeater
                         station entry block sits over the through dust and
                         doubles as the isolation.
  * crossing_dipunder -- one bus level; the crossing bus dips 1 y onto
                         glass supports and runs under the through bus's
                         station entry block (which needs no support).

Verification: for each tile, all 4 input combos through real levers; each
output must equal its own input (conduction + per-bit isolation).  Then all
levers off, settled, outputs asserted dark, and the schematic saved BAKED
AT REST as crossing_tiles.schem.
"""
import os

import nucleation as n

import rs
import materials as mt

b = rs.Build("crossing_tiles")
LV = []


def lever_col(x, y, z):
    for yy in range(0, y):
        b.stone(x, yy, z)
    b.force(x, y, z, rs.LEVER_OFF)
    LV.append((x, y, z))
    return len(LV) - 1


# tile 1: y-parity crossing at (4,1,5); tile 2: dip-under at (20,2,5)
par = mt.crossing_parity(b, 4, 1, 5)
dip = mt.crossing_dipunder(b, 20, 2, 5)

ia = lever_col(par["a_in"][0] - 1, par["a_in"][1], par["a_in"][2])
ib = lever_col(par["b_in"][0], par["b_in"][1], par["b_in"][2] - 1)
ic = lever_col(dip["c_in"][0], dip["c_in"][1], dip["c_in"][2] - 1)
id_ = lever_col(dip["d_in"][0] - 1, dip["d_in"][1], dip["d_in"][2])

sim = b.sim()
lv = rs.Levers(sim, LV)

bad = 0
for p in (0, 1):
    for q in (0, 1):
        bits = [0] * len(LV)
        bits[ia] = bits[ic] = p
        bits[ib] = bits[id_] = q
        lv.set(bits)
        got = (sim.power(*par["a_out"]) > 0, sim.power(*par["b_out"]) > 0,
               sim.power(*dip["c_out"]) > 0, sim.power(*dip["d_out"]) > 0)
        want = (p == 1, q == 1, p == 1, q == 1)
        ok = got == want
        print("%s combo (%d,%d): parity a=%d b=%d | dipunder c=%d d=%d"
              % (("PASS" if ok else "FAIL"), p, q,
                 got[0], got[1], got[2], got[3]))
        if not ok:
            print("      got %s want %s" % (got, want))
            bad += 1

# bake at rest
lv.set([0] * len(LV))
rest = [sim.power(*par["a_out"]), sim.power(*par["b_out"]),
        sim.power(*dip["c_out"]), sim.power(*dip["d_out"])]
assert all(x == 0 for x in rest), rest
tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_crossing_tight.schem")
b.s.save_to_file(tmp)
tight = n.Schematic.open(tmp)
changed = sim.sim.bake_to(tight)
out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                   "crossing_tiles.schem")
tight.save_to_file(out)
print("baked %d settled states; saved %s" % (changed, out))
print("crossing_tiles: %s (4 combos x 2 tiles, per-bit conduction + isolation)"
      % ("PASS" if bad == 0 else "FAIL"))
raise SystemExit(1 if bad else 0)
