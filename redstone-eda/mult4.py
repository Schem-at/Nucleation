"""A 4x4 multiplier as STACKED planes with maze-routed vertical interconnect.

Architecture (shift-add, data flows upward only):

    plane 0 (y 0..):   16 AND nodes  pp_j_i = A_i & B_j   (levers live here)
    plane 1 (y 12..):  4-bit KS adder  acc0>>1 + pp1      -> P1 + carry bits
    plane 2 (y 24..):  4-bit KS adder  row1>>1 + pp2      -> P2 + carry bits
    plane 3 (y 36..):  4-bit KS adder  row2>>1 + pp3      -> P3..P7

Each plane is compiled by the same PLA compiler; inter-plane nets are found by
the A* maze router (dust, stairs, torch-ladder climbs), with unused adder
inputs simply left unrouted (an undriven rail is a constant 0).  P0 is read
directly off plane 0.

This is the chip flow in miniature: verified macros, 3D placement, global
routing, then whole-design DRC (audit + net shorts) and simulation.
"""
import argparse

import build_ppa as bp
import router
import rs

PITCH = 12


def pp_netlist(_n):
    """Plane 0: pp_j_i = A_i & B_j, two 1-term nodes per slice."""
    nodes, levers = {}, []
    for j in range(4):
        for i in range(4):
            idx = j * 4 + i
            nodes.setdefault(idx // 2, []).append(
                ("pp%d_%d" % (j, i), [["A%d" % i, "B%d" % j]]))
    levers = [(0, "A%d" % i, 0) for i in range(4)] + \
             [(0, "B%d" % j, 1) for j in range(4)]
    return [dict(name="pp", nodes=sorted(nodes.items()), local=False,
                 inverters=[], levers=levers)]


def renamed_adder(pfx):
    """The verified 4-bit KS adder netlist, signals prefixed, levers -> ext."""
    def make(_n):
        stages = bp.netlist(4)
        ren = lambda s: pfx + s
        for st in stages:
            st["nodes"] = [(sl, [(ren(o), [[ren(s) for s in t] for t in terms])
                                 for o, terms in groups])
                           for sl, groups in st["nodes"]]
            st["inverters"] = [(sl, ren(a), ren(b)) for sl, a, b in st["inverters"]]
            st["ext"] = [(sl, ren(s), c) for sl, s, c in st["levers"]]
            st["levers"] = []
        return stages
    return make


def drive_cell(ppa, sig, dy):
    """West-end dust cell of an ext rail == where a routed wire must land."""
    for st in ppa.stages:
        if sig in st.get("spans", {}):
            return (st["drive"][sig], 1 + dy, ppa.rail_z(st, sig))
    raise KeyError(sig)


def build():
    """Compile the four planes, stack them at PITCH and globally route.

    Returns everything a checker needs, so a DRC or a regression test can
    inspect the real design instead of re-deriving it: `prewired` is the cell
    set BEFORE any routing (the compiled plane geometry), which is what tells a
    rail cell apart from a routed one.
    """
    planes = [bp.PPA(4, make=pp_netlist, name="pp_plane")]
    for k in (1, 2, 3):
        planes.append(bp.PPA(4, make=renamed_adder("m%d" % k), name="row%d" % k))
    for p in planes:
        p.build()

    # ---- merge, planes stacked at PITCH ------------------------------------
    b, labels, probe = rs.Build("mult4"), {}, {}
    for k, p in enumerate(planes):
        dy = PITCH * k
        for (x, y, z), blk in p.b.cells.items():
            b.put(x, y + dy, z, blk)
        for (x, y, z), lab in p.labels.items():
            labels[(x, y + dy, z)] = lab
        for sig, (x, y, z) in p.probe.items():
            probe[sig] = (x, y + dy, z)
    levers = [(sig, pos) for sig, pos in planes[0].levers]
    prewired = set(b.cells)

    # ---- global routing -----------------------------------------------------
    r = router.Router(b, labels)
    hops = []
    for i in range(1, 4):                      # acc0>>1 into row1's A
        hops.append(("pp0_%d" % i, 0, "m1A%d" % (i - 1)))
    for j in (1, 2, 3):                        # pp_j up to row j's B
        for i in range(4):
            hops.append(("pp%d_%d" % (j, i), 0, "m%dB%d" % (j, i)))
    for k in (1, 2):                           # carry bits up to the next row
        for i in (1, 2, 3):
            hops.append(("m%ds%d" % (k, i), k, "m%dA%d" % (k + 1, i - 1)))
        hops.append(("m%dcout" % k, k, "m%dA3" % (k + 1)))
    # route the longest nets first, while space is uncongested
    hops.sort(key=lambda h_: -abs(int(h_[2][1]) - h_[1]))
    total, aliases, routed = 0, [], []
    for sig, src_k, dst_sig in hops:
        src = probe[sig]
        dst_k = int(dst_sig[1])
        dst = drive_cell(planes[dst_k], dst_sig, PITCH * dst_k)
        # the route joins two named nets into one -- tell both the router
        # (clearance) and the net checker (deliberate alias, not a short)
        n = r.route(src, dst, dst_sig, friendly={sig, dst_sig})
        aliases.append((sig, dst_sig))
        routed.append((sig, dst_sig, src, dst))
        total += n
    return dict(planes=planes, b=b, labels=labels, probe=probe, levers=levers,
                aliases=aliases, routed=routed, prewired=prewired,
                routed_cells=total, hops=hops)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-sim", action="store_true")
    ap.add_argument("--out", default="")
    args = ap.parse_args()

    d = build()
    b, labels, probe = d["b"], d["labels"], d["probe"]
    planes, levers, aliases = d["planes"], d["levers"], d["aliases"]
    print("routed %d inter-plane nets, %d cells"
          % (len(d["hops"]), d["routed_cells"]))

    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    vol = (x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)
    print("mult4: %d blocks, extent x %d..%d y %d..%d z %d..%d, fill %.1f%%"
          % (len(b.cells), x0, x1, y0, y1, z0, z1, 100 * len(b.cells) / vol))

    import audit, nets, drc
    problems = audit.audit(b.cells)
    shorts = nets.check(b.cells, labels, aliases)
    for kind, items in problems.items():
        if items:
            print("STRUCTURAL %s x%d e.g. %s" % (kind, len(items), items[0]))
    print("net check: %d shorts" % len(shorts))
    for s in shorts[:6]:
        print("   ", s)
    # This is the build the ring rule was written for: m1B2's route once joined
    # the rail it drives and latched at 15 (64/256, everything else green).
    rings_ok = drc.check_rings("mult4", b.cells)
    if any(problems.values()) or shorts or not rings_ok:
        return 1
    if args.no_sim:
        return 0

    sim = b.sim(settle=4000)
    print("placed; quiescent:", sim.sim.is_quiescent())
    names = [s for s, _ in levers]
    lv = rs.Levers(sim, [p for _, p in levers])
    outs = [probe["pp0_0"]] + [probe["m%ds0" % k] for k in (1, 2, 3)] \
        + [probe["m3s%d" % i] for i in (1, 2, 3)] + [probe["m3cout"]]

    bad = 0
    for A in range(16):
        for B in range(16):
            bits = [(A >> int(n[1])) & 1 if n[0] == "A" else (B >> int(n[1])) & 1
                    for n in names]
            lv.set(bits)
            got = sum(int(sim.on(*p)) << i for i, p in enumerate(outs))
            if got != A * B:
                bad += 1
                if bad <= 6:
                    print("   WRONG %d*%d = %d, got %d" % (A, B, A * B, got))
    print("multiplier: %d/256 correct" % (256 - bad))
    if bad:
        return 1
    if args.out:
        lv.set([0] * len(names))
        print("baked %d states" % bp.bake(b, sim))
        b.s.save_to_file(args.out)
        print("saved", args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
