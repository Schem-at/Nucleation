"""BUS LEVEL-SHIFT: a dense N-bit 2y-pitch stack changes level by K, in form.

Generalizes the dip-under's +-1 (bus8_cross.py v2) to an arbitrary K.  The
whole stack moves in LOCKSTEP, so the 2y pitch -- and therefore the
interleave that isolates the bits -- is invariant through the shift.  Every
cell is COMPUTED from materials.py's predicates; no material is hardcoded
for a constrained cell.

GEOMETRY (flow along +axis, one column per offset, all bits in lockstep):

  a level costs TWO columns -- step off, land -- so the slope is 1y per 2
  cells.  A continuous 1y-per-1x staircase is IMPOSSIBLE for a dense stack:
  every cell would be both the step-UPPER of the next diagonal (support must
  CONDUCT, diode law) and the cap over bit n-1's in-use lower diagonal
  (support must INSULATE, cut law).  pick_support() refuses it.  Landing flat
  for one cell splits those two roles across two columns -- the ALTERNATION.

  DESCENDING          ASCENDING
  o even  step off    solid support   |  o even  step off    GLASS (bit>0)
  o odd   landed      GLASS (bit>0)   |  o odd   landed      solid support
  o = 2K  resume      solid           |  o = 2K  resume      solid

  THE TRANSPARENT PARITY FLIPS WITH DIRECTION.  Descending, the step-off
  cell's support must conduct (a 1y step passes DOWN only over a conducting
  upper support) and the landed cell's support caps bit n-1's live diagonal.
  Ascending, up-steps conduct over anything, so the roles swap: the step-off
  support is the cap that must insulate, and the landed support is the SOLID
  that severs the cross-bit diagonal (bit n's step-off dust and bit n-1's
  landed dust are 1y apart in adjacent columns -- an unintended diagonal,
  cut by the conductor above the lower one).

  Dust decays 1/cell and stairs cannot host repeaters, so every
  `station_every` levels the tile inserts the same block-sandwich station
  the dip tile uses (entry block / repeater on a solid floor / exit block),
  which restores a fresh 15.  K is therefore unbounded.

VERIFICATION: walking-ones (8), all-on, alternating 0xAA/0x55 and 8 seeded
random patterns, per K in {1,2,3,5,8} x {down,up} -- 19 patterns x 8 outputs
x 10 configs, zero crosstalk tolerated.
"""
import random

import rs
from rs import DUST
from materials import pick_support, separator, GLASS

BITS = 8
# Dust cells tolerated between refresh stations inside a shift.  Stairs
# cannot host repeaters, so the staircase spends the signal 2 cells per
# level; 12 leaves the exit cell plus a joint inside dust's 15-cell reach.
DUST_CAP = 12


# -- the column plan --------------------------------------------------------

def plan_shift(k, down=True, since0=0, cap=DUST_CAP):
    """Column plan for a k-level shift: ([(offset, dy, kind)], since_out).

    dy is the level RELATIVE to the entry level (negative descending).
    kinds: 'step' (step off this cell), 'land' (landed after the step),
    'flat' (resume the form), 'entry'/'rep'/'exit' (refresh station).

    `since0` is the dust already spent since the last refresh on the way in;
    a station is inserted BEFORE any level whose two dust cells would blow
    the cap, so K is unbounded and a stale arrival is refreshed on entry.
    """
    if k < 1:
        raise ValueError("k must be >= 1")
    sgn = -1 if down else 1
    cols, o, dy, since = [], 0, 0, since0
    for _ in range(k):
        if since + 2 > cap:
            for kind in ("entry", "rep", "exit"):
                cols.append((o, dy, kind))
                o += 1
            since = 0
        cols.append((o, dy, "step"))
        o += 1
        dy += sgn
        cols.append((o, dy, "land"))
        o += 1
        since += 2
    cols.append((o, dy, "flat"))
    return cols, since + 1


def shift_len(k, down=True, since0=0, cap=DUST_CAP):
    """Cells of run a k-level shift consumes (offset 0 .. len-1)."""
    return plan_shift(k, down, since0, cap)[0][-1][0] + 1


def support_for(kind, bit, down):
    """The support material under one shift cell, COMPUTED from its two
    constraints.  Returns None where the cell needs no support."""
    if kind in ("entry", "exit"):
        return None                      # station blocks float, as in the dip
    if kind == "rep":
        return pick_support(need_conductor=True, solid=rs.PALETTE["lid"],
                            why="level-shift station repeater floor")
    if kind == "flat":
        return separator(solid=rs.PALETTE["gate"])
    # A staircase cell. Descending: the step-off cell is the diagonal's UPPER
    # dust (must conduct downhill) and the landed cell caps bit n-1's live
    # diagonal. Ascending: up-steps conduct over anything, so the roles swap.
    if down:
        conductor, insulator = kind == "step", kind == "land"
    else:
        conductor, insulator = kind == "land", kind == "step"
    return pick_support(need_conductor=conductor,
                        need_insulator=insulator and bit > 0,
                        solid=rs.PALETTE["gate"], transparent=GLASS,
                        why="level-shift %s cell (%s)"
                            % (kind, "down" if down else "up"))


def emit_shift(b, x0, y0, z, bits, k, down=True, since0=0, facing="west"):
    """Stamp the shift tile along +X from x0; bit n enters at y0 + 2n.
    Returns (x_after, dy) -- the first free column and the level delta."""
    cols, _ = plan_shift(k, down, since0)
    for bit in range(bits):
        base = y0 + 2 * bit
        for o, dy, kind in cols:
            x, y = x0 + o, base + dy
            sup = support_for(kind, bit, down)
            if sup is not None:
                b.put(x, y - 1, z, sup)
            if kind == "rep":
                b.put(x, y, z, rs.repeater(facing))
            elif kind in ("entry", "exit"):
                b.put(x, y, z, rs.PALETTE["route"])
            else:
                b.put(x, y, z, DUST)
    return x0 + cols[-1][0] + 1, cols[-1][1]


# -- harness ----------------------------------------------------------------

def flat_run(b, xs, y0, z, bits, facing="west", station_at=None):
    """Plain 2y-pitch stack over the x range `xs`; optional 3-cell station
    starting at `station_at` (entry/rep/exit) so the shift enters at 15."""
    for bit in range(bits):
        y = y0 + 2 * bit
        for x in xs:
            if station_at is not None and station_at <= x <= station_at + 2:
                continue
            b.put(x, y - 1, z, separator(solid=rs.PALETTE["gate"]))
            b.put(x, y, z, DUST)
        if station_at is not None:
            b.put(station_at, y, z, rs.PALETTE["route"])
            b.put(station_at + 1, y - 1, z,
                  pick_support(need_conductor=True, solid=rs.PALETTE["lid"],
                               why="feed station repeater floor"))
            b.put(station_at + 1, y, z, rs.repeater(facing))
            b.put(station_at + 2, y, z, rs.PALETTE["route"])


def lever_column(b, x, z, y0, bits):
    levers = []
    for bit in range(bits):
        y = y0 + 2 * bit
        b.stone(x, y - 1, z)
        b.force(x, y, z, rs.LEVER_OFF)
        levers.append((x, y, z))
    return levers


def build_case(k, down, bits=BITS, z=0, since0=0):
    """lever -> flat run (+station when since0==0) -> shift(k) -> flat -> outs.

    With since0 > 0 the feed station is dropped, so the shift really does
    arrive stale and must refresh itself on entry."""
    # Keep every level (and every support, one lower) at y >= 0.
    y0 = 2 + (k if down else 0)
    b = rs.Build("levelshift_%s%d_s%d" % ("dn" if down else "up", k, since0))
    flat_run(b, range(0, 5), y0, z, bits,
             station_at=(2 if since0 == 0 else None))
    x_after, dy = emit_shift(b, 5, y0, z, bits, k, down=down, since0=since0)
    flat_run(b, range(x_after, x_after + 3), y0 + dy, z, bits)
    levers = lever_column(b, -1, z, y0, bits)
    outs = [(x_after + 2, y0 + dy + 2 * bit, z) for bit in range(bits)]
    return b, levers, outs, dy


def run_case(k, down, bits=BITS, verbose=False, since0=0):
    b, levers, outs, dy = build_case(k, down, bits, since0=since0)
    sim = b.sim()
    lv = rs.Levers(sim, levers)
    rng = random.Random(9000 + k + (0 if down else 1))
    patterns = ([("walk-%d" % i, 1 << i) for i in range(bits)]
                + [("all-on", (1 << bits) - 1), ("alt-AA", 0xAA),
                   ("alt-55", 0x55)]
                + [("rand-%d" % i, rng.randrange(1 << bits)) for i in range(8)])
    good = total = 0
    for label, p in patterns:
        lv.set([(p >> i) & 1 for i in range(bits)])
        got = [1 if sim.power(*q) > 0 else 0 for q in outs]
        want = [(p >> i) & 1 for i in range(bits)]
        good += sum(1 for g, w in zip(got, want) if g == w)
        total += bits
        if verbose or got != want:
            fmt = lambda v: "".join(map(str, reversed(v)))
            print("    %s %-8s want %s got %s"
                  % ("PASS" if got == want else "FAIL", label,
                     fmt(want), fmt(got)))
    lv.set([0] * bits)
    rest = [sim.power(*q) for q in outs]
    quiet = all(v == 0 for v in rest)
    n_glass = sum(1 for v in b.cells.values() if v == GLASS)
    print("  %s K=%-2d %-4s since0=%-3d dy=%-3d len=%-3d glass=%-3d %d/%d %s"
          % ("PASS" if good == total and quiet else "FAIL", k,
             "down" if down else "up", since0, dy,
             shift_len(k, down, since0), n_glass,
             good, total, "" if quiet else "NOT QUIET AT REST %r" % rest))
    return good == total and quiet


def main():
    print("bus level-shift template: %d-bit dense stack, 2y pitch" % BITS)
    ok = True
    for since0 in (0, 11):
        for down in (True, False):
            for k in (1, 2, 3, 5, 8):
                ok = run_case(k, down, since0=since0) and ok
    print("ALL GREEN" if ok else "RED")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
