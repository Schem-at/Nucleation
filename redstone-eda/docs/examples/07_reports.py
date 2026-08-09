"""Reading the checker reports: DRC over a build, STA over a gate netlist."""
import json
import os
from collections import Counter

import nucleation as n
from _common import EDA

s = n.Schematic.open(os.path.join(EDA, "rca4_cells.schem"))

violations = json.loads(n.Routing.drc(s, False))        # support, torches, decay, cycles
print("DRC: %d violations %s"
      % (len(violations), dict(Counter(v["kind"] for v in violations))))

# STA wants the intended gate netlist; each comparator cell costs 2 rt.
gates = [g for k in range(4) for g in (
    {"out": "s%d" % k, "ins": ["a%d" % k, "b%d" % k, "cin" if k == 0 else "c%d" % k],
     "delay_rt": 2},
    {"out": "c%d" % (k + 1), "ins": ["a%d" % k, "b%d" % k, "cin" if k == 0 else "c%d" % k],
     "delay_rt": 2})]
netlist = {"inputs": ["cin"] + ["%s%d" % (p, k) for k in range(4) for p in "ab"],
           "gates": gates}
sta = json.loads(n.Routing.sta(s, json.dumps(netlist)))
print("STA: cout arrives at %d rt, critical path %s"
      % (sta["arrival_rt"]["c4"], " -> ".join(sta["critical"])))
assert sta["arrival_rt"]["c4"] == 8
print("OK 07_reports")
