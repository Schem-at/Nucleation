#!/usr/bin/env python3
"""Typed-cell demo: drive the compiled seg7 PLA by PORT NAME and WORD VALUE.

The built-in compiler (nucleation.Hdl, crates/nucleation-hdl) emits a
CellContract next to the schematic: BLIF vector ports (d[0]..d[3],
seg[0]..seg[6]) grouped back into typed words, input bits mapped to their
drive levers, output bits to their dust probes, plus a physical sidecar
(bounds keepout, ESTIMATED per-port delays from levelization depth,
paste_safe=false).

This script proves the contract is sufficient to run the cell: it sets
`d = 0x0..0xF` and reads the 7-bit `seg` word back, with EVERY coordinate
taken from the contract -- none live in this file. Expected values come from
hdl2redstone.py's Python reference model (the verified spec).

Run from redstone-eda/:  python hdl/typed_demo.py
"""
import json
import os
import sys

HDL_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HDL_DIR)                    # hdl2redstone (reference model)
sys.path.insert(0, os.path.dirname(HDL_DIR))   # rs (sim helpers)

import hdl2redstone as h
import rs
import nucleation as n

TOP = "seg7"
BLIF = os.path.join(HDL_DIR, "build", "%s.blif" % TOP)


def reference_model(blif_path):
    """The pure-Python prim-graph eval, prepared exactly like main()."""
    inputs, outputs, raw_nodes, _latches = h.parse_blif(blif_path)
    nodes, consts = h.fold(inputs, raw_nodes)
    comp = h.Compiler(inputs, outputs, nodes, consts)
    po_val = {po: comp.value(po, 1) for po in outputs}
    comp.peephole()
    po_val = {po: (v if isinstance(v, tuple) else comp.resolve(v))
              for po, v in po_val.items()}
    comp.levelise()
    return inputs, outputs, comp, po_val


def main():
    text = open(BLIF).read()

    # 1. compile: schematic + typed-cell contract, one call each
    contract = json.loads(n.Hdl.compile_blif_contract(text, TOP))
    schem = n.Hdl.compile_blif(text, TOP, False)
    d_port = contract["io"]["inputs"]["d"]
    seg_port = contract["io"]["outputs"]["seg"]
    print("contract: input d %s (%d levers), output seg %s (%d probes)"
          % (d_port["io_type"], len(d_port["positions"]),
             seg_port["io_type"], len(seg_port["positions"])))
    est = [e for e in contract["physical"]["delays_rt"]
           if e["from"] == "d" and e["to"] == "seg"]
    print("estimated d->seg arrival: %s rt (levelization depth * 2, not measured)"
          % (est[0]["delay_rt"] if est else "?"))

    # 2. simulate (tighten round-trip, exactly rs.Build.sim / run_rust)
    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_hdl_typed.schem")
    schem.save_to_file(tmp)
    tight = n.Schematic.open(tmp)
    sim = n.TickSimulation.from_schematic(tight, n.TickSettleMode.Placement,
                                          0, 0, 0, rs.EXTRA_STATES)
    sim.run_until_quiescent(4000)
    # the contract's bounds keepout is the cell frame the ports live in
    origin = tuple(contract["physical"]["keepouts"][0]["min"])
    s = rs.Sim(sim, origin)

    # 3. drive by name and word: every position comes from the contract
    levers = rs.Levers(s, [tuple(p) for p in d_port["positions"]])
    probes = [tuple(p) for p in seg_port["positions"]]

    inputs, outputs, comp, po_val = reference_model(BLIF)
    ok = 0
    for value in range(16):
        levers.set([(value >> i) & 1 for i in range(len(d_port["positions"]))])
        got = sum(int(s.on(*p)) << i for i, p in enumerate(probes))
        bits = {net: (value >> i) & 1
                for i, net in enumerate(["d[0]", "d[1]", "d[2]", "d[3]"])}
        want_val = comp.eval([bits[net] for net in inputs])
        want = 0
        for i in range(7):
            v = po_val["seg[%d]" % i]
            b = v[1] if isinstance(v, tuple) else want_val[v]
            want |= b << i
        mark = "ok" if got == want else "WRONG"
        if got == want:
            ok += 1
        print("  set d=0x%X  read seg=0b%07d  want 0b%07d  %s"
              % (value, int(format(got, "b")), int(format(want, "b")), mark))
    print("%s typed drive: %d/16" % (TOP, ok))
    return 0 if ok == 16 else 1


if __name__ == "__main__":
    sys.exit(main())
