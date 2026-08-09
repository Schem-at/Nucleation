"""Probe the user's packing claim: "spiral staircases can tile if they are
offset 180 degrees".

They can.  This finds the exact law, which is stronger and more useful than the
180-degree special case:

  A ring riser is dust on the perimeter of an sx-by-sz footprint, one cell per
  y, so **y == path index**.  Two cells of that path are planar-adjacent iff
  they are consecutive on it -- PROVIDED the cycle is CHORDLESS, which the
  perimeter is exactly when sx >= 3 AND sz >= 3.  Put a second bit on the same
  ring at path-index offset `sep` and, in every column the two share, they sit
  `sep` apart in y.  So the whole legality question collapses to one integer:

      sep = 1  the two are one step apart in every shared column -> ONE net
      sep = 2  the second bit's SUPPORT lands directly above the first bit's
               dust: it is a conductor in that bit's CUT cell, so it severs
               the step -- BOTH bits die (this is the quiet failure: nothing
               leaks, nothing conducts)
      sep >= 3 legal.  Any two bits are >= 3 apart in y in every shared
               column, so no step and no merge exists.

  A period-8 ring (3x3) therefore takes exactly TWO bits, and their only legal
  offsets are 3 and 4 -- offset 4 of 8 IS "180 degrees".  Bigger perimeters
  take floor(perimeter / 3) bits.

  A 2-wide ring is NOT chordless -- ring(3,2) has (1,0) at index 1 planar
  adjacent to (1,1) at index 4, a chord of length 3 -- so its bits at offset 3
  land on the SAME level and merge.  That is why the footprint must be >= 3x3.

Groups:
  T1 the chordless precondition: 2-wide rings fail at every offset,
  T2 the offset law on a 3x3 ring: sweep 1..4, only 3 and 4 pass,
  T3 sep=2 (90 degrees) fails QUIETLY -- both bits dead, zero leak,
  T4 floor(perimeter/3) bits on wider rings: 4x3 -> 3, 5x3 -> 4,
  T5 a whole BYTE on one 11x3 ring: 8 bits, sampled + walking patterns,
  T6 the same ring riser carries all its bits DOWNWARD too,
  T7 tiling ring risers next to each other: a 1-column gap is required,
  T8 the closed-form predictor agrees with every sim verdict above.

Run: ~/eda-venv/bin/python probe_spiral_tiling.py
"""
import itertools
import rs
import audit
import vforms as vf
from rs import DUST, STONE, LEVER_OFF, repeater

VERDICTS = []
PREDICTED = []          # (label, predicted_ok, measured_ok)


def V(ok, text, extra=""):
    VERDICTS.append((bool(ok), text, extra))


# ------------------------------------------------------------ the closed form
def legal(sx, sz, offsets):
    """The predictor the router would use.  No simulation."""
    if sx < 3 or sz < 3:
        return False                      # perimeter has a chord
    m = 2 * (sx + sz) - 4
    for a, b in itertools.combinations(offsets, 2):
        d = (a - b) % m
        if min(d, m - d) < 3:
            return False
    return True


def patterns(n, cap=8):
    """Full matrix while it is small; walking ones/zeros + 0xAA/0x55 above."""
    if n <= cap:
        return list(itertools.product((0, 1), repeat=n))
    pats = [tuple([0] * n), tuple([1] * n)]
    for i in range(n):
        pats.append(tuple(1 if j == i else 0 for j in range(n)))
        pats.append(tuple(0 if j == i else 1 for j in range(n)))
    pats.append(tuple(j % 2 for j in range(n)))
    pats.append(tuple((j + 1) % 2 for j in range(n)))
    return pats


# ------------------------------------------------------------------- harness
def rig(rings, n, from_top=False, stub=2):
    """`rings` = [(sx, sz, ox, oz, [offsets...]), ...].  One lever per bit.

    Each bit is driven lever -> dust -> REPEATER -> its entry port, so the
    riser's first cell starts at a full 15 and the exit still reads 5-6 after a
    10-cell climb.  That matters: with a bare lever the exit sat at ss 1, and a
    leak arriving one cell longer than the intended path read 0 -- an ss-starved
    rig reports "no crosstalk" for a circuit that is shorted.  It is read
    `stub` dust cells PAST the exit port (the crosswire rule) AND every one of
    the bit's own dust cells is checked for leakage, which no ss budget can
    mask.
    """
    b = rs.Build("tiling")
    levs, outs, nets = [], [], []
    for (sx, sz, ox, oz, offs) in rings:
        rg = vf.ring(sx, sz)
        for p in offs:
            cs = vf.ring_riser(b, rg, ox, oz, 4, n, p)
            nets.append(cs)
            src, snk = (cs[-1], cs[0]) if from_top else (cs[0], cs[-1])
            d = vf.ring_outward(src, ox, oz, sx, sz)
            x, y, z = src
            # repeater 1 out, facing the riser; lever 3 out
            # the repeater sits one cell OUTWARD, so it must output back inward:
            # its input side is the direction it was displaced along.
            back = {(0, -1): "north", (0, 1): "south",
                    (-1, 0): "west", (1, 0): "east"}[d]
            b.force(x + d[0], y - 1, z + d[1], STONE)
            b.force(x + d[0], y, z + d[1], repeater(back))
            for k in (2, 3):
                b.force(x + d[0] * k, y - 1, z + d[1] * k, STONE)
                b.force(x + d[0] * k, y, z + d[1] * k, DUST)
            lc = (x + d[0] * 4, y, z + d[1] * 4)
            b.force(lc[0], lc[1] - 1, lc[2], STONE)
            b.force(*lc, LEVER_OFF)
            levs.append(lc)
            # readout: `stub` cells out of the exit port
            e = vf.ring_outward(snk, ox, oz, sx, sz)
            x, y, z = snk
            for k in range(1, stub + 1):
                b.force(x + e[0] * k, y - 1, z + e[1] * k, STONE)
                b.force(x + e[0] * k, y, z + e[1] * k, DUST)
            outs.append((x + e[0] * stub, y, z + e[1] * stub))
    return b, levs, outs, nets


def measure(label, rings, n=None, from_top=False, predict=None):
    n = N if n is None else n
    nb = sum(len(r[4]) for r in rings)
    b, levs, outs, nets = rig(rings, n, from_top)
    floating = sum(len(v) for v in audit.audit(b.cells).values())
    sim = b.sim()
    lv = rs.Levers(sim, levs)
    lo = [None] * nb
    hi = [0] * nb
    pats = patterns(nb)
    for bits in pats:
        lv.set(list(bits))
        for j in range(nb):
            v = sim.power(*outs[j])
            if bits[j]:
                lo[j] = v if lo[j] is None else min(lo[j], v)
            else:
                # a leak ANYWHERE in an undriven bit counts, not only at its
                # exit -- the exit alone can be masked by ss decay
                hi[j] = max([v] + [sim.power(*c) for c in nets[j]] + [hi[j]])
    ok = all(v and v > 0 for v in lo) and all(v == 0 for v in hi) and not floating
    if predict is not None:
        PREDICTED.append((label, predict, ok))
    return ok, lo, hi, len(pats), floating


N = 10

# ------------------------------------------------- T1 chordless precondition
for (sx, sz) in ((3, 2), (4, 2)):
    m = 2 * (sx + sz) - 4
    for sep in range(1, m // 2 + 1):
        ok, lo, hi, np_, flt = measure(
            "ring%dx%d sep=%d" % (sx, sz, sep), [(sx, sz, 0, 0, [0, sep])], N,
            predict=legal(sx, sz, [0, sep]))
        V(not ok,
          "T1 a 2-wide ring (%dx%d, period %d) at offset %d is NOT usable -- "
          "the perimeter has a chord of length 3, so the cycle is not "
          "chordless and y == path index no longer separates the bits"
          % (sx, sz, m, sep),
          "driven=%s leak=%s" % (lo, hi))

# ------------------------------------------------------- T2/T3 the offset law
for sep in (1, 2, 3, 4):
    ok, lo, hi, np_, flt = measure("ring3x3 sep=%d" % sep,
                                   [(3, 3, 0, 0, [0, sep])], N,
                                   predict=legal(3, 3, [0, sep]))
    want = sep >= 3
    V(ok == want,
      "T2 two bits on ONE 3x3 ring riser at path offset %d (of period 8): %s"
      % (sep, "PACK, zero crosstalk over 4/4 patterns" if want
         else "illegal"),
      "driven=%s leak=%s" % (lo, hi))
    if sep == 2:
        V(any(v is None or v <= 0 for v in lo) and any(v > 0 for v in hi),
          "T3 offset 2 (90 degrees) fails as a SEVERED bit, not as a loud "
          "short: the upper bit's SOLID support sits in the lower bit's CUT "
          "cell, so one bit is DEAD (never reaches its exit in any pattern) "
          "while foreign power sits inside its cells.  Its EXIT is quiet in "
          "all four patterns, so an exit-only, ss-starved probe scores this "
          "configuration as crosstalk-free -- which is why every cell of every "
          "undriven bit is checked here",
          "driven=%s worst-leak-anywhere=%s" % (lo, hi))
    if sep == 4:
        V(ok, "T2 offset 4 of period 8 IS the user's \"offset 180 degrees\": "
              "two spiral staircases interleave in ONE 3x3 footprint with zero "
              "crosstalk -- 4.5 xz cells per bit", "driven=%s" % lo)

# --------------------------------------------- T4 floor(perimeter/3) bits
for (sx, sz, nbits) in ((4, 3, 3), (5, 3, 4), (6, 3, 4)):
    m = 2 * (sx + sz) - 4
    offs = vf.ring_bits(m, 3)
    ok, lo, hi, np_, flt = measure("ring%dx%d %dbits" % (sx, sz, len(offs)),
                                   [(sx, sz, 0, 0, offs)], N,
                                   predict=legal(sx, sz, offs))
    V(ok and len(offs) == nbits,
      "T4 a %dx%d ring riser (period %d) carries %d bits at offset 3 -- "
      "%.2f xz cells per bit, %d/%d patterns"
      % (sx, sz, m, len(offs), sx * sz / float(len(offs)), np_, np_),
      "offsets=%s driven=%s leak=%s" % (offs, lo, hi))

# ------------------------------------------------------------- T5 a full byte
offs8 = vf.ring_bits(24, 3)
ok, lo, hi, np_, flt = measure("ring11x3 byte", [(11, 3, 0, 0, offs8)], N,
                               predict=legal(11, 3, offs8))
V(ok and len(offs8) == 8,
  "T5 a whole BYTE rises on ONE 11x3 ring riser: 8 bits at offset 3, "
  "%d patterns (all-off/all-on/walking-ones/walking-zeros/0xAA/0x55), zero "
  "crosstalk -- %.3f xz cells per bit" % (np_, 33 / 8.0),
  "offsets=%s driven=%s leak=%s floating=%d" % (offs8, lo, hi, flt))

# ---------------------------------------------------------- T6 the same, DOWN
for (sx, sz, lbl) in ((3, 3, "3x3 pair"), (5, 3, "5x3 quad")):
    m = 2 * (sx + sz) - 4
    offs = vf.ring_bits(m, 3)
    ok, lo, hi, np_, flt = measure("%s down" % lbl, [(sx, sz, 0, 0, offs)], N,
                                   from_top=True, predict=legal(sx, sz, offs))
    V(ok,
      "T6 the %s ring riser carries all %d bits DOWNWARD in the same "
      "footprint (drive at the top, read at the bottom) -- the packing is "
      "direction-agnostic" % (lbl, len(offs)),
      "driven=%s leak=%s" % (lo, hi))

# ----------------------------------------------------------------- T7 tiling
# Two rings side by side.  Sweep the x-pitch AND the second ring's rotation:
# report the smallest pitch at which SOME rotation is clean, and whether the
# evenly-spread offsets from `ring_bits` are among the clean ones.
for (sx, sz) in ((2, 2), (3, 3), (5, 3)):
    m = 2 * (sx + sz) - 4
    offs = vf.ring_bits(m, 3) or [0]     # a period-4 ring holds a single bit
    for pit in (sx, sx + 1):
        clean = []
        for q in range(m):
            offs2 = [(o + q) % m for o in offs]
            ok, lo, hi, np_, flt = measure(
                "two %dx%d pitch=%d q=%d" % (sx, sz, pit, q),
                [(sx, sz, 0, 0, offs), (sx, sz, pit, 0, offs2)], N)
            if ok:
                clean.append(q)
        V(bool(clean) == (pit > sx),
          "T7 two %dx%d ring risers (%d bits each, offsets %s) at x-pitch %d: "
          "%s" % (sx, sz, len(offs), offs, pit,
                  "%d of %d relative rotations are clean (%s) -- a wide bus "
                  "tiles with ONE empty column between rings"
                  % (len(clean), m, clean) if clean else
                  "no rotation is clean; FLUSH rings always leak, because the "
                  "seam leaves two foreign dusts 1 cell apart"),
          "clean rotations=%s" % clean)

# ------------------------------------------------------- T8 predictor agrees
mismatch = [(l, p, m_) for (l, p, m_) in PREDICTED if p != m_]
V(not mismatch,
  "T8 the closed-form predictor `legal(sx, sz, offsets)` (footprint >= 3x3 and "
  "every pair of offsets >= 3 apart around the perimeter) agrees with all %d "
  "simulated configurations" % len(PREDICTED),
  "mismatches=%s" % mismatch)

bad = 0
for ok, text, extra in VERDICTS:
    print("%s %s%s" % ("PASS" if ok else "FAIL", text,
                       ("   [%s]" % extra) if extra else ""))
    bad += 0 if ok else 1
print("probe_spiral_tiling: %d/%d" % (len(VERDICTS) - bad, len(VERDICTS)))
raise SystemExit(1 if bad else 0)
