"""4-bit register: DFF stamps at DFF_PITCH in z + clock chain by abutment.

Bit k's clk_out (10,1,5+7k) sits directly north-adjacent to bit k+1's clk_in
(10,1,6+7k): the clock connects by abutment alone, one refresh repeater per
cell.  Abutment rule (learned from the rca_cells repeater-ring root cause):
the boundary rows z=-1 and z=5 of every cell hold NOTHING but the x=10 clock
column, so no cell-internal net can couple across a seam.

Verification: fixed-tick clocked protocol -- random write/hold sequences,
never run_until_quiescent after the initial placement settle.
"""
import random

import nucleation as n
import nets
import drc
import rs
import seq_cells as sc
from seq_probe import run_gt, bake_states, reload_sim, ticks_to_quiescent

BITS = 4
# clock skew: one chain repeater per cell => 2 gt/bit; widen the functional
# pulse so every bit sees a full half-period
PULSE = sc.PULSE + 2 * BITS
SETUP = sc.SETUP


def build_register(frag, bits=BITS, pitch=sc.DFF_PITCH):
    b = rs.Build("register%d" % bits)
    labels = {}
    ports = []
    aliases = []
    for k in range(bits):
        rn = lambda s, k=k: "clk" if s == "clk" else "b%d.%s" % (k, s)
        ports.append(frag.stamp(b, labels, 0, 0, k * pitch, rename=rn))
    return b, labels, ports, aliases


def harness(b, ports):
    """D levers west of each bit, CLK lever north of bit0's clk_in."""
    d_levers = []
    for pk in ports:
        x, y, z = pk["d"]
        b.stone(x - 2, 0, z); b.force(x - 2, y, z, rs.LEVER_OFF)
        b.stone(x - 1, 0, z); b.put(x - 1, y, z, rs.DUST)
        d_levers.append((x - 2, y, z))
    x, y, z = ports[0]["clk_in"]
    b.stone(x, 0, z - 1); b.put(x, y, z - 1, rs.DUST)
    b.stone(x, 0, z - 2); b.force(x, y, z - 2, rs.LEVER_OFF)
    return d_levers, (x, y, z - 2)


def set_bits(sim, levers, states, bits):
    for i, lv in enumerate(levers):
        want = bool((bits >> i) & 1)
        if states[i] != want:
            sim.use(*lv)
            states[i] = want
            run_gt(sim, 1)


def read_q(sim, ports):
    return sum(int(sim.on(*pk["q"])) << k for k, pk in enumerate(ports))


def pulse(sim, clk):
    sim.use(*clk); run_gt(sim, PULSE)
    sim.use(*clk); run_gt(sim, PULSE)


def verify(save_baked=None):
    frag = sc.build_dff()
    b, labels, ports, aliases = build_register(frag)
    d_levers, clk = harness(b, ports)
    shorts = nets.check(b.cells, labels, aliases)
    assert not shorts, "shorts: %s" % shorts[:4]
    # A register has no feedback: its state lives in the DFFs' locked repeaters,
    # not in a conduction ring.  Any ring here is an accident.
    assert drc.check_rings("register%d" % BITS, b.cells), "diode ring"
    sim = b.sim()          # placement settle of the fresh, unclocked build
    ok = read_q(sim, ports) == 0
    print("register4: init Q=0000 %s" % ("PASS" if ok else "FAIL"))

    states = [False] * BITS
    random.seed(11)
    q_model = 0
    good = True
    for step in range(16):
        val = random.randint(0, 15)
        set_bits(sim, d_levers, states, val)
        run_gt(sim, SETUP)
        pulse(sim, clk)
        q_model = val
        got = read_q(sim, ports)
        if got != q_model:
            print("   write %2d: got %2d" % (val, got))
            good = False
        # hold: scramble D without a clock edge
        distract = random.randint(0, 15)
        set_bits(sim, d_levers, states, distract)
        run_gt(sim, 30)
        got = read_q(sim, ports)
        if got != q_model:
            print("   hold after write %2d (D=%2d): got %2d" % (val, distract, got))
            good = False
    print("register4: 16 random write/hold rounds %s"
          % ("PASS" if good else "FAIL"))
    ok = ok and good

    # baked initial state at Q=0: rebuild fresh, settle, bake, reload InWorld
    b2 = rs.Build("register%d_baked" % BITS)
    labels2 = {}
    ports2 = []
    for k in range(BITS):
        rn = lambda s, k=k: "clk" if s == "clk" else "b%d.%s" % (k, s)
        ports2.append(frag.stamp(b2, labels2, 0, 0, k * sc.DFF_PITCH, rename=rn))
    sim2 = b2.sim()
    baked = bake_states(b2, sim2)
    s3, raw = reload_sim(baked, b2.bounds()[0], n.TickSettleMode.InWorld)
    tq = ticks_to_quiescent(raw)
    bq = read_q(s3, ports2)
    bok = tq == 0 and bq == 0
    print("register4 baked Q=0: InWorld quiescent in %d gt, Q=%d %s"
          % (tq, bq, "PASS" if bok else "FAIL"))
    if save_baked:
        baked.save_to_file(save_baked)
        print("saved %s" % save_baked)
    return ok and bok


if __name__ == "__main__":
    import sys
    out = sys.argv[1] if len(sys.argv) > 1 else None
    print("seq_register4:", "ALL PASS" if verify(out) else "FAILURES")
