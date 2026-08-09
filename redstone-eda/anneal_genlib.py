"""Anneal the genlib-mapped seg7's instance placements with pnr-core.

The cheap honest wiring of pnr-core's annealer to the Python compositor:
`cargo run -p nucleation-routing --example anneal_place` is a deterministic
text filter (SplitMix64, seeded) that anneals instance BOXES (each cell's
claimed obstruction region: fragment + approach corridors + out stubs) under
HPWL + overlap + binned congestion.  This driver:

  1. maps seg7 exactly like genlib_map --design seg7 (baseline = the
     levelized placer), routes it, measures, and sim-verifies 16/16;
  2. exports boxes + port pins + nets, anneals them in Rust;
  3. rebuilds with the annealed placements, measures, sim-verifies;
  4. reports the deltas; saves showcase schems ONLY for verified-green
     builds (--save).

Usage: ~/eda-venv/bin/python anneal_genlib.py [--seed 42] [--save]
"""
import argparse
import os
import subprocess
import sys

import genlib_map as gm
import rs

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

# a cell's claimed box relative to its stamp origin (see compose():
# corridors reach 5 west of a port at local x0 -> box x -6; out stubs reach
# +2 east -> +3; z margin 2)
BOX_W, BOX_E, BOX_Z = 6, 3, 2


def build(name, nl, lib, order, layout, tag):
    b, labels, inst_ports, driver, lever_at, wire_rt, wire_path = \
        gm.compose(name, nl, lib, order, layout)
    arrival, worst = gm.structural_sta(nl, lib, order, wire_rt)
    blocks, dims = gm.measure(tag, b)
    wire_cells = sum(len(p) for p in wire_path.values())
    print("%s: STA %d rt -> %s; %d route-path cells"
          % (tag, arrival[worst], worst, wire_cells))
    sim = gm.verify_design(name, nl, lib, order, b, labels, driver, lever_at)
    return {"b": b, "sim": sim, "blocks": blocks, "dims": dims,
            "sta": arrival[worst], "wire_cells": wire_cells,
            "lever_at": lever_at, "nl": nl, "ok": sim is not None}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--save", action="store_true")
    args = ap.parse_args()

    lib = gm.build_library()
    os.makedirs(gm.BUILD_DIR, exist_ok=True)
    genlib = os.path.join(gm.BUILD_DIR, "cells.genlib")
    gm.write_genlib(lib, genlib)
    blif = os.path.join(gm.BUILD_DIR, "seg7_map.blif")
    gm.run_yosys(os.path.join(HERE, "hdl", "seg7.v"), genlib, blif)
    nl = gm.parse_blif(blif)
    order = gm.topo_gates(nl)

    placements, pi_pos = gm.place(nl, lib, order)
    base = build("seg7", nl, lib, order, (placements, pi_pos), "baseline")
    if not base["ok"]:
        print("baseline failed verification -- aborting")
        return 1

    # ---- export boxes + pins + nets to the Rust annealer ----------------
    lines = []
    xs = [p[0] for p in placements] + [120]
    zs = [p[2] for p in placements] + [120]
    # box-origin window: stamp origins stay east of the PI column (x >= 12)
    # and boxes may reach BOX_Z north of z=0, like the baseline's row 0
    area = (12 - BOX_W, -BOX_Z, max(xs) + 40, max(zs) + 30)
    lines.append("area %d %d %d %d" % area)
    lines.append("seed %d" % args.seed)
    pin_idx = {}
    pins = []

    def add_pin(cell_id, dx, dz):
        pins.append("pin %s %d %d" % (cell_id, dx, dz))
        pin_idx[len(pins) - 1] = None
        return len(pins) - 1

    for n, (x, y, z) in pi_pos.items():
        cid = "pi_" + n.replace("[", "_").replace("]", "")
        lines.append("cell %s 3 1 %d %d 1" % (cid, x - 2, z))
    for i, (kind, pinsmap) in enumerate(order):
        cell = lib[kind]
        px, pz = placements[i][0], placements[i][2]
        # box origin = stamp origin - west reach; ports relative to box
        lines.append("cell g%d %d %d %d %d 0"
                     % (i, cell.w + BOX_W + BOX_E, cell.d + 2 * BOX_Z,
                        px - BOX_W, pz - BOX_Z))
    net_pins = {}
    for n in nl.inputs:
        cid = "pi_" + n.replace("[", "_").replace("]", "")
        net_pins.setdefault(n, []).append(add_pin(cid, 2, 0))
    for i, (kind, pinsmap) in enumerate(order):
        cell = lib[kind]
        for pname, port in cell.frag.ports.items():
            if pname == "out":
                net = pinsmap["O"]
            elif pname in pinsmap:
                net = pinsmap[pname]
            else:
                continue
            net_pins.setdefault(net, []).append(
                add_pin("g%d" % i, port[0] + BOX_W, port[2] + BOX_Z))
    for net, ps in sorted(net_pins.items()):
        if len(ps) > 1:
            lines.append("net " + " ".join(str(p) for p in ps))
    lines = lines[:2] + sorted(l for l in lines[2:] if l.startswith("cell")) \
        + pins + [l for l in lines[2:] if l.startswith("net")] \
        + [l for l in lines[2:] if not l.startswith(("cell", "net"))]

    proc = subprocess.run(
        ["cargo", "run", "--quiet", "-p", "nucleation-routing",
         "--example", "anneal_place"],
        input="\n".join(lines) + "\n", capture_output=True, text=True,
        cwd=ROOT)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        return 1
    annealed = {}
    for line in proc.stdout.splitlines():
        t = line.split()
        if t[0] == "cost":
            print("anneal: cost %s -> %s, HPWL %s -> %s "
                  "(%d accepted / %d proposed)"
                  % (t[1], t[2], t[3], t[4], int(t[5]), int(t[6])))
        elif t[0] == "place" and t[1].startswith("g"):
            annealed[int(t[1][1:])] = (int(t[2]), int(t[3]))
    new_placements = [
        (annealed[i][0] + BOX_W, 0, annealed[i][1] + BOX_Z)
        for i in range(len(order))]

    ann = build("seg7", nl, lib, order, (new_placements, pi_pos), "annealed")

    print("\n== deltas (baseline -> annealed) ==")
    for key, fmt in (("blocks", "%d"), ("wire_cells", "%d"), ("sta", "%d rt")):
        print("  %-11s %s -> %s" % (key, fmt % base[key], fmt % ann[key]))
    print("  bbox        %s -> %s" % ("x".join(map(str, base["dims"])),
                                      "x".join(map(str, ann["dims"]))))
    print("  verified    %s -> %s"
          % ("16/16" if base["ok"] else "FAIL",
             "16/16" if ann["ok"] else "FAIL"))

    if args.save:
        import build_ppa as bp
        for tag, res in (("baseline", base), ("annealed", ann)):
            if not res["ok"]:
                print("NOT saving %s (unverified)" % tag)
                continue
            lv = rs.Levers(res["sim"],
                           [res["lever_at"][n] for n in nl.inputs])
            lv.set([0] * len(nl.inputs))
            bp.bake(res["b"], res["sim"])
            path = os.path.join(HERE, "showcase",
                                "genlib_seg7_anneal_%s.schem" % tag)
            res["b"].s.save_to_file(path)
            print("saved", path)
    return 0 if (base["ok"] and ann["ok"]) else 1


if __name__ == "__main__":
    raise SystemExit(main())
