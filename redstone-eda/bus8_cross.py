"""Two dense vertical 8-bit buses (2y pitch) crossing at 90 degrees.

Geometry -- technique 3a from notes-material-model.md ("y-parity
interpenetration: the block-sandwich stations' solid blocks double as the
isolation") generalized to full 8-bit stacks:

  * bus A runs +X at z=ZC, bits at y = 1+2n (odd levels);
  * bus B runs +Z at x=XC, bits at y = 2+2n (even levels);
  * at the shared column (XC, ZC) EVERY level is a station ENTRY BLOCK --
    A's at odd y, B's at even y: a 16-block tower.  Each entry block is
    weak-powered only by its own bus's dead-end approach dust (probed
    A14/A15) and feeds its own repeater; block-to-block never propagates,
    so the tower is simultaneously all 64 bit-pair isolations.
  * repeater floors are solid cells that land on the bit below's repeater
    (block on block, inert); everything else between bits is slab-top
    (probed UNDER stack).

So the whole 8x8 crossing (64 bit pairs) collapses into ONE 3x3-ground
station core: A crosses in 3 x-cells, B in 3 z-cells, 17 tall, 16 repeaters
(1 per bit -- each bus pays 1 rt, and the station doubles as its refresh).

Verification (all through real levers, outputs read on far-end dust):
walking-ones on each bus separately (8+8), both-all-on, alternating pairs
(0xAA/0x55 both ways), plus 8 seeded random joint patterns -- 27 patterns,
ALL 16 outputs asserted each time = 432 checks, zero crosstalk tolerated.
Saved BAKED at rest as showcase/bus_cross8.schem.
"""
import os
import random

import nucleation as n

import rs
from rs import DUST
from materials import SLAB_TOP

BITS = 8
XC, ZC = 8, 8                 # crossing column
END = 15                      # both runs span 0..END on their axis


def build_bus_a(b):
    """Bus A: +X at z=ZC, bit n dust at y=1+2n.  Station at x=XC..XC+2."""
    for x in range(END + 1):
        b.stone(x, 0, ZC, "rail_floor")           # bit0 floor + station floors
    outs = []
    for bit in range(BITS):
        y = 1 + 2 * bit
        for x in list(range(0, XC)) + list(range(XC + 3, END + 1)):
            b.put(x, y, ZC, DUST)
            if bit > 0 and (x, y - 1, ZC) not in b.cells:
                b.put(x, y - 1, ZC, SLAB_TOP)
        b.put(XC, y, ZC, rs.PALETTE["route"])     # entry block: tower cell
        b.stone(XC + 1, y - 1, ZC, "lid")         # solid repeater floor
        b.put(XC + 1, y, ZC, rs.repeater("west")) # flows +X
        b.put(XC + 2, y, ZC, rs.PALETTE["route"]) # exit block: fresh 15
        outs.append((END, y, ZC))
    return outs


def build_bus_b(b):
    """Bus B: +Z at x=XC, bit n dust at y=2+2n.  Station at z=ZC..ZC+2.
    Its entry blocks fill the tower's even levels; its slab separators live
    at odd levels, which off the crossing column belong to no A cell."""
    outs = []
    for bit in range(BITS):
        y = 2 + 2 * bit
        for z in list(range(0, ZC)) + list(range(ZC + 3, END + 1)):
            b.put(XC, y, z, DUST)
            if (XC, y - 1, z) not in b.cells:
                b.put(XC, y - 1, z, SLAB_TOP)     # bit0 too: B floats at y=1
        b.put(XC, y, ZC, rs.PALETTE["gate"])      # entry block: tower cell
        b.stone(XC, y - 1, ZC + 1, "lid")         # solid repeater floor
        b.put(XC, y, ZC + 1, rs.repeater("north"))  # flows +Z
        b.put(XC, y, ZC + 2, rs.PALETTE["gate"])  # exit block
        outs.append((XC, y, END))
    return outs


def lever_column(b, x, z, y_first):
    """8 stacked drivers: lever at y_first+2n on a block below; the levers'
    attachment blocks only neighbour slab separators (inert)."""
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


def main():
    b = rs.Build("bus_cross8")
    outs_a = build_bus_a(b)
    outs_b = build_bus_b(b)
    lev_a = lever_column(b, -1, ZC, 1)
    lev_b = lever_column(b, XC, -1, 2)
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
    print("bus_cross8: %d/%d output checks (%d patterns x 16 bits; "
          "8x8 crossing core 3x3 ground x 17 tall, 16 repeaters)"
          % (good, total, len(patterns)))

    # bake at rest and save the showcase piece
    lv.set([0] * (2 * BITS))
    rest = [sim.power(*p) for p in outs_a + outs_b]
    assert all(v == 0 for v in rest), rest
    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_cross8_tight.schem")
    b.s.save_to_file(tmp)
    tight = n.Schematic.open(tmp)
    changed = sim.sim.bake_to(tight)
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "showcase", "bus_cross8.schem")
    tight.save_to_file(out)
    print("baked %d settled states; saved %s" % (changed, out))
    return 0 if good == total else 1


if __name__ == "__main__":
    raise SystemExit(main())
