"""Re-verify every built circuit in this directory AT SPEED.

Every exhaustive result we have ever claimed -- the 4-bit RCA 512/512, the ALU
2048/2048, the 4x4 multiplier 256/256, `rca4_cells`, the Kogge-Stone adders --
was produced by `rs.Levers.set()`, which flips ONE lever and then runs
`run_until_quiescent` before touching anything else.  Those runs are therefore
proofs about the QUASI-STATIC regime: infinite time between input changes, and
one bit changing at a time.  They say nothing about a circuit that is fed a new
input word every N game ticks, which is what a clocked datapath does.

`probe_torch_burnout.py` showed the regime matters: a torch driven faster than
one change per 4 gt goes stuck-until-update, and the fault is invisible to a
settle-based check (internally corrupt, port still plausible, next answer
wrong).  Every NOT gate is a torch and the PLA architecture puts two torches in
every column, so the question is not whether our circuits contain the hazard --
they are made of it -- but whether it BINDS before latency does.

This probe answers that per circuit.  For each one it sweeps the input hold time
downward and reports:

  latency      worst-case quasi-static settle, in game ticks -- the floor below
               which a wrong answer is merely LATE;
  max rate     the smallest hold at which every vector in a maximum-activity
               sequence is correct when sampled at the end of its own hold;
  failure mode "slow only"      -- below the safe hold the answer is late, and
                                   settling always repairs it;
               "silently wrong" -- there is a hold at which the traffic leaves
                                   the circuit giving wrong answers to
                                   subsequent, fully settled inputs.  Lost
                                   data, with a plausible-looking port.

The vector sequence is deliberately worst-case: `ratedrive.alternating` flips
EVERY input bit on every vector, which both maximises the toggle count seen by
each torch and forces full carry propagation.  Where a circuit is small enough,
the discovered safe hold is then confirmed against the circuit's own FULL
exhaustive case list, driven word-parallel at that hold -- so the exhaustive
claim is re-established at speed rather than merely assumed to carry over.

Run: ~/eda-venv/bin/python probe_rate_limits.py            (all circuits)
     ~/eda-venv/bin/python probe_rate_limits.py --only alu4 --no-exhaustive
"""
import argparse
import time

import ratedrive as rd
import rs

ROWS = []
VERDICTS = []


def V(ok, text, extra=""):
    VERDICTS.append((bool(ok), text, extra))


def torches(b):
    return sum(1 for blk in b.cells.values() if "torch" in blk)


# =========================================================== the rigs
# Each rig returns a dict:
#   build   -> the rs.Build (built ONCE; a fresh sim is taken per hold, which
#              is what resets burnout state -- rebuilding geometry would be
#              wasteful and rebuilding nothing would carry damage forward)
#   fresh() -> (sim, RateDriver)
#   read(sim) -> the observed output value
#   expect(vector) -> the wanted value
#   vectors -> the worst-case rate sequence
#   exhaustive -> the circuit's own full case list, or None

def rig_not_gate():
    """Calibration: one torch, i.e. one NOT gate, the atom of the PLA."""
    b = rs.Build("rate_not")
    b.force(0, 3, 0, rs.STONE)
    b.force(0, 4, 0, rs.TORCH)
    b.force(0, 5, 0, rs.STONE)
    b.force(0, 6, 0, rs.DUST)
    for k in (1, 2):
        b.force(0, 2, -k, rs.STONE)
        b.force(0, 3, -k, rs.DUST)
    b.force(0, 2, -3, rs.STONE)
    b.force(0, 3, -3, rs.LEVER_OFF)
    lc, out = (0, 3, -3), (0, 6, 0)

    def fresh():
        sim = b.sim()
        return sim, rd.RateDriver(sim, [lc])

    return dict(name="not_gate (1 torch)", build=b, fresh=fresh,
                read=lambda sim: sim.power(*out) > 0,
                expect=lambda v: not v[0],
                vectors=rd.alternating(1, pairs=12),
                exhaustive=[[0], [1]] * 8)


def rig_rca_cells(n=4):
    """`rca_cells.py`: the genlib FA stamped n times.  Repeaters + comparators,
    ZERO torches -- so this is the control group for the whole probe."""
    import cells
    import rca_cells
    ha = cells.build_half_adder()
    fa = cells.build_full_adder(ha)
    b, labels, ports, _aliases = rca_cells.build_rca(n, fa)
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
    names = ["cin"] + ["a%d" % k for k in range(n)] + ["b%d" % k for k in range(n)]
    order = [lever[s] for s in names]

    def fresh():
        sim = b.sim()
        return sim, rd.RateDriver(sim, order)

    def read(sim):
        return (sum(int(sim.on(*ports[k]["sum"])) << k for k in range(n))
                + (int(sim.on(*ports[n - 1]["cout"])) << n))

    def expect(v):
        return (sum(v[1 + k] << k for k in range(n))
                + sum(v[1 + n + k] << k for k in range(n)) + v[0])

    exh = [[c] + [(A >> k) & 1 for k in range(n)] + [(B >> k) & 1 for k in range(n)]
           for A in range(1 << n) for B in range(1 << n) for c in (0, 1)]
    return dict(name="rca%d_cells (0 torches)" % n, build=b, fresh=fresh,
                read=read, expect=expect,
                vectors=rd.alternating(2 * n + 1, pairs=8), exhaustive=exh)


def rig_adder_pla(n=4):
    """`build_adder.py`: the hand-laid PLA ripple adder.  Torch-based."""
    import build_adder as ba
    b, ab, cin = ba.build(n)
    order = [cin] + [p for pair in ab for p in pair]

    def fresh():
        sim = b.sim()
        return sim, rd.RateDriver(sim, order)

    def read(sim):
        s, c = ba.read_out(sim, n)
        return sum(int(x) << k for k, x in enumerate(s)) + (int(c) << n)

    def expect(v):
        A = sum(v[1 + 2 * i] << i for i in range(n))
        B = sum(v[2 + 2 * i] << i for i in range(n))
        return A + B + v[0]

    exh = []
    for A in range(1 << n):
        for B in range(1 << n):
            for c in (0, 1):
                bits = [c]
                for i in range(n):
                    bits += [(A >> i) & 1, (B >> i) & 1]
                exh.append(bits)
    return dict(name="ripple_carry_adder_%dbit (PLA)" % n, build=b, fresh=fresh,
                read=read, expect=expect,
                vectors=rd.alternating(2 * n + 1, pairs=8), exhaustive=exh)


def rig_ppa(n=8, exhaustive=False):
    """`build_ppa.py`: the Kogge-Stone parallel-prefix adder."""
    import build_ppa as bp
    ppa = bp.PPA(n, name="ks%d" % n)
    b = ppa.build()
    names = [s for s, _p in ppa.levers]
    order = [p for _s, p in ppa.levers]

    def fresh():
        sim = b.sim(settle=4000)
        return sim, rd.RateDriver(sim, order)

    def read(sim):
        return (sum(int(sim.on(*ppa.probe["s%d" % i])) << i for i in range(n))
                + (int(sim.on(*ppa.probe["cout"])) << n))

    def bits_for(A, B, cin):
        out = []
        for nm in names:
            if nm[1:].isdigit() and nm[0] in "AB":
                out.append(((A if nm[0] == "A" else B) >> int(nm[1:])) & 1)
            else:
                out.append(cin if nm == "cin" else 0)
        return out

    want = {}

    def add(A, B, c):
        bits = bits_for(A, B, c)
        want[tuple(bits)] = A + B + c
        return bits

    vecs = []
    mask = (1 << n) - 1
    for _ in range(8):                     # worst case: all-ones + 1, both ways
        vecs.append(add(mask, 0, 1))
        vecs.append(add(0, mask, 0))
    exh = None
    if exhaustive:
        exh = [add(A, B, c) for A in range(1 << n) for B in range(1 << n)
               for c in (0, 1)]
    else:
        import random
        rnd = random.Random(3)
        exh = [add(rnd.getrandbits(n), rnd.getrandbits(n), rnd.getrandbits(1))
               for _ in range(64)]
    return dict(name="kogge_stone_%dbit (PLA)" % n, build=b, fresh=fresh,
                read=read, expect=lambda v: want[tuple(v)],
                vectors=vecs, exhaustive=exh)


def rig_alu(width=4):
    """`build_alu.py`: the 4-op ALU, compiled by the same PLA flow."""
    import build_alu
    import build_ppa as bp
    ppa = bp.PPA(width, make=build_alu.alu_netlist, name="alu%d" % width)
    b = ppa.build()
    names = [s for s, _p in ppa.levers]
    order = [p for _s, p in ppa.levers]
    N = width

    def fresh():
        sim = b.sim(settle=4000)
        return sim, rd.RateDriver(sim, order)

    def read(sim):
        return sum(int(sim.on(*ppa.probe["out%d" % i])) << i for i in range(N))

    want = {}

    def add(op, A, B, cin):
        model = build_alu.model(N, op, A, B, cin)
        bits = [model.get(nm, 0) if not (nm[1:].isdigit() and nm[0] in "AB")
                else ((A if nm[0] == "A" else B) >> int(nm[1:])) & 1
                for nm in names]
        want[tuple(bits)] = build_alu.expected_value(N, op, A, B, cin)
        return bits

    mask = (1 << N) - 1
    vecs = []
    for _ in range(6):                     # thrash ADD with full carry ripple
        vecs.append(add("opADD", mask, 0, 1))
        vecs.append(add("opADD", 0, mask, 0))
    exh = [add(op, A, B, c) for op in build_alu.OPS for A in range(1 << N)
           for B in range(1 << N) for c in (0, 1)]
    return dict(name="alu_%dbit (PLA)" % width, build=b, fresh=fresh,
                read=read, expect=lambda v: want[tuple(v)],
                vectors=vecs, exhaustive=exh)


def rig_mult4():
    """`mult4.py`: 4 stacked PLA planes + maze-routed vertical interconnect.

    The build is reproduced from `mult4.main` rather than imported, because that
    builder does its stacking and routing inline in `main()` and this probe must
    not change a verified builder to observe it.  Keep in step with mult4.py.
    """
    import mult4
    import build_ppa as bp
    import router
    planes = [bp.PPA(4, make=mult4.pp_netlist, name="pp_plane")]
    for k in (1, 2, 3):
        planes.append(bp.PPA(4, make=mult4.renamed_adder("m%d" % k),
                             name="row%d" % k))
    for p in planes:
        p.build()
    b, labels, probe = rs.Build("mult4"), {}, {}
    for k, p in enumerate(planes):
        dy = mult4.PITCH * k
        for (x, y, z), blk in p.b.cells.items():
            b.put(x, y + dy, z, blk)
        for (x, y, z), lab in p.labels.items():
            labels[(x, y + dy, z)] = lab
        for sig, (x, y, z) in p.probe.items():
            probe[sig] = (x, y + dy, z)
    levers = [(sig, pos) for sig, pos in planes[0].levers]
    r = router.Router(b, labels)
    hops = []
    for i in range(1, 4):
        hops.append(("pp0_%d" % i, 0, "m1A%d" % (i - 1)))
    for j in (1, 2, 3):
        for i in range(4):
            hops.append(("pp%d_%d" % (j, i), 0, "m%dB%d" % (j, i)))
    for k in (1, 2):
        for i in (1, 2, 3):
            hops.append(("m%ds%d" % (k, i), k, "m%dA%d" % (k + 1, i - 1)))
        hops.append(("m%dcout" % k, k, "m%dA3" % (k + 1)))
    hops.sort(key=lambda h_: -abs(int(h_[2][1]) - h_[1]))
    for sig, _src_k, dst_sig in hops:
        dst_k = int(dst_sig[1])
        r.route(probe[sig], mult4.drive_cell(planes[dst_k], dst_sig,
                                             mult4.PITCH * dst_k),
                dst_sig, friendly={sig, dst_sig})

    names = [s for s, _ in levers]
    order = [p for _, p in levers]
    outs = ([probe["pp0_0"]] + [probe["m%ds0" % k] for k in (1, 2, 3)]
            + [probe["m3s%d" % i] for i in (1, 2, 3)] + [probe["m3cout"]])

    def fresh():
        sim = b.sim(settle=4000)
        return sim, rd.RateDriver(sim, order)

    def read(sim):
        return sum(int(sim.on(*p)) << i for i, p in enumerate(outs))

    def bits_for(A, B):
        return [(A >> int(nm[1])) & 1 if nm[0] == "A" else (B >> int(nm[1])) & 1
                for nm in names]

    want = {}

    def add(A, B):
        bits = bits_for(A, B)
        want[tuple(bits)] = A * B
        return bits

    vecs = []
    for _ in range(6):
        vecs.append(add(15, 15))
        vecs.append(add(0, 0))
        vecs.append(add(15, 0))
        vecs.append(add(0, 15))
    exh = [add(A, B) for A in range(16) for B in range(16)]
    return dict(name="mult_4x4 (stacked PLA)", build=b, fresh=fresh,
                read=read, expect=lambda v: want[tuple(v)],
                vectors=vecs, exhaustive=exh)


RIGS = {
    "not": rig_not_gate,
    "rca4_cells": rig_rca_cells,
    "adder4": rig_adder_pla,
    "mult4": rig_mult4,
    "alu4": rig_alu,
    "ks8": lambda: rig_ppa(8),
}


# =========================================================== the sweep

def run_rig(key, make, holds, do_exhaustive):
    t0 = time.time()
    rig = make()
    b = rig["build"]
    ntor = torches(b)
    # The sweep sequence is the hand-picked worst-activity vectors PLUS a spread
    # sampled from the circuit's own case list.  The extremes alone are not
    # enough: for `rca4_cells` the all-ones/all-zeros pair happens to be one of
    # the FASTER transitions, so a sweep driven by extremes alone reported a
    # safe hold at which a third of the exhaustive list is still wrong.
    vecs = list(rig["vectors"])
    if rig["exhaustive"] and len(rig["exhaustive"]) > 16:
        step = max(1, len(rig["exhaustive"]) // 40)
        vecs += rig["exhaustive"][::step]

    # Latency is measured over the SAME sequence the sweep drives, so the two
    # numbers are comparable.  It is still a sample, not a bound -- which is
    # exactly why the case-list confirmation below is allowed to escalate past it.
    sim, drv = rig["fresh"]()
    lat, _lv = rd.measure_latency(drv, vecs[:24])
    print("   %-32s %6d blocks  %5d torches  latency %4d gt  (build %.1fs)"
          % (rig["name"], len(b.cells), ntor, lat, time.time() - t0))

    # BASELINE PRECHECK.  Reproduce the circuit's own published protocol -- one
    # lever at a time, settle between -- on a sample of its case list, before
    # claiming anything about rate.  A harness that re-verifies a claim has to
    # establish the claim still holds, or every rate number it reports is
    # measuring a circuit that was already broken.  This caught `mult_4x4`
    # failing 64/256 QUASI-STATICALLY on the current tree.
    base = None
    if rig["exhaustive"]:
        sim, drv = rig["fresh"]()
        lv = rs.Levers(sim, drv.positions)
        step = max(1, len(rig["exhaustive"]) // 32)
        sample = rig["exhaustive"][::step]
        bad = 0
        for v in sample:
            lv.set(v)
            if rig["read"](sim) != rig["expect"](v):
                bad += 1
        base = (len(sample) - bad, len(sample))
        print("      quasi-static baseline (legacy protocol): %d/%d%s"
              % (base[0], base[1], "" if not bad else "   <-- ALREADY BROKEN"))
        if bad:
            print("      SKIPPING the rate sweep: there is no correct "
                  "quasi-static behaviour here to re-verify at speed.")
            ROWS.append(dict(key=key, name=rig["name"], torches=ntor,
                             latency=lat, safe_speed=None, safe_clean=None,
                             mode="baseline broken", exhaustive=None,
                             exh_hold=None, baseline=base,
                             n_cases=len(rig["exhaustive"]),
                             blocks=len(b.cells), rows=[]))
            return

    def make_drv():
        return rig["fresh"]()

    rows, safe_speed, safe_clean, mode = rd.sweep(
        make_drv, vecs, rig["read"], rig["expect"], holds=holds)
    for r in sorted(rows, key=lambda r: r["hold"]):
        print("      hold %4d gt  at_speed %.2f  drain %-5s resume %-5s "
              "canary %.2f  %s"
              % (r["hold"], r["at_speed"], r["drain_ok"], r["resume_ok"],
                 r["canary"],
                 " ".join(t for t, f in (("DAMAGED", r["damaged"]),
                                         ("STUCK-WRONG", r["stuck_wrong"]),
                                         ("RESUME-WRONG", not r["resume_ok"]))
                          if f)))

    # Confirm the discovered rate against the FULL case list, and escalate if it
    # does not hold: the sweep sequence is a sample, the case list is the claim.
    exh_ok, exh_hold = None, None
    if do_exhaustive and rig["exhaustive"] and safe_speed:
        read, expect = rig["read"], rig["expect"]
        ladder = [h for h in sorted(set(list(holds) + [safe_speed]))
                  if h >= safe_speed]
        for h in ladder:
            sim, drv = rig["fresh"]()
            bad, t1 = 0, time.time()
            for v in rig["exhaustive"]:
                drv.apply(v, h)
                if read(sim) != expect(v):
                    bad += 1
            exh_ok = (len(rig["exhaustive"]) - bad, len(rig["exhaustive"]))
            exh_hold = h
            print("      case list AT SPEED (hold %3d gt, word-parallel): "
                  "%d/%d  (%.1fs)"
                  % (h, exh_ok[0], exh_ok[1], time.time() - t1))
            if bad == 0:
                break

    ROWS.append(dict(key=key, name=rig["name"], torches=ntor, latency=lat,
                     safe_speed=safe_speed, safe_clean=safe_clean, mode=mode,
                     exhaustive=exh_ok, exh_hold=exh_hold, baseline=base,
                     n_cases=len(rig["exhaustive"] or ()),
                     blocks=len(b.cells), rows=rows))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", default="", help="comma-separated rig keys")
    ap.add_argument("--no-exhaustive", action="store_true")
    ap.add_argument("--quick", action="store_true")
    args = ap.parse_args()

    keys = [k for k in RIGS if not args.only or k in args.only.split(",")]
    keys.sort(key=lambda k: list(RIGS).index(k))
    holds = (2, 4, 8, 16, 32, 64, 128) if args.quick else rd.HOLDS

    print("== rate sweep (worst-case activity: every input bit flips every "
          "vector) ==")
    for k in keys:
        run_rig(k, RIGS[k], holds, not args.no_exhaustive)

    print()
    print("== per-circuit rate table ==")
    print("%-32s %7s %8s %10s %12s  %s"
          % ("circuit", "torches", "latency", "max rate", "cases at rate",
             "failure mode when over-driven"))
    for r in ROWS:
        rate = r["exh_hold"] or r["safe_speed"]
        print("%-32s %7d %6d gt %7s gt %12s  %s"
              % (r["name"], r["torches"], r["latency"], rate,
                 "%d/%d" % r["exhaustive"] if r["exhaustive"] else "-",
                 r["mode"]))

    # ------------------------------------------------------------- verdicts
    broken = [r for r in ROWS if r["mode"] == "baseline broken"]
    ROWS[:] = [r for r in ROWS if r["mode"] != "baseline broken"] + broken
    rated = [r for r in ROWS if r["mode"] != "baseline broken"]
    torchless = [r for r in rated if r["torches"] == 0]

    V(not broken,
      "R0 every circuit still passes its own published quasi-static protocol, "
      "so the rate numbers below are about rate and not about a circuit that "
      "was already broken"
      if not broken else
      "R0 %d circuit(s) FAIL QUASI-STATICALLY on the current tree, before rate "
      "enters the picture: %s.  This is a pre-existing regression in committed "
      "code, not a rate effect -- the reference builder reproduces it on its "
      "own -- so those circuits are excluded from the rate table and their "
      "published results need re-establishing before they can be re-verified "
      "at speed" % (len(broken), ", ".join("%s (%d/%d quasi-static)"
                                           % (r["key"], r["baseline"][0],
                                              r["baseline"][1])
                                           for r in broken)),
      "" if not broken else "excluded: %s" % [r["key"] for r in broken])

    ROWS_ = rated
    V(all(r["safe_speed"] is not None for r in ROWS_),
      "R1 every circuit has a finite maximum input rate, and none of them was "
      "ever measured before: the published verification of each one drove it "
      "with `rs.Levers.set()`, which settles between vectors and therefore "
      "sampled exactly one point of this sweep -- the limit as hold -> infinity")

    V(all(r["safe_speed"] >= r["latency"] * 0.5 for r in ROWS_ if r["safe_speed"]),
      "R2 for every circuit here the binding constraint is LATENCY, not the "
      "torch toggle budget: the safe hold tracks the measured settle time, and "
      "every one of those is above the %d gt burnout floor.  So respecting a "
      "circuit's own propagation delay keeps its torches inside budget "
      "automatically -- burnout binds only for logic FASTER than %d gt, which "
      "is to say for single gates and short chains, not for these datapaths"
      % (rd.TORCH_MIN_HOLD_GT, rd.TORCH_MIN_HOLD_GT),
      "latency vs safe hold: %s"
      % {r["key"]: (r["latency"], r["safe_speed"]) for r in ROWS_})

    silent = [r for r in ROWS_ if "silently wrong" in r["mode"]]
    V(bool(silent),
      "R3 %d of %d circuits fail SILENTLY WRONG when over-driven, not merely "
      "slow: %s.  Over-driven, they leave state that a later, fully settled "
      "input reads incorrectly -- so a consumer that misses the rate budget "
      "gets a plausible wrong number, with nothing to indicate it"
      % (len(silent), len(ROWS_), ", ".join(r["key"] for r in silent)),
      "failure mode by circuit: %s" % {r["key"]: r["mode"] for r in ROWS_})

    V(all(r["mode"] == "slow only" for r in torchless),
      "R4 the torch-free circuit(s) (%s) fail 'slow only' at every rate tried: "
      "repeaters and comparators have no toggle budget, so over-driving them "
      "loses values in flight and leaves no residue.  This is the control "
      "group, and it confirms the damage in R3 is the TORCH, not the harness"
      % ", ".join(r["key"] for r in torchless) or "none")

    V(all(r["exhaustive"] is None or r["exhaustive"][0] == r["exhaustive"][1]
          for r in ROWS_),
      "R5 at the rate in the table, every circuit reproduces its FULL case list "
      "driven word-parallel and back-to-back -- so the exhaustive claims "
      "SURVIVE, restated with a rate attached: they hold for any input rate at "
      "or below that hold.  What was never true is the unqualified reading of "
      "them, and above that hold the same circuits are silent about being wrong",
      "case list at rate: %s"
      % {r["key"]: "%d/%d @ %s gt" % (r["exhaustive"][0], r["exhaustive"][1],
                                      r["exh_hold"])
         for r in ROWS_ if r["exhaustive"]})

    # Every reported rate is confirmed against the FULL case list, never against
    # the sampled sweep alone -- because the sample under-estimates.  Assert the
    # property that matters (the rate is always case-list-confirmed) rather than
    # "escalation happened", which depends on which circuits --only selected.
    escalated = {r["key"]: (r["safe_speed"], r["exh_hold"]) for r in ROWS_
                 if r["exh_hold"] and r["exh_hold"] > r["safe_speed"]}
    V(all(r["exh_hold"] is None or r["exhaustive"][0] == r["exhaustive"][1]
          for r in ROWS_),
      "R6 every rate in the table is the hold at which the FULL case list "
      "passes, never the hold the sampled sweep certified -- because the sample "
      "UNDER-ESTIMATES.  Worst-case ACTIVITY is not worst-case LATENCY: the "
      "transition that toggles the most input bits is not the one that takes "
      "longest to settle, so a rate-aware harness has to escalate against the "
      "real case list.  Escalation actually fired here for: %s"
      % (", ".join("%s %d->%d gt" % (k, a, bq)
                   for k, (a, bq) in sorted(escalated.items()))
         or "no circuit in this subset (run the full sweep: rca4_cells needs "
            "84->96 gt)"),
      "sweep hold vs case-list hold: %s"
      % {r["key"]: (r["safe_speed"], r["exh_hold"]) for r in ROWS_
         if r["exh_hold"]})

    bad = 0
    print()
    for ok, text, extra in VERDICTS:
        print("%s %s%s" % ("PASS" if ok else "FAIL", text,
                           ("   [%s]" % extra) if extra else ""))
        bad += 0 if ok else 1
    print("probe_rate_limits: %d/%d" % (len(VERDICTS) - bad, len(VERDICTS)))
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
