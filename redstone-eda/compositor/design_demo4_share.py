"""Design-document serialization demo (DESIGN_SPEC.md section 8), in PURE
binding calls: build the bus-cross design of design_demo2, save the FULL
document as `.nucm` (project tier: cells, transforms, ports with scanned
hardware, bus layers incl. fragments and states), reload it, prove the
reloaded document is live (bake + 4/4 typed spot checks through the
embedded contract), export the LAYERED `.litematic` (interchange tier:
one named region per layer + a `NucleationDesign` manifest), reimport it
as a design, and reopen the very same file as a plain multi-region
litematic.
"""
import json
import os

import nucleation as n

STONE = "minecraft:stone"
DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]"
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"

N, Y0, STEP = 8, 2, 2
HERE = os.path.dirname(os.path.abspath(__file__))


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


def build_design():
    s = n.Schematic.create("crossing")
    a_in = lever_bank(s, 0, 8, 1, 0)
    a_out = lamp_bank(s, 16, 8)
    b_in = lever_bank(s, 8, 0, 0, 1)
    b_out = lamp_bank(s, 8, 16)
    d = n.Design.for_schematic("crossing", s)
    d.declare_input("a_in", *a_in, 0, STEP, 0, N, "uint")
    d.declare_output("a_out", *a_out, 0, STEP, 0, N, "uint")
    d.declare_input("b_in", *b_in, 0, STEP, 0, N, "uint")
    d.declare_output("b_out", *b_out, 0, STEP, 0, N, "uint")
    st_a = d.route_bus("bus_a", "a_in", '["a_out"]', "[]",
                       '{"bus_block":"minecraft:lime_concrete"}')
    st_b = d.route_bus("bus_b", "b_in", '["b_out"]', "[]",
                       '{"bus_block":"minecraft:cyan_concrete",'
                       '"transparent_block":"minecraft:cyan_stained_glass"}')
    assert st_a == "routed" and st_b == "routed", (st_a, st_b)
    return d


def spot_check(d, label):
    """4 typed spot checks through the embedded contract of a bake."""
    baked = d.bake(4000)
    cell = n.CellExecutor.for_schematic(baked)
    cell.settle(4000)

    def word(v):
        return n.Value.from_u32(v)

    patterns = [(0x01, 0x00), (0x80, 0x01), (0xAA, 0x55), (0xFF, 0xFF)]
    good = 0
    for pa, pb in patterns:
        cell.set_input("a_in", word(pa))
        cell.set_input("b_in", word(pb))
        cell.settle(400)
        a = cell.read_output("a_out").as_u32()
        b = cell.read_output("b_out").as_u32()
        ok = a == pa and b == pb
        good += ok
        print("%s %s %02x/%02x -> a_out=%02x b_out=%02x"
              % ("PASS" if ok else "FAIL", label, pa, pb, a, b))
    print("%s: %d/%d spot checks" % (label, good, len(patterns)))
    return good == len(patterns)


def main():
    d = build_design()

    # ---- .nucm project tier: save the FULL document, reload it. --------
    nucm_path = os.path.join(HERE, "..", "showcase", "bus_cross8_design.nucm")
    d.save_nucm(nucm_path)
    print("saved %s (%d bytes)"
          % (os.path.normpath(nucm_path), os.path.getsize(nucm_path)))
    d2 = n.Design.load_nucm(nucm_path)

    # The reloaded document kept its bus layers and their states…
    assert d2.bus_state("bus_a") == "routed", d2.bus_state("bus_a")
    assert d2.bus_state("bus_b") == "routed", d2.bus_state("bus_b")
    report = json.loads(d2.check())
    assert report["clean"], json.dumps(report, indent=2)
    print("reloaded .nucm: buses routed, check clean")

    # …and is LIVE: bake + 4/4 typed spot checks through the contract.
    if not spot_check(d2, "nucm-reload"):
        return 1

    # ---- .litematic interchange tier: layered export + manifest. ------
    lit_path = os.path.join(HERE, "..", "showcase", "bus_cross8_design.litematic")
    d2.export_litematic(lit_path)
    print("exported %s (%d bytes)"
          % (os.path.normpath(lit_path), os.path.getsize(lit_path)))

    # Reimport as a design (references degrade to embedded copies).
    d3 = n.Design.import_litematic(lit_path)
    assert d3.bus_state("bus_a") == "routed", d3.bus_state("bus_a")
    assert d3.bus_state("bus_b") == "routed", d3.bus_state("bus_b")
    report = json.loads(d3.check())
    assert report["clean"], json.dumps(report, indent=2)
    print("reimported .litematic as a design: buses routed, check clean")

    # The very same file also opens as a PLAIN multi-region litematic.
    plain = n.Schematic.open(lit_path)
    regions = json.loads(plain.region_names_json())
    print("plain litematic regions:")
    for r in sorted(regions):
        print("  -", r)
    assert "bus:bus_a" in regions and "bus:bus_b" in regions, regions
    print("design_demo4_share: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
