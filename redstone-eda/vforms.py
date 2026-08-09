"""Closed-form VERTICAL TRANSPORT templates (the ones notes-vertical-transport.md
tabulates).  Every function here is a formula in a level index or a path index,
so it ports to Rust mechanically -- exactly like `pivot_tiles.py` and the
`crosswire_tiles.md` formulas.

Verified by `probe_vertical_forms.py` and `probe_spiral_tiling.py`.

Four forms:

  torch_ladder()   1x1 column, 2 y per torch, inverts per torch, refreshes to
                   15.  UP ONLY.  Densest and fastest *on paper*, and towers
                   may stand at x-pitch 1 provided their PORTS alternate sides
                   -- but **NOT USABLE FOR SWITCHING BUS DATA**: it costs 1 gt
                   per y of rise on the critical path, and its torches BURN OUT
                   when the input is toggled faster than one change per 4 gt
                   (`probe_torch_burnout.py`).  Static or low-toggle control
                   signals only; call `assert_data_safe()` first.
  glass_tower()    2x1 column of dust on all-glass supports, 1 y per cell,
                   0 gt, non-inverting, -1 ss per y.  UP ONLY (transparent
                   diode).  z-pitch 2.
  ring_riser()     dust on the CHORDLESS perimeter of an sx-by-sz footprint
                   (both >= 3), 1 y per cell, all-solid supports.
                   BIDIRECTIONAL -- this is the spiral staircase, and the only
                   form that goes DOWN.  Hosts floor(perimeter/3) independent
                   bits at path-index separation 3; separation 4 on a period-8
                   ring is the user's "offset 180 degrees".
  half_slope()     horizontal level change, 1 y per 2 x cells, supports
                   alternating SOLID (even x) / TRANSPARENT (odd x), which is
                   what lets a second line ride 2 y above it.

Coordinate convention: every builder takes an origin and returns the list of
DUST cells in signal order, so cs[0] is the entry port and cs[-1] the exit.
"""
import rs
from rs import DUST, STONE

GLASS = "minecraft:glass"
SLAB_TOP = "minecraft:smooth_stone_slab[type=top,waterlogged=false]"


# --------------------------------------------------------------------------
# 0. the form contract -- latency and TOGGLE-RATE limits are first-class
# --------------------------------------------------------------------------
# mc-tick's constants (crates/mc-tick/src/components.rs).  Only turn-offs count
# toward the budget, so a full toggle cycle spends one.
TORCH_BURNOUT_WINDOW = 60      # game ticks the engine looks back over
TORCH_MAX_TURNOFFS = 8         # turn-offs in that window before it goes dead
# Measured, not derived: probe_torch_burnout.py B1/B2.  Hold-time per input
# state, in game ticks, below which a torch starts dropping transitions.
TORCH_MIN_HOLD_GT = 4

#: One row per form.  A router must read `max_toggle_gt` and `burnout_risk`
#: BEFORE it reads the density: a carrier that cannot survive the traffic is
#: not a cheap carrier, it is a broken one.
#:
#:   gt_per_y            latency added per y of level change
#:   blocks_per_y_per_bit marginal block cost of one more y (measured, section M
#:                       of probe_vertical_forms.py)
#:   xz_claim_per_bit    footprint a router must reserve, incl. neighbour pitch
#:   max_toggle_gt       minimum hold time per input state.  None = unlimited
#:   burnout_risk        "none" | "disqualifying"
#:   data_safe           may this carry continuously switching bus data?
FORMS = {
    "torch_ladder": dict(
        direction="up", rate="2 y per torch", gt_per_y=1.0,
        blocks_per_y_per_bit=1.0, xz_claim_per_bit=3.0,
        inverting=True, ss_per_y=0, reach_y=None,
        max_toggle_gt=TORCH_MIN_HOLD_GT, burnout_risk="disqualifying",
        data_safe=False,
        note="fastest and densest on paper; burns out under switching data"),
    "glass_tower": dict(
        direction="up", rate="1 y per cell", gt_per_y=0.0,
        blocks_per_y_per_bit=2.0, xz_claim_per_bit=4.0,
        inverting=False, ss_per_y=1, reach_y=14,
        max_toggle_gt=None, burnout_risk="none", data_safe=True,
        note="pure dust; also a free one-way diode"),
    "ring_riser": dict(
        direction="both", rate="1 y per cell", gt_per_y=0.0,
        blocks_per_y_per_bit=2.0, xz_claim_per_bit=3.75,
        inverting=False, ss_per_y=1, reach_y=14,
        max_toggle_gt=None, burnout_risk="none", data_safe=True,
        note="5x3 / 4 bits; the only form that descends"),
    "half_slope": dict(
        direction="both", rate="1 y per 2 x cells", gt_per_y=0.0,
        blocks_per_y_per_bit=2.0, xz_claim_per_bit=None,
        inverting=False, ss_per_y=2, reach_y=7,
        max_toggle_gt=None, burnout_risk="none", data_safe=True,
        note="a SLOPE, not a shaft: footprint grows with rise but is shared "
             "by every bit in the stack"),
    "stair_1to1": dict(
        direction="both", rate="1 y per x cell", gt_per_y=0.0,
        blocks_per_y_per_bit=2.0, xz_claim_per_bit=None,
        inverting=False, ss_per_y=1, reach_y=14,
        max_toggle_gt=None, burnout_risk="none", data_safe=True,
        note="cannot stack a second line at any pitch"),
    "repeater_drop": dict(
        direction="down", rate="1 y per station", gt_per_y=2.0,
        blocks_per_y_per_bit=3.0, xz_claim_per_bit=2.0,
        inverting=False, ss_per_y=0, reach_y=None,
        max_toggle_gt=None, burnout_risk="none", data_safe=True,
        note="repeaters do not burn out; only the single stage is probed"),
}


def data_safe(form, toggle_period_gt=None):
    """HARD precondition, not a cost: may `form` carry switching data?

    `toggle_period_gt` is how long the caller guarantees each input state is
    held.  Omit it to ask the unconditional question ("is this safe for a bus
    whose traffic I cannot bound?"), which is the question a router asks.
    """
    f = FORMS[form]
    lim = f["max_toggle_gt"]
    if lim is None:
        return True
    if toggle_period_gt is None:
        return False
    return toggle_period_gt >= lim


def assert_data_safe(form, toggle_period_gt=None):
    if not data_safe(form, toggle_period_gt):
        f = FORMS[form]
        raise AssertionError(
            "%s is not safe for this traffic: it needs each input state held "
            ">= %d gt (torch burnout: %d turn-offs per %d gt), and the caller "
            "%s. Use a torch-free form -- ring_riser descends and climbs at "
            "0 gt with no toggle limit."
            % (form, f["max_toggle_gt"], TORCH_MAX_TURNOFFS,
               TORCH_BURNOUT_WINDOW,
               "did not bound its toggle rate" if toggle_period_gt is None
               else "holds only %d gt" % toggle_period_gt))


# --------------------------------------------------------------------------
# 1. torch ladder -- 1x1, UP only, 2 y per torch, inverts per torch
# --------------------------------------------------------------------------

def torch_ladder(b, x, z, y0, torches, port_side=-1, entry_axis="z"):
    """A 1x1 torch tower at column (x, z), base block at y0.

    Cell formula, for t in 0 .. torches-1:
        (x, y0 + 2t,     z)  solid          <- torch t's attachment
        (x, y0 + 2t + 1, z)  redstone_torch
        (x, y0 + 2*torches, z) solid        <- the CAP: strongly powered
    Rise = 2 * torches.  Inverting iff `torches` is odd.

    PORTS.  The entry is a dust cell one step to `port_side` of the BASE, which
    dead-ends into it (weak power; the torch reads weak).  The exit is a dust
    cell one step to `port_side` of the CAP at cap level, which reads the cap's
    STRONG power on its side face.  Both ports live on side `port_side` so that
    two ladders at x-pitch 1 alternate sides (+1/-1) and never share a lane:
    same-side pitch-1 ports MERGE into one dust net, and a merged net turns
    into a T that no longer points into the bases (the POINTING LAW) -- the
    whole array then reads a constant.  Probed: probe_vertical_forms L4/L5.

    Returns (entry_cell, exit_cell, cap_cell).
    """
    d = (0, port_side) if entry_axis == "z" else (port_side, 0)
    for t in range(torches):
        b.force(x, y0 + 2 * t, z, STONE)
        b.force(x, y0 + 2 * t + 1, z, rs.TORCH)
    capy = y0 + 2 * torches
    b.force(x, capy, z, STONE)
    entry = (x + d[0], y0, z + d[1])
    b.force(entry[0], y0 - 1, entry[2], STONE)
    b.force(*entry, DUST)
    exit_ = (x + d[0], capy, z + d[1])
    b.force(exit_[0], capy - 1, exit_[2], STONE)
    b.force(*exit_, DUST)
    return entry, exit_, (x, capy, z)


def ladder_bus(b, x0, z, y0, nbits, torches, axis="x"):
    """`nbits` torch ladders at PITCH 1, ports alternating sides.

    1 xz cell per bit -- the densest legal vertical bus.  Returns
    [(entry, exit), ...] per bit.
    """
    out = []
    for i in range(nbits):
        cx = x0 + i if axis == "x" else x0
        cz = z if axis == "x" else z + i
        side = -1 if i % 2 == 0 else +1
        e, o, _ = torch_ladder(b, cx, cz, y0, torches, port_side=side,
                               entry_axis="z" if axis == "x" else "x")
        out.append((e, o))
    return out


# --------------------------------------------------------------------------
# 2. transparent ("glass") tower -- 2x1, UP only, 1 y per cell, 0 gt
# --------------------------------------------------------------------------

def glass_tower(b, x, z, y0, n, axis="x", support=GLASS):
    """A 2-cell-wide vertical zigzag: dust at (x + k%2, y0 + k, z).

    Every support is the CUT cell of the dust below it, so every support must
    be a NON-CONDUCTOR -- which makes every step's upper dust sit on a
    non-conductor, which is the transparent-diode law: the tower conducts UP
    ONLY.  All-solid supports cut every step and the tower is dead after one
    cell (probe_vertical_forms G2).

    -1 ss per cell, so a single tower reaches 14 y from a 15 source.
    z-pitch 2 between towers (pitch 1 leaks: the glass supports keep the
    cross-net diagonals alive too).
    """
    cs = []
    for k in range(n):
        c = (x + (k % 2), y0 + k, z) if axis == "x" else (x, y0 + k, z + (k % 2))
        cs.append(c)
    for (cx, cy, cz) in cs:
        b.force(cx, cy - 1, cz, support)
        b.force(cx, cy, cz, DUST)
    return cs


# --------------------------------------------------------------------------
# 3. ring riser (the spiral staircase) -- BIDIRECTIONAL, 1 y per cell
# --------------------------------------------------------------------------

def ring(sx, sz):
    """Clockwise perimeter of an sx-by-sz footprint, starting at (0,0).

    For sx >= 3 and sz >= 3 this cycle is CHORDLESS in the grid graph: two
    perimeter cells are planar-adjacent only if they are consecutive on the
    path.  That is the whole legality argument for stacking several bits on
    one ring (see ring_riser), and it is exactly why a 2-wide ring fails:
    ring(3,2) has (1,0) at index 1 planar-adjacent to (1,1) at index 4, a
    chord of length 3, so nets 3 apart land on the SAME level and merge.
    """
    cs = [(x, 0) for x in range(sx)]
    cs += [(sx - 1, z) for z in range(1, sz)]
    cs += [(x, sz - 1) for x in range(sx - 2, -1, -1)]
    cs += [(0, z) for z in range(sz - 2, 0, -1)]
    return cs


def ring_riser(b, rg, ox, oz, y0, n, phase=0, support=STONE):
    """One bit climbing/descending the ring `rg`: dust at path index (k+phase),
    level y0+k.  Because y == path index, two planar-adjacent cells of a
    chordless ring differ by exactly 1 in y -- the intended step -- and every
    other pair differs by >= 2, which is why nothing else in the column
    connects.  All supports SOLID and every cut cell (directly above a dust)
    is AIR, so the form conducts BOTH WAYS at -1 ss per y.
    """
    m = len(rg)
    cs = [(ox + rg[(k + phase) % m][0], y0 + k, oz + rg[(k + phase) % m][1])
          for k in range(n)]
    for (x, y, z) in cs:
        b.force(x, y - 1, z, support)
        b.force(x, y, z, DUST)
    return cs


def ring_bits(m, sep=3):
    """The phases for a multi-bit ring riser of perimeter `m`, spread EVENLY.

    A net at phase p and one at phase q sit |p-q| apart in y in every shared
    column.  A separation of 1 IS the step (they would merge), 2 puts a foreign
    support directly above a dust and cuts both, and 3 is the first legal
    value -- so a ring holds floor(m/3) bits.  They are then spread as evenly
    as that count allows rather than bunched at exactly 3, because the spare
    separation is what buys isolation from the NEXT ring across the seam: a
    period-8 ring holds 2 bits, and placing them at 0 and 4 -- the user's
    "offset 180 degrees" -- is what lets two such rings tile at x-pitch 4,
    while 0 and 3 leaks across the seam.  Probed: probe_spiral_tiling T2/T7.
    """
    k = m // sep
    return [int(round(i * m / float(k))) % m for i in range(k)]


def ring_bus(b, sx, sz, ox, oz, y0, n, sep=3, support=STONE):
    """A full multi-bit ring riser.  Returns [cells_per_bit, ...].

    Density: sx*sz xz cells for floor(perimeter/sep) bits.
    """
    rg = ring(sx, sz)
    return [ring_riser(b, rg, ox, oz, y0, n, p, support)
            for p in ring_bits(len(rg), sep)]


def ring_outward(cell, ox, oz, sx, sz):
    """A direction that leaves the ring's bbox from a perimeter cell -- where a
    port stub must go."""
    x, _, z = cell
    lx, lz = x - ox, z - oz
    if lz == 0:
        return (0, -1)
    if lz == sz - 1:
        return (0, +1)
    if lx == 0:
        return (-1, 0)
    return (+1, 0)


# --------------------------------------------------------------------------
# 4. half slope -- horizontal level change that a second line can ride above
# --------------------------------------------------------------------------

def slope_levels(ncells, rise_every=2, up=True):
    """y offsets of a `1 y per rise_every x cells` slope."""
    s = 1 if up else -1
    return [s * (i // rise_every) for i in range(ncells)]


def lower_end_of_step(ys):
    """Per index: is this dust the LOWER end of an in-use 1-y step?

    That is the ONLY predicate the support material depends on, because the
    support of a line is the CUT cell of the line 2 y beneath it (support of
    index i sits at y_i - 1 == (y_i - 2) + 1 == the cut cell of the lower
    line's dust at the same index).
    """
    out = []
    for i, y in enumerate(ys):
        lo = ((i + 1 < len(ys) and ys[i + 1] == y + 1)
              or (i > 0 and ys[i - 1] == y + 1))
        out.append(lo)
    return out


def half_slope(b, x0, z, y0, ncells, alternate=True, up=True, rise_every=2,
               transparent=GLASS):
    """A horizontal level change at 1 y per `rise_every` x cells.

    Support material formula (`alternate=True`):

        support[i] = TRANSPARENT  if dust i is the LOWER end of a step
                     SOLID       otherwise

    which on an ASCENDING half slope is glass at odd i and solid at even i --
    the user's "every second block is transparent".  The transparent cells let
    THIS line's steps survive under the next line's support; the solid cells
    sever the diagonal that would otherwise reach from this line's dust up to
    the next line's dust one x back, which is exactly how the two levels stay
    separate.  `alternate=False` makes every support transparent: the negative
    control, in which two lines 2 y apart MERGE (probe_vertical_forms H3/H4).
    """
    ys = slope_levels(ncells, rise_every, up)
    cs = [(x0 + i, y0 + ys[i], z) for i in range(ncells)]
    lo = lower_end_of_step(ys)
    for i, (x, y, zz) in enumerate(cs):
        sup = transparent if (lo[i] or not alternate) else STONE
        b.force(x, y - 1, zz, sup)
        b.force(x, y, zz, DUST)
    return cs
