"""Minimal redstone helpers over nucleation's Schematic + mc-tick TickSimulation.

Only set_block_from_string is used to author geometry; state is read back out of
the simulation as vanilla block-state strings.
"""
import re
import nucleation as n

STONE = "minecraft:stone"
# Authoring bare "minecraft:redstone_wire" interns a property-less state that the
# engine never normalises unless the cell happens to catch a placement update --
# such cells sit inert and read back with no power= at all.  Always spell the
# default state out so it interns as a real wire.
DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]"
TORCH = "minecraft:redstone_torch[lit=true]"
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER_OFF = "minecraft:lever[face=floor,facing=north,powered=false]"

DIRS = ("north", "south", "east", "west")

# Every state the sim may need to intern. mc-tick binds behaviour to interned
# states at construction time; a state that only shows up later sits inert.
EXTRA_STATES = ";".join(
    ["minecraft:lever[face=floor,facing=%s,powered=%s]" % (d, p)
     for d in DIRS for p in ("true", "false")]
    + ["minecraft:redstone_torch[lit=true]", "minecraft:redstone_torch[lit=false]"]
    + ["minecraft:redstone_wall_torch[facing=%s,lit=%s]" % (d, p)
       for d in DIRS for p in ("true", "false")]
    + ["minecraft:redstone_lamp[lit=true]", "minecraft:redstone_lamp[lit=false]"]
    + ["minecraft:repeater[facing=%s,delay=%d,locked=%s,powered=%s]" % (d, dl, lk, pw)
       for d in DIRS for dl in (1, 2) for lk in ("true", "false")
       for pw in ("true", "false")]
    # Material-model blocks (probe_materials.py): transparent supports and
    # slabs.  These have no powered variants, but interning them here keeps
    # late placements (routers, templates) from creating inert states.
    + ["minecraft:glass", "minecraft:white_stained_glass",
       "minecraft:smooth_stone_slab[type=top,waterlogged=false]",
       "minecraft:smooth_stone_slab[type=bottom,waterlogged=false]"]
)


# Colour-code structure by role.  Concrete is a full solid block and conducts
# exactly like stone, so this is purely cosmetic -- but it makes a 200k-block
# build readable: you can see the rails, the lids and the wiring at a glance.
PALETTE = {
    "rail_floor": "minecraft:gray_concrete",
    "lid":        "minecraft:light_blue_concrete",
    "tap":        "minecraft:red_concrete",
    "coll_floor": "minecraft:lime_concrete",
    "gate":       "minecraft:orange_concrete",
    "lane":       "minecraft:yellow_concrete",
    "route":      "minecraft:magenta_concrete",
    "inv":        "minecraft:purple_concrete",
    "plain":      "minecraft:stone",
}


def wall_torch(face_dir, lit=True):
    """A wall torch whose attachment block is one step OPPOSITE face_dir.

    Verified: redstone_wall_torch[facing=south] at (x,y,z) hangs off (x,y,z-1).
    """
    return "minecraft:redstone_wall_torch[facing=%s,lit=%s]" % (face_dir, str(lit).lower())


def repeater(input_from, delay=1):
    """A repeater whose INPUT side is `input_from`; it outputs the opposite way.

    Verified: repeater[facing=west] conducts toward +X.
    """
    return "minecraft:repeater[facing=%s,delay=%d,locked=false,powered=false]" % (input_from, delay)


class Build:
    def __init__(self, name):
        self.s = n.Schematic.create(name)
        self.cells = {}

    def put(self, x, y, z, block):
        prev = self.cells.get((x, y, z))
        if prev is not None and prev != block:
            raise AssertionError(
                "cell collision at (%d,%d,%d): %s vs %s" % (x, y, z, prev, block))
        self.cells[(x, y, z)] = block
        self.s.set_block_from_string(x, y, z, block)

    def force(self, x, y, z, block):
        """Deliberately overwrite whatever is already in this cell."""
        self.cells[(x, y, z)] = block
        self.s.set_block_from_string(x, y, z, block)

    def stone(self, x, y, z, role="plain"):
        # Structural blocks legitimately coincide (a route's support IS the rail
        # floor beneath it).  First role to claim a cell keeps its colour; the
        # block is solid either way, so this is cosmetic only.
        if self.solid_at(x, y, z):
            return
        self.put(x, y, z, PALETTE[role])

    def solid_at(self, x, y, z):
        b = self.cells.get((x, y, z))
        return b is not None and ("concrete" in b or b == STONE or "lamp" in b)

    def dust(self, x, y, z, support=True):
        if support:
            self.stone(x, y - 1, z)
        self.put(x, y, z, DUST)

    def sim(self, mode=None, settle=400):
        """Build a simulation that is addressable in THIS build's coordinates.

        from_schematic normalises the build to its own tight bounds, so a build
        whose lowest block is y=1 comes up shifted down by one.  Sim wraps that.
        """
        mode = mode or n.TickSettleMode.Placement
        # The engine sizes its world from the schematic's ALLOCATED region, not
        # from the blocks actually in it, and that region is padded well beyond
        # the content -- enough to blow the 8M-cell cap on a big build.  A .schem
        # save normalises to tight, non-negative bounds, so round-trip first.
        import os
        tmp = os.path.join(os.environ.get("TMPDIR", "/tmp"), "_rs_tighten.schem")
        self.s.save_to_file(tmp)
        tight = n.Schematic.open(tmp)
        sim = n.TickSimulation.from_schematic(tight, mode, 0, 0, 0, EXTRA_STATES)
        sim.run_until_quiescent(settle)
        return Sim(sim, self.bounds()[0])

    def bounds(self):
        xs = [p[0] for p in self.cells]
        ys = [p[1] for p in self.cells]
        zs = [p[2] for p in self.cells]
        return (min(xs), min(ys), min(zs)), (max(xs), max(ys), max(zs))


class Sim:
    def __init__(self, sim, offset):
        self.sim = sim
        self.ox, self.oy, self.oz = offset

    def _p(self, x, y, z):
        return x - self.ox, y - self.oy, z - self.oz

    def block(self, x, y, z):
        return self.sim.get_block(*self._p(x, y, z))

    def power(self, x, y, z):
        m = _POWER.search(self.block(x, y, z))
        return int(m.group(1)) if m else -1

    def on(self, x, y, z):
        return self.power(x, y, z) > 0

    def lit(self, x, y, z):
        m = _LIT.search(self.block(x, y, z))
        return None if m is None else m.group(1) == "true"

    def powered(self, x, y, z):
        m = _PWD.search(self.block(x, y, z))
        return None if m is None else m.group(1) == "true"

    def use(self, x, y, z):
        self.sim.use_block(*self._p(x, y, z))

    def settle(self, budget=400):
        return self.sim.run_until_quiescent(budget)


_POWER = re.compile(r"power=(\d+)")
_LIT = re.compile(r"lit=(true|false)")
_PWD = re.compile(r"powered=(true|false)")


def power(sim, x, y, z):
    m = _POWER.search(sim.get_block(x, y, z))
    return int(m.group(1)) if m else -1


def lit(sim, x, y, z):
    m = _LIT.search(sim.get_block(x, y, z))
    return None if m is None else m.group(1) == "true"


def levered(sim, x, y, z):
    m = _PWD.search(sim.get_block(x, y, z))
    return None if m is None else m.group(1) == "true"


class Levers:
    """Toggle-to-target driver: levers only respond to use_block, never to a
    direct state write, so we track state and toggle toward the target."""

    def __init__(self, sim, positions):
        self.sim = sim
        self.positions = list(positions)
        self.state = [bool(sim.powered(*p)) for p in self.positions]

    def set(self, bits, settle=400):
        """Flip one lever at a time, letting the circuit settle after each.

        Flipping several inside a single tick is a legal thing to ask for, but it
        injects transients that a ripple chain can latch on to; a player flips
        levers one by one, so that is what we model.
        """
        ok = True
        for i, b in enumerate(bits):
            if self.state[i] != bool(b):
                self.sim.use(*self.positions[i])
                self.state[i] = bool(b)
                ok = self.sim.settle(settle) and ok
        ok = self.sim.settle(settle) and ok
        return ok
