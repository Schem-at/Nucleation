"""Design API acceptance, sketch (1) of DESIGN_SPEC.md, on the idiomatic
veneer: endpoint hardware with raw set_block, typed keyword ports, two
crossing 8-bit buses (crossing IMPLICIT, per-bus styles), strict check,
bake, typed walking-ones 8+8 + isolation through the embedded contract,
and the flat .schem artifact saved to showcase/.
"""
import json
import os

import nucleation as n

STONE = "minecraft:stone"
DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]"
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"
N, STEP = 8, (0, 2, 0)


def lever_bank(s, x, z, dx, dz):
    for i in range(N):
        y = 2 + 2 * i
        s.set_block_from_string(x, y - 1, z, STONE)
        s.set_block_from_string(x, y, z, LEVER)
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE)
        s.set_block_from_string(x + dx, y, z + dz, DUST)
    return (x + dx, 2, z + dz)


def lamp_bank(s, x, z):
    for i in range(N):
        y = 2 + 2 * i
        s.set_block_from_string(x, y - 1, z, LAMP)
        s.set_block_from_string(x, y, z, DUST)
    return (x, 2, z)


def main():
    # Loose layer: bus A runs +X at z=8, bus B runs +Z at x=8 — they MUST cross.
    s = n.Schematic.create("crossing")
    a_in, a_out = lever_bank(s, 0, 8, 1, 0), lamp_bank(s, 16, 8)
    b_in, b_out = lever_bank(s, 8, 0, 0, 1), lamp_bank(s, 8, 16)

    d = n.Design.for_schematic("crossing", s)
    d.declare_input("a_in", anchor=a_in, step=STEP, width=N, ty="uint")
    d.declare_output("a_out", anchor=a_out, step=STEP, width=N, ty="uint")
    d.declare_input("b_in", anchor=b_in, step=STEP, width=N, ty="uint")
    d.declare_output("b_out", anchor=b_out, step=STEP, width=N, ty="uint")

    bus_a = d.route_bus("bus_a", driver="a_in", sinks=["a_out"],
                        style=n.Style(bus_block="minecraft:lime_concrete"))
    bus_b = d.route_bus("bus_b", driver="b_in", sinks=["b_out"],
                        style=n.Style(bus_block="minecraft:cyan_concrete",
                                      transparent_block="minecraft:cyan_stained_glass"))
    assert bus_a.state == "routed" and bus_b.state == "routed", (bus_a, bus_b)

    d.check(strict=True)                 # DRC + LVS over the flattened artifact
    baked = d.bake(4000)                 # settled in mc-tick, contract stamped Baked
    ex = baked.executor()                # typed I/O over the EMBEDDED contract

    patterns = ([(1 << i, 0) for i in range(N)] + [(0, 1 << i) for i in range(N)]
                + [(0xFF, 0xFF), (0xAA, 0x55), (0x55, 0xAA)])
    good = 0
    for pa, pb in patterns:
        ex["a_in"], ex["b_in"] = pa, pb
        ex.settle(400)
        a, b = ex["a_out"], ex["b_out"]
        good += (ok := a == pa and b == pb)
        print("%s %02x/%02x  a_out=%02x b_out=%02x" % ("PASS" if ok else "FAIL", pa, pb, a, b))
    print("design_demo2: %d/%d patterns" % (good, len(patterns)))
    if good != len(patterns):
        return 1

    ex["a_in"], ex["b_in"] = 0, 0        # at rest, then save the flat artifact
    ex.settle(400)
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "..", "showcase", "bus_cross8_design.schem")
    baked.save(out)
    back = json.loads(n.Schematic.open(out).resolve_cell_contract_json())
    assert back["contract"]["name"] == "crossing" and back["warnings"] == []
    print("saved %s (contract autodetected on reopen)" % os.path.normpath(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
