"""An N-bit 4-op ALU (ADD / SUB / AND / XOR) on the Kogge-Stone PLA compiler.

The pretty part: the adder already computes the logic ops internally.
g_i = A&B IS the AND result and p_i = A^B IS the XOR result, so the whole ALU
is the verified prefix adder plus three things:

  * a B-select stage: effB_i = B_i XOR opSUB (two-term PLA node using a
    nopSUB inverter rail), so SUB = A + ~B + 1 reuses the adder unchanged;
  * a forced carry-in: cin_eff = cin&opADD + opSUB;
  * a 4-term output mux folded into the sum stage:
      out_i = (p^c)&opADDSUB + g&opAND + p&opXOR.

Op select is one-hot (opADD/opSUB/opAND/opXOR levers).  All op machinery lives
in slices at NEGATIVE x, west of bit 0: their route corridors then cross bands
where nothing else exists, which is what makes the channel allocation work.

Geometry is the same primitive library, with a wider 21-block slice: 4 wiring
channels (a signal routed far needs an exclusive corridor per group) and up to
4 columns per node (the output mux has 4 product terms).
"""
import argparse
import random

import build_ppa as bp
import rs

# -- wider slice: 4 channels + 4 columns ------------------------------------
bp.SLICE_W = 21
bp.CH = (0, 2, 4, 6)
bp.INV_X = 8
bp.INV_RAIL_X = 11
bp.COL_A = (13, 15)
bp.COL_B = (17,)
bp.EXTRA_COLS = (19,)
bp.RAIL_E = 19

OPS = ("opADD", "opSUB", "opAND", "opXOR")


def alu_netlist(N):
    L = N.bit_length() - 1
    stages = []

    # ---- bsel: effB = B ^ opSUB, plus the op plumbing in negative slices ----
    inv = [(i, "B%d" % i, "nB%d" % i) for i in range(N)] + [(0, "opSUB", "nopSUB")]
    nodes = [(i, [("effB%d" % i, [["B%d" % i, "nopSUB"], ["nB%d" % i, "opSUB"]])])
             for i in range(N)]
    nodes += [(-1, [("cin_eff", [["cin", "opADD"], ["opSUB"]])]),
              (-2, [("opADDSUB_b", [["opADD"], ["opSUB"]])]),
              (-3, [("opAND_b", [["opAND"]])]),
              (-4, [("opXOR_b", [["opXOR"]])])]
    levers = [(i, "B%d" % i, 0) for i in range(N)] + \
             [(-1, "cin", 0), (-2, "opADD", 1), (-2, "opSUB", 0),
              (-3, "opAND", 0), (-4, "opXOR", 0)]
    stages.append(dict(name="bsel", nodes=nodes, local=True,
                       inverters=inv, levers=levers))

    # ---- pg: identical to the adder, with effB in place of B ---------------
    inv = []
    for i in range(N):
        inv += [(i, "A%d" % i, "nA%d" % i), (i, "effB%d" % i, "neffB%d" % i)]
    nodes = [(i, [("p%d" % i, [["A%d" % i, "neffB%d" % i], ["nA%d" % i, "effB%d" % i]]),
                  ("g%d" % i, [["A%d" % i, "effB%d" % i]])]) for i in range(N)]
    stages.append(dict(name="pg", nodes=nodes, local=True, inverters=inv,
                       levers=[(i, "A%d" % i, 0) for i in range(N)]))

    # ---- Kogge-Stone prefix: unchanged ------------------------------------
    gp = lambda k, i: ("g%d" % i, "p%d" % i) if k == 0 else \
        ("G%d_%d" % (k, i), "P%d_%d" % (k, i))
    for k in range(1, L + 1):
        s = 1 << (k - 1)
        nodes = []
        for i in range(N):
            gh, ph = gp(k - 1, i)
            go, po = gp(k, i)
            if i >= s:
                gl, pl = gp(k - 1, i - s)
                nodes.append((i, [(go, [[gh], [ph, gl]]), (po, [[ph, pl]])]))
            else:
                nodes.append((i, [(go, [[gh]]), (po, [[ph]])]))
        stages.append(dict(name="prefix%d" % k, nodes=nodes, local=False,
                           inverters=[], levers=[], stride=s, width=N))

    # ---- carries, from cin_eff --------------------------------------------
    nodes = [(0, [("c0", [["cin_eff"]])])]
    for i in range(1, N + 1):
        g, p = gp(L, i - 1)
        out = "cout" if i == N else "c%d" % i
        nodes.append((i, [(out, [[g], [p, "cin_eff"]])]))
    stages.append(dict(name="carry", nodes=nodes, local=False,
                       inverters=[], levers=[]))

    # ---- output mux folded into the sum stage ------------------------------
    inv = []
    for i in range(N):
        inv += [(i, "p%d" % i, "np%d" % i), (i, "c%d" % i, "nc%d" % i)]
    nodes = [(i, [("out%d" % i,
                   [["p%d" % i, "nc%d" % i, "opADDSUB_b"],
                    ["np%d" % i, "c%d" % i, "opADDSUB_b"],
                    ["g%d" % i, "opAND_b"],
                    ["p%d" % i, "opXOR_b"]])]) for i in range(N)]
    stages.append(dict(name="outmux", nodes=nodes, local=True,
                       inverters=inv, levers=[]))
    return stages


def model(N, op, A, B, cin):
    """Reference values for every named signal, for per-node checking."""
    mask = (1 << N) - 1
    v = {"cin": cin}
    for o in OPS:
        v[o] = int(o == op)
    v["opSUB"], v["nopSUB"] = v["opSUB"], 1 - v["opSUB"]
    effB = (B ^ mask) if op == "opSUB" else B
    v["cin_eff"] = (cin & v["opADD"]) | v["opSUB"]
    v["opADDSUB_b"] = v["opADD"] | v["opSUB"]
    v["opAND_b"], v["opXOR_b"] = v["opAND"], v["opXOR"]
    G, P = {}, {}
    for i in range(N):
        a, b, eb = (A >> i) & 1, (B >> i) & 1, (effB >> i) & 1
        v["A%d" % i], v["nA%d" % i] = a, 1 - a
        v["B%d" % i], v["nB%d" % i] = b, 1 - b
        v["effB%d" % i], v["neffB%d" % i] = eb, 1 - eb
        v["p%d" % i], v["g%d" % i] = a ^ eb, a & eb
        G[i], P[i] = a & eb, a ^ eb
    for k in range(1, N.bit_length()):
        s, nG, nP = 1 << (k - 1), {}, {}
        for i in range(N):
            nG[i] = G[i] | (P[i] & G[i - s]) if i >= s else G[i]
            nP[i] = P[i] & P[i - s] if i >= s else P[i]
            v["G%d_%d" % (k, i)], v["P%d_%d" % (k, i)] = nG[i], nP[i]
        G, P = nG, nP
    v["c0"] = v["cin_eff"]
    for i in range(1, N):
        v["c%d" % i] = G[i - 1] | (P[i - 1] & v["cin_eff"])
    v["cout"] = G[N - 1] | (P[N - 1] & v["cin_eff"])
    for i in range(N):
        s_i = v["p%d" % i] ^ v["c%d" % i]
        v["np%d" % i], v["nc%d" % i] = 1 - v["p%d" % i], 1 - v["c%d" % i]
        v["out%d" % i] = (s_i & v["opADDSUB_b"]) | (v["g%d" % i] & v["opAND"]) \
            | (v["p%d" % i] & v["opXOR"])
    return v


def expected_value(N, op, A, B, cin):
    mask = (1 << N) - 1
    if op == "opADD":
        return (A + B + cin) & mask
    if op == "opSUB":
        return (A - B) & mask                    # cin ignored: SUB forces +1
    return (A & B) if op == "opAND" else (A ^ B)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--width", type=int, default=8)
    ap.add_argument("--cases", type=int, default=0, help="0 = exhaustive")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--out", default="")
    ap.add_argument("--no-sim", action="store_true")
    args = ap.parse_args()
    N = args.width

    ppa = bp.PPA(N, make=alu_netlist, name="alu_%dbit" % N)
    b = ppa.build()
    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    vol = (x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)
    print("ALU width %d: %d blocks, extent x %d..%d y %d..%d z %d..%d (%.2fM cells)"
          % (N, len(b.cells), x0, x1, y0, y1, z0, z1, vol / 1e6))

    import audit, nets, drc
    problems = audit.audit(b.cells)
    shorts = nets.check(b.cells, ppa.labels)
    for kind, items in problems.items():
        if items:
            print("STRUCTURAL %s x%d e.g. %s" % (kind, len(items), items[0]))
    print("net check: %d shorted nets" % len(shorts))
    for la, lb, pa, pb in shorts[:8]:
        print("   SHORT %s <-> %s  at %s / %s" % (la, lb, pa, pb))
    rings_ok = drc.check_rings("build_alu", b.cells)
    if any(problems.values()) or shorts or not rings_ok:
        return 1
    if args.no_sim:
        return 0

    sim = b.sim(settle=4000)
    print("placed; quiescent:", sim.sim.is_quiescent(), "non-air:", sim.sim.non_air_count())
    names = [s for s, _p in ppa.levers]
    lv = rs.Levers(sim, [p for _s, p in ppa.levers])

    if args.cases:
        rnd = random.Random(args.seed)
        mask = (1 << N) - 1
        cases = []
        for op in OPS:
            cases += [(op, 0, 0, 0), (op, mask, 1, 0), (op, mask, mask, 1),
                      (op, 0x55 & mask, 0xAA & mask, 0), (op, 1, mask, 0)]
            cases += [(op, rnd.getrandbits(N), rnd.getrandbits(N), rnd.getrandbits(1))
                      for _ in range(args.cases // 4)]
    else:
        cases = [(op, a, bb, c) for op in OPS for a in range(1 << N)
                 for bb in range(1 << N) for c in (0, 1)]
    print("checking %d cases" % len(cases))

    bad, sig_bad = 0, {}
    for op, A, B, cin in cases:
        want = model(N, op, A, B, cin)
        bits = [want.get(n, 0) if not n[1:].isdigit() or n[0] not in "AB"
                else ((A if n[0] == "A" else B) >> int(n[1:])) & 1 for n in names]
        lv.set(bits)
        got = {s: int(sim.on(*p)) for s, p in ppa.probe.items() if s in want}
        wrong = [s for s in got if got[s] != want[s]]
        result = sum(got["out%d" % i] << i for i in range(N))
        if result != expected_value(N, op, A, B, cin):
            bad += 1
            if bad <= 5:
                print("   WRONG %s A=%d B=%d cin=%d -> %d (want %d)"
                      % (op, A, B, cin, result, expected_value(N, op, A, B, cin)))
        for s in wrong:
            sig_bad[s] = sig_bad.get(s, 0) + 1
    print("ALU results correct: %d/%d" % (len(cases) - bad, len(cases)))
    if sig_bad:
        print("signals that disagreed (%d):" % len(sig_bad))
        for s, c in sorted(sig_bad.items(), key=lambda kv: -kv[1])[:12]:
            print("   %-12s wrong in %d/%d" % (s, c, len(cases)))
    if bad or sig_bad:
        return 1

    if args.out:
        lv.set([0] * len(names))          # save at rest, all levers off
        print("baked %d states" % bp.bake(b, sim))
        b.s.save_to_file(args.out)
        print("saved", args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
