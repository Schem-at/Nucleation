"""Guard the TEMPLATE FORMULAS in crosswire_tiles.md against the schematics.

`verify_crosswire.py` proves the ground-truth .schem files work.  This proves
that the closed-form cell formulas written down for the Rust port describe
EXACTLY those files -- so a transcription slip in the doc fails here instead of
being discovered in Rust.  Also asserts the physical invariants each template
leans on (every dust support conducts; every bump's cut cells are clear; the
updown intersection cell is air).

Run:  ~/eda-venv/bin/python crosswire/test_crosswire_templates.py
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import materials as M
import verify_crosswire as V


# --------------------------------------------------- xw_buffered  (CROSSWIRE002)
def classic_unit(y):
    """Unit whose Z-line level is y; the X-line is at y+1.  y-1 is the previous
    unit's cap layer, which doubles as this unit's Z supports."""
    return {
        "dust": ([(2, y, 0), (2, y, 1), (2, y, 4)]
                 + [(0, y + 1, 2), (1, y + 1, 2), (3, y + 1, 2), (4, y + 1, 2)]),
        "repeater": [(2, y, 3), (2, y + 1, 2)],
        "solid": ([(2, y - 1, z) for z in (0, 1, 3, 4)]     # Z supports
                  + [(x, y, 2) for x in range(5)]           # X supports; x=2 is
                  + [(2, y + 1, z) for z in (0, 1, 3, 4)]),  # the STRONG cell
    }


# -------------------------------------------------------- xw_hop  (region A)
def hop_line(y):
    """Dust cells of the region-A line whose PORT level is y (period 4 from
    y=1).  Each line hops 1 up (the cell at y+1) over the line at y-1."""
    r = (y - 1) % 4
    if r == 0:                                    # X-line, lane z=3, hop at x=5
        return [(0, y, 4), (1, y, 4), (1, y, 3), (2, y, 3), (3, y, 3),
                (4, y, 3), (5, y + 1, 3), (6, y, 3)]
    if r == 1:                                    # Z-line, lane x=3, hop at z=3
        return [(4, y, 0), (4, y, 1), (3, y, 1), (3, y, 2), (3, y + 1, 3),
                (3, y, 4), (3, y, 5), (3, y, 6), (3, y, 7), (4, y, 7), (4, y, 8)]
    if r == 2:                                    # X-line, lane z=5, hop at x=3
        return [(0, y, 4), (1, y, 4), (1, y, 5), (2, y, 5), (3, y + 1, 5),
                (4, y, 5), (5, y, 5), (6, y, 5)]
    return [(4, y, 0), (4, y, 1), (5, y, 1), (5, y, 2), (5, y, 3), (5, y, 4),
            (5, y + 1, 5), (5, y, 6), (4, y, 7), (5, y, 7), (4, y, 8)]


def hop_travel_axis(y):
    return (1, 0, 0) if (y - 1) % 4 in (0, 2) else (0, 0, 1)


# ----------------------------------------------------- xw_updown  (region B)
def updown_lines(y):
    """(X-line, Z-line) whose PORTS are both at level y (period 4 from y=3).
    On y = 3+4k the X dips and the Z bumps; on y = 5+4k they swap."""
    if (y - 3) % 4 == 0:
        xline = ([(0, y, 13), (1, y, 13)]
                 + [(x, y - 1, 12) for x in range(1, 6)]        # DIP, -z jog
                 + [(5, y, 13), (6, y, 13)])
        zline = ([(3, y, 10), (3, y, 11), (4, y, 11)]
                 + [(4, y + 1, z) for z in (12, 13, 14)]        # BUMP, +x jog
                 + [(4, y, 15), (3, y, 15), (3, y, 16)])
    else:
        zline = ([(3, y, 10), (3, y, 11)]
                 + [(2, y - 1, z) for z in range(11, 16)]       # DIP, -x jog
                 + [(3, y, 15), (3, y, 16)])
        xline = ([(0, y, 13), (1, y, 13), (1, y, 14)]
                 + [(x, y + 1, 14) for x in (2, 3, 4)]          # BUMP, +z jog
                 + [(5, y, 14), (5, y, 13), (6, y, 13)])
    return xline, zline


# --------------------------------------------------------------------- checks
FAILS = []


def eq(label, formula, actual):
    f, a = set(formula), set(actual)
    if f != a:
        FAILS.append("%s: formula-only %s / schem-only %s"
                     % (label, sorted(f - a), sorted(a - f)))
    return f == a


def nets_by_min_y(region):
    return {min(c[1] for c in comp): comp for comp, _e, _a in V.wire_nets(region)}


def main():
    cl = V.load("CROSSWIRE002_classic_crosswire")
    for unit in range(5):
        y = 1 + 2 * unit
        t = classic_unit(y)
        act = {k: v for k, v in cl.items() if y <= k[1] <= y + 1}
        eq("classic u%d dust" % unit, t["dust"],
           [k for k, v in act.items() if "wire" in v])
        eq("classic u%d repeater" % unit, t["repeater"],
           [k for k, v in act.items() if "repeater" in v])
        if unit < 4:      # the file truncates the top unit's cap layer
            band = {k: v for k, v in cl.items() if y - 1 <= k[1] <= y + 1}
            eq("classic u%d solid" % unit, t["solid"],
               [k for k, v in band.items() if "wool" in v])
    #   the crossing cell is the Z-repeater's output block, and the X-repeater
    #   stands directly on it
    for unit in range(5):
        y = 1 + 2 * unit
        assert M.conducts(cl[(2, y, 2)]), "the crossing cell must be a conductor"
        assert "repeater" in cl[(2, y + 1, 2)]
        for x in (1, 3):                       # its two NEIGHBOURS carry the
            assert M.conducts(cl[(x, y, 2)])   # X-line's dust and stay dead
            assert "wire" in cl[(x, y + 1, 2)]

    whole = V.load("CROSSWIRE001_instant_crosswire")

    regA = V.slab_z(whole, 0, 8)
    netsA = nets_by_min_y(regA)
    for y in sorted(netsA):
        if y > 16:
            continue
        eq("hop y=%d" % y, hop_line(y), netsA[y])
        cells = hop_line(y)
        bump = [c for c in cells if c[1] == y + 1][0]
        support = (bump[0], bump[1] - 1, bump[2])
        assert M.conducts(regA.get(support)), ("hop support must conduct", y)
        #   ...and that same support is the CUT cell of the crossed line's two
        #   would-be diagonals: the crossed line's dust is directly under it.
        #   (y=1 hops over the line of the unit BELOW, which the file omits.)
        if y > 1:
            crossed = regA.get((support[0], support[1] - 1, support[2]), "")
            assert "wire" in crossed, (y, support, crossed)
        #   the bump's own two cut cells (along its travel axis) must be clear
        d = hop_travel_axis(y)
        for s in (-1, 1):
            cc = (bump[0] + s * d[0], bump[1], bump[2] + s * d[2])
            assert not M.cuts_step(regA.get(cc)), ("hop cut cell", y, cc)
    for y, comp in netsA.items():
        for c in comp:
            assert M.conducts(regA.get((c[0], c[1] - 1, c[2]))), ("support", c)

    regB = V.slab_z(whole, 10, 16)
    #   A complete tile's ports both leave the region; a net that dead-ends
    #   inside it is the file's truncated top unit (also skipped by
    #   verify_crosswire.auto_lines) and has no formula to check.
    def leaves(c):
        return c[0] in (0, 6) or c[2] in (10, 16)

    by_port, truncated = {}, set()
    for comp, ends, _a in V.wire_nets(regB):
        py = ends[0][1]
        if not all(leaves(c) for c in ends):
            truncated.add(py)
            continue
        by_port.setdefault(py, []).append(comp)
    for py in sorted(by_port):
        if py in truncated or len(by_port[py]) != 2:
            print("  (port level %d: incomplete tile in the file -- skipped)"
                  % py)
            continue
        xl, zl = updown_lines(py)
        a, b = by_port[py]
        xa, za = (a, b) if max(c[0] for c in a) - min(c[0] for c in a) == 6 \
            else (b, a)
        eq("updown y=%d X" % py, xl, xa)
        eq("updown y=%d Z" % py, zl, za)
        #   THE point of this family: the intersection cell is empty
        assert regB.get((3, py, 13)) is None, ("intersection must be air", py)
    for comp, _e, _a in V.wire_nets(regB):
        for c in comp:
            assert M.conducts(regB.get((c[0], c[1] - 1, c[2]))), ("support", c)

    for f in FAILS:
        print("FAIL %s" % f)
    print("test_crosswire_templates: %s"
          % ("all formulas match the schematics"
             if not FAILS else "%d mismatches" % len(FAILS)))
    return 1 if FAILS else 0


if __name__ == "__main__":
    raise SystemExit(main())
