"""Static net checker: prove no two distinct signals share a redstone net.

Both of the real bugs so far were unintended adjacency -- a tap whose output block
touched its own input rail, and a lamp that was adjacent to a wire pointing the
wrong way.  Simulation only says "wrong answer somewhere"; this says which two
signals are touching, and where.

Redstone dust connects to dust at the 4 horizontal neighbours, and diagonally
one step up/down, with the up-diagonal blocked when the cell above the LOWER
dust is a CONDUCTOR (that is exactly what the rail lids exploit).

Material model (probe_materials.py, PROBED table in notes-material-model.md):
the cut rule runs on conductivity, not solidity.  Glass, stained glass and
both slab halves are non-conductors -- they never cut a diagonal -- while
still (glass / slab-top) supporting dust.  `is_solid` here has always meant
"conductor" (stone / lamp / concrete hints); the material classes make that
explicit and keep transparent supports from cutting.

The diode: a diagonal step whose UPPER dust sits on a non-conductor passes
signal UP only (`step_conducts_down`).  `neighbours` stays symmetric on
purpose -- for short checking, a one-way electrical touch is still a short.
"""
import materials as _mt

DUST_KEY = "redstone_wire"
SOLID_HINTS = ("minecraft:stone", "redstone_lamp", "concrete")


def material(cells, pos):
    """Material class at pos: air/solid/transparent/slab_top/slab_bottom/other."""
    return _mt.classify(cells.get(pos))


def is_solid(cells, pos):
    """Does the block at pos cut diagonals / conduct weak power?

    Conductor semantics (probed): true for the solid hints, false for air,
    glass, stained glass and slabs -- identical to the historical hint list
    for every block the suites place, now backed by the material table."""
    b = cells.get(pos)
    if b is None:
        return False
    if any(h in b for h in SOLID_HINTS):
        return True
    return _mt.conductor(b)


def step_conducts_down(cells, lower, upper):
    """May power flow DOWN a diagonal step (upper -> lower)?

    Only if the upper dust's support is a conductor (the probed transparent
    diode): the lower dust's up-read is gated on that side block."""
    return is_solid(cells, (upper[0], upper[1] - 1, upper[2]))


def neighbours(cells, pos):
    """Cells whose dust is electrically connected to the dust at `pos`."""
    x, y, z = pos
    out = []
    for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        side = (x + dx, y, z + dz)
        if DUST_KEY in cells.get(side, ""):
            out.append(side)
        # dust one step DOWN in that direction: blocked if the cell above the
        # lower dust is solid
        low = (x + dx, y - 1, z + dz)
        if DUST_KEY in cells.get(low, "") and not is_solid(cells, (x + dx, y, z + dz)):
            out.append(low)
        # dust one step UP: blocked if the cell above THIS dust is solid
        high = (x + dx, y + 1, z + dz)
        if DUST_KEY in cells.get(high, "") and not is_solid(cells, (x, y + 1, z)):
            out.append(high)
    return out


def check(cells, labels, aliases=()):
    """labels: pos -> signal label.  Returns list of (labelA, labelB, posA, posB).

    aliases: iterable of label pairs that are DELIBERATELY the same electrical
    net (a routed wire joining a producer lane to a consumer rail)."""
    lparent = {}

    def lfind(a):
        lparent.setdefault(a, a)
        while lparent[a] != a:
            lparent[a] = lparent[lparent[a]]
            a = lparent[a]
        return a

    for a, b in aliases:
        lparent[lfind(a)] = lfind(b)
    parent = {}

    def find(a):
        parent.setdefault(a, a)
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    def union(a, b):
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb

    dust = [p for p, b in cells.items() if DUST_KEY in b]
    for p in dust:
        find(p)
        for q in neighbours(cells, p):
            union(p, q)

    comps = {}
    for p in dust:
        comps.setdefault(find(p), []).append(p)

    shorts = []
    for members in comps.values():
        seen = {}
        for p in members:
            lab = labels.get(p)
            if lab is not None:
                seen.setdefault(lfind(lab), p)
        if len(seen) > 1:
            items = sorted(seen.items())
            (la, pa), (lb, pb) = items[0], items[1]
            shorts.append((la, lb, pa, pb))
    return shorts
