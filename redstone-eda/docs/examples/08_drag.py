"""Dragging: a gate move reroutes only 2 segments; a blocked bus FAILS visibly."""
import json

import nucleation as n
from _common import STONE, lamp_bank, lever_bank

s = n.Schematic.create("drag")
a_in, a_out = lever_bank(s, 0, 8, 1, 0), lamp_bank(s, 24, 8)
d = n.Design.for_schematic("drag", s)
d.declare_input("a_in", anchor=a_in, step=(0, 2, 0), width=8, ty="uint")
d.declare_output("a_out", anchor=a_out, step=(0, 2, 0), width=8, ty="uint")
bus = d.route_bus("bus_a", driver="a_in", sinks=["a_out"],
                  gates=[n.Gate(anchor=(8, 2, 8), step=(0, 2, 0)),
                         n.Gate(anchor=(16, 2, 8), step=(0, 2, 0))])

moved = bus.move_gate(0, (8, 2, 12))                 # drag g0 four cells south
assert moved["state"] == "routed" and moved["rerouted_segments"] == 2, moved
print("gate drag: exactly %d segments rerouted" % moved["rerouted_segments"])

cube = n.Schematic.create("cube")                    # an obstacle with no ports
for x in range(3):
    for y in range(3):
        for z in range(3):
            cube.set_block_from_string(x, y, z, STONE)
cube.set_cell_contract_json(json.dumps(
    {"name": "cube", "io": {"inputs": {}, "outputs": {}, "buses": {}}}))
d.add_cell("cube", cube)
d.place("c0", "cube", at=(4, 0, 20))                 # parked clear of the bus

r = d.move_instance("c0", at=(4, 0, 12))             # drag it ONTO the corridor
assert "bus_a" in r["failed"] and bus.state.startswith("failed"), (r, bus.state)
print("blocked -> %s (the move always lands; the bus goes red, never half-routed)"
      % bus.state[:38])

r = d.move_instance("c0", at=(4, 0, 20))             # drag away: it re-attempts
assert r["rerouted"] == ["bus_a"] and d.check().clean, (r, bus.state)
print("recovered -> %s  OK 08_drag" % bus.state)
