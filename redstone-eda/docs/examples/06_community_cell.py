"""A community build with an embedded contract: IO autodetected, then poked."""
import json
import os

import nucleation as n
from _common import ROOT

path = os.path.join(ROOT, "computational_schematics", "enhanced",
                    "ADD007_8bit_cca_matt_enhanced.schem")
s = n.Schematic.open(path)

found = json.loads(s.resolve_cell_contract_json())      # from .schem metadata
assert found["warnings"] == [], found["warnings"]
io = found["contract"]["io"]
print("%s: inputs %s -> outputs %s"
      % (found["contract"]["name"], list(io["inputs"]), list(io["outputs"])))

ex = n.design.Executor.for_schematic(s)                 # somebody else's redstone,
ex["a"], ex["b"] = 37, 5                                # driven by name
ex.settle()
assert ex["sum"] == 42, ex["sum"]
print("37 + 5 = %d  OK 06_community_cell" % ex["sum"])
