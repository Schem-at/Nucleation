"""Design API acceptance, sketch (1) of DESIGN_SPEC.md, in PURE binding
calls: endpoint hardware with raw set_block, typed ports declared over it,
two crossing 8-bit buses routed with the crossing IMPLICIT (dip-under tile,
per-bus styles), flatten / check / bake, typed walking-ones 8+8 + isolation
through the EMBEDDED contract, and the flat .schem artifact saved to
showcase/.

Everything here goes through the generated `nucleation` wheel — no helper
modules, no raw simulation coordinates after the ports are declared.
"""
import json
import os

import nucleation as n

STONE = "minecraft:stone"
DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]"
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"

N, Y0, STEP = 8, 2, 2


def lever_bank(s, x, z, dx, dz):
    """8 levers at 2y pitch, each with its connection dust one step in."""
    for i in range(N):
        y = Y0 + STEP * i
        s.set_block_from_string(x, y - 1, z, STONE)
        s.set_block_from_string(x, y, z, LEVER)
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE)
        s.set_block_from_string(x + dx, y, z + dz, DUST)
    return (x + dx, Y0, z + dz)


def lamp_bank(s, x, z):
    """8 lamps at 2y pitch, each lamp supporting its own connection dust."""
    for i in range(N):
        y = Y0 + STEP * i
        s.set_block_from_string(x, y - 1, z, LAMP)
        s.set_block_from_string(x, y, z, DUST)
    return (x, Y0, z)


def main():
    # Loose block layer: bus A runs +X at z=8, bus B runs +Z at x=8 -- the
    # two buses MUST cross.
    s = n.Schematic.create("crossing")
    a_in = lever_bank(s, 0, 8, 1, 0)
    a_out = lamp_bank(s, 16, 8)
    b_in = lever_bank(s, 8, 0, 0, 1)
    b_out = lamp_bank(s, 8, 16)

    d = n.Design.for_schematic("crossing", s)

    # Typed ports: anchor + step + width + type; hardware capabilities are
    # scanned (lever => drivable, lamp => readable) and validated loudly.
    d.declare_input("a_in", *a_in, 0, STEP, 0, N, "uint")
    d.declare_output("a_out", *a_out, 0, STEP, 0, N, "uint")
    d.declare_input("b_in", *b_in, 0, STEP, 0, N, "uint")
    d.declare_output("b_out", *b_out, 0, STEP, 0, N, "uint")

    # Two buses, per-bus styles; the crossing is IMPLICIT.
    st_a = d.route_bus("bus_a", "a_in", '["a_out"]', "[]",
                       '{"bus_block":"minecraft:lime_concrete"}')
    st_b = d.route_bus("bus_b", "b_in", '["b_out"]', "[]",
                       '{"bus_block":"minecraft:cyan_concrete",'
                       '"transparent_block":"minecraft:cyan_stained_glass"}')
    print("bus_a:", st_a, "| bus_b:", st_b)
    assert st_a == "routed" and st_b == "routed", (st_a, st_b)
    assert d.bus_state("bus_a") == "routed"

    # check(): DRC + LVS over the flattened artifact.
    report = json.loads(d.check())
    assert report["clean"], json.dumps(report, indent=2)
    print("check: clean (drc=%d, lvs matched=%d)"
          % (len(report["drc"]), len(report["lvs"]["matched"])))

    # bake(): settled in mc-tick, states written back, InitialState::Baked
    # stamped into the embedded contract.
    baked = d.bake(4000)
    contract = json.loads(baked.cell_contract_json())
    assert contract["name"] == "crossing"
    assert contract["physical"]["initial_state"]["kind"] == "baked", contract["physical"]
    regions = json.loads(baked.region_names_json())
    assert "bus:bus_a" in regions and "bus:bus_b" in regions, regions
    print("baked; regions:", regions)

    # Typed walking-ones through the EMBEDDED contract -- port names and
    # word values only, no coordinates.
    cell = n.CellExecutor.for_schematic(baked)
    cell.settle(4000)

    def word(v):
        return n.Value.from_u32(v)

    good = 0
    for i in range(N):
        cell.set_input("a_in", word(1 << i))
        cell.set_input("b_in", word(0))
        cell.settle(400)
        a, b = cell.read_output("a_out").as_u32(), cell.read_output("b_out").as_u32()
        ok = a == 1 << i and b == 0
        good += ok
        print("%s walkA-%d  a_out=%02x b_out=%02x" % ("PASS" if ok else "FAIL", i, a, b))
    for i in range(N):
        cell.set_input("a_in", word(0))
        cell.set_input("b_in", word(1 << i))
        cell.settle(400)
        a, b = cell.read_output("a_out").as_u32(), cell.read_output("b_out").as_u32()
        ok = b == 1 << i and a == 0
        good += ok
        print("%s walkB-%d  a_out=%02x b_out=%02x" % ("PASS" if ok else "FAIL", i, a, b))
    for pa, pb in [(0xFF, 0xFF), (0xAA, 0x55), (0x55, 0xAA)]:
        cell.set_input("a_in", word(pa))
        cell.set_input("b_in", word(pb))
        cell.settle(400)
        a, b = cell.read_output("a_out").as_u32(), cell.read_output("b_out").as_u32()
        ok = a == pa and b == pb
        good += ok
        print("%s joint %02x/%02x  a_out=%02x b_out=%02x"
              % ("PASS" if ok else "FAIL", pa, pb, a, b))
    total = 2 * N + 3
    print("design_demo2: %d/%d patterns" % (good, total))
    if good != total:
        return 1

    # At-rest inputs, then save the flat artifact (embedded contract rides).
    cell.set_input("a_in", word(0))
    cell.set_input("b_in", word(0))
    cell.settle(400)
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "..", "showcase", "bus_cross8_design.schem")
    baked.save_to_file(out)
    reopened = n.Schematic.open(out)
    back = json.loads(reopened.resolve_cell_contract_json())
    assert back["contract"]["name"] == "crossing" and back["warnings"] == []
    print("saved %s (contract autodetected on reopen)" % os.path.normpath(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
