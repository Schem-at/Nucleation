"""Two dense vertical 8-bit buses (2y pitch) crossing at 90 degrees.

TWO verified variants, both saved to showcase/:

v2 CANONICAL (bus_cross8.schem) -- COMPOSABLE: both buses run at the SAME
canonical levels y = 2+2n, so a bus's level never depends on its crossing
history.  At the crossing, bus B dips 1 down (to 1+2n), passes under bus
A's level through its own block-sandwich station, and climbs back up:

  * bus A (+X at z=ZC): straight run, aligned station at x = XC..XC+2
    (entry blocks at even levels 2+2n in the shared column);
  * bus B (+Z at x=XC): steps DOWN at z=ZC-2 -> ZC-1, one dip dust cell,
    then entry block / repeater / exit block at z = ZC..ZC+2 occupying the
    shared column's ODD levels 1+2n -- interleaving A's entry blocks --
    one dip-exit dust cell at ZC+3, and steps UP at ZC+3 -> ZC+4;
  * every cell is computed from the predicates (materials.py).
    TRANSPARENT blocks appear ONLY at the dip's slope transitions: bit n's
    dip-dust supports at (XC, 2n, ZC-1|ZC+3) sit directly above the lower
    dust of bit n-1's step diagonal, which must survive -> glass (bit 0:
    nothing below -> solid).  The matching SOLID step supports at
    (XC, 1+2n, ZC-2|ZC+4) satisfy the diode law (step conducts downhill)
    AND sever the cross-bit diagonals -- the alternation.
  * each bus pays 1 repeater/bit which doubles as its refresh.
  Crossing core: 3(x) x 7(z) x 17(y); 16 repeaters; 14 glass.

v1 TOWER (bus_cross8_tower.schem) -- kept as an alternative with genuine
wins: core is only 3 x 3 x 17 and uses NO transparent blocks; the cost is
y-parity (B's bits at 2+2n vs A's 1+2n), i.e. crossing it SHIFTS a bus's
level -- the composability defect v2 removes.

Verification (per variant, all through real levers, outputs on far dust):
walking-ones each bus (8+8), both-all-on, alternating pairs both ways,
8 seeded random joint patterns -- 27 patterns x 16 outputs = 432 checks,
zero crosstalk tolerated.  Saved BAKED at rest, only on green.
"""
import os
import random

import nucleation as n

import rs
from rs import DUST
from materials import pick_support, separator

BITS = 8
XC, ZC = 8, 8                 # crossing column
END = 15                      # both runs span 0..END on their axis


# -- v2: both buses at canonical levels y = 2+2n ----------------------------

def canon_bus_a(b):
    """Bus A: +X at z=ZC, bit n dust at y=2+2n, aligned station XC..XC+2."""
    outs = []
    for bit in range(BITS):
        y = 2 + 2 * bit
        for x in list(range(0, XC)) + list(range(XC + 3, END + 1)):
            b.put(x, y - 1, ZC, separator())      # solid: straight run
            b.put(x, y, ZC, DUST)
        b.put(XC, y, ZC, rs.PALETTE["route"])     # entry block (floats; also
        #   the cap severing every read to/from B's dip dust one level down)
        b.put(XC + 1, y - 1, ZC,
              pick_support(need_conductor=True, solid=rs.PALETTE["lid"],
                           why="A repeater floor"))
        b.put(XC + 1, y, ZC, rs.repeater("west"))  # flows +X
        b.put(XC + 2, y, ZC, rs.PALETTE["route"])  # exit block: fresh 15
        outs.append((END, y, ZC))
    return outs


def canon_bus_b(b):
    """Bus B: +Z at x=XC, bit n dust at y=2+2n, dipping to 1+2n across the
    crossing.  Its station blocks fill the shared column's odd levels."""
    outs = []
    for bit in range(BITS):
        y = 2 + 2 * bit                           # canonical level
        d = y - 1                                 # dip level
        for z in list(range(0, ZC - 1)) + list(range(ZC + 4, END + 1)):
            # straight canonical run; at ZC-2 / ZC+4 the dust is the STEP
            # UPPER: the same solid also satisfies the diode law and severs
            # the cross-bit diagonals at the slope
            b.put(XC, y - 1, z,
                  pick_support(need_conductor=(z in (ZC - 2, ZC + 4)),
                               solid=rs.PALETTE["gate"], why="B run z=%d" % z))
            b.put(XC, y, z, DUST)
        for z in (ZC - 1, ZC + 3):                # the two dip dust cells:
            # their support sits directly above the LOWER dust of bit n-1's
            # step diagonal, which must survive -> transparent (bit 0:
            # nothing below -> solid).  THE ONLY TRANSPARENT CELLS.
            b.put(XC, d - 1, z,
                  pick_support(need_insulator=(bit > 0),
                               solid=rs.PALETTE["gate"],
                               why="B dip support z=%d" % z))
            b.put(XC, d, z, DUST)
        b.put(XC, d, ZC, rs.PALETTE["gate"])      # entry block (floats in the
        #   tower's odd levels, under A's entry; nothing conductive chains)
        b.put(XC, d - 1, ZC + 1,
              pick_support(need_conductor=True, solid=rs.PALETTE["lid"],
                           why="B repeater floor"))
        b.put(XC, d, ZC + 1, rs.repeater("north"))  # flows +Z
        b.put(XC, d, ZC + 2, rs.PALETTE["gate"])    # exit block: fresh 15
        outs.append((XC, y, END))
    return outs


# -- v1 tower variant (kept: 3x3 core, zero glass; costs y-parity) ----------

def tower_bus_a(b):
    """Bus A: +X at z=ZC, bit n dust at y=1+2n.  Station at x=XC..XC+2."""
    for x in range(END + 1):
        b.stone(x, 0, ZC, "rail_floor")           # bit0 floor + station floors
    outs = []
    for bit in range(BITS):
        y = 1 + 2 * bit
        for x in list(range(0, XC)) + list(range(XC + 3, END + 1)):
            b.put(x, y, ZC, DUST)
            if bit > 0 and (x, y - 1, ZC) not in b.cells:
                b.put(x, y - 1, ZC, separator())
        b.put(XC, y, ZC, rs.PALETTE["route"])     # entry block: tower cell
        b.stone(XC + 1, y - 1, ZC, "lid")         # solid repeater floor
        b.put(XC + 1, y, ZC, rs.repeater("west"))  # flows +X
        b.put(XC + 2, y, ZC, rs.PALETTE["route"])  # exit block: fresh 15
        outs.append((END, y, ZC))
    return outs


def tower_bus_b(b):
    """Bus B: +Z at x=XC, bit n dust at y=2+2n (the y-parity shift).
    Its entry blocks fill the tower's even levels."""
    outs = []
    for bit in range(BITS):
        y = 2 + 2 * bit
        for z in list(range(0, ZC)) + list(range(ZC + 3, END + 1)):
            b.put(XC, y, z, DUST)
            if (XC, y - 1, z) not in b.cells:
                b.put(XC, y - 1, z, separator())  # bit0 too: B floats at y=1
        b.put(XC, y, ZC, rs.PALETTE["gate"])      # entry block: tower cell
        b.stone(XC, y - 1, ZC + 1, "lid")         # solid repeater floor
        b.put(XC, y, ZC + 1, rs.repeater("north"))  # flows +Z
        b.put(XC, y, ZC + 2, rs.PALETTE["gate"])  # exit block
        outs.append((XC, y, END))
    return outs


def lever_column(b, x, z, y_first):
    """8 stacked drivers: lever at y_first+2n on a block below; the levers'
    attachment blocks only neighbour solid separators (inert)."""
    levers = []
    for bit in range(BITS):
        y = y_first + 2 * bit
        b.stone(x, y - 1, z)
        b.force(x, y, z, rs.LEVER_OFF)
        levers.append((x, y, z))
    return levers


def check(sim, outs_a, outs_b, pa, pb, label):
    got_a = [1 if sim.power(*p) > 0 else 0 for p in outs_a]
    got_b = [1 if sim.power(*p) > 0 else 0 for p in outs_b]
    want_a = [(pa >> i) & 1 for i in range(BITS)]
    want_b = [(pb >> i) & 1 for i in range(BITS)]
    ok = got_a == want_a and got_b == want_b
    fmt = lambda v: "".join(map(str, reversed(v)))
    print("%s %-12s A want %s got %s | B want %s got %s"
          % ("PASS" if ok else "FAIL", label,
             fmt(want_a), fmt(got_a), fmt(want_b), fmt(got_b)))
    return (sum(1 for g, w in zip(got_a, want_a) if g == w)
            + sum(1 for g, w in zip(got_b, want_b) if g == w))


def run_variant(name, build_a, build_b, y_first_a, y_first_b, out_name):
    b = rs.Build(name)
    outs_a = build_a(b)
    outs_b = build_b(b)
    lev_a = lever_column(b, -1, ZC, y_first_a)
    lev_b = lever_column(b, XC, -1, y_first_b)
    sim = b.sim()
    lv = rs.Levers(sim, lev_a + lev_b)

    rng = random.Random(1234)
    patterns = ([("walkA-%d" % i, 1 << i, 0) for i in range(BITS)]
                + [("walkB-%d" % i, 0, 1 << i) for i in range(BITS)]
                + [("all-on", 0xFF, 0xFF),
                   ("alt-AA/55", 0xAA, 0x55), ("alt-55/AA", 0x55, 0xAA)]
                + [("rand-%d" % i, rng.randrange(256), rng.randrange(256))
                   for i in range(8)])
    good = total = 0
    for label, pa, pb in patterns:
        lv.set([(pa >> i) & 1 for i in range(BITS)]
               + [(pb >> i) & 1 for i in range(BITS)])
        good += check(sim, outs_a, outs_b, pa, pb, label)
        total += 2 * BITS
    print("%s: %d/%d output checks (%d patterns x 16 bits)"
          % (name, good, total, len(patterns)))
    if good != total:
        print("NOT saving %s (suite red)" % out_name)
        return False

    # bake at rest and save the showcase piece
    lv.set([0] * (2 * BITS))
    rest = [sim.power(*p) for p in outs_a + outs_b]
    assert all(v == 0 for v in rest), rest
    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"),
                       "_%s_tight.schem" % name)
    b.s.save_to_file(tmp)
    tight = n.Schematic.open(tmp)
    changed = sim.sim.bake_to(tight)
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "showcase", out_name)
    tight.save_to_file(out)
    print("baked %d settled states; saved %s" % (changed, out))
    return True


def main():
    ok_v2 = run_variant("bus_cross8", canon_bus_a, canon_bus_b, 2, 2,
                        "bus_cross8.schem")
    ok_v1 = run_variant("bus_cross8_tower", tower_bus_a, tower_bus_b, 1, 2,
                        "bus_cross8_tower.schem")
    return 0 if ok_v2 and ok_v1 else 1


if __name__ == "__main__":
    raise SystemExit(main())
