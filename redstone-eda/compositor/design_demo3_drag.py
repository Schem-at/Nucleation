"""Design API acceptance, sketches (2) and (3) of DESIGN_SPEC.md, in PURE
binding calls — the Phase 2 Lane A drag surface:

  * gate drag: EXACTLY the two adjacent segments are ripped and rerouted,
    then typed walking-ones prove conduction through the embedded contract;
  * component drag THROUGH a bus: the move always succeeds (the document's
    truth), the bus fails VISIBLY (`failed: ...`), and dragging the
    component away re-attempts and reroutes it;
  * multi-sink trunk (1 driver -> 2 sinks) and the explicit wired-OR merge
    (`route_bus_or`), both verified in-sim;
  * per-bus STA/skew via `bus_skew` and `max_len_rt` net-class rule
    enforcement in `check()`.

Everything goes through the generated `nucleation` wheel — no helper
modules, no raw simulation coordinates after the ports are declared.
"""
import json

import nucleation as n

STONE = "minecraft:stone"
DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]"
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"

N, Y0, STEP = 8, 2, 2

GOOD = 0
TOTAL = 0


def report(ok, label):
    global GOOD, TOTAL
    TOTAL += 1
    GOOD += bool(ok)
    print("%s %s" % ("PASS" if ok else "FAIL", label))
    return ok


def lever_bank(s, x, z, dx, dz):
    for i in range(N):
        y = Y0 + STEP * i
        s.set_block_from_string(x, y - 1, z, STONE)
        s.set_block_from_string(x, y, z, LEVER)
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE)
        s.set_block_from_string(x + dx, y, z + dz, DUST)
    return (x + dx, Y0, z + dz)


def lamp_bank(s, x, z):
    for i in range(N):
        y = Y0 + STEP * i
        s.set_block_from_string(x, y - 1, z, LAMP)
        s.set_block_from_string(x, y, z, DUST)
    return (x, Y0, z)


def word(v):
    return n.Value.from_u32(v)


def executor(d):
    baked = d.bake(4000)
    cell = n.CellExecutor.for_schematic(baked)
    cell.settle(4000)
    return cell


def part_gate_drag():
    """Sketch (2): drag g0; only its 2 segments reroute; still conducts."""
    print("-- sketch 2: gate drag --")
    s = n.Schematic.create("drag")
    a_in = lever_bank(s, 0, 8, 1, 0)
    a_out = lamp_bank(s, 24, 8)
    d = n.Design.for_schematic("drag", s)
    d.declare_input("a_in", *a_in, 0, STEP, 0, N, "uint")
    d.declare_output("a_out", *a_out, 0, STEP, 0, N, "uint")
    gates = json.dumps([
        {"name": "g0", "anchor": [8, 2, 8], "step": [0, 2, 0]},
        {"name": "g1", "anchor": [16, 2, 8], "step": [0, 2, 0]},
    ])
    state = d.route_bus("bus_a", "a_in", '["a_out"]', gates, "{}")
    report(state == "routed", "bus_a routed through g0 + g1 (%s)" % state)
    report(json.loads(d.check())["clean"], "check clean before the drag")

    moved = json.loads(d.move_gate("bus_a", "g0", 8, 2, 12))
    report(moved["state"] == "routed", "gate drag rerouted (%s)" % moved["state"])
    report(moved["rerouted_segments"] == 2,
           "EXACTLY 2 segments rerouted (%d)" % moved["rerouted_segments"])
    report(json.loads(d.check())["clean"], "check clean after the drag")

    skew = json.loads(d.bus_skew("bus_a"))
    report(len(skew["per_bit_rt"]) == N and skew["skew_rt"] == 0,
           "skew report: per-bit %s, skew %d rt" % (skew["per_bit_rt"], skew["skew_rt"]))

    # Net-class rule: an impossible delay budget turns check() dirty…
    d.set_bus_rule("bus_a", '{"max_len_rt": 0}')
    dirty = json.loads(d.check())
    report(not dirty["clean"] and dirty["rules"], "max_len_rt=0 flags a rule violation")
    # …a sane budget is enforced clean.
    d.set_bus_rule("bus_a", '{"max_len_rt": 100}')
    report(json.loads(d.check())["clean"], "max_len_rt=100 passes")

    cell = executor(d)
    ok = True
    for i in range(N):
        cell.set_input("a_in", word(1 << i))
        cell.settle(400)
        got = cell.read_output("a_out").as_u32()
        ok = ok and got == 1 << i
    report(ok, "typed walking-ones conduct after the drag")


def part_component_drag():
    """Sketch (3): c0 dragged through bus_a -> visible FAILED; away -> reroute."""
    print("-- sketch 3: component drag --")
    s = n.Schematic.create("blocker")
    a_in = lever_bank(s, 0, 8, 1, 0)
    a_out = lamp_bank(s, 16, 8)
    d = n.Design.for_schematic("blocker", s)
    d.declare_input("a_in", *a_in, 0, STEP, 0, N, "uint")
    d.declare_output("a_out", *a_out, 0, STEP, 0, N, "uint")

    cube = n.Schematic.create("cube")
    for x in range(3):
        for y in range(3):
            for z in range(3):
                cube.set_block_from_string(x, y, z, STONE)
    cube.set_cell_contract_json(json.dumps(
        {"name": "cube", "io": {"inputs": {}, "outputs": {}, "buses": {}}}))
    d.add_cell("cube", cube)
    d.place("c0", "cube", 4, 0, 20, 0)

    state = d.route_bus("bus_a", "a_in", '["a_out"]', "[]", "{}")
    report(state == "routed", "bus_a routed with c0 parked clear (%s)" % state)

    # Drag c0 THROUGH the bus: move succeeds, bus fails visibly.
    r = json.loads(d.move_instance("c0", 4, 0, 8, 0))
    report("bus_a" in r["failed"], "co-reroute failed VISIBLY: %s" % r["failed"])
    report(d.bus_state("bus_a").startswith("failed"),
           "bus state is the red layer (%s…)" % d.bus_state("bus_a")[:40])

    # Drag it away: the FAILED bus is re-attempted and reroutes.
    r = json.loads(d.move_instance("c0", 4, 0, 20, 0))
    report(r["rerouted"] == ["bus_a"], "dragged away -> rerouted %s" % r["rerouted"])
    report(json.loads(d.check())["clean"], "check clean after recovery")


def part_fanout():
    """Multi-sink trunk: one driver, two sinks, both conduct in-sim."""
    print("-- fanout trunk --")
    s = n.Schematic.create("fanout")
    a_in = lever_bank(s, 0, 8, 1, 0)
    a_out = lamp_bank(s, 16, 8)
    c_out = lamp_bank(s, 8, 16)
    d = n.Design.for_schematic("fanout", s)
    d.declare_input("a_in", *a_in, 0, STEP, 0, N, "uint")
    d.declare_output("a_out", *a_out, 0, STEP, 0, N, "uint")
    d.declare_output("c_out", *c_out, 0, STEP, 0, N, "uint")
    state = d.route_bus("fan", "a_in", '["a_out", "c_out"]', "[]", "{}")
    report(state == "routed", "fanout trunk + branch routed (%s)" % state)
    report(json.loads(d.check())["clean"], "fanout check clean")
    cell = executor(d)
    ok = True
    for i in range(N):
        cell.set_input("a_in", word(1 << i))
        cell.settle(400)
        a = cell.read_output("a_out").as_u32()
        c = cell.read_output("c_out").as_u32()
        ok = ok and a == 1 << i and c == 1 << i
    report(ok, "both sinks read every walking-one")


def part_wired_or():
    """Explicit wired-OR merge: sink reads a | b."""
    print("-- wired-OR merge --")
    s = n.Schematic.create("wor")
    a_in = lever_bank(s, 0, 8, 1, 0)
    b_in = lever_bank(s, 8, 0, 0, 1)
    a_out = lamp_bank(s, 16, 8)
    d = n.Design.for_schematic("wor", s)
    d.declare_input("a_in", *a_in, 0, STEP, 0, N, "uint")
    d.declare_input("b_in", *b_in, 0, STEP, 0, N, "uint")
    d.declare_output("a_out", *a_out, 0, STEP, 0, N, "uint")
    state = d.route_bus_or("wor", '["a_in", "b_in"]', '["a_out"]', "[]", "{}")
    report(state == "routed", "wired-OR routed (%s)" % state)
    report(json.loads(d.check())["clean"], "wired-OR check clean (ONE LVS net)")
    cell = executor(d)
    ok = True
    for a, b in [(0, 0), (0x55, 0xAA), (0xAA, 0x55), (0x0F, 0xF0), (0x81, 0x18)]:
        cell.set_input("a_in", word(a))
        cell.set_input("b_in", word(b))
        cell.settle(400)
        got = cell.read_output("a_out").as_u32()
        ok = ok and got == (a | b)
    report(ok, "sink reads a | b for every driven pair")


def main():
    part_gate_drag()
    part_component_drag()
    part_fanout()
    part_wired_or()
    print("design_demo3_drag: %d/%d checks" % (GOOD, TOTAL))
    return 0 if GOOD == TOTAL else 1


if __name__ == "__main__":
    raise SystemExit(main())
