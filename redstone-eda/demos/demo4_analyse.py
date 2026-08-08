"""demo4: static analysis -- DRC + STA over the real 4-bit adder, and the
repeater-cycle check catching a deliberately built ring latch.

DRC audits a schematic with no extra information: support, unattached
torches, decay, and directed diode cycles.  STA takes a gate netlist and
reports arrival times + the critical path.  The ring latch is the exact bug
class the cycle check was built for: an accidental unclocked storage element.
"""
import json
import os
from collections import Counter

import _common
from _common import n, rs

RCA = os.path.join(os.path.dirname(_common.HERE), "rca4_cells.schem")
print("== demo4: DRC + STA over the 4-bit adder ==")
s = n.Schematic.open(RCA)

violations = json.loads(n.Routing.drc(s, False))
kinds = Counter(v["kind"] for v in violations)
print(f"DRC over {os.path.basename(RCA)}: {len(violations)} violations {dict(kinds)}")
for v in violations[:3]:
    print(f"   e.g. {v}")

# STA: the ripple-carry structure as a gate netlist.  Each bit computes
# sum_k and carry_{k+1} from (a_k, b_k, c_k); comparators cost 2 rt each.
gates = []
for k in range(4):
    ins = [f"a{k}", f"b{k}", "cin" if k == 0 else f"c{k}"]
    gates.append({"out": f"s{k}", "ins": ins, "delay_rt": 2})
    gates.append({"out": f"c{k + 1}", "ins": ins, "delay_rt": 2})
netlist = {"inputs": ["cin"] + [f"{s_}{k}" for k in range(4) for s_ in "ab"],
           "gates": gates}
report = json.loads(n.Routing.sta(s, json.dumps(netlist)))
arr = report["arrival_rt"]
print(f"STA: sum arrivals {[arr[f's{k}'] for k in range(4)]} rt, "
      f"cout arrives at {arr['c4']} rt")
print(f"     critical path: {' -> '.join(report['critical'])}")
assert arr["c4"] == 8 and report["critical"][-1] in ("c4", "s3")

# The ring latch: a dust loop with one repeater bridging a cut -- a diode
# feeding its own input net.  Perfectly legal to build, silently stateful.
print("-- now a deliberate ring latch:")
latch = rs.Build("ring_latch")
for x, z in [(1, 1), (2, 1), (3, 1), (3, 2), (3, 3), (2, 3), (1, 3)]:
    latch.dust(x, 1, z)
latch.stone(1, 0, 2)
latch.put(1, 1, 2, "minecraft:repeater[facing=south,delay=1,locked=false,powered=false]")
found = json.loads(n.Routing.drc(latch.s, False))
cycles = [v for v in found if v["kind"] == "repeater_cycle"]
print(f"DRC on the latch: {len(found)} violations, "
      f"{len(cycles)} repeater_cycle: diodes {cycles[0]['diodes'] if cycles else '-'}")
assert cycles, found
print("demo4 PASS: timing bounded, and the latch did not slip past DRC")
