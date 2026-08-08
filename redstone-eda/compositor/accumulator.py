"""Clocked 4-bit ACCUMULATOR, composed with the Compositor MVP.

    Q -> adder.a   (feedback corridors, counter4 geometry)
    B -> adder.b   (external 4-bit bus, west levers, flyover over the
                    feedback columns)
    sum -> D       (north escape corridor)
    shared clock   (x38 column, chained by gap dust + one repeater)

Every clock edge: Q <- (Q + B) mod 16.  cin is left low (no torch).

Verification: alias-aware nets.check + audit, placement settle to Q=0/D=0,
reset-by-bake (InWorld reload quiescent in 0 gt at Q=0), then >=20 clocked
random-B steps checking the running sum mod 16 at every step, on the
RELOADED baked sim (deployment conditions).  Bridge DRC + LVS reported.
Clock: HIGH 30 / LOW 110 gt (counter4's measured min period is 100 gt for
the same loop; DFF cell alone is 20 gt -- seq_README characterization).
"""
import json
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

import nucleation as n            # noqa: E402
import cells                      # noqa: E402
import rs                         # noqa: E402
import seq_cells as sc            # noqa: E402
from seq_probe import run_gt, reload_sim, ticks_to_quiescent  # noqa: E402
from compositor import Compositor  # noqa: E402

BITS = 4
FA_P = cells.PITCH                # 13
REG_X = 28
COL_X = {k: -8 + 2 * k for k in range(BITS)}    # Q->a feedback columns
HIGH, LOW = 30, 110
SETTLE_B = 100                    # gt after B changes before the edge: lever
                                  # flips launch carry-chain glitches that
                                  # need ~60-80 gt to wash out (measured)


def build():
    ha = cells.build_half_adder()
    fa = cells.build_full_adder(ha)
    dff = sc.build_dff()
    c = Compositor("accumulator4")
    fains, regs = [], []
    for k in range(BITS):
        fains.append(c.add("fa%d" % k, fa, (0, 0, FA_P * k)))
        regs.append(c.add("r%d" % k, dff, (REG_X, 0, FA_P * k + 8),
                          shared=("clk",)))
    for k in range(BITS - 1):     # carry chain connects by ABUTMENT
        assert c.connect(fains[k].ref("cout"), fains[k + 1].ref("cin")) == "abut"
    for k in range(BITS):         # feedback nets: realized as corridors below
        c.alias("fa%d.sum" % k, "r%d.d" % k)
        c.alias("r%d.q" % k, "fa%d.a" % k)

    # clock: lever north of bit0 clk_in; gap dust + repeater between cells
    c.dust(38, 1, 6, "clk")
    c.b.stone(38, 0, 5)
    c.b.force(38, 1, 5, rs.LEVER_OFF)
    clk_lever = (38, 1, 5)
    for k in range(BITS - 1):
        z = FA_P * k
        for zz in (14, 15, 16, 18, 19):
            c.dust(38, 1, z + zz, "clk")
        c.rep(38, 1, z + 17, "north")

    b_levers = []
    for k in range(BITS):
        z = FA_P * k
        s_net, q_net = "r%d.d" % k, "fa%d.a" % k

        # ---- sum -> D (guards sever diagonals into the FA cout flyover)
        c.guard(19, 2, z + 7)
        c.guard(20, 2, z + 6)
        c.guard(21, 2, z + 6)
        c.dust(19, 1, z + 7, s_net, "lane")
        for x in (19, 20, 21, 22, 23):
            c.dust(x, 1, z + 6, s_net, "lane")
        c.rep(24, 1, z + 6, "west")
        for x in (25, 26, 27):
            c.dust(x, 1, z + 6, s_net, "lane")
        c.dust(27, 1, z + 7, s_net, "lane")
        c.dust(27, 1, z + 8, s_net, "lane")

        # ---- Q -> a feedback corridor (counter4 geometry, verified)
        c.dust(41, 1, z + 8, q_net, "tap")
        for zz in (9, 10, 11):
            c.dust(41, 1, z + zz, q_net, "tap")
        c.rep(41, 1, z + 12, "north")
        for zz in (13, 14, 15):
            c.dust(41, 1, z + zz, q_net, "tap")
        c.dust(40, 1, z + 15, q_net, "tap")
        c.b.stone(39, 1, z + 15, "tap"); c.dust(39, 2, z + 15, q_net, floor=False)
        c.guard(38, 2, z + 15)                       # cap clock gap
        c.dust(38, 3, z + 15, q_net, floor=False)
        c.b.stone(37, 1, z + 15, "tap"); c.dust(37, 2, z + 15, q_net, floor=False)
        c.dust(36, 1, z + 15, q_net, "tap")
        c.rep(35, 1, z + 15, "east")
        for x in range(34, 23, -1):
            c.dust(x, 1, z + 15, q_net, "tap")
        c.dust(24, 1, z + 14, q_net, "tap")
        c.rep(24, 1, z + 13, "south")
        c.dust(24, 1, z + 12, q_net, "tap")
        c.dust(23, 1, z + 12, q_net, "tap")
        c.b.stone(22, 1, z + 12, "tap"); c.dust(22, 2, z + 12, q_net, floor=False)
        c.guard(21, 2, z + 12)                       # cap carry column
        c.dust(21, 3, z + 12, q_net, floor=False)
        c.b.stone(20, 1, z + 12, "tap"); c.dust(20, 2, z + 12, q_net, floor=False)
        for x in range(19, 12, -1):
            c.dust(x, 1, z + 12, q_net, "tap")
        c.rep(12, 1, z + 12, "east")
        for x in range(11, 4, -1):
            c.dust(x, 1, z + 12, q_net, "tap")
        c.guard(4, 2, z + 13)                        # guard xor diagonal
        c.b.stone(4, 1, z + 12, "tap"); c.dust(4, 2, z + 12, q_net, floor=False)
        for x in (3, 2, 1, 0):
            c.b.stone(x, 2, z + 12, "tap")
            c.dust(x, 3, z + 12, q_net, floor=False)
        c.b.stone(-1, 1, z + 12, "tap"); c.dust(-1, 2, z + 12, q_net, floor=False)
        c.dust(-1, 1, z + 11, q_net, "tap")
        xk = COL_X[k]
        c.dust(-2, 1, z + 11, q_net, "tap")
        if xk < -2:
            c.rep(-3, 1, z + 11, "east")
            for x in range(-4, xk - 1, -1):
                c.dust(x, 1, z + 11, q_net, "tap")
        c.rep(xk, 1, z + 10, "south")
        for zz in range(z + 9, z - 1, -1):
            c.dust(xk, 1, zz, q_net, "tap")
        entry_rep_x = xk + 1 if xk >= -4 else -4
        for x in range(xk + 1, 0):
            if x == entry_rep_x:
                c.rep(x, 1, z, "west")
            else:
                c.dust(x, 1, z, q_net, "tap")
        if k:   # guard: bit k-1's descent dust diagonally over this entry
            c.guard(-1, 2, z)

        # ---- external B feed: lever west, flyover OVER the Q->a column xk
        b_net = "fa%d.b" % k
        c.b.stone(xk - 6, 0, z + 4)
        c.b.force(xk - 6, 1, z + 4, rs.LEVER_OFF)
        b_levers.append((xk - 6, 1, z + 4))
        c.dust(xk - 5, 1, z + 4, b_net, "lane")
        c.dust(xk - 4, 1, z + 4, b_net, "lane")
        c.rep(xk - 3, 1, z + 4, "west")
        c.dust(xk - 2, 1, z + 4, b_net, "lane")
        c.b.stone(xk - 1, 1, z + 4, "lane"); c.dust(xk - 1, 2, z + 4, b_net, floor=False)
        c.guard(xk, 2, z + 4)                        # cap the feedback column
        c.dust(xk, 3, z + 4, b_net, floor=False)
        c.b.stone(xk + 1, 1, z + 4, "lane"); c.dust(xk + 1, 2, z + 4, b_net, floor=False)
        if xk + 2 < 0:                               # descend + refresh
            c.dust(xk + 2, 1, z + 4, b_net, "lane")
            if xk + 3 < 0:
                c.rep(xk + 3, 1, z + 4, "west")
            for x in range(xk + 4, 0):
                c.dust(x, 1, z + 4, b_net, "lane")
    return c, fains, regs, clk_lever, b_levers


def read_q(sim, regs):
    return sum(int(sim.on(*r.ports["q"])) << k for k, r in enumerate(regs))


def set_b(sim, b_levers, states, val):
    for i, lv in enumerate(b_levers):
        want = bool((val >> i) & 1)
        if states[i] != want:
            sim.use(*lv)
            states[i] = want
            run_gt(sim, 1)


def pulse(sim, clk, high=HIGH, low=LOW):
    sim.use(*clk); run_gt(sim, high)
    sim.use(*clk); run_gt(sim, low)


def run_steps(sim, regs, clk, b_levers, states, seq, high=HIGH, low=LOW,
              settle_b=SETTLE_B):
    """Clock the B sequence in; returns (ok, mc-tick Q per step)."""
    acc, got = read_q(sim, regs), []
    ok = True
    for bval in seq:
        set_b(sim, b_levers, states, bval)
        run_gt(sim, settle_b)
        pulse(sim, clk, high, low)
        acc = (acc + bval) % 16
        q = read_q(sim, regs)
        got.append(q)
        if q != acc:
            ok = False
    return ok, got


def main():
    c, fains, regs, clk, b_levers = build()
    print("accumulator4: %d blocks, %d instances" % (len(c.b.cells), len(c.insts)))
    clean, problems, shorts = c.check()
    print("audit+nets.check (alias-aware): %s (%d shorts)"
          % ("CLEAN" if clean else "DIRTY", len(shorts)))
    if not clean:
        return False

    sim = c.sim(settle=800)
    q0 = read_q(sim, regs)
    d0 = sum(int(sim.on(*r.ports["d"])) << k for k, r in enumerate(regs))
    print("settled: Q=%d D=%d (want 0/0)" % (q0, d0))
    ok = q0 == 0 and d0 == 0

    out = os.path.join(os.path.dirname(HERE), "showcase", "accumulator4.schem")
    baked = c.bake(sim, out)
    print("saved %s" % out)

    # ---- reset-by-bake: reload the baked artifact InWorld, prove Q=0/0gt
    s2, raw = reload_sim(baked, c.b.bounds()[0], n.TickSettleMode.InWorld)
    tq = ticks_to_quiescent(raw)
    bq = read_q(s2, regs)
    print("baked reload (InWorld): quiescent in %d gt, Q=%d %s"
          % (tq, bq, "PASS" if (tq == 0 and bq == 0) else "FAIL"))
    ok = ok and tq == 0 and bq == 0

    # ---- >=20 random-B clocked steps on the RELOADED sim
    random.seed(20260808)
    seq = [random.randint(0, 15) for _ in range(24)]
    states = [bool(s2.powered(*lv)) for lv in b_levers]
    good, got = run_steps(s2, regs, clk, b_levers, states, seq)
    model = []
    acc = 0
    for v in seq:
        acc = (acc + v) % 16
        model.append(acc)
    print("24 clocked accumulate steps (period %d gt): %s"
          % (HIGH + LOW, "PASS" if good else "FAIL"))
    ok = ok and good
    with open(os.path.join(HERE, "accumulator_trace.json"), "w") as f:
        json.dump({"B": seq, "Q_mc_tick": got, "Q_model": model,
                   "high_gt": HIGH, "low_gt": LOW, "settle_b_gt": SETTLE_B}, f)

    # ---- measure the required B-settle (external-input setup, gt)
    min_settle = None
    for sb in (20, 40, 60, 80, 100):
        s3, _ = reload_sim(baked, c.b.bounds()[0], n.TickSettleMode.InWorld)
        st3 = [bool(s3.powered(*lv)) for lv in b_levers]
        g3, _ = run_steps(s3, regs, clk, b_levers, st3, seq[:8], settle_b=sb)
        print("   B-settle %3d gt (8 steps): %s" % (sb, "PASS" if g3 else "FAIL"))
        if g3 and min_settle is None:
            min_settle = sb
    print("min B-settle: %s gt (feedback-loop min period stays 100 gt, "
          "counter4 measurement)" % min_settle)
    ok = ok and min_settle is not None

    # ---- bridge analyses (informational for a sequential design)
    drc = c.drc()
    kinds = {}
    for v in drc:
        kinds[v["kind"]] = kinds.get(v["kind"], 0) + 1
    hard = sum(kinds.get(k, 0) for k in ("short", "floating", "unattached_wall_torch"))
    print("bridge DRC: %s (repeater_cycle=%d is the DFF/feedback storage)"
          % (kinds, kinds.get("repeater_cycle", 0)))
    ok = ok and hard == 0
    # LVS: conduction nets merge THROUGH components (probed), so between
    # logical nets "shorts" enumerate intended gate edges and "cycles" the
    # sequential loops; the wiring check is opens == 0 (every intent net
    # conducts end-to-end).
    lvs = c.lvs()
    opens = len(lvs.get("opens", []))
    print("bridge LVS: opens=%d (wiring intact) | gate-edge 'shorts'=%d, "
          "sequential cycles=%d" % (opens, len(lvs.get("shorts", [])),
                                    len(lvs.get("cycles", []))))
    ok = ok and opens == 0
    return ok


if __name__ == "__main__":
    print("accumulator4:", "ALL PASS" if main() else "FAILURES")
