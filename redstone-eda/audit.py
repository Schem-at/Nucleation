"""Structural audit: every block that needs something to hold it up must have it.

The simulator is happy to tick a floating wire, so this is checked statically --
otherwise the schematic only works until someone actually pastes it.
"""
import materials as _mt

NEEDS_FLOOR = ("redstone_wire", "repeater", "comparator",
               "lever[face=floor", "redstone_torch")   # standing torch, not wall
SOLID_HINTS = ("minecraft:stone", "redstone_lamp", "concrete")


def is_solid(block):
    return block is not None and (any(h in block for h in SOLID_HINTS)
                                  or _mt.conductor(block))


def is_sturdy(block):
    """May a floor-mounted component legally sit on this block?

    canSurviveOn (probed): full cubes INCLUDING glass/stained glass, plus
    top-half slabs.  Bottom slabs and air are vanilla-illegal even though
    mc-tick happily ticks the floating wire -- which is exactly why this
    check is static."""
    return is_solid(block) or (block is not None and _mt.sturdy(block))


def audit(cells):
    problems = {"floating": [], "unattached_wall_torch": [], "floating_solid": []}
    for (x, y, z), block in sorted(cells.items()):
        if "redstone_wall_torch" in block:
            face = block.split("facing=")[1].split(",")[0].rstrip("]")
            back = {"north": (0, 0, 1), "south": (0, 0, -1),
                    "east": (-1, 0, 0), "west": (1, 0, 0)}[face]
            anchor = (x + back[0], y + back[1], z + back[2])
            if not is_solid(cells.get(anchor)):
                problems["unattached_wall_torch"].append(((x, y, z), face, anchor))
            continue
        if any(k in block for k in NEEDS_FLOOR):
            if not is_sturdy(cells.get((x, y - 1, z))):
                problems["floating"].append(((x, y, z), block.split("[")[0],
                                             cells.get((x, y - 1, z))))
    return problems


if __name__ == "__main__":
    import build_adder as ad
    b, _, _ = ad.build(4)
    p = audit(b.cells)
    print("cells: %d" % len(b.cells))
    for kind, items in p.items():
        print("%s: %d" % (kind, len(items)))
        for it in items[:8]:
            print("   ", it)
