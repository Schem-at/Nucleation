"""HexAnalog (E4): a 4-bit value on ONE wire as signal strength 0-15.

encoder: staged comparator subtraction.  chain rear = const 15, then
         subtract nb3*8, nb2*4, nb1*2, nb0*1 (nb = NOT bit, wall-torch
         inverters gating torch-fed decay lanes of exact strength).
         out = 15 - (15 - v) = v.
trunk:   alternating [comparator][dust] cells -- a comparator regenerates
         its rear value losslessly (probed), so one wire carries the full
         nibble any distance.
decoder: 3 stages of threshold + gated subtract + comparator-merge:
           t  = max(v - c, 0)        c = 2^k - 1   -> bit_k = t > 0
           u  = max(v - g, 0)        g = 2^k       (const decay lane)
           w  = max(v - 15*bit, 0)                 (repeater-gated pass)
           v' = max(u, w)            two comparators aimed at ONE dust
                                     cell merge losslessly (probed)
         b0 = final remainder (0/1).

All comparators run in SUBTRACT mode with empty sides as pass-throughs --
compare-mode gating stays unused (its semantics were the unverified ones).
Every analog fact used is PROBED in mc-tick first (run this file).
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

import nucleation as n            # noqa: E402
import cells                      # noqa: E402  (interns comparator states)
import nets                       # noqa: E402
import rs                        # noqa: E402
from seq_probe import bake_states, reload_sim, ticks_to_quiescent  # noqa: E402

COMP = "minecraft:comparator[facing=%s,mode=subtract,powered=false]"


def comp(b, x, z, facing):
    b.stone(x, 0, z)
    b.put(x, 1, z, COMP % facing)


def dust(b, labels, x, z, lab):
    b.stone(x, 0, z)
    b.put(x, 1, z, rs.DUST)
    labels[(x, 1, z)] = lab


def torch(b, x, z):
    b.stone(x, 0, z)
    b.put(x, 1, z, rs.TORCH)


# ------------------------------------------------------------------ probes

def probe():
    """Verify every analog primitive the design leans on."""
    ok = True

    def chk(c, m):
        nonlocal ok
        print("   %-58s %s" % (m, "PASS" if c else "FAIL"))
        ok = ok and c

    # rig A: torch -> dust(15) -> [comp sub, side dust 6] -> out dust
    b = rs.Build("p_a")
    labels = {}
    torch(b, 0, 0)
    dust(b, labels, 1, 0, "back")
    comp(b, 2, 0, "west")
    dust(b, labels, 3, 0, "out")
    # side: torch at z=10, lane z=9..1 arrives at (2,1) with 15-8=7
    torch(b, 2, 10)
    for z in range(9, 0, -1):
        dust(b, labels, 2, z, "side")
    sim = b.sim()
    chk(sim.power(1, 1, 0) == 15, "torch-adjacent dust reads 15")
    chk(sim.power(2, 1, 1) == 7, "decay lane: 9 cells from torch -> 7")
    chk(sim.power(3, 1, 0) == 8, "subtract via SIDE DUST: 15 - 7 = 8 on out dust")

    # rig B: comp->comp->comp chain regenerates exactly (trunk primitive)
    b = rs.Build("p_b")
    labels = {}
    torch(b, 0, 0)
    for z in range(1, 7):
        dust(b, labels, 0, z, "lane")        # value 15-... at z=6: 10
    dust(b, labels, 1, 6, "v")               # 9
    comp(b, 2, 6, "west")
    dust(b, labels, 3, 6, "t1")
    comp(b, 4, 6, "west")                     # comp reading dust behind
    comp(b, 5, 6, "west")                     # comp reading COMP directly
    dust(b, labels, 6, 6, "t2")
    sim = b.sim()
    v = sim.power(1, 1, 6)
    chk(v == 9, "rear feed dust is 9")
    chk(sim.power(3, 1, 6) == v, "comp pass-through (no side) preserves value")
    chk(sim.power(6, 1, 6) == v, "comp->comp->dust chain still exact (%d)" % v)

    # rig C: two comparators aimed at ONE dust cell -> max (lossless merge)
    b = rs.Build("p_c")
    labels = {}
    torch(b, 0, 0)
    for x in range(1, 6):
        dust(b, labels, x, 0, "a")            # 15..11
    comp(b, 6, 0, "west")                     # outputs east -> M (7,0)
    torch(b, 7, 4)
    for z in range(3, 1, -1):
        dust(b, labels, 7, z, "b")            # (7,2): 14
    comp(b, 7, 1, "south")                    # input south (7,2), out north -> M
    dust(b, labels, 7, 0, "m")                # shared M cell
    sim = b.sim()
    chk(sim.power(7, 1, 0) == 14, "two comps into ONE dust cell: max(11,14)=14")

    # rig D: repeater into a comparator SIDE gates 15 exactly
    b = rs.Build("p_d")
    labels = {}
    torch(b, 0, 0)
    dust(b, labels, 1, 0, "back")             # 15
    comp(b, 2, 0, "west")
    dust(b, labels, 3, 0, "out")
    b.stone(2, 0, 1)
    b.put(2, 1, 1, rs.repeater("south"))      # input (2,1,2) -> north into side
    dust(b, labels, 2, 2, "g")
    dust(b, labels, 2, 3, "g")
    b.stone(2, 0, 4); b.force(2, 1, 4, rs.LEVER_OFF)
    sim = b.sim()
    chk(sim.power(3, 1, 0) == 15, "lever off: out = 15 - 0")
    sim.use(2, 1, 4)
    sim.settle()
    chk(sim.power(3, 1, 0) == 0, "repeater-gated side: 15 - 15 = 0")
    print("   probes:", "ALL PASS" if ok else "FAILURES")
    return ok


# ----------------------------------------------------------------- encoder

def build_encoder(b, labels):
    """Levers b0..b3 -> one wire at (8,1,0) carrying v.  Returns levers."""
    torch(b, 2, 0)
    dust(b, labels, 3, 0, "e.const15")
    for x in (4, 5, 6, 7):
        comp(b, x, 0, "west")
    dust(b, labels, 8, 0, "e.s")              # encoder output S = v

    levers = {}
    # (bit, comp x, side dir, strength) — south lanes z>0, north z<0
    for bit, x, sgn, c in ((3, 4, 1, 8), (1, 6, 1, 2), (2, 5, -1, 4), (0, 7, -1, 1)):
        zb = sgn * (18 - c)                   # inverter base block
        lane = "e.nb%d" % bit
        b.stone(x, 0, zb)
        b.put(x, 1, zb, rs.PALETTE["inv"])   # torch base block
        tz = zb - sgn                        # wall torch cell
        b.put(x, 1, tz, rs.wall_torch("north" if sgn > 0 else "south"))
        labels[(x, 1, tz)] = lane
        for z in range(1, 18 - c - 1):       # lane dust: |z| = 1 .. |zb|-2
            dust(b, labels, x, sgn * z, lane)
        for dz in (1, 2):                    # dead-end input into the base
            dust(b, labels, x, zb + sgn * dz, "e.b%d" % bit)
        b.stone(x, 0, zb + sgn * 3)
        b.force(x, 1, zb + sgn * 3, rs.LEVER_OFF)
        levers[bit] = (x, 1, zb + sgn * 3)
    return levers


# ----------------------------------------------------------------- decoder

def build_stage(b, labels, ox, oz, cp, g, tag):
    """One decode stage at origin (ox, oz).  cp = threshold const seen by
    the v-1 dust (2^k - 2), g = subtract const (2^k).  Returns (bit-port,
    next-P0 position)."""
    L = lambda s: "%s.%s" % (tag, s)
    X, Z = ox, oz

    # pipeline: dust at even x, pass comps at odd x (all subtract, no sides)
    for i, x in enumerate(range(0, 9)):
        if x % 2 == 0:
            dust(b, labels, X + x, Z, L("v"))
        else:
            comp(b, X + x, Z, "west")
    # t branch (west): B_t, v-1 dust, T with const side, refresh, b out
    comp(b, X + 0, Z + 1, "north")
    dust(b, labels, X + 0, Z + 2, L("vb"))
    dust(b, labels, X - 1, Z + 2, L("vb"))            # v-1 (one decay step)
    comp(b, X - 1, Z + 3, "north")                    # T: t = (v-1) - cp
    if cp > 0:                                        # const column + jog
        dust(b, labels, X - 2, Z + 3, L("c"))
        m = 14 - cp
        for dz in range(0, m + 1):
            dust(b, labels, X - 3, Z + 3 + dz, L("c"))
        torch(b, X - 3, Z + 4 + m)
    dust(b, labels, X - 1, Z + 4, L("t"))
    b.stone(X - 1, 0, Z + 5)
    b.put(X - 1, 1, Z + 5, rs.repeater("north"))
    dust(b, labels, X - 1, Z + 6, L("t"))             # bit port (15 when set)
    bit_port = (X - 1, 1, Z + 6)
    # b path to W's gate repeater
    for x in (0, 1, 2):
        dust(b, labels, X + x, Z + 6, L("t"))
    for dz in (5, 4, 3):
        dust(b, labels, X + 2, Z + dz, L("t"))
    b.stone(X + 3, 0, Z + 3)
    b.put(X + 3, 1, Z + 3, rs.repeater("west"))       # -> W west side (15*b)

    # W branch (middle): w = v - 15*bit
    comp(b, X + 4, Z + 1, "north")
    dust(b, labels, X + 4, Z + 2, L("vw"))
    comp(b, X + 4, Z + 3, "north")                    # W
    dust(b, labels, X + 4, Z + 4, L("w"))
    comp(b, X + 4, Z + 5, "north")
    dust(b, labels, X + 4, Z + 6, L("w"))
    comp(b, X + 5, Z + 6, "west")                     # -> M

    # U branch (east): u = v - g, const lane east on row Z+3
    comp(b, X + 8, Z + 1, "north")
    dust(b, labels, X + 8, Z + 2, L("vu"))
    comp(b, X + 8, Z + 3, "north")                    # U
    Lg = 25 - g
    for x in range(9, Lg):
        dust(b, labels, X + x, Z + 3, L("g"))
    torch(b, X + Lg, Z + 3)
    dust(b, labels, X + 8, Z + 4, L("u"))
    comp(b, X + 8, Z + 5, "north")
    dust(b, labels, X + 8, Z + 6, L("u"))
    comp(b, X + 7, Z + 6, "east")                     # -> M

    # merge + transport south to the next stage's P0
    dust(b, labels, X + 6, Z + 6, L("m"))             # M = max(u, w) = v'
    for dz in (7, 9, 11):
        comp(b, X + 6, Z + dz, "north")
    for dz in (8, 10):
        dust(b, labels, X + 6, Z + dz, L("m"))
    dust(b, labels, X + 6, Z + 12, "%s.next" % tag)   # = next stage P0
    return bit_port, (X + 6, Z + 12)


def build_decoder(b, labels, ox):
    """P0 at (ox, 1, 0) must already be v.  Returns bit ports [b0..b3]."""
    p3, nxt = build_stage(b, labels, ox, 0, 6, 8, "d3")
    p2, nxt2 = build_stage(b, labels, nxt[0], nxt[1], 2, 4, "d2")
    p1, nxt3 = build_stage(b, labels, nxt2[0], nxt2[1], 0, 2, "d1")
    b0 = (nxt3[0], 1, nxt3[1])                        # remainder 0/1
    return [b0, (p1[0], p1[1], p1[2]), (p2[0], p2[1], p2[2]),
            (p3[0], p3[1], p3[2])]


def build_trunk(b, labels, x0, x1):
    """Alternating comp/dust from x0..x1 on row z=0 (comp first)."""
    for x in range(x0, x1 + 1):
        if (x - x0) % 2 == 0:
            comp(b, x, 0, "west")
        else:
            dust(b, labels, x, 0, "trunk")


def main():
    if not probe():
        return False

    # ---- encoder alone: 16/16
    b = rs.Build("hex_encoder")
    labels = {}
    levers = build_encoder(b, labels)
    shorts = nets.check(b.cells, labels)
    assert not shorts, shorts[:4]
    sim = b.sim()
    lv = rs.Levers(sim, [levers[i] for i in range(4)])
    good = 0
    for v in range(16):
        lv.set([(v >> i) & 1 for i in range(4)])
        got = sim.power(8, 1, 0)
        if got == v:
            good += 1
        else:
            print("   encoder v=%2d -> S=%2d" % (v, got))
    print("encoder exhaustive: %d/16" % good)
    if good != 16:
        return False

    # ---- encoder -> 12-cell one-wire trunk -> decoder: 16/16
    b = rs.Build("hexanalog_trunk")
    labels = {}
    levers = build_encoder(b, labels)
    build_trunk(b, labels, 9, 19)          # comps at 9,11,..,19; dust between
    dust(b, labels, 20, 0, "trunk")        # P0 = trunk exit
    bits = build_decoder(b, labels, 20)
    shorts = nets.check(b.cells, labels)
    if shorts:
        for s in shorts[:8]:
            print("   SHORT", s)
        return False
    sim = b.sim(settle=800)
    lv = rs.Levers(sim, [levers[i] for i in range(4)])
    good = 0
    for v in range(16):
        lv.set([(v >> i) & 1 for i in range(4)], settle=800)
        entry = sim.power(20, 1, 0)
        got = sum(int(sim.on(*bits[i])) << i for i in range(4))
        if entry == v and got == v:
            good += 1
        else:
            print("   v=%2d: trunk exit=%2d decoded=%2d" % (v, entry, got))
    print("trunk+decoder exhaustive: %d/16 (12-block one-wire trunk, "
          "comparator-regenerated)" % good)
    if good != 16:
        return False

    lv.set([0, 0, 0, 0], settle=800)
    out = os.path.join(os.path.dirname(HERE), "showcase", "hexanalog_trunk.schem")
    baked = bake_states(b, sim)
    baked.save_to_file(out)
    print("saved %s" % out)
    return True


if __name__ == "__main__":
    print("hexanalog:", "ALL PASS" if main() else "FAILURES")
