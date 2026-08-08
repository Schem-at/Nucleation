"""A 4-bit ripple-carry adder in redstone, built with nucleation's Python API.

Each bit is one full adder laid out as a small PLA:

  * six horizontal RAILS run along X carrying C, ~C, A, ~A, B, ~B.  Every rail is
    dust at y=1 on a stone floor with a stone LID at y=2 -- the lid is what makes
    the whole thing possible, because it kills every diagonal that would
    otherwise short a rail to the wiring above it.
  * one COLUMN per product term.  A column spurs off each rail it needs, the spur
    dead-ends into a block, and a torch on top of that block injects NOT(rail)
    into a collector dust line running above the rails at y=4.
  * the collector dead-ends into a block and a torch on top inverts it, so a
    column outputs AND(its rails).  A repeater partway along the collector keeps
    the far taps from decaying to nothing.

  sum   = A'B'C + A'BC' + AB'C' + ABC          (4 columns, needs both polarities)
  carry = AB + AC + BC                          (3 columns, positive literals only)

Column outputs are OR-ed by simply merging them onto one dust lane, and the carry
lane walks over to the next bit's C rail.  Rail order puts C first so that hop is
short.

Verified against every one of the 512 input combinations with mc-tick.
"""
import argparse
import itertools
import rs
from rs import DUST, LEVER_OFF, repeater

# ---- rails ---------------------------------------------------------------
C, NC, A, NA, B, NB = range(6)
RAIL_NAMES = ["C", "~C", "A", "~A", "B", "~B"]
rail_z = lambda r: 3 * r          # 0, 3, 6, 9, 12, 15
spur_z = lambda r: 3 * r + 1
tap_z = lambda r: 3 * r + 2

X_DRIVE = 0                       # rail drivers (levers / arriving carry)
X_INV = 2                         # head inverters that make the ~rails
X_LID0 = 6                        # lids start here, east of the head
X_RAIL_END = 24
# A rail loses one signal level per block, so it cannot reach the far columns on
# its own.  Refresh it at these x -- they must miss every column, because a
# repeater ignores dust on its sides and a column taps the rail from the side.
X_RAIL_REPEATER = (7, 16)

# ---- per-column wiring ---------------------------------------------------
Z_COLL0 = 2                       # collector spans the tap columns
Z_COLL1 = 3 * 5 + 2 + 1           # 18
Z_COLL_REPEATER = 9               # a rail-z, so never a tap column
Z_GATE = Z_COLL1 + 1              # 19
Z_LANE = Z_GATE + 1               # 20  (output lanes, y=7)
Z_CARRY = Z_LANE + 2              # 22  (carry runs west along here)
Y_COLL = 4
Y_LANE = 7

# sum needs both polarities; carry is a majority so positive literals suffice
SUM_TERMS = [(NA, NB, C), (NA, B, NC), (A, NB, NC), (A, B, C)]
CARRY_TERMS = [(A, B), (A, C), (B, C)]
SUM_X = [8, 10, 12, 14]
CARRY_X = [18, 20, 22]
X_SUM_LAMP = 9                    # a gap column inside the sum lane's span
X_CARRY_LAMP = 19                 # ditto for the carry lane

CELL_DZ = 24


def facing_toward(frm, to):
    """The compass name of the step from `frm` to `to` (a repeater's facing names
    its INPUT side, so pass the previous cell as `to`)."""
    dx, dz = to[0] - frm[0], to[2] - frm[2]
    if dx == -1:
        return "west"
    if dx == 1:
        return "east"
    if dz == -1:
        return "north"
    if dz == 1:
        return "south"
    raise ValueError("not a unit horizontal step: %s -> %s" % (frm, to))


def run_path(b, cells, repeat_every=14):
    """Lay a dust path through `cells`, adding a stone support under each and
    dropping in a repeater every so often so nothing decays to zero.

    Pitch 14 is probe-verified (probe_station I15: a repeater fires from ss1
    dust, true max 15 between refreshes) minus 2 levels of margin; the first
    refresh stays within 5 cells because the source may arrive decayed."""
    since = max(0, repeat_every - 5)
    for i, cell in enumerate(cells):
        x, y, z = cell
        b.stone(x, y - 1, z)
        prev, nxt = (cells[i - 1] if i else None), (cells[i + 1] if i + 1 < len(cells) else None)
        straight = (prev and nxt and prev[1] == y == nxt[1]
                    and (prev[0] == x == nxt[0] or prev[2] == z == nxt[2]))
        # bank a refresh at the last straight cell before a repeater-free
        # tail (stairs/landings) so the tail never starts nearly spent
        nxt2 = cells[i + 2] if i + 2 < len(cells) else None
        cont = (nxt2 is not None and nxt[1] == y == nxt2[1]
                and (nxt[0] == x == nxt2[0] or nxt[2] == z == nxt2[2]))
        since += 1
        if straight and since >= (repeat_every if cont else repeat_every - 6):
            b.put(x, y, z, repeater(facing_toward(cell, prev)))
            since = 0
        else:
            b.put(x, y, z, DUST)


def head_inverter(b, zb, r_in, r_out):
    """Drive rail r_out with NOT(rail r_in): spur off the input rail, invert with
    a torch, then stair the result back down onto the output rail."""
    zs, zt = zb + spur_z(r_in), zb + tap_z(r_in)
    b.stone(X_INV, 0, zs)                     # floor, so the spur is not floating
    b.stone(X_INV, 0, zt)
    b.put(X_INV, 1, zs, DUST)                 # spur, dead-ends into the block east
    b.stone(X_INV, 1, zt)
    b.put(X_INV, 2, zt, rs.TORCH)
    b.stone(X_INV, 3, zt)
    b.put(X_INV, 4, zt, DUST)
    # stair down 3 levels heading east, then step onto the output rail
    for k, (x, y) in enumerate([(X_INV + 1, 3), (X_INV + 2, 2), (X_INV + 3, 1)]):
        b.stone(x, y - 1, zt)
        b.put(x, y, zt, DUST)
    assert X_INV + 3 == 5 and zb + rail_z(r_out) == zt + 1


def column(b, zb, x, taps):
    """One product term: tap the given rails, OR them, invert -> AND(rails)."""
    for r in taps:
        b.stone(x, 0, zb + spur_z(r))         # floor, so the spur is not floating
        b.stone(x, 0, zb + tap_z(r))
        b.put(x, 1, zb + spur_z(r), DUST)     # spur branches off the rail
        b.stone(x, 1, zb + tap_z(r))          # ...and dead-ends into this block
        b.put(x, 2, zb + tap_z(r), rs.TORCH)  # torch on top inverts the rail
    for z in range(zb + Z_COLL0, zb + Z_COLL1 + 1):
        b.stone(x, 3, z)                      # collector floor (= tap outputs)
        if z == zb + Z_COLL_REPEATER:
            b.put(x, Y_COLL, z, repeater("north"))
        else:
            b.put(x, Y_COLL, z, DUST)
    b.stone(x, Y_COLL, zb + Z_GATE)           # collector dead-ends into this
    b.put(x, 5, zb + Z_GATE, rs.TORCH)
    b.stone(x, 6, zb + Z_GATE)
    b.put(x, Y_LANE, zb + Z_GATE, DUST)       # column output


def lane(b, zb, xs, lamp_x):
    """Merge a group of column outputs onto one dust lane == wired OR.

    The lamp goes UNDER the lane, because dust always weakly powers the block
    beneath it.  A lamp set beside the lane does NOT light: the lane's end cell
    also turns north into its column, so it is a corner and points away from the
    lamp rather than into it.
    """
    for x in range(min(xs), max(xs) + 1):
        b.put(x, 6, zb + Z_LANE, rs.LAMP if x == lamp_x else rs.STONE)
        b.put(x, Y_LANE, zb + Z_LANE, DUST)


def fa_cell(b, i, drive_c_with_lever):
    """One full adder. Returns (A lever, B lever, C lever or None)."""
    zb = i * CELL_DZ
    # rails: C and the two operand rails are driven at x=0, the ~rails at x=5
    for r in range(6):
        z, x0 = zb + rail_z(r), (5 if r in (NC, NA, NB) else X_DRIVE)
        for x in range(x0, X_RAIL_END + 1):
            b.stone(x, 0, z)
            b.put(x, 1, z, repeater("west") if x in X_RAIL_REPEATER else DUST)
            if x >= X_LID0:
                b.stone(x, 2, z)              # the lid
    levers = []
    for r in (A, B):
        b.force(X_DRIVE, 1, zb + rail_z(r), LEVER_OFF)
        levers.append((X_DRIVE, 1, zb + rail_z(r)))
    if drive_c_with_lever:
        b.force(X_DRIVE, 1, zb + rail_z(C), LEVER_OFF)
        levers.append((X_DRIVE, 1, zb + rail_z(C)))

    for r_in, r_out in ((C, NC), (A, NA), (B, NB)):
        head_inverter(b, zb, r_in, r_out)

    for x, taps in zip(SUM_X, SUM_TERMS):
        column(b, zb, x, taps)
    for x, taps in zip(CARRY_X, CARRY_TERMS):
        column(b, zb, x, taps)
    lane(b, zb, SUM_X, X_SUM_LAMP)
    lane(b, zb, CARRY_X, X_CARRY_LAMP)
    return levers


def carry_cells(i):
    """The ordered dust path taking bit i's carry lane to bit i+1's C rail."""
    zb = i * CELL_DZ
    x_from = CARRY_X[-1]
    cells = [(x_from, Y_LANE, zb + Z_LANE + 1), (x_from, Y_LANE, zb + Z_CARRY)]
    cells += [(x, Y_LANE, zb + Z_CARRY) for x in range(x_from - 1, X_LID0 - 1, -1)]
    # stair down six levels heading west, landing one z short of the next C rail
    z_drop = zb + CELL_DZ - 1
    cells += [(X_LID0, Y_LANE, z_drop)]
    for k in range(1, 7):
        cells.append((X_LID0 - k, Y_LANE - k, z_drop))
    cells += [(X_DRIVE, 1, zb + CELL_DZ)]     # step onto the next C rail
    return cells


def carry_hop(b, i):
    run_path(b, carry_cells(i))


def build(nbits=4):
    b = rs.Build("ripple_carry_adder_%dbit" % nbits)
    ab_levers, cin = [], None
    for i in range(nbits):
        lv = fa_cell(b, i, drive_c_with_lever=(i == 0))
        ab_levers.append((lv[0], lv[1]))
        if i == 0:
            cin = lv[2]
    for i in range(nbits - 1):
        carry_hop(b, i)
    return b, ab_levers, cin


def read_out(sim, nbits):
    """Sum bits come off each cell's sum lane; the final carry off the top cell."""
    s = [sim.on(SUM_X[0], Y_LANE, i * CELL_DZ + Z_LANE) for i in range(nbits)]
    cout = sim.on(CARRY_X[0], Y_LANE, (nbits - 1) * CELL_DZ + Z_LANE)
    return s, cout


def read_lamps(sim, nbits):
    """The same values as read_out, but as a player sees them: the output lamps.

    Checked against the wires on every case -- a lamp that is merely decorative
    is a lamp nobody notices is dark."""
    s = [sim.lit(X_SUM_LAMP, Y_LANE - 1, i * CELL_DZ + Z_LANE) for i in range(nbits)]
    cout = sim.lit(X_CARRY_LAMP, Y_LANE - 1, (nbits - 1) * CELL_DZ + Z_LANE)
    return s, cout


def bake_states(b, sim):
    """Write the settled world back into the schematic.

    Authoring with plain set_block stores every wire as an unconnected dot at
    power 0; Minecraft re-derives connections when the build is pasted, so it
    works, but the FILE describes a circuit full of disconnected dots and any
    static reader (renderer, mesher) draws it that way.  This is what
    `set_block("...{simulate=true}")` does per placement -- place through the
    engine, keep whatever the world became -- done once for the whole build
    instead of re-settling the world a thousand times.

    Bake from the all-levers-off rest state, so the file is saved at rest.
    """
    changed = 0
    for pos in sorted(b.cells):
        state = sim.block(*pos)
        if state != b.cells[pos] and "air" not in state:
            b.force(*pos, state)
            changed += 1
    return changed


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bits", type=int, default=4)
    ap.add_argument("--out", default="ripple_carry_adder_4bit.schem")
    ap.add_argument("--quick", action="store_true", help="test a sample, not all cases")
    args = ap.parse_args()

    b, ab_levers, cin = build(args.bits)
    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    print("build: %d blocks, extent x %d..%d  y %d..%d  z %d..%d"
          % (len(b.cells), x0, x1, y0, y1, z0, z1))

    # The simulator will happily tick a floating wire, so check support statically
    # before trusting anything the simulation says.
    import audit
    problems = audit.audit(b.cells)
    if any(problems.values()):
        for kind, items in problems.items():
            if items:
                print("STRUCTURAL: %s x%d, e.g. %s" % (kind, len(items), items[0]))
        return 1
    print("structural audit: clean (nothing floating or unattached)")

    sim = b.sim()
    print("placed; quiescent:", sim.sim.is_quiescent(), "non-air:", sim.sim.non_air_count())

    order = [cin] + [p for pair in ab_levers for p in pair]
    lv = rs.Levers(sim, order)

    n = args.bits
    cases = list(itertools.product([0, 1], repeat=2 * n + 1))
    if args.quick:
        cases = cases[::37]
    bad, lamp_bad = [], []
    for bits in cases:
        ci = bits[0]
        av = [bits[1 + 2 * i] for i in range(n)]
        bv = [bits[2 + 2 * i] for i in range(n)]
        lv.set(bits)
        s, cout = read_out(sim, n)
        ls, lcout = read_lamps(sim, n)
        a = sum(v << i for i, v in enumerate(av))
        bb = sum(v << i for i, v in enumerate(bv))
        want = a + bb + ci
        got = sum(int(v) << i for i, v in enumerate(s)) + (int(cout) << n)
        if got != want:
            bad.append((a, bb, ci, got, want))
        if [bool(v) for v in ls] != [bool(v) for v in s] or bool(lcout) != bool(cout):
            lamp_bad.append((a, bb, ci, s, ls, cout, lcout))
    print("exhaustive check: %d/%d correct" % (len(cases) - len(bad), len(cases)))
    for a, bb, ci, got, want in bad[:10]:
        print("   %d + %d + %d = %d, got %d" % (a, bb, ci, want, got))
    print("output lamps agree with the wires: %d/%d" % (len(cases) - len(lamp_bad), len(cases)))
    for rec in lamp_bad[:5]:
        print("   %d+%d+%d wires=%s lamps=%s cout=%s lamp_cout=%s" % rec)
    if lamp_bad:
        bad = bad or lamp_bad

    if bad:
        return 1

    lv.set([0] * len(order))          # back to rest before recording the world
    wires_before = sum("redstone_wire" in v for v in b.cells.values())
    print("baked %d of %d block states from the settled world (%d wires)"
          % (bake_states(b, sim), len(b.cells), wires_before))

    b.s.save_to_file(args.out)
    print("saved", args.out)

    # Re-verify the FILE, not just the build we happen to hold in memory: a save
    # normalises the origin and round-trips the palette, so prove the artifact
    # someone will actually paste still adds correctly.  InWorld trusts the saved
    # states as-is, so it also proves the bake produced a genuinely resting world.
    import nucleation as nuc
    reloaded = nuc.Schematic.open(args.out)
    rsim = rs.Sim(nuc.TickSimulation.from_schematic(
        reloaded, nuc.TickSettleMode.InWorld, 0, 0, 0, rs.EXTRA_STATES), (0, 0, 0))
    print("reloaded InWorld: already at rest?", rsim.sim.is_quiescent(),
          "(ticks needed:", rsim.sim.tick_count(), ")")
    print("saved rail wire :", reloaded.get_block_string(10, 1, 6))
    rsim.settle()
    if "lever" not in rsim.block(*order[0]):
        print("!! reloaded schematic is not origin-aligned; found %r at %s"
              % (rsim.block(*order[0]), order[0]))
        return 1
    rlv = rs.Levers(rsim, order)
    rbad = 0
    for bits in cases:
        rlv.set(bits)
        a = sum(bits[1 + 2 * i] << i for i in range(n))
        bb = sum(bits[2 + 2 * i] << i for i in range(n))
        s, cout = read_out(rsim, n)
        got = sum(int(v) << i for i, v in enumerate(s)) + (int(cout) << n)
        rbad += (got != a + bb + bits[0])
    print("reloaded .schem re-check: %d/%d correct" % (len(cases) - rbad, len(cases)))

    print("\nlever map (schematic coords, x y z):")
    print("  Cin      %s" % (cin,))
    for i, (la, lb) in enumerate(ab_levers):
        print("  A%d %-10s B%d %s" % (i, la, i, lb))
    print("output lamps: sum bit i at (%d, %d, %d + %d*i), carry-out at (%d, %d, %d)"
          % (X_SUM_LAMP, Y_LANE - 1, Z_LANE, CELL_DZ,
             X_CARRY_LAMP, Y_LANE - 1, (args.bits - 1) * CELL_DZ + Z_LANE))
    return 1 if rbad else 0


if __name__ == "__main__":
    raise SystemExit(main())
