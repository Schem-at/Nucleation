"""Verify the two user-supplied wire-crossing schematics in mc-tick.

Ground truth (copies; the originals in ~/Downloads are never touched):

  CROSSWIRE002_classic_crosswire.schem   5 x 11 x 5, 90 cells
      A tileable 2-y unit: a Z-line whose repeater STRONGLY POWERS the
      crossing block, and an X-line whose repeater sits ON that block one
      level up.  Buffered (1 rt per line), refreshes both signals to 15.

  CROSSWIRE001_instant_crosswire.schem   7 x 19 x 17, 609 cells
      TWO variants side by side in z, each tiled 4x vertically:
        region A (z=0..8)   "hop":    lines at 1-y pitch, axes alternating
                                      per level; each line hops 1 up over
                                      the line directly beneath it.
        region B (z=10..16) "updown": both lines enter/leave at the SAME y;
                                      one dips 1 down, the other bumps 1 up,
                                      and the intersection cell is AIR.
      Pure dust: 0-tick delay, cost is signal strength only.

What this script does, per crossing instance:

  1. rebuilds the cells into a fresh Build (offset into positive space),
  2. finds each line's two PORTS (net endpoints, auto-derived from the
     vanilla wire-connection rules) or uses the hardcoded classic ports
     (whose nets are split by repeaters),
  3. extends every port outward with 2 dust + a lever,
  4. drives the full 2^n lever matrix over the crossing's n lines and
     asserts each output is hot IFF its own input is hot -- zero crosstalk,
  5. reports arrival signal strength and the tick delay of each line.

Run: ~/eda-venv/bin/python crosswire/verify_crosswire.py
"""
import os
import sys
import itertools

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import nucleation as n
import rs
from rs import DUST

HERE = os.path.dirname(os.path.abspath(__file__))
WOOL = ("minecraft:red_wool", "minecraft:blue_wool")
rs.EXTRA_STATES += ";" + ";".join(WOOL)

OFF = (8, 1, 8)          # push the ground truth into strictly positive space

HOR = ((1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1))


# --------------------------------------------------------------- schem loading
def load(name):
    """{(x,y,z): blockstate} for every non-air cell of a .schem."""
    s = n.Schematic.open(os.path.join(HERE, name + ".schem"))
    mn, mx = s.tight_bounds_min(), s.tight_bounds_max()
    cells = {}
    for x in range(mn.x, mx.x + 1):
        for y in range(mn.y, mx.y + 1):
            for z in range(mn.z, mx.z + 1):
                b = s.get_block_string(x, y, z)
                if b and "air" not in b:
                    cells[(x, y, z)] = b
    return cells


def slab_z(cells, z0, z1):
    return {k: v for k, v in cells.items() if z0 <= k[2] <= z1
            and "glass" not in v}          # the 2 glass cells are bbox markers


# ------------------------------------------------------- vanilla wire topology
# RedStoneWireBlock.calculateTargetStrength (1.20):
#   for each horizontal neighbour np of self:
#       read wire AT np                                       (flat)
#       if np isRedstoneConductor and above(self) is not  -> read wire at np+up
#       elif np is not isRedstoneConductor                -> read wire at np+down
# The two diagonal reads gate on DIFFERENT cells; see TRANSPORT_MODEL.md.
def is_dust(b):
    return b is not None and "redstone_wire" in b


def conducts(b):
    """isRedstoneConductor: full opaque cube.  Wool/concrete/stone yes; glass,
    slabs, air, and every device (repeater/lever/dust) no."""
    return b is not None and ("wool" in b or "concrete" in b
                              or b == "minecraft:stone")


def wire_edges(cells):
    """Undirected 'same net' edges (either read direction is enough)."""
    out = []
    for p in [k for k in cells if is_dust(cells[k])]:
        x, y, z = p
        for dx, _dy, dz in HOR:
            side = (x + dx, y, z + dz)
            if is_dust(cells.get(side)):
                out.append((p, side))
                continue
            up = (x + dx, y + 1, z + dz)
            if is_dust(cells.get(up)) and conducts(cells.get(side)) \
                    and not conducts(cells.get((x, y + 1, z))):
                out.append((p, up))
            dn = (x + dx, y - 1, z + dz)
            if is_dust(cells.get(dn)) and not conducts(cells.get(side)):
                out.append((p, dn))
    return out


def wire_nets(cells):
    """[(sorted cells, [endpoints])] -- one entry per isolated dust net."""
    edges = wire_edges(cells)
    adj = {}
    for a, b in edges:
        adj.setdefault(a, set()).add(b)
        adj.setdefault(b, set()).add(a)
    seen, nets = set(), []
    for p in [k for k in cells if is_dust(cells[k])]:
        if p in seen:
            continue
        comp, stack = set(), [p]
        while stack:
            q = stack.pop()
            if q in comp:
                continue
            comp.add(q)
            stack.extend(adj.get(q, ()))
        seen |= comp
        ends = sorted(c for c in comp if len(adj.get(c, ())) <= 1)
        nets.append((sorted(comp, key=lambda t: (t[1], t[2], t[0])), ends, adj))
    return nets


def outward(port, adj):
    """Horizontal direction that leaves the net at `port`."""
    nb = list(adj.get(port, ()))
    if not nb:
        raise AssertionError("isolated dust %r: give it an explicit port dir" % (port,))
    d = (port[0] - nb[0][0], 0, port[2] - nb[0][2])
    # a diagonal neighbour still fixes the axis; normalise to unit length
    return (max(-1, min(1, d[0])), 0, max(-1, min(1, d[2])))


# ---------------------------------------------------------------- the test rig
class Rig:
    """One crossing instance: ground-truth cells + lever-driven port stubs."""

    def __init__(self, name, cells):
        self.name = name
        self.b = rs.Build(name)
        self.cells = cells
        for (x, y, z), blk in cells.items():
            self.b.force(x + OFF[0], y + OFF[1], z + OFF[2], blk)
        self.lines = []                     # (label, lever_pos, out_pos)
        self.levers = []
        self.truncated = []

    def _p(self, c):
        return (c[0] + OFF[0], c[1] + OFF[1], c[2] + OFF[2])

    def _rig_put(self, c, blk):
        """Place a RIG cell.  Never allowed to overwrite ground truth: a stub
        that lands inside the tile is a mis-derived port, not a test.  (This
        guard is what caught the truncated top unit of CROSSWIRE001.)"""
        if c in self.cells and self.cells[c] != blk:
            raise AssertionError(
                "%s: rig cell %r would overwrite ground truth %s with %s"
                % (self.name, c, self.cells[c], blk))
        self.b.force(*self._p(c), block=blk)

    def _stub_dust(self, port, d, cells):
        p = port
        for i in range(1, cells + 1):
            c = (port[0] + d[0] * i, port[1], port[2] + d[2] * i)
            self._rig_put((c[0], c[1] - 1, c[2]), rs.STONE)
            self._rig_put(c, DUST)
            p = c
        return p

    def stub(self, port, d, cells=2):
        """dust*cells + lever outward from `port`; returns the lever position."""
        p = self._stub_dust(port, d, cells)
        lv = (port[0] + d[0] * (cells + 1), port[1], port[2] + d[2] * (cells + 1))
        self._rig_put((lv[0], lv[1] - 1, lv[2]), rs.STONE)
        self._rig_put(lv, rs.LEVER_OFF)
        return lv, p

    def stub_out(self, port, d, cells=2):
        """Output stub: dust only, no lever.  The far cell becomes the probe,
        so a leak has to cross the tile AND two fresh dust cells to score."""
        return self._stub_dust(port, d, cells)

    def line(self, label, in_port, in_dir, out_port, out_dir, stub_out=2):
        lv, _ = self.stub(in_port, in_dir)
        probe = self.stub_out(out_port, out_dir, stub_out) if stub_out else out_port
        self.levers.append(lv)
        self.lines.append((label, len(self.levers) - 1, probe, in_port, out_port))

    def run(self, exhaustive=None):
        sim = self.b.sim()
        lv = rs.Levers(sim, [self._p(p) for p in self.levers])
        nl = len(self.lines)
        if exhaustive is None:
            exhaustive = nl <= 8
        combos = (list(itertools.product((0, 1), repeat=nl)) if exhaustive
                  else [tuple(0 for _ in range(nl))]
                  + [tuple(1 if i == k else 0 for i in range(nl)) for k in range(nl)]
                  + [tuple(1 for _ in range(nl))]
                  + [tuple((i + k) % 2 for i in range(nl)) for k in (0, 1)])
        bad, checks, ss = 0, 0, {}
        for combo in combos:
            lv.set(combo)
            for (label, li, probe, _ip, _op) in self.lines:
                want = bool(combo[li])
                got = sim.power(*self._p(probe))
                checks += 1
                if bool(got > 0) != want:
                    bad += 1
                    print("    FAIL %-22s combo=%s want=%s got_ss=%d"
                          % (label, "".join(map(str, combo)), want, got))
                if want and got > 0:
                    ss.setdefault(label, set()).add(got)
        lv.set([0] * nl)
        return sim, lv, bad, checks, ss

    def delays(self, sim, lv):
        """Game ticks from a lever flip to the output going hot, per line."""
        out = {}
        for (label, li, probe, _ip, _op) in self.lines:
            lv.set([0] * len(self.lines))
            bits = [0] * len(self.lines)
            bits[li] = 1
            # flip WITHOUT settling, then step one game tick at a time
            sim.use(*self._p(self.levers[li]))
            t = 0
            while t < 40 and sim.power(*self._p(probe)) == 0:
                sim.sim.step()
                t += 1
            out[label] = t if sim.power(*self._p(probe)) > 0 else None
            sim.settle()
            sim.use(*self._p(self.levers[li]))
            sim.settle()
        return out


# --------------------------------------------------------------- the instances
def auto_lines(rig, cells, want_levels=None):
    """One line per dust net, ports = net endpoints (pure-dust designs).

    A real port leaves the tile, so both endpoints must sit on an x/z boundary
    plane of the region.  A net that dead-ends INSIDE the region is a unit the
    schematic truncated (the top tile of CROSSWIRE001 loses one cell) -- it is
    reported and skipped rather than driven through a fabricated port.
    """
    xs = [c[0] for c in cells]
    zs = [c[2] for c in cells]
    lo, hi = (min(xs), min(zs)), (max(xs), max(zs))

    def on_edge(c):
        return c[0] in (lo[0], hi[0]) or c[2] in (lo[1], hi[1])

    skipped = []
    for comp, ends, adj in wire_nets(cells):
        if len(ends) != 2:
            raise AssertionError("%s: net %r has %d endpoints"
                                 % (rig.name, comp[0], len(ends)))
        y0 = min(c[1] for c in comp)
        if want_levels is not None and y0 not in want_levels:
            continue
        a, b = ends
        if not (on_edge(a) and on_edge(b)):
            skipped.append((y0, a, b))
            continue
        rig.line("y%02d_%s" % (y0, "".join(map(str, a))),
                 a, outward(a, adj), b, outward(b, adj))
    rig.truncated = skipped


def classic_rig(unit):
    """One 2-y unit of the classic crossing (yz = Z-line, yz+1 = X-line)."""
    cells = load("CROSSWIRE002_classic_crosswire")
    yz = 1 + 2 * unit
    keep = {k: v for k, v in cells.items() if yz - 1 <= k[1] <= yz + 2}
    r = Rig("classic_u%d" % unit, keep)
    #  Z-line: dot dust (2,yz,4) -> repeater[facing=south] (2,yz,3)
    #          -> STRONG block (2,yz,2) -> dust (2,yz,1),(2,yz,0)
    r.line("Zline", (2, yz, 4), (0, 0, 1), (2, yz, 0), (0, 0, -1))
    #  X-line: dust (4..3,yz+1,2) -> repeater[facing=east] (2,yz+1,2)
    #          -> dust (1,yz+1,2),(0,yz+1,2)
    r.line("Xline", (4, yz + 1, 2), (1, 0, 0), (0, yz + 1, 2), (-1, 0, 0))
    return r


def classic_stack():
    """All five stacked units at once (the tileability check)."""
    r = Rig("classic_stack", load("CROSSWIRE002_classic_crosswire"))
    for unit in range(5):
        yz = 1 + 2 * unit
        r.line("Z%d" % unit, (2, yz, 4), (0, 0, 1), (2, yz, 0), (0, 0, -1))
        r.line("X%d" % unit, (4, yz + 1, 2), (1, 0, 0), (0, yz + 1, 2), (-1, 0, 0))
    return r


def instant_rig(region, levels=None):
    cells = load("CROSSWIRE001_instant_crosswire")
    z0, z1 = (0, 8) if region == "A" else (10, 16)
    r = Rig("instant_%s" % region, slab_z(cells, z0, z1))
    auto_lines(r, r.cells, levels)
    return r


def main():
    fails = 0
    for rig in ([classic_rig(0), classic_stack()]
                + [instant_rig("A", levels=(1, 2, 3, 4)),
                   instant_rig("B", levels=(2, 3, 4, 5)),
                   instant_rig("A"), instant_rig("B")]):
        sim, lv, bad, checks, ss = rig.run()
        dl = rig.delays(sim, lv)
        print("%-16s lines=%2d checks=%4d %s" %
              (rig.name, len(rig.lines), checks,
               "OK" if bad == 0 else "%d FAIL" % bad))
        for (label, _li, _pr, ip, op) in rig.lines:
            print("    %-18s in%-12s out%-12s ss=%-10s delay=%s gt"
                  % (label, ip, op,
                     sorted(ss.get(label, ())), dl.get(label)))
        for (y0, a, b) in rig.truncated:
            print("    (skipped truncated net y0=%d %s..%s -- the schematic's "
                  "top tile is incomplete)" % (y0, a, b))
        fails += bad
    print("verify_crosswire: %s" % ("PASS" if fails == 0 else "%d FAILURES" % fails))
    return 1 if fails else 0


if __name__ == "__main__":
    raise SystemExit(main())
