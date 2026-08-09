"""Material model: PRIMITIVE PREDICATES first, everything else COMPUTED.

ARCHITECTURE (the expandability contract):

  1. PRIMITIVE PREDICATES -- one boolean per probed physical property
     (PROBED table, notes-material-model.md; probe_materials.py):

         sturdy(block)         may dust / a repeater legally sit on it?
         conducts(block)       is_conductor: carries weak power into a
                               device back (dust -> block -> repeater)
         cuts_diagonal(block)  == conducts(block): the CUT LAW runs on
                               conductivity, not solidity

  2. LAWS -- physics derived from the predicates, each one a function:

         step_conducts(upper_support, downhill)
                               the transparent-diode law: a 1-y step passes
                               UP always, DOWN only if the UPPER dust's
                               support conducts
         cap_is_harmful(cap, dust_uses_diagonal_here)
                               a cap above dust matters ONLY if that dust is
                               the lower end of an in-use diagonal

  3. CELL CHOOSERS -- pick_support() / separator() turn a cell's
     CONSTRAINTS (must sever a diagonal? must let one survive? diode?) into
     a material and assert the choice against the predicates.  Every placed
     cell is justified by a computation, or refused (over-constrained cell
     == the geometry itself is wrong).

  4. PATTERNS -- geometry generators (slope, pass-under, dip) that COMPUTE
     each support/cap through the choosers from local geometric facts.
     They never hardcode a material for a constrained cell.

  Future rules slot in as new predicates + laws; patterns pick them up
  through the choosers.

WHEN TRANSPARENCY: a transparent block appears ONLY where a diagonal must
survive -- i.e. the cell sits directly above the lower dust of an in-use
1-y step (slope transitions of interleaved lines).  Everywhere else --
straight runs, separators, repeater floors, caps -- SOLID is correct:
straight dust has no diagonals in use (cap law), weak power never lights
dust, and a conductor also severs any stray cross-net diagonal.
"""
import rs
from rs import DUST, STONE

GLASS = "minecraft:glass"
SGLASS = "minecraft:white_stained_glass"
SLAB_TOP = "minecraft:smooth_stone_slab[type=top,waterlogged=false]"
SLAB_BOT = "minecraft:smooth_stone_slab[type=bottom,waterlogged=false]"

TRANSPARENT = (GLASS, SGLASS)

# name -> (dust_sits, conducts_weak, cuts_diagonal, step_down_off_it)
MATERIAL_TABLE = {
    "solid":       (True,  True,  True,  True),
    "transparent": (True,  False, False, False),
    "slab_top":    (True,  False, False, False),
    "slab_bottom": (False, False, False, False),
    "air":         (False, False, False, False),
}


def classify(block):
    """Material class of a block-state string (None = air)."""
    if block is None or block.startswith("minecraft:air"):
        return "air"
    if any(block.startswith(t) for t in TRANSPARENT) or "stained_glass" in block:
        return "transparent"
    if "_slab" in block:
        if "type=double" in block:
            return "solid"
        return "slab_top" if "type=top" in block else "slab_bottom"
    if ("concrete" in block or block == STONE or "lamp" in block
            or "_wool" in block or "planks" in block or "terracotta" in block):
        return "solid"
    return "other"


# -- 1. primitive predicates (each cites a probed table column) -------------

def sturdy(block):
    """May dust/repeaters legally sit on this block? (canSurviveOn; probed
    'dust sits' -- static legality only, the sim never pops dust)."""
    return classify(block) in ("solid", "transparent", "slab_top")


def conducts(block):
    """is_redstone_conductor: carries weak power into a device back
    (probed 'conducts weak')."""
    return classify(block) == "solid"


#   THREE RULES, ONE PREDICATE, THREE DIFFERENT CELLS.  All three read
#   is_redstone_conductor, which is why they used to be one function -- but
#   they apply at different cells and have different consequences, so a
#   planner that conflates them cannot express the crossing tricks.
#   Probed as an explicit 2x2x2 matrix in probe_transport.py (group C, 8/8).

def cuts_step(block):
    """CUT RULE.  Cell: directly ABOVE THE LOWER dust of a 1-y step.
    A conductor there severs the step in BOTH directions."""
    return conducts(block)


def gates_downhill(block):
    """DIODE RULE.  Cell: the UPPER dust's SUPPORT.  A conductor there lets
    the step conduct downhill as well as up; a non-conductor (glass, top
    slab) makes the step a one-way UPHILL diode.  Note this is the *other*
    cell from cuts_step -- a glass DIODE cell does not cut anything, and a
    solid CUT cell blocks even when the diode cell is solid."""
    return conducts(block)


def carries_weak(block):
    """WEAK-CARRY RULE.  Cell: the block a dust POINTS INTO.  A conductor
    there becomes a weakly powered block, which drives device backs but
    lights no dust at all (probe_transport W1/W2)."""
    return conducts(block)


cuts_diagonal = cuts_step     # backward-compat alias (the old conflated name)
conductor = conducts          # backward-compat alias


# -- 2. laws ----------------------------------------------------------------

def step_conducts(upper_support, downhill=True):
    """Transparent-diode law (probed): a 1-y step conducts UP always; DOWN
    only if the UPPER dust sits on a conductor (the lower dust's read of the
    wire above is gated on that side block conducting)."""
    return (not downhill) or conducts(upper_support)


def cap_is_harmful(cap, dust_uses_diagonal_here):
    """A cap above dust is harmful ONLY if that dust is the lower end of an
    in-use diagonal (cut law).  Flat/straight dust tolerates any cap: weak
    power never lights dust, blocks never chain power to blocks."""
    return dust_uses_diagonal_here and cuts_diagonal(cap)


def dust_drives_block(entry_on_pointing_axis):
    """POINTING LAW (probe_pivot.py A/B/C): dust weak-powers only the
    blocks it POINTS into -- the axes of its connections (dust, levers,
    repeaters count; a single connection extends to the opposite side; a
    true dot powers all sides).  A run that turns a corner does NOT fire a
    station entry block around the corner: every entry block must be
    preceded by >= 1 dust cell on the station's own axis.  Strong power is
    exempt (probe D): a station EXIT block lights dust on ANY side, so a
    corner directly after an exit block is free."""
    return entry_on_pointing_axis


def step_reads(cut_block, upper_support, downhill):
    """The full 1-y step law, both cells at once (probe_transport group C).

        uphill   conducts iff the CUT cell is clear
        downhill conducts iff the CUT cell is clear AND the DIODE cell conducts

    `step_conducts()` above is this with the cut cell assumed clear; it stays
    for the existing patterns."""
    if cuts_step(cut_block):
        return False
    return (not downhill) or gates_downhill(upper_support)


# -- 2b. TRANSPORT MECHANISMS ----------------------------------------------
#
# One row per way a signal MOVES.  A row declares, in a canonical frame whose
# forward is +X:
#
#   support   what must be under it            (static legality)
#   attach    what it hangs off, if anything   (torches)
#   emits     (kind, offset) pairs it energises
#   reads     offsets it takes input from
#   reader    which emission kinds it can pick up
#
# Everything else -- occupancy legality, what interferes with what -- is
# COMPUTED from those five fields, so a new trick is a new row, not a new
# special case.  See TRANSPORT_MODEL.md for the same table in prose plus the
# probe that fixed each field.

STRONG, WEAK, WIRE, SOURCE = "strong", "weak", "wire", "source"

#   Reader classes: which emission kinds actually reach this consumer.
#   The single most useful line in this file: DUST DOES NOT READ WEAK.
READER_KINDS = {
    "dust":   (STRONG, SOURCE, WIRE),                 # probe_transport W1
    "device": (STRONG, WEAK, SOURCE, WIRE),           # probe_transport W2
    "none":   (),
}

NB6 = ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1))
NB4 = ((1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1))
DIRV = {"east": (1, 0, 0), "west": (-1, 0, 0),
        "south": (0, 0, 1), "north": (0, 0, -1)}


def _xform(off, fwd):
    """Canonical (forward, up, right) offset -> world offset for `fwd`."""
    f = fwd
    r = (-f[2], 0, f[0])
    a, b, c = off
    return tuple(a * f[i] + b * (0, 1, 0)[i] + c * r[i] for i in range(3))


def _add(p, d):
    return (p[0] + d[0], p[1] + d[1], p[2] + d[2])


class Mechanism(object):
    def __init__(self, name, block, support=None, attach=None, emits=(),
                 reads=(), reader="none", decay=0, refresh=False, exact=False,
                 delay_gt=0, inverts=False, probe="", note=""):
        self.name, self.block = name, block
        self.support = support          # "sturdy" | None  (cell (0,-1,0))
        self.attach = attach            # offset that must be sturdy-faced
        self.emits = tuple(emits)
        self.reads = tuple(reads)
        self.reader = reader
        self.decay = decay              # signal strength lost per cell
        self.refresh = refresh          # re-emits 15 regardless of input
        self.exact = exact              # preserves the analog value
        self.delay_gt = delay_gt        # game ticks
        self.inverts = inverts
        self.probe = probe              # the probe that fixed this row
        self.note = note

    def __repr__(self):
        return "<mech %s>" % self.name

    # ---- what the row REQUIRES of its neighbourhood
    def requires(self, cell, fwd=(1, 0, 0)):
        """{role: (cell, predicate_name)} that must hold for this placement."""
        need = {}
        if self.support == "sturdy":
            need["support"] = (_add(cell, (0, -1, 0)), "sturdy")
        if self.attach is not None:
            need["attach"] = (_add(cell, _xform(self.attach, fwd)), "sturdy")
        return need

    # ---- what the row EMITS
    def emission(self, cell, fwd=(1, 0, 0)):
        """{cell: kind} this placement energises.  ALL_SIDES expands to the 6
        neighbours (probe_transport S2); POINTING is data-dependent and is
        supplied by the caller via `pointing=`."""
        out = {}
        for kind, off in self.emits:
            if off == "ALL_SIDES":
                for d in NB6:
                    out[_add(cell, d)] = kind
            elif off == "SELF":
                out[cell] = kind
            elif off == "POINTING":
                pass                    # see dust_emission()
            else:
                out[_add(cell, _xform(off, fwd))] = kind
        return out

    def inputs(self, cell, fwd=(1, 0, 0)):
        return [_add(cell, _xform(o, fwd)) for o in self.reads]

    def sensitive(self, cell, fwd=(1, 0, 0)):
        """The cells where a FOREIGN emission energises this placement.

        Dust is energised by anything that emits INTO ITS OWN CELL (it scans
        its 6 neighbours, which is the same relation seen from the source).
        A device is energised only at its declared input cells -- that
        asymmetry is why a repeater may stand on a hard-powered block and why
        a repeater beside a live rail is inert (probe_transport S4,
        probe_station S)."""
        if self.reader == "none":
            return []
        if self.reader == "dust":
            return [cell]
        return self.inputs(cell, fwd)


def dust_emission(cell, pointing):
    """Dust's emission depends on its CONNECTION AXES (the pointing law,
    probe_pivot A/B/C): WEAK into the block below and into every block it
    points at; the cell itself carries WIRE.  It never powers the block
    ABOVE it (probe_transport W3) -- that is what lets a lid over a live run
    carry a foreign line."""
    out = {cell: WIRE, _add(cell, (0, -1, 0)): WEAK}
    for d in pointing:
        out[_add(cell, d)] = WEAK
    return out


MECH = {}


def _m(*a, **kw):
    m = Mechanism(*a, **kw)
    MECH[m.name] = m
    return m


#   ------------------------------------------------------------ the carriers
_m("dust", DUST, support="sturdy", reader="dust", decay=1,
   emits=((WIRE, "SELF"), (WEAK, (0, -1, 0)), (WEAK, "POINTING")),
   reads=list(NB6),
   probe="probe_materials.py SIT; probe_transport.py W1/W3; probe_pivot A/B/C",
   note="the only decaying carrier: 15 cells max, no delay, no refresh")

_m("dust_step", DUST, support="sturdy", reader="dust", decay=1,
   emits=((WIRE, "SELF"), (WEAK, (0, -1, 0)), (WEAK, "POINTING")),
   reads=list(NB6),
   probe="probe_transport.py C (8/8 matrix); probe_pivot E",
   note="a 1-y dust step: legality is step_reads(cut cell, diode cell, dir)")

#   NOTE ON "SELF": a powered block is read two ways.  Dust sitting on one of
#   its FACES reads it as a neighbour (offset ALL_SIDES); a device whose input
#   cell IS that block reads it in place (offset SELF).  Same physical fact,
#   two consumer geometries -- emitting both is what lets `interferes()` use
#   one uniform rule for dust and for device backs.
_m("strong_block", None, reader="none",
   emits=((STRONG, "ALL_SIDES"), (STRONG, "SELF")),
   probe="probe_transport.py S1/S2/S3",
   note="a conductor driven by a repeater/comparator/torch/lever output face."
        " Lights dust on ALL SIX faces at 15 and NEVER chains to another"
        " block -- so two of them may be neighbours.  THE crossing primitive."
        " Needs no support of its own and carries dust on top of it.")

_m("weak_block", None, reader="none",
   emits=((WEAK, "ALL_SIDES"), (WEAK, "SELF")),
   probe="probe_transport.py W1/W2",
   note="a conductor a dust POINTS INTO. Drives device backs; lights no dust."
        " Two weak blocks may touch, and a weak block may carry foreign dust.")

_m("repeater", None, support="sturdy", reader="device", refresh=True,
   delay_gt=2, emits=((STRONG, (1, 0, 0)),), reads=((-1, 0, 0),),
   probe="probe_station.py A/I/B/Z/F/S; probe_transport.py S4",
   note="reads its BACK only: immune to its floor, its sides and the block"
        " above (no quasi-connectivity), so it may stand ON a hard-powered"
        " block.  delay_gt = 2*delay property. Locks from the side.")

_m("comparator", None, support="sturdy", reader="device", exact=True,
   delay_gt=2, emits=((STRONG, (1, 0, 0)),),
   reads=((-1, 0, 0), (0, 0, 1), (0, 0, -1)),
   probe="probe_station.py C1..C4",
   note="the only ANALOG-exact relay; also reads both sides (max) and reads"
        " container fill through a conductor")

_m("torch_floor", rs.TORCH,
   attach=(0, -1, 0), reader="device", refresh=True, delay_gt=2, inverts=True,
   emits=(((STRONG, (0, 1, 0)), (SOURCE, "SELF"))
          + tuple((SOURCE, d) for d in NB4)),
   reads=((0, -1, 0),),
   probe="probe_transport.py T1/T2/T3",
   note="INVERTS its attachment block; strongly powers the block ABOVE it and"
        " gives 15 to the 4 horizontal dusts; never powers its own attachment"
        " block, which is what makes a torch TOWER the vertical carrier"
        " (2 gt and one inversion per level)")

_m("redstone_block", "minecraft:redstone_block", reader="none",
   emits=((SOURCE, "SELF"),) + tuple((SOURCE, d) for d in NB6),
   probe="probe_transport.py B1/B2",
   note="an unconditional 15 SOURCE, not a powered block: dust beside it"
        " reads 15, but the solid block beside it stays dead")

_m("lever", rs.LEVER_OFF, attach=(0, -1, 0), reader="none",
   emits=(((STRONG, (0, -1, 0)), (SOURCE, "SELF"))
          + tuple((SOURCE, d) for d in NB6)),
   probe="probe_pivot.py A; every rig in this directory",
   note="strong-powers its attachment block; the test-rig source")

#   -------------------------------------------------- the passive space-savers
_m("solid_support", STONE, reader="none",
   probe="probe_materials.py; bus8_probe.py P1",
   note="sturdy AND conducting: occupy a DIODE cell to make a step two-way,"
        " occupy a CUT cell to sever a foreign step, or lay it over a live"
        " straight run as a separator/lid that carries another line")

_m("transparent_support", GLASS, reader="none",
   probe="probe_materials.py CUT_UP/CUT_DOWN/DIODE_*; probe_transport.py C",
   note="sturdy and NON-conducting: in a CUT cell it lets a step survive; in"
        " a DIODE cell it makes the step one-way UPHILL; it never carries"
        " weak power")


# -------------------------------------------------- mechanism-level questions

def can_occupy(mech, cell, grid, fwd=(1, 0, 0)):
    """(ok, reason).  May mechanism `mech` sit at `cell` in `grid`?"""
    m = MECH[mech] if isinstance(mech, str) else mech
    if grid.get(cell) is not None:
        return False, "cell %r is occupied by %s" % (cell, grid[cell])
    for role, (c, pred) in m.requires(cell, fwd).items():
        if pred == "sturdy" and not sturdy(grid.get(c)):
            return False, "%s cell %r is %r, not sturdy" % (role, c, grid.get(c))
    return True, "ok"


def wire_connects(grid, p, q):
    """Do two dust cells belong to the same net?  Exactly vanilla's
    calculateTargetStrength neighbour scan, which is where the CUT cell and
    the DIODE cell come from (see probe_transport.py group C)."""
    if p == q:
        return True
    dx, dy, dz = q[0] - p[0], q[1] - p[1], q[2] - p[2]
    if abs(dx) + abs(dz) != 1 or abs(dy) > 1:
        return False                       # dust has no planar diagonal (P3)
    side = (p[0] + dx, p[1], p[2] + dz)
    if dy == 0:
        return True
    if dy == 1:                            # p is LOWER, reads up
        return (gates_downhill(grid.get(side))
                and not cuts_step(grid.get((p[0], p[1] + 1, p[2]))))
    return not cuts_step(grid.get(side))    # p is UPPER, reads down


def interferes(a, b, grid=None):
    """Do two PLACEMENTS of different nets interact?  (ok, reason).

    `a`/`b` are (mech_name, cell, fwd, net) tuples; `pointing` for dust is
    taken from the grid when one is supplied.  Interference is exactly
    'one side's emission lands where the other side reads, in a kind the
    other side can actually read'.  Everything the crossing tiles exploit
    falls out of that: STRONG never lands on a block, WEAK never lands on a
    dust, and a repeater only reads one cell."""
    ma, ca, fa, na = a
    mb, cb, fb, nb = b
    if na == nb:
        return False, "same net"
    A, B = MECH[ma], MECH[mb]
    for (m_src, c_src, f_src), (m_dst, c_dst, f_dst) in (
            ((ma, ca, fa), (mb, cb, fb)), ((mb, cb, fb), (ma, ca, fa))):
        src, dst = MECH[m_src], MECH[m_dst]
        emit = src.emission(c_src, f_src)
        kinds = READER_KINDS[dst.reader]
        for c in dst.sensitive(c_dst, f_dst):
            k = emit.get(c)
            if k is not None and k in kinds and k != WIRE:
                return True, ("%s at %r emits %s into %s's %r"
                              % (m_src, c_src, k, m_dst, c))
    if A.reader == "dust" and B.reader == "dust" and grid is not None:
        if wire_connects(grid, ca, cb):
            return True, "the two dust cells are wire-connected at %r/%r" % (ca, cb)
    return False, "isolated"


def mechanism_table():
    """The rows, in the order TRANSPORT_MODEL.md tabulates them."""
    return [MECH[k] for k in (
        "dust", "dust_step", "strong_block", "weak_block", "repeater",
        "comparator", "torch_floor", "redstone_block", "lever",
        "solid_support", "transparent_support")]


# -- 3. cell choosers -------------------------------------------------------

def pick_support(need_conductor=False, need_insulator=False,
                 solid=None, transparent=GLASS, why=""):
    """One support cell, computed from its constraints:

       need_conductor -- the dust on this cell is a step-UPPER whose slope
           must conduct downhill (diode law), OR this cell caps the lower
           dust of a cross-net diagonal that must be SEVERED;
       need_insulator -- this cell sits directly above the lower dust of an
           IN-USE diagonal and must not cut it.

    Both at once means the geometry is impossible -> refuse.  Unconstrained
    cells default to SOLID ('transparent only where a diagonal must
    survive')."""
    if need_conductor and need_insulator:
        raise ValueError("over-constrained support cell (re-plan geometry): "
                         + why)
    block = transparent if need_insulator else (solid or rs.PALETTE["lane"])
    assert sturdy(block), (block, why)
    if need_conductor:
        assert conducts(block), (block, why)
    if need_insulator:
        assert not cuts_diagonal(block), (block, why)
    return block


def separator(solid=None):
    """Between STACKED STRAIGHT runs (2y pitch: dust/sep/dust) the layer is
    SOLID, computed: straight dust has no diagonal in use, so
    cap_is_harmful()==False; weak power from the dust above dead-ends in the
    block; and the conductor severs any stray diagonal.  NO transparency on
    straight runs."""
    block = solid or rs.PALETTE["lane"]
    assert conducts(block) and sturdy(block), block
    assert not cap_is_harmful(block, dust_uses_diagonal_here=False)
    return block


# -- 4. patterns (every cell computed through the choosers) -----------------

def half_slope_2line(b, x0, y0, z0, n=9, transparent=GLASS, cap=None):
    """Two independent lines climbing +1 y per 2 x along +X, in adjacent
    z-rows (z0 and z0+1), vertically interleaved: at odd columns the upper
    line's dust is exactly 1 y above the lower line's dust.

    Cell derivation (nothing hardcoded):
      * step-UPPER dust cells sit on pick_support(need_conductor=True):
        the diode law, so the slope conducts BOTH ways;
      * at every 1-y column the two lines' dusts form a CROSS-LINE diagonal
        (lower: z0 dust, upper: z0+1 dust); the cell above its lower dust
        must sever it -> solid cap.  Legal by the cap law: the z0 dust is
        flat at odd columns (its own climbs happen at even columns);
      * no support cell here sits above the lower dust of an in-use
        diagonal, so no cell NEEDS transparency -> all supports solid.
        (`cap`/`transparent` args exist for negative controls.)

    y0 must be >= 1.  Returns {"low": [p0, p1], "high": [p0, p1]}.
    """
    assert y0 >= 1, "half_slope_2line: y0 must be >= 1"
    lo_first = hi_first = lo_last = hi_last = None
    for x in range(n):
        h0 = y0 + (x + 1) // 2          # lower line, climbs even->odd
        h1 = y0 + 2 + x // 2            # upper line, climbs odd->even
        lo_upper = x % 2 == 1           # z0 dust is a step-UPPER at odd x
        hi_upper = x % 2 == 0           # z0+1 dust is a step-UPPER at even x
        b.put(x0 + x, h0 - 1, z0,
              pick_support(need_conductor=lo_upper, transparent=transparent,
                           why="z0 col %d" % x))
        b.put(x0 + x, h0, z0, DUST)
        b.put(x0 + x, h1 - 1, z0 + 1,
              pick_support(need_conductor=hi_upper, transparent=transparent,
                           why="z0+1 col %d" % x))
        b.put(x0 + x, h1, z0 + 1, DUST)
        if x % 2 == 1:
            c = cap or rs.PALETTE["lid"]
            if cap is None:             # negative controls may violate this
                assert cuts_diagonal(c), "cap must sever the cross-line pair"
                assert not cap_is_harmful(c, dust_uses_diagonal_here=False)
            b.put(x0 + x, h0 + 1, z0, c)
        if x == 0:
            lo_first, hi_first = (x0, h0, z0), (x0, h1, z0 + 1)
        if x == n - 1:
            lo_last = (x0 + x, h0, z0)
            hi_last = (x0 + x, h1, z0 + 1)
    return {"low": [lo_first, hi_first], "high": [lo_last, hi_last]}


# -- pattern: y-parity 90-degree crossing (station block = isolation) -------

def crossing_parity(b, x0, y0, z0):
    """Through bus A along +X at y0; crossing bus B along +Z at y0+1.
    B runs through a block-sandwich repeater station whose entry block sits
    DIRECTLY OVER A's dust.  Derivation: the entry block must conduct (it
    carries B's weak drive to the repeater) and it caps A's dust -- legal
    because A is STRAIGHT here (cap_is_harmful()==False); the same
    conductivity severs every read between the nets.
    B pays one repeater (1 rt); A passes untouched.
    Returns ports {"a_in","a_out","b_in","b_out"}."""
    for x in range(x0 - 3, x0 + 4):      # A: plain dust run
        b.stone(x, y0 - 1, z0, "rail_floor")
        b.put(x, y0, z0, DUST)
    for z in range(z0 - 3, z0):          # B trunk (straight, dead-ends into
        b.stone(x0, y0, z, "route")      # the entry block)
        b.put(x0, y0 + 1, z, DUST)
    entry = rs.PALETTE["route"]
    assert conducts(entry)                              # carries B's weak drive
    assert not cap_is_harmful(entry, dust_uses_diagonal_here=False)  # A straight
    b.put(x0, y0 + 1, z0, entry)                        # entry block, over A
    b.stone(x0, y0, z0 + 1, "route")                    # repeater support
    b.put(x0, y0 + 1, z0 + 1, rs.repeater("north"))     # flows +Z
    b.put(x0, y0 + 1, z0 + 2, rs.PALETTE["route"])      # exit block (strong)
    for z in (z0 + 3, z0 + 4):
        b.stone(x0, y0, z, "route")
        b.put(x0, y0 + 1, z, DUST)
    return {"a_in": (x0 - 3, y0, z0), "a_out": (x0 + 3, y0, z0),
            "b_in": (x0, y0 + 1, z0 - 3), "b_out": (x0, y0 + 1, z0 + 4)}


# -- pattern: dip-under 90-degree crossing (single bus level) ---------------

def crossing_dipunder(b, x0, y0, z0, transparent=GLASS):
    """Through bus D along +X at y0 runs through a station; crossing bus C
    along +Z steps down 1, runs at y0-1 under the ENTRY block (which needs
    no support -- that is what frees the cell), and steps back up.

    Cell derivation: both of C's step-UPPER dusts sit on
    pick_support(need_conductor=True) (diode -> bidirectional); the dip
    supports are UNCONSTRAINED in this flat tile (no dust below them), so
    they compute to SOLID.  In a stacked context the same computation turns
    them transparent -- see bus8_cross.py, where a lower bit's step diagonal
    lives directly beneath.  The entry block above the dip severs both
    up-reads (cut law) and is a legal cap (C's dip dust is straight there).
    D pays one repeater (1 rt); C pays only the dip.  y0 must be >= 2.
    Returns ports {"c_in","c_out","d_in","d_out"}."""
    assert y0 >= 2, "crossing_dipunder: y0 must be >= 2"
    for x in range(x0 - 3, x0):          # D trunk
        b.stone(x, y0 - 1, z0, "rail_floor")
        b.put(x, y0, z0, DUST)
    entry = rs.PALETTE["route"]
    assert conducts(entry)                              # severs the up-reads
    assert not cap_is_harmful(entry, dust_uses_diagonal_here=False)  # dip straight at z0
    b.put(x0, y0, z0, entry)                            # entry block, no support
    b.stone(x0 + 1, y0 - 1, z0, "route")                # repeater support
    b.put(x0 + 1, y0, z0, rs.repeater("west"))          # flows +X
    b.put(x0 + 2, y0, z0, rs.PALETTE["route"])          # exit block
    for x in (x0 + 3, x0 + 4):
        b.stone(x, y0 - 1, z0, "rail_floor")
        b.put(x, y0, z0, DUST)
    for z in (z0 - 3, z0 - 2):           # C approach, bus level; z0-2 dust is
        b.put(x0, y0 - 1, z,             # the down-step's UPPER -> diode
              pick_support(need_conductor=(z == z0 - 2),
                           solid=rs.PALETTE["route"], why="C approach"))
        b.put(x0, y0, z, DUST)
    for z in (z0 - 1, z0, z0 + 1):       # the dip: unconstrained supports
        b.put(x0, y0 - 2, z,             # (nothing below) -> solid
              pick_support(transparent=transparent, why="dip floor"))
        b.put(x0, y0 - 1, z, DUST)
    for z in (z0 + 2, z0 + 3):           # C exit; z0+2 dust is the up-step's
        b.put(x0, y0 - 1, z,             # UPPER -> diode (reverse flow)
              pick_support(need_conductor=(z == z0 + 2),
                           solid=rs.PALETTE["route"], why="C exit"))
        b.put(x0, y0, z, DUST)
    return {"c_in": (x0, y0, z0 - 3), "c_out": (x0, y0, z0 + 3),
            "d_in": (x0 - 3, y0, z0), "d_out": (x0 + 4, y0, z0)}


PATTERNS = {
    "half_slope_2line": half_slope_2line,
    "crossing_parity": crossing_parity,
    "crossing_dipunder": crossing_dipunder,
}


# -- verification gate: the 2-line slope, both directions -------------------

def _lever_col(b, x, y, z, levers):
    for yy in range(0, y):
        b.stone(x, yy, z)
    b.force(x, y, z, rs.LEVER_OFF)
    levers.append((x, y, z))
    return len(levers) - 1


def _verify_slope():
    b = rs.Build("half_slope_2line")
    LV = []
    N, Y0 = 9, 1
    # instance 1 (z=0): driven at the LOW end -> uphill flow
    up = half_slope_2line(b, 0, Y0, 0, N)
    iu0 = _lever_col(b, -1, up["low"][0][1], 0, LV)
    iu1 = _lever_col(b, -1, up["low"][1][1], 1, LV)
    # instance 2 (z=8): driven at the HIGH end -> downhill flow
    dn = half_slope_2line(b, 0, Y0, 8, N)
    id0 = _lever_col(b, N, dn["high"][0][1], 8, LV)
    id1 = _lever_col(b, N, dn["high"][1][1], 9, LV)
    # instance 3 (z=16): NEGATIVE CONTROL -- caps made transparent.  The
    # cap law in reverse: without a conductor over the cross-line pair's
    # lower dust, driving only the lower line must light the upper line.
    ng = half_slope_2line(b, 0, Y0, 16, N, cap=GLASS)
    in0 = _lever_col(b, -1, ng["low"][0][1], 16, LV)
    sim = b.sim()
    lv = rs.Levers(sim, LV)
    probes = {"up_lo": up["high"][0], "up_hi": up["high"][1],
              "dn_lo": dn["low"][0], "dn_hi": dn["low"][1]}
    bad = 0
    for a in (0, 1):
        for c in (0, 1):
            bits = [0] * len(LV)
            bits[iu0] = bits[id0] = a
            bits[iu1] = bits[id1] = c
            bits[in0] = a
            lv.set(bits)
            got = {k: sim.power(*p) > 0 for k, p in probes.items()}
            want = {"up_lo": a == 1, "up_hi": c == 1,
                    "dn_lo": a == 1, "dn_hi": c == 1}
            ok = got == want
            print("%s combo lo=%d hi=%d -> uphill (%d,%d) downhill (%d,%d)"
                  % ("PASS" if ok else "FAIL", a, c,
                     got["up_lo"], got["up_hi"], got["dn_lo"], got["dn_hi"]))
            bad += 0 if ok else 1
    # negative control: lower line driven, transparent caps -> levels mix
    bits = [0] * len(LV)
    bits[in0] = 1
    lv.set(bits)
    mixed = sim.power(*ng["high"][1]) > 0
    print("%s negative control: transparent caps mix the levels"
          % ("PASS" if mixed else "FAIL"))
    bad += 0 if mixed else 1
    print("half_slope_2line: %d/9 checks (4 combos x both directions "
          "+ mix control)" % (9 - bad))
    return bad


if __name__ == "__main__":
    raise SystemExit(1 if _verify_slope() else 0)
