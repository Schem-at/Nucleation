"""Dense VERTICAL 8-bit bus v2: 2y pitch, 1 wide, NO transparent blocks,
ALIGNED station columns.

v1 used slab-top separators and STAGGERED the per-bit refresh stations
diagonally (bit n's entry at x = S + n) so no repeater floor would ever sit
over live dust.  Both precautions are unnecessary, computed from the
predicates (materials.py; PROBED table, notes-material-model.md):

  * separators: a straight run uses NO diagonals, so the cut law never has
    anything to cut -- a SOLID separator is a harmless cap
    (cap_is_harmful()==False), the dust above it weak-powers it into a
    dead end (weak power never lights dust), and it severs any stray
    diagonal.  Transparency is reserved for slope transitions, which a
    straight bus does not have.
  * aligned stations: stacking every bit's station at the same x only adds
    solid-over-live-dust and block-on-block adjacencies, both inert.  The
    one genuinely new adjacency -- a solid repeater-floor-style cap sitting
    directly over the bit below's LIVE run dust -- is probed here (P1).

PROBES (fresh empirical checks, run before the build):

  P1  solid caps over a live straight run: the capped run still conducts,
      and a second driven line ON TOP of the caps (2y pitch, the aligned-
      column adjacency) is isolated from it in all 4 lever combos.
  P2  a repeater fires from a SLAB-TOP floor: sturdiness is static-legality
      only, so v1's solid 'lid' floors were never load-bearing either --
      cited only to close the question; v2 floors are solid anyway.

BUS: bit n dust at y = 1+2n; a full SOLID layer at y = 2n for bit n > 0
(bit 0: ground floor); station entries ALIGNED at x = 8 and x = 26 for
every bit (the second at the probed max pitch: 15 dust after the first
exit).  Verified: walking-ones + all-on + alternating + all-off at the far
end, 12 patterns x 8 bits = 96 checks, zero crosstalk tolerated.  Saved
BAKED at rest as showcase/bus8_run.schem only on green.
"""
import os

import nucleation as n

import rs
from rs import DUST
from materials import SLAB_TOP, separator

RUN = 40                      # x = 0 .. RUN-1
STAGES = (8, 26)              # every bit's station entry at these x (ALIGNED)
BITS = 8


def y_of(bit):
    return 1 + 2 * bit


# -- probes -----------------------------------------------------------------

def probe():
    """P1 + P2 (see module docstring).  Returns number of failed verdicts."""
    b = rs.Build("bus8_v2_probes")
    LV = []

    def lever(x, y, z):
        b.stone(x, y - 1, z)
        b.force(x, y, z, rs.LEVER_OFF)
        LV.append((x, y, z))
        return len(LV) - 1

    # P1: line A x=0..8 at y=1; SOLID caps at y=2 over x=3..8; line B rides
    # the caps at y=3 (dust/solid/dust -- exactly the aligned-column stack).
    iA = lever(-1, 1, 0)
    for x in range(9):
        b.stone(x, 0, 0)
        b.put(x, 1, 0, DUST)
    for x in range(3, 9):
        b.put(x, 2, 0, separator())     # computed: solid cap, straight run
        b.put(x, 3, 0, DUST)
    iB = lever(9, 3, 0)                 # its block at (9,2,0) touches no dust
    pA, pB = (8, 1, 0), (3, 3, 0)

    # P2: repeater whose floor is a SLAB-TOP (station at x=4..6, z=6)
    iC = lever(-1, 1, 6)
    for x in range(4):
        b.stone(x, 0, 6)
        b.put(x, 1, 6, DUST)
    b.put(4, 1, 6, rs.PALETTE["route"])           # entry block
    b.put(5, 0, 6, SLAB_TOP)                      # the probed floor
    b.put(5, 1, 6, rs.repeater("west"))
    b.put(6, 1, 6, rs.PALETTE["route"])           # exit block
    for x in (7, 8):
        b.stone(x, 0, 6)
        b.put(x, 1, 6, DUST)
    pC = (8, 1, 6)

    sim = b.sim()
    lv = rs.Levers(sim, LV)
    p1 = []
    for a in (0, 1):
        for bb in (0, 1):
            bits = [0] * len(LV)
            bits[iA], bits[iB] = a, bb
            lv.set(bits)
            p1.append((sim.power(*pA) > 0) == (a == 1)
                      and (sim.power(*pB) > 0) == (bb == 1))
    bits = [0] * len(LV)
    bits[iC] = 1
    lv.set(bits)
    c_on = sim.power(*pC) > 0
    bits[iC] = 0
    lv.set(bits)
    c_off = sim.power(*pC) > 0

    verdicts = [
        ("P1 solid caps over a live straight run are harmless "
         "+ 2y dust/solid/dust isolated (4 combos)", all(p1)),
        ("P2 repeater fires from a slab-top floor (sturdiness is "
         "static-only; moot in v2 -- floors go solid)", c_on and not c_off),
    ]
    bad = 0
    for text, ok in verdicts:
        print("%s %s" % ("PASS" if ok else "FAIL", text))
        bad += 0 if ok else 1
    return bad


# -- the bus ----------------------------------------------------------------

def build_bus(b, z=0):
    """The dense vertical bus along +X at this z.  Returns out-probe cells."""
    for x in range(RUN):
        b.stone(x, 0, z, "rail_floor")            # bit0 conductive floor
    station_x = set()
    for e in STAGES:
        station_x.update((e, e + 1, e + 2))
    outs = []
    for bit in range(BITS):
        y = y_of(bit)
        if bit > 0:
            for x in range(RUN):                  # full SOLID separator layer:
                b.put(x, y - 1, z, separator())   # caps the run below (P1) or
        for x in range(RUN):                      # sits block-on-block (inert)
            if x not in station_x:
                b.put(x, y, z, DUST)              # the bit's rail
        for e in STAGES:                          # ALIGNED station columns
            b.put(e, y, z, rs.PALETTE["route"])   # entry block (floats)
            b.put(e + 1, y, z, rs.repeater("west"))   # on the solid layer
            b.put(e + 2, y, z, rs.PALETTE["route"])   # exit block (fresh 15)
        outs.append((RUN - 1, y, z))
    return outs


def lever_column(b, x, z):
    """8 stacked drivers: block at even y, lever at odd y (bit n at 1+2n)."""
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
    probe_bad = probe()

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
          "%d-block run, 2 ALIGNED refresh stages, 2y pitch, no glass/slabs)"
          % (good, total, len(patterns), RUN))

    if probe_bad or good != total:
        print("NOT saving showcase piece (probes or suite red)")
        return 1

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
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
