"""Drag-and-drop composition: place two cells, route a bus, check, bake, save."""
import json
import os

import nucleation as n
from _common import EDA, OUT, lamp_bank, lever_bank, synth

blif = open(synth(os.path.join(EDA, "hdl", "seg7.v"), "seg7")).read()
cell = n.Hdl.compile_blif(blif, "seg7", True)          # a cell needs a contract to be
cell.set_cell_contract_json(n.Hdl.compile_blif_contract(blif, "seg7"))   # placeable
keepout = json.loads(cell.cell_contract_json())["physical"]["keepouts"][0]
pitch = keepout["max"][2] - keepout["min"][2] + 8      # space instances by the keepout

s = n.Schematic.create("compose")
a_in, a_out = lever_bank(s, -40, 8, 1, 0), lamp_bank(s, -20, 8)
d = n.Design.for_schematic("compose", s)
d.add_cell("seg7", cell)
d.place("u1", "seg7", at=(0, 0, 40))
d.place("u2", "seg7", at=(0, 0, 40 + pitch))

d.declare_input("a_in", anchor=a_in, step=(0, 2, 0), width=8, ty="uint")
d.declare_output("a_out", anchor=a_out, step=(0, 2, 0), width=8, ty="uint")
bus = d.route_bus("bus_a", driver="a_in", sinks=["a_out"])

d.check(strict=True)                                   # raises DesignCheckError if dirty
d.bake().save(os.path.join(OUT, "composed.schem"))     # settled in mc-tick, then written
print("2 instances + %s, %d blocks  OK 03_compose" % (bus, d.flatten().block_count()))
