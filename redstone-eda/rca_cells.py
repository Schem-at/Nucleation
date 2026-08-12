"""N-bit ripple-carry adder = FA cells stamped at PITCH in z.  Zero routing.

The FA port contract (cin north / cout south at the same x) means bit k's cout
cell sits directly adjacent to bit k+1's cin cell when stamped at dz=k*PITCH:
the carry chain connects by abutment.  a/b enter the west face at (z0, z4) +
k*PITCH -- a pitch-aligned word bus by construction; sums leave the east face.
"""
import argparse
import cells
import rs


def build_rca(n, fa):
    b = rs.Build("rca%d_cells" % n)
    labels = {}
    ports = []
    aliases = []
    for k in range(n):
        pk = fa.stamp(b, labels, 0, 0, k * cells.PITCH,
                      rename=lambda s, k=k: "b%d.%s" % (k, s))
        ports.append(pk)
        if k:
            aliases.append(("b%d.cout" % (k - 1), "b%d.cin" % k))
    return b, labels, ports, aliases


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bits", type=int, default=4)
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    n = args.bits

    ha = cells.build_half_adder()
    fa = cells.build_full_adder(ha)
    b, labels, ports, aliases = build_rca(n, fa)

    lever = {}
    for k in range(n):
        for sig in ("a", "b"):
            x, y, z = ports[k][sig]
            b.stone(x - 2, y - 1, z)
            b.force(x - 2, y, z, rs.LEVER_OFF)
            b.stone(x - 1, y - 1, z)
            b.put(x - 1, y, z, rs.DUST)
            lever["%s%d" % (sig, k)] = (x - 2, y, z)
    cx, cy, cz = ports[0]["cin"]
    b.stone(cx, cy - 1, cz - 2)
    b.force(cx, cy, cz - 2, rs.LEVER_OFF)
    b.stone(cx, cy - 1, cz - 1)
    b.put(cx, cy, cz - 1, rs.DUST)
    lever["cin"] = (cx, cy, cz - 2)

    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    vol = (x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)
    print("rca%d: %d blocks, %dx%dx%d, fill %.0f%%"
          % (n, len(b.cells), x1 - x0 + 1, y1 - y0 + 1, z1 - z0 + 1,
             100 * len(b.cells) / vol))

    import audit, nets, drc
    problems = audit.audit(b.cells)
    shorts = nets.check(b.cells, labels, aliases)
    for kind, items in problems.items():
        if items:
            print("STRUCTURAL %s x%d e.g. %s" % (kind, len(items), items[0]))
    print("net check: %d shorts" % len(shorts))
    # The first of the three ring latches was here: two cells' port approaches
    # formed opposite-facing repeaters in a ring on an aliased net.
    rings_ok = drc.check_rings("rca%d" % n, b.cells)
    if any(problems.values()) or shorts or not rings_ok:
        for s in shorts[:4]:
            print("   ", s)
        return 1

    sim = b.sim()
    names = ["cin"] + ["a%d" % k for k in range(n)] + ["b%d" % k for k in range(n)]
    lv = rs.Levers(sim, [lever[s] for s in names])
    bad = 0
    total = 0
    for A in range(1 << n):
        for B in range(1 << n):
            for c in (0, 1):
                lv.set([c] + [(A >> k) & 1 for k in range(n)]
                       + [(B >> k) & 1 for k in range(n)])
                got = sum(int(sim.on(*ports[k]["sum"])) << k for k in range(n))
                got += int(sim.on(*ports[n - 1]["cout"])) << n
                total += 1
                if got != A + B + c:
                    bad += 1
                    if bad <= 5:
                        print("   WRONG %d+%d+%d=%d got %d" % (A, B, c, A + B + c, got))
    print("rca%d exhaustive: %d/%d correct" % (n, total - bad, total))
    if bad:
        return 1
    if args.out:
        lv.set([0] * len(names))
        import build_ppa as bp
        print("baked %d states" % bp.bake(b, sim))
        b.s.save_to_file(args.out)
        print("saved", args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
