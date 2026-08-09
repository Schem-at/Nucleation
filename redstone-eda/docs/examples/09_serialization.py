"""Three serialization tiers: .nucm keeps the project, .litematic the layers,
.schem the artifact."""
import os

import nucleation as n
from _common import OUT, lamp_bank, lever_bank

s = n.Schematic.create("tiers")
a_in, a_out = lever_bank(s, 0, 8, 1, 0), lamp_bank(s, 16, 8)
d = n.Design.for_schematic("tiers", s)
d.declare_input("a_in", anchor=a_in, step=(0, 2, 0), width=8, ty="uint")
d.declare_output("a_out", anchor=a_out, step=(0, 2, 0), width=8, ty="uint")
d.route_bus("bus_a", driver="a_in", sinks=["a_out"])

for suffix in (".nucm", ".litematic", ".schem"):        # save() dispatches on suffix
    d.save(os.path.join(OUT, "tiers" + suffix))

back = n.Design.load_nucm(os.path.join(OUT, "tiers.nucm"))   # project tier reopens
assert back.bus_state("bus_a") == "routed", back.bus_state("bus_a")
assert back.flatten().block_count() == d.flatten().block_count()
print("nucm round-trip: bus_a %s, %d blocks  OK 09_serialization"
      % (back.bus_state("bus_a"), back.flatten().block_count()))
