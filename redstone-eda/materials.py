"""Material-aware building patterns, backed by the probed table
(probe_materials.py; PROBED section of notes-material-model.md).

Material classes and their probed properties in mc-tick:

    class        dust sits  weak power  cuts diagonal  vertical step
    solid        yes        yes         YES            both directions
    transparent  yes        no          no             UP ONLY (diode)
    slab_top     yes        no          no             UP ONLY (diode)
    slab_bottom  vanilla-NO no          no             --
    air          vanilla-NO no          no             --

The one non-obvious consequence (the diode): a 1-y climb conducts DOWNHILL
only if the UPPER dust sits on a conductor.  So every pattern here places
climb-uppers on SOLID and uses transparent supports only under climb-lowers
and flats -- which is exactly the user's alternation: "dust on TRANSPARENT
at (x,y); its next step (+x,+y) on SOLID".

Patterns (each verified in-sim by its own gate):

  half_slope_2line  -- two independent lines climbing 1 y per 2 x in
                       ADJACENT z-rows, interleaved at 1-y offset.  Glass
                       supports under climb-lowers let each line make its
                       own diagonals; solid caps above the lower line at
                       the 1-y columns block the other level's dust.
  crossing_parity   -- 90-degree crossing, buses offset 1 in y; the
                       crossing bus runs through a block-sandwich repeater
                       station whose ENTRY block sits directly over the
                       through bus's dust: the station block doubles as the
                       isolation (it cuts the through dust's up-diagonals
                       and blocks the trunk's down-read).
  crossing_dipunder -- 90-degree crossing at ONE bus level: the crossing
                       bus steps down 1, runs under the through bus's
                       station ENTRY block (which needs no support), and
                       steps back up.  Dip supports are glass; both steps
                       keep their uppers on solid, so the crossing conducts
                       both ways.
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
    if "concrete" in block or block == STONE or "lamp" in block:
        return "solid"
    return "other"


def sturdy(block):
    """May dust/repeaters legally sit on this block? (canSurviveOn)"""
    return classify(block) in ("solid", "transparent", "slab_top")


def conductor(block):
    """Does this block cut diagonals / carry weak power? (isRedstoneConductor)"""
    return classify(block) == "solid"


# -- pattern: two-line half slope at 1-y pitch ------------------------------

def half_slope_2line(b, x0, y0, z0, n=9, transparent=GLASS, cap=None):
    """Two independent lines climbing +1 y per 2 x along +X, in adjacent
    z-rows (z0 and z0+1), vertically interleaved: at odd columns the upper
    line's dust is exactly 1 y above the lower line's dust, and the upper
    line's TRANSPARENT support sits level with the lower dust.

    Isolation, per the probed table:
      * lower line cannot pull the upper: its up-read side cell is the
        upper's transparent support (non-conductor blocks the read);
      * upper line cannot pull the lower: a SOLID cap above the lower dust
        at every 1-y column blocks the down-read;
      * both lines' climb-uppers sit on SOLID, so the slope conducts in
        both directions (no transparent diode in the path).

    y0 must be >= 1 (the lower line's first support sits at y0-1).
    Returns {"low": [p0, p1], "high": [p0, p1], "cells": [...]}.
    """
    assert y0 >= 1, "half_slope_2line: y0 must be >= 1"
    lo_first = hi_first = lo_last = hi_last = None
    for x in range(n):
        h0 = y0 + (x + 1) // 2          # lower line, climbs even->odd
        h1 = y0 + 2 + x // 2            # upper line, climbs odd->even
        # lower line (z0): transparent under climb-lowers (even x),
        # solid under climb-uppers (odd x)
        b.put(x0 + x, h0 - 1, z0,
              transparent if x % 2 == 0 else rs.PALETTE["lane"])
        b.put(x0 + x, h0, z0, DUST)
        # upper line (z0+1): solid under climb-uppers (even x),
        # transparent under climb-lowers (odd x)
        b.put(x0 + x, h1 - 1, z0 + 1,
              rs.PALETTE["lane"] if x % 2 == 0 else transparent)
        b.put(x0 + x, h1, z0 + 1, DUST)
        if x % 2 == 1:
            # 1-y column: cap above the lower dust blocks the upper line's
            # down-read (mix cut); harmless to the lower line (it is flat
            # here -- its climbs are governed at even columns)
            b.put(x0 + x, h0 + 1, z0, cap or rs.PALETTE["lid"])
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
    DIRECTLY OVER A's dust: the block conducts B's weak drive to the
    repeater while cutting every read between the two nets.
    B pays one repeater (1 rt); A passes untouched.
    Returns ports {"a_in","a_out","b_in","b_out"}."""
    for x in range(x0 - 3, x0 + 4):      # A: plain dust run
        b.stone(x, y0 - 1, z0, "rail_floor")
        b.put(x, y0, z0, DUST)
    for z in range(z0 - 3, z0):          # B trunk (straight, dead-ends into
        b.stone(x0, y0, z, "route")      # the entry block)
        b.put(x0, y0 + 1, z, DUST)
    b.put(x0, y0 + 1, z0, rs.PALETTE["route"])          # entry block, over A
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
    along +Z steps down 1 (upper of the step on SOLID -> bidirectional),
    runs at y0-1 under the ENTRY block (which needs no support -- that is
    what frees the cell), and steps back up.  The dip's supports are
    transparent; the entry block above the dip blocks both up-reads.
    D pays one repeater (1 rt); C pays only the dip.
    y0 must be >= 2 (dip supports sit at y0-2).
    Returns ports {"c_in","c_out","d_in","d_out"}."""
    assert y0 >= 2, "crossing_dipunder: y0 must be >= 2"
    for x in range(x0 - 3, x0):          # D trunk
        b.stone(x, y0 - 1, z0, "rail_floor")
        b.put(x, y0, z0, DUST)
    b.put(x0, y0, z0, rs.PALETTE["route"])              # entry block, no support
    b.stone(x0 + 1, y0 - 1, z0, "route")                # repeater support
    b.put(x0 + 1, y0, z0, rs.repeater("west"))          # flows +X
    b.put(x0 + 2, y0, z0, rs.PALETTE["route"])          # exit block
    for x in (x0 + 3, x0 + 4):
        b.stone(x, y0 - 1, z0, "rail_floor")
        b.put(x, y0, z0, DUST)
    for z in (z0 - 3, z0 - 2):           # C approach, bus level
        b.stone(x0, y0 - 1, z, "route")
        b.put(x0, y0, z, DUST)
    for z in (z0 - 1, z0, z0 + 1):       # the dip: dust at y0-1 on glass,
        b.put(x0, y0 - 2, z, transparent)               # under the entry block
        b.put(x0, y0 - 1, z, DUST)
    for z in (z0 + 2, z0 + 3):           # C exit, back at bus level
        b.stone(x0, y0 - 1, z, "route")
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
    # user's claim: all-glass mixes the levels.  Driving only the lower
    # line must light the upper line here (down-read no longer blocked).
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
