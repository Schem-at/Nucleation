"""Form-pivot adapter tiles: vertical(2y stack) <-> horizontal(2z flat) bus
form conversion, plus the flat 90-degree corner.  All cells COMPUTED from the
material predicates (materials.py; PROBED table, notes-material-model.md).

THREE TILES (each verified through real levers, saved BAKED at rest):

pivot_v2h / pivot_h2v -- the same geometry, repeater direction flipped
(repeaters make the tile one-way, so the two directions are DISTINCT tiles):

  * VERTICAL port at x=0, z=0: bit n dust at (0, 1+2n, 0) -- the dense
    vertical form (bus8_probe.py: 2y pitch, solid separators).
  * FAN column at x=0: bit n runs +Z at its own level y=1+2n from z=0 to
    its lane z=2n.  The fan is a straight-run stack: bit n's support layer
    at y=2n doubles as the solid cap over bit n-1's fan dust -- computed by
    the cap law exactly like bus8_run's separator layers (no diagonals in
    use on a straight run; probed P1).
  * POINTING LAW (probe_pivot.py, NEW probed physics): dust weak-powers
    only the blocks it POINTS into -- its connection axes (dust, levers,
    repeaters count; a single connection extends to the opposite side; a
    true dot powers all sides).  A run that turns a corner does NOT fire a
    station entry block around the corner.  Consequences: every station
    entry is preceded by >= 1 dust cell on the station's own axis, and a
    station EXIT block may sit AT a corner for free (strong power lights
    dust on any side -- probe D).
  * LANE APPROACH dust at x=1 (its corner neighbour gives it the X axis),
    then the block-sandwich STATION at x=2..4 (probe_station.py).  v2h
    flows +X (repeater[facing=west]); h2v flows -X (facing=east), entry
    and exit blocks swapped.  Refreshing BEFORE the staircase makes every
    lane's decay budget uniform.  Bit 7's fan is 16 cells port->lane, one
    over budget, so bit 7 (alone) gets an inline fan station at z=6..8.
  * STAIRCASE + flat run at x=5..19 in lane z=2n: bit n descends 1y per 1x
    from y=1+2n to y=1 (2n steps), then runs flat; x-advance is uniform,
    so every bit sees exactly 15 dust cells after its station exit (last
    at ss1 -- probed legal: A15/I15, an ss1 dust still drives a station).
    Step-UPPER dusts sit on pick_support(need_conductor=True): the diode
    law (a 1-y step passes DOWN only if the upper dust's support conducts)
    -- v2h needs downhill (probe E), and solid supports make the same
    geometry conduct uphill for h2v (UP always passes).  No cell sits
    above the lower dust of an in-use diagonal, so NO transparency
    anywhere: lanes are 2 apart in z and dust reads span only 1 cell.
  * HORIZONTAL port at x=19: bit n dust at (19, 1, 2n) -- the flat form.
  Footprint: 20(x) x 16(y) x 15(z).  9 repeaters.  Zero glass.

pivot_flat90 -- the flat form turns a corner (+X run becomes +Z run),
CONCENTRIC lanes: bit n enters at z=2n and corners at x=14-2n, so the bus
order is PRESERVED in the travel frame (bit 0 stays the leftmost lane;
coordinate mapping z=2n -> x=14-2n).  Concentric corners are the unique
planar, crossing-free layout.  Stations obey the pointing law: S1 on the
in-leg at x=1..3 for bits with corner x >= 4 (bits 0..5); S2 for every bit
on the out-leg at z=zn+2..zn+4, entered through the one dust cell after
the corner (which acquires the Z axis).  Worst dust run is 15 cells
(bit 0's out-leg, probed-legal ss1 at the port).
  Footprint: 15(x) x 2(y) x 20(z).  14 repeaters.  Flat, all-solid.
  One verified orientation; the other 3 are Y-rotations/mirrors of the
  template (coordinate transform only, no new physics).

Verification per tile: all-off + walking-ones + all-on + 0xAA + 0x55 =
12 patterns x 8 bits = 96 checks, zero crosstalk tolerated; 288 total.
"""
import os

import nucleation as n

import rs
from rs import DUST
from materials import pick_support, separator

BITS = 8
XSTAIR = 5               # first staircase dust cell after the lane station
XPORT = 19               # horizontal port x (15 dusts after station exit)
ZPORT = 19               # flat90 out-port z
FANSTA = 6               # bit 7's fan refresh station z = 6..8


def yv(bit):
    return 1 + 2 * bit


def lane(bit):
    return 2 * bit


# -- tile: vertical <-> horizontal pivot ------------------------------------

def build_pivot(b, to_horizontal):
    """The form adapter.  to_horizontal=True: v2h (flows V->H, repeaters +X);
    False: h2v (flows H->V, repeaters -X).  Returns (v_ports, h_ports).

    POINTING LAW (probe_pivot.py): dust weak-powers only the blocks it
    points into -- its connection axes (a lever counts; a dead end extends
    to the opposite side).  So every station ENTRY is preceded by at least
    one dust cell on the station's own axis; station EXIT blocks are
    strongly powered and light dust on any side (probe D), so a corner may
    follow an exit for free."""
    v_ports, h_ports = [], []
    for bit in range(BITS):
        y, zn = yv(bit), lane(bit)
        fansta = zn + 2 > 15        # port->lane-dust exceeds 15 cells: bit 7
        # fan column x=0: port dust at z=0, straight run to the corner z=2n.
        # Support layer y=2n is ALSO the cap over bit n-1's fan dust below:
        # straight run -> solid separator, computed (cap law; probed P1).
        for z in range(0, zn + 1):
            b.put(0, y - 1, z, separator())
            if fansta and z in (FANSTA, FANSTA + 1, FANSTA + 2):
                continue            # station cells, filled below
            b.put(0, y, z, DUST)
        if fansta:                  # inline fan refresh (straight, +Z axis)
            if to_horizontal:
                b.put(0, y, FANSTA, rs.PALETTE["route"])       # entry
                b.put(0, y, FANSTA + 1, rs.repeater("north"))  # flows +Z
                b.put(0, y, FANSTA + 2, rs.PALETTE["route"])   # exit
            else:
                b.put(0, y, FANSTA + 2, rs.PALETTE["route"])   # entry
                b.put(0, y, FANSTA + 1, rs.repeater("south"))  # flows -Z
                b.put(0, y, FANSTA, rs.PALETTE["route"])       # exit
        v_ports.append((0, y, 0))
        # lane approach dust at x=1: its corner neighbour (or the strong
        # exit block behind it) gives it the X axis -> it points into the
        # lane station's entry block.
        b.put(1, y - 1, zn, pick_support(solid=rs.PALETTE["lane"],
                                         why="lane dust bit %d" % bit))
        b.put(1, y, zn, DUST)
        # block-sandwich lane station at x=2..4 (probe_station.py)
        floor = pick_support(need_conductor=True, solid=rs.PALETTE["lid"],
                             why="pivot repeater floor bit %d" % bit)
        if to_horizontal:
            b.put(2, y, zn, rs.PALETTE["route"])        # entry (weak drive)
            b.put(3, y - 1, zn, floor)
            b.put(3, y, zn, rs.repeater("west"))        # flows +X
            b.put(4, y, zn, rs.PALETTE["route"])        # exit (fresh 15)
        else:
            b.put(4, y, zn, rs.PALETTE["route"])        # entry
            b.put(3, y - 1, zn, floor)
            b.put(3, y, zn, rs.repeater("east"))        # flows -X
            b.put(2, y, zn, rs.PALETTE["route"])        # exit
        # staircase + flat run, x=5..19: descend 1y/1x to y=1, then flat.
        for x in range(XSTAIR, XPORT + 1):
            yy = max(1, y - (x - XSTAIR))
            upper = yy > 1              # this dust is the UPPER end of a step
            b.put(x, yy - 1, zn,
                  pick_support(need_conductor=upper, solid=rs.PALETTE["gate"],
                               why="stair bit %d x=%d" % (bit, x)))
            b.put(x, yy, zn, DUST)
        h_ports.append((XPORT, 1, zn))
    return v_ports, h_ports


# -- tile: flat 90-degree corner --------------------------------------------

def build_flat90(b):
    """Flat-form corner, +X run -> +Z run, concentric lanes: bit n enters at
    z=2n and corners at x=14-2n.  Stations obey the pointing law:

      * S1 (bits with corner x >= 4, i.e. 0..5): on the in-leg at x=1..3 --
        the port dust's lever + dead-end extension give it the X axis;
      * S2 (every bit): on the out-leg at z=zn+2..zn+4 -- one dust cell
        after the corner acquires the Z axis and points into the entry.

    Returns (in_ports, out_ports), both indexed by bit."""
    in_ports, out_ports = [], []
    for bit in range(BITS):
        zn = lane(bit)
        xc = 14 - 2 * bit               # concentric corner column
        s1 = xc >= 4                    # room for the in-leg station
        for x in range(0, xc + 1):      # in-leg +X at z=zn
            if s1 and x in (1, 2, 3):
                continue                # S1 cells, filled below
            b.put(x, 0, zn, separator())
            b.put(x, 1, zn, DUST)
        if s1:
            b.put(1, 1, zn, rs.PALETTE["route"])        # entry
            b.put(2, 0, zn,
                  pick_support(need_conductor=True, solid=rs.PALETTE["lid"],
                               why="flat90 S1 floor bit %d" % bit))
            b.put(2, 1, zn, rs.repeater("west"))        # flows +X
            b.put(3, 1, zn, rs.PALETTE["route"])        # exit (fresh 15)
        in_ports.append((0, 1, zn))
        # out-leg +Z: one dust past the corner, then S2, then the run out
        b.put(xc, 0, zn + 1, separator())
        b.put(xc, 1, zn + 1, DUST)                      # acquires the Z axis
        b.put(xc, 1, zn + 2, rs.PALETTE["route"])       # S2 entry
        b.put(xc, 0, zn + 3,
              pick_support(need_conductor=True, solid=rs.PALETTE["lid"],
                           why="flat90 S2 floor bit %d" % bit))
        b.put(xc, 1, zn + 3, rs.repeater("north"))      # flows +Z
        b.put(xc, 1, zn + 4, rs.PALETTE["route"])       # exit (fresh 15)
        for z in range(zn + 5, ZPORT + 1):
            b.put(xc, 0, z, separator())
            b.put(xc, 1, z, DUST)
        out_ports.append((xc, 1, ZPORT))
    return in_ports, out_ports


# -- verification harness ---------------------------------------------------

def lever_for(b, port, dx, levers):
    """A lever `dx` cells from `port` along x, on its own block."""
    x, y, z = port
    b.stone(x + dx, y - 1, z)
    b.force(x + dx, y, z, rs.LEVER_OFF)
    levers.append((x + dx, y, z))


def check(sim, outs, pattern, label):
    got = [1 if sim.power(*p) > 0 else 0 for p in outs]
    want = [(pattern >> i) & 1 for i in range(BITS)]
    ok = got == want
    print("%s %-10s want %s got %s" % ("PASS" if ok else "FAIL", label,
                                       "".join(map(str, reversed(want))),
                                       "".join(map(str, reversed(got)))))
    return BITS if ok else sum(1 for g, w in zip(got, want) if g == w)


PATTERNS = ([(0, "all-off")]
            + [(1 << i, "walk-%d" % i) for i in range(BITS)]
            + [(0xFF, "all-on"), (0xAA, "alt-AA"), (0x55, "alt-55")])


def run_tile(name, builder, lever_dx, out_name):
    b = rs.Build(name)
    ins, outs = builder(b)
    levers = []
    for p in ins:
        lever_for(b, p, lever_dx, levers)
    sim = b.sim()
    lv = rs.Levers(sim, levers)
    good = total = 0
    for pat, label in PATTERNS:
        lv.set([(pat >> i) & 1 for i in range(BITS)])
        good += check(sim, outs, pat, label)
        total += BITS
    print("%s: %d/%d output checks (%d patterns x 8 bits)"
          % (name, good, total, len(PATTERNS)))
    if good != total:
        print("NOT saving %s (suite red)" % out_name)
        return False
    # bake at rest and save
    lv.set([0] * BITS)
    rest = [sim.power(*p) for p in outs]
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
    ok = run_tile("pivot_v2h",
                  lambda b: build_pivot(b, True), -1, "pivot_v2h.schem")
    ok &= run_tile("pivot_h2v",
                   lambda b: tuple(reversed(build_pivot(b, False))), +1,
                   "pivot_h2v.schem")
    ok &= run_tile("pivot_flat90", build_flat90, -1, "pivot_flat90.schem")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
