"""4-bit synchronous counter: register + the existing FA cell column as the
increment (b=0, cin tied high by a torch), feedback Q -> a and sum -> D.
This closes the sequential loop.

Geometry (z' = 13k per bit):
  FA cells   x0..21,  z' + (-1..11)   (rca_cells stamping, carry by abutment)
  register   x28..40, z' + (7..13)    (DFF v2 stamps, data row at z'+8 = the
                                       FA sum row; clock chain x38 with gap
                                       dust + one repeater between cells)
  sum -> D   escapes NORTH of the sum port (the FA's own cout flyover owns
             y2..5 over z'+7..8, x16..21), runs east along z'+6 and drops
             into D from the west.  Three GUARD blocks sever the diagonal
             dust connections to the cout flyover (a guard above a straight
             flat run is legal capping).
  Q -> a     leaves Q east (x41), runs south to z'+15, flies OVER the clock
             gap column (cap on x38), west to x24, north to the z'+12
             boundary row, over the carry column (cap on x21), west with a
             y3 flyover over bit k+1's a-lane (x0..5), then down to z'+11,
             west to this bit's own column x_k = -8+2k, north to z', and
             east into the a port.  Guards protect every descent diagonal.

All corridor dust is labelled and the whole build passes the alias-aware
shorts check before simulation.  Verification is fixed-tick only.
"""
import nucleation as n
import nets
import drc
import rs
import cells
import seq_cells as sc
from seq_probe import run_gt, bake_states, reload_sim, ticks_to_quiescent

BITS = 4
FA_P = cells.PITCH            # 13
REG_X = 28
COL_X = {k: -8 + 2 * k for k in range(BITS)}   # bit0 farthest west
HIGH, LOW = 30, 110           # clock phases for functional tests (gt)

TORCH_ON = "minecraft:redstone_torch[lit=true]"


def dust(b, labels, x, y, z, net, role="route", floor=True):
    if floor:
        b.stone(x, y - 1, z, role)
    b.put(x, y, z, rs.DUST)
    labels[(x, y, z)] = net


def rep(b, x, y, z, input_from):
    b.stone(x, y - 1, z, "gate")
    b.put(x, y, z, rs.repeater(input_from))


def build_counter():
    ha = cells.build_half_adder()
    fa = cells.build_full_adder(ha)
    dff = sc.build_dff()
    b = rs.Build("counter4")
    labels = {}
    aliases = []
    fp, rp = [], []
    for k in range(BITS):
        fp.append(fa.stamp(b, labels, 0, 0, FA_P * k,
                           rename=lambda s, k=k: "b%d.%s" % (k, s)))
        rp.append(dff.stamp(b, labels, REG_X, 0, FA_P * k + 8,
                            rename=lambda s, k=k:
                            "clk" if s == "clk" else "r%d.%s" % (k, s)))
        if k:
            aliases.append(("b%d.cout" % (k - 1), "b%d.cin" % k))
        aliases.append(("b%d.sum" % k, "r%d.d" % k))
        aliases.append(("r%d.q" % k, "b%d.a" % k))

    # cin of bit0 tied HIGH: torch beside the cin port
    b.stone(21, 0, -2)
    b.put(21, 1, -2, TORCH_ON)
    labels[(21, 1, -2)] = "b0.cin"

    # clock: lever north of bit0's clk_in, gap dust between register cells
    b.stone(38, 0, 6); b.put(38, 1, 6, rs.DUST); labels[(38, 1, 6)] = "clk"
    b.stone(38, 0, 5); b.force(38, 1, 5, rs.LEVER_OFF)
    clk_lever = (38, 1, 5)
    for k in range(BITS - 1):
        z = FA_P * k
        for zz in (14, 15, 16, 18, 19):
            dust(b, labels, 38, 1, z + zz, "clk")
        rep(b, 38, 1, z + 17, "north")

    for k in range(BITS):
        z = FA_P * k
        s_net, q_net = "r%d.d" % k, "b%d.a" % k

        # ---- sum -> D  (guards sever diagonals into the FA cout flyover)
        b.put(19, 2, z + 7, rs.PALETTE["lid"])
        b.put(20, 2, z + 6, rs.PALETTE["lid"])
        b.put(21, 2, z + 6, rs.PALETTE["lid"])
        dust(b, labels, 19, 1, z + 7, s_net, "lane")
        for x in (19, 20, 21, 22, 23):
            dust(b, labels, x, 1, z + 6, s_net, "lane")
        rep(b, 24, 1, z + 6, "west")
        for x in (25, 26, 27):
            dust(b, labels, x, 1, z + 6, s_net, "lane")
        dust(b, labels, 27, 1, z + 7, s_net, "lane")
        dust(b, labels, 27, 1, z + 8, s_net, "lane")

        # ---- Q -> a feedback corridor
        dust(b, labels, 41, 1, z + 8, q_net, "tap")
        for zz in (9, 10, 11):
            dust(b, labels, 41, 1, z + zz, q_net, "tap")
        rep(b, 41, 1, z + 12, "north")
        for zz in (13, 14, 15):
            dust(b, labels, 41, 1, z + zz, q_net, "tap")
        dust(b, labels, 40, 1, z + 15, q_net, "tap")
        b.stone(39, 1, z + 15, "tap"); dust(b, labels, 39, 2, z + 15, q_net, floor=False)
        b.put(38, 2, z + 15, rs.PALETTE["lid"])          # cap clock gap
        dust(b, labels, 38, 3, z + 15, q_net, floor=False)
        b.stone(37, 1, z + 15, "tap"); dust(b, labels, 37, 2, z + 15, q_net, floor=False)
        dust(b, labels, 36, 1, z + 15, q_net, "tap")
        rep(b, 35, 1, z + 15, "east")
        for x in range(34, 23, -1):
            dust(b, labels, x, 1, z + 15, q_net, "tap")
        dust(b, labels, 24, 1, z + 14, q_net, "tap")
        rep(b, 24, 1, z + 13, "south")
        dust(b, labels, 24, 1, z + 12, q_net, "tap")
        dust(b, labels, 23, 1, z + 12, q_net, "tap")
        b.stone(22, 1, z + 12, "tap"); dust(b, labels, 22, 2, z + 12, q_net, floor=False)
        b.put(21, 2, z + 12, rs.PALETTE["lid"])          # cap carry column
        dust(b, labels, 21, 3, z + 12, q_net, floor=False)
        b.stone(20, 1, z + 12, "tap"); dust(b, labels, 20, 2, z + 12, q_net, floor=False)
        for x in range(19, 12, -1):
            dust(b, labels, x, 1, z + 12, q_net, "tap")
        rep(b, 12, 1, z + 12, "east")
        for x in range(11, 4, -1):
            dust(b, labels, x, 1, z + 12, q_net, "tap")
        # y3 flyover over bit k+1's a-lane (x0..2) and xor dust (x4)
        b.put(4, 2, z + 13, rs.PALETTE["lid"])           # guard xor diagonal
        b.stone(4, 1, z + 12, "tap"); dust(b, labels, 4, 2, z + 12, q_net, floor=False)
        for x in (3, 2, 1, 0):
            b.stone(x, 2, z + 12, "tap")
            dust(b, labels, x, 3, z + 12, q_net, floor=False)
        b.stone(-1, 1, z + 12, "tap"); dust(b, labels, -1, 2, z + 12, q_net, floor=False)
        dust(b, labels, -1, 1, z + 11, q_net, "tap")
        xk = COL_X[k]
        dust(b, labels, -2, 1, z + 11, q_net, "tap")
        if xk < -2:
            rep(b, -3, 1, z + 11, "east")
            for x in range(-4, xk - 1, -1):
                dust(b, labels, x, 1, z + 11, q_net, "tap")
        rep(b, xk, 1, z + 10, "south")
        for zz in range(z + 9, z - 1, -1):
            dust(b, labels, xk, 1, zz, q_net, "tap")
        # entry row east into the a port, refreshed once more (the column
        # repeater's 15 dies over 9 column cells + up to 7 entry cells)
        entry_rep_x = xk + 1 if xk >= -4 else -4
        for x in range(xk + 1, 0):
            if x == entry_rep_x:
                rep(b, x, 1, z, "west")
            else:
                dust(b, labels, x, 1, z, q_net, "tap")
        if k:   # guard: bit k-1's descent dust diagonally over this entry
            b.put(-1, 2, z, rs.PALETTE["lid"])

    return b, labels, aliases, fp, rp, clk_lever


def read_q(sim, rp):
    return sum(int(sim.on(*p["q"])) << k for k, p in enumerate(rp))


def read_d(sim, rp):
    return sum(int(sim.on(*p["d"])) << k for k, p in enumerate(rp))


def pulse(sim, clk):
    sim.use(*clk); run_gt(sim, HIGH)
    sim.use(*clk); run_gt(sim, LOW)


def main():
    import os
    b, labels, aliases, fp, rp, clk = build_counter()
    print("counter4: %d blocks" % len(b.cells))
    shorts = nets.check(b.cells, labels, aliases)
    if shorts:
        for s in shorts[:6]:
            print("   SHORT", s)
        return False
    # This build closes the sequential loop ON PURPOSE (Q -> a, sum -> D), so it
    # holds exactly one diode ring per bit -- the only declared latch count in
    # the suite.  Any other number means a ring formed that nobody designed.
    if not drc.check_rings("counter%d" % BITS, b.cells, expect=BITS):
        return False
    sim = b.sim(settle=800)
    q0, d0 = read_q(sim, rp), read_d(sim, rp)
    print("counter4: settled Q=%d D=%d (want 0 / 1)" % (q0, d0))
    ok = q0 == 0 and d0 == 1

    baked = bake_states(b, sim)          # Q=0, CLK=0: the shipping artifact
    off = b.bounds()[0]

    good = True
    for i in range(1, 25):
        pulse(sim, clk)
        got = read_q(sim, rp)
        if got != i % 16:
            print("   step %2d: Q=%2d want %2d" % (i, got, i % 16))
            good = False
    print("counter4: 24 clocked steps %s" % ("PASS" if good else "FAIL"))
    ok = ok and good

    # deployment verify: baked schematic, InWorld, 0-tick quiescent, counts
    s2, raw = reload_sim(baked, off, n.TickSettleMode.InWorld)
    tq = ticks_to_quiescent(raw)
    bq = read_q(s2, rp)
    bok = tq == 0 and bq == 0
    for i in range(1, 6):
        pulse(s2, clk)
        bok = bok and read_q(s2, rp) == i
    print("counter4 baked: InWorld quiescent %d gt, Q=0, 5 steps after "
          "reload %s" % (tq, "PASS" if bok else "FAIL"))

    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "showcase", "counter4.schem")
    baked.save_to_file(out)
    print("saved %s" % out)
    return ok and bok


if __name__ == "__main__":
    print("seq_counter:", "ALL PASS" if main() else "FAILURES")
