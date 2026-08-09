"""demo3: compose a 4-bit adder from the verified Python cell library.

cells.build_half_adder / build_full_adder produce truth-tabled comparator
cells with port contracts on their faces; rca_cells.build_rca stamps four FA
cells at PITCH so the carry chain connects by pure abutment -- zero routing.
We wire levers to the inputs, simulate, and spot-check real sums.
"""
import os

import _common
from _common import rs
import cells
import rca_cells

N = 4
print("== demo3: compose a 4-bit ripple-carry adder from cells ==")
ha = cells.build_half_adder()
fa = cells.build_full_adder(ha)
b, labels, ports, aliases = rca_cells.build_rca(N, fa)
print(f"stamped {N} FA cells at pitch {cells.PITCH}: {len(b.cells)} blocks")

# Levers: two per bit (a, b) feeding the west ports, one for cin.
lever = {}
for k in range(N):
    for sig in ("a", "b"):
        x, y, z = ports[k][sig]
        b.stone(x - 2, y - 1, z)
        b.force(x - 2, y, z, rs.LEVER_OFF)
        b.stone(x - 1, y - 1, z)
        b.put(x - 1, y, z, rs.DUST)
        lever[f"{sig}{k}"] = (x - 2, y, z)
cx, cy, cz = ports[0]["cin"]
b.stone(cx, cy - 1, cz - 2)
b.force(cx, cy, cz - 2, rs.LEVER_OFF)
b.stone(cx, cy - 1, cz - 1)
b.put(cx, cy, cz - 1, rs.DUST)
lever["cin"] = (cx, cy, cz - 2)

sim = b.sim()
names = ["cin"] + [f"a{k}" for k in range(N)] + [f"b{k}" for k in range(N)]
lv = rs.Levers(sim, [lever[s] for s in names])

CASES = [(0, 0, 0), (1, 2, 0), (3, 5, 1), (9, 6, 0), (15, 15, 1), (7, 8, 1)]
bad = 0
for A, B, c in CASES:
    lv.set([c] + [(A >> k) & 1 for k in range(N)]
           + [(B >> k) & 1 for k in range(N)])
    got = sum(int(sim.on(*ports[k]["sum"])) << k for k in range(N))
    got += int(sim.on(*ports[N - 1]["cout"])) << N
    ok = got == A + B + c
    bad += not ok
    print(f"  {A:2d} + {B:2d} + {c} = {A + B + c:2d}   sim says {got:2d}   "
          f"{'ok' if ok else 'WRONG'}")
assert not bad, f"{bad} wrong sums"

out = os.path.join(_common.HERE, "rca4_composed.schem")
b.s.save_to_file(out)
print(f"saved {out}")
print(f"demo3 PASS: {len(CASES)}/{len(CASES)} spot-checked sums correct")
