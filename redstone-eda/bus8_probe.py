"""Dense VERTICAL 8-bit bus: bits stacked in y at 2y pitch, 1 block wide.

Stack (per the PROBED material table, notes-material-model.md):

    y=15  bit7 dust
    y=14  slab[type=top]     <- separator: supports the dust above, conducts
    ...                         nothing, cuts nothing (probed UNDER: the
    y=3   bit1 dust             dust/slab/dust 2y stack conducts both lines,
    y=2   slab[type=top]        fully isolated)
    y=1   bit0 dust
    y=0   solid rail floor

Solid appears ONLY where a level must host a repeater (station floors) --
everything else between bits is slab-top.  Cross-section: 1 wide x 16 tall
for 8 bits (2 cells/bit).

Refresh: dust arrives at a block-sandwich station entry with ss>=1 after at
most 15 cells (probed A15; A16 is dead), and the exit block re-emits a fresh
15 (probed B).  Max clean pitch between refreshes is therefore 18 (15 dust +
3 station).  Stations must sit on conductive floors, so station cells are
STAGGERED per bit along the run: bit n refreshes at x = S + n, giving each
repeater a solid floor cell that lands on the bit below's station blocks
(block-on-block, electrically inert) instead of on live dust.

This script builds a 40-block run with TWO staggered refresh stages per bit
(second stage at the probed max pitch, 15 dust after the first exit), drives
it with a stacked lever column, and verifies walking-ones + all-on +
alternating + all-off at the far end: 12 patterns x 8 bits = 96 checks,
zero crosstalk tolerated.  Saved BAKED at rest as showcase/bus8_run.schem.
"""
import os

import nucleation as n

import rs
from rs import DUST
from materials import SLAB_TOP

RUN = 40                      # x = 0 .. RUN-1
STAGES = (8, 26)              # bit n's station entries at x = stage + n
BITS = 8


def y_of(bit):
    return 1 + 2 * bit


def build_bus(b, z=0):
    """The dense vertical bus along +X at this z.  Returns out-probe cells."""
    # ground floor: bit0's conductive floor (and every station-0 floor)
    for x in range(RUN):
        b.stone(x, 0, z, "rail_floor")
    outs = []
    for bit in range(BITS):
        y = y_of(bit)
        entries = [s + bit for s in STAGES]
        station_x = set()
        for e in entries:
            station_x.update((e, e + 1, e + 2))
        for x in range(RUN):
            if x in station_x:
                continue                      # filled below
            b.put(x, y, z, DUST)              # the bit's rail
            if bit > 0 and (x, y - 1, z) not in b.cells:
                b.put(x, y - 1, z, SLAB_TOP)  # separator = dust support
        for e in entries:
            b.put(e, y, z, rs.PALETTE["route"])         # entry block (floats)
            b.stone(e + 1, y - 1, z, "lid")             # solid repeater floor
            b.put(e + 1, y, z, rs.repeater("west"))     # flows +X
            b.put(e + 2, y, z, rs.PALETTE["route"])     # exit block (fresh 15)
        outs.append((RUN - 1, y, z))
    return outs


def lever_column(b, x, z):
    """8 stacked drivers: block at even y, lever at odd y (bit n at 1+2n).
    A lever strongly powers only its attachment block below; that block's
    only non-lever neighbours are the bus's slab separators (inert)."""
    levers = []
    for bit in range(BITS):
        b.stone(x, 2 * bit, z)
        b.force(x, 2 * bit + 1, z, rs.LEVER_OFF)
        levers.append((x, 2 * bit + 1, z))
    return levers


def check(sim, outs, pattern, label):
    got = [1 if sim.power(*p) > 0 else 0 for p in outs]
    want = [(pattern >> i) & 1 for i in range(BITS)]
    ok = got == want
    print("%s %-12s want %s got %s" % ("PASS" if ok else "FAIL", label,
                                       "".join(map(str, reversed(want))),
                                       "".join(map(str, reversed(got)))))
    return BITS if ok else sum(1 for g, w in zip(got, want) if g == w)


def main():
    b = rs.Build("bus8_run")
    outs = build_bus(b)
    levers = lever_column(b, -1, 0)
    sim = b.sim()
    lv = rs.Levers(sim, levers)

    patterns = ([(0, "all-off")]
                + [(1 << i, "walk-%d" % i) for i in range(BITS)]
                + [(0xFF, "all-on"), (0xAA, "alt-AA"), (0x55, "alt-55")])
    good = total = 0
    for pat, label in patterns:
        lv.set([(pat >> i) & 1 for i in range(BITS)])
        good += check(sim, outs, pat, label)
        total += BITS
    print("bus8_run: %d/%d output checks (8 bits x %d patterns, "
          "%d-block run, 2 refresh stages/bit, 2y pitch)"
          % (good, total, len(patterns), RUN))

    # bake at rest and save the showcase piece
    lv.set([0] * BITS)
    assert all(sim.power(*p) == 0 for p in outs)
    tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_bus8_tight.schem")
    b.s.save_to_file(tmp)
    tight = n.Schematic.open(tmp)
    changed = sim.sim.bake_to(tight)
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "showcase", "bus8_run.schem")
    tight.save_to_file(out)
    print("baked %d settled states; saved %s" % (changed, out))
    return 0 if good == total else 1


if __name__ == "__main__":
    raise SystemExit(main())
