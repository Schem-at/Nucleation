"""demo2: look INSIDE a running simulation.

Build a lever -> dust run -> lamp, flip the lever, then use the three new
introspection APIs: conduction_trace explains WHO powers a cell and why (a
recursive power-source tree), read_probes reads many cells in one call, and
bake_to writes the settled state back into a schematic for saving.
"""
import json
import os

import _common
from _common import n, rs

b = rs.Build("demo2_introspect")
b.stone(0, 0, 0)
b.force(0, 1, 0, rs.LEVER_OFF)
for x in range(1, 7):
    b.dust(x, 1, 0)          # dust on stone, x = 1..6
b.stone(7, 0, 0)
b.put(7, 1, 0, rs.LAMP)

print("== demo2: conduction_trace / read_probes / bake_to ==")
sim, (ox, oy, oz) = _common.simulate(b.s)
sim.use_block(0 - ox, 1 - oy, 0 - oz)
sim.run_until_quiescent(400)


def show(node, depth=0):
    pad = "  " * depth
    pos, kind, power = node.get("pos"), node.get("kind", "leaf"), node.get("power")
    print(f"{pad}{kind} at {pos} (power {power})")
    for inp in node.get("inputs", []):
        src = inp.get("source")
        tag = " [cycle]" if inp.get("cycle") else ""
        print(f"{pad}  <- {inp['mechanism']} carrying {inp['power']}{tag}")
        if src and not inp.get("cycle"):
            show(src, depth + 2)


# Why is the LAST dust cell powered?  Walk the tree back to the lever.
trace = json.loads(sim.conduction_trace(6 - ox, 1 - oy, 0 - oz))
print("-- conduction trace of the dust at (6,1,0):")
show(trace)

# Batch-read the whole dust run in one call.
probes = [[x - ox, 1 - oy, 0 - oz] for x in range(1, 7)]
states = json.loads(sim.read_probes(json.dumps(probes)))
powers = [s.split("power=")[1].split(",")[0].rstrip("]") for s in states]
print(f"-- read_probes over the run: powers {powers}")
assert all(int(p) > 0 for p in powers), powers

lamp = sim.get_block(7 - ox, 1 - oy, 0 - oz)
print(f"-- the lamp reads {lamp}")
assert "lit=true" in lamp

# Bake the LIVE state back into a schematic and save it.
baked = n.Schematic.create("demo2_baked")
for (x, y, z), blk in b.cells.items():
    baked.set_block_from_string(x - ox, y - oy, z - oz, blk)
changed = sim.bake_to(baked)
out = os.path.join(_common.HERE, "introspected_baked.schem")
baked.save_to_file(out)
print(f"-- bake_to rewrote {changed} cells (dust powers, lever, lit lamp); saved {out}")
assert changed > 0
print("demo2 PASS: the sim explained itself and baked its state")
