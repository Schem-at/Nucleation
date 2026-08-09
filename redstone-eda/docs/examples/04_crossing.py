"""Two perpendicular 8-bit buses: the 90-degree crossing is IMPLICIT."""
import nucleation as n
from _common import lamp_bank, lever_bank

s = n.Schematic.create("crossing")
a_in, a_out = lever_bank(s, 0, 8, 1, 0), lamp_bank(s, 16, 8)   # A runs +X at z=8
b_in, b_out = lever_bank(s, 8, 0, 0, 1), lamp_bank(s, 8, 16)   # B runs +Z at x=8

d = n.Design.for_schematic("crossing", s)
for name, anchor in [("a_in", a_in), ("b_in", b_in)]:
    d.declare_input(name, anchor=anchor, step=(0, 2, 0), width=8, ty="uint")
for name, anchor in [("a_out", a_out), ("b_out", b_out)]:
    d.declare_output(name, anchor=anchor, step=(0, 2, 0), width=8, ty="uint")

d.route_bus("bus_a", driver="a_in", sinks=["a_out"],
            style=n.Style(bus_block="minecraft:lime_concrete"))
d.route_bus("bus_b", driver="b_in", sinks=["b_out"],       # the router picks a crossing
            style=n.Style(bus_block="minecraft:cyan_concrete",
                          transparent_block="minecraft:cyan_stained_glass"))
d.check(strict=True)
ex = d.bake().executor()
ex["a_in"], ex["b_in"] = 0x55, 0xAA                        # both buses at once
ex.settle(400)
assert (ex["a_out"], ex["b_out"]) == (0x55, 0xAA), (ex["a_out"], ex["b_out"])
print("a_out=0x%02X b_out=0x%02X isolated  OK 04_crossing" % (ex["a_out"], ex["b_out"]))
