"""Cell library: dense, flat, abutment-composable logic cells.

A Cell is a verified template: a block fragment plus PORT contracts (inputs on
the west face, outputs on the east face, at fixed y=1 and declared z rows).
Cells compose by placing fragments so that an output port cell IS the next
cell's input port cell -- connection by abutment, no routing.  Internal
crossings are braided by the maze router at build time, then the whole cell is
truth-tabled in mc-tick before it may be stamped.

The comparator half adder (all flat, no torch towers -- height 2):

    xor   = (a - b) OR (b - a)     two subtract comparators, outputs merged
    carry = a - xor                one subtract comparator

Comparator side inputs are fed through repeaters: a repeater is directional
and connects to no neighbouring dust, so it taps a line without shorting the
braid.  1 redstone tick per comparator.
"""
import rs
import router as router_mod

COMP = "minecraft:comparator[facing=%s,mode=%s,powered=false]"
if "comparator" not in rs.EXTRA_STATES:
    # intern BOTH powered states: a state that first appears mid-simulation
    # (a comparator turning on) is otherwise inert -- the classic trap
    rs.EXTRA_STATES += ";" + ";".join(
        "minecraft:comparator[facing=%s,mode=%s,powered=%s]" % (f, m, pw)
        for f in ("north", "south", "east", "west")
        for m in ("compare", "subtract") for pw in ("false", "true"))


class Fragment:
    """A placeable piece: cells + labels + named ports (build coordinates)."""

    def __init__(self):
        self.cells, self.labels, self.ports = {}, {}, {}

    def stamp(self, build, labels, dx, dy, dz, rename=None):
        rn = rename or (lambda s: s)
        for (x, y, z), blk in self.cells.items():
            build.put(x + dx, y + dy, z + dz, blk)
        for (x, y, z), lab in self.labels.items():
            labels[(x + dx, y + dy, z + dz)] = rn(lab)
        return {name: (x + dx, y + dy, z + dz)
                for name, (x, y, z) in self.ports.items()}


def build_half_adder():
    """Construct + internally route the HA fragment.  Returns Fragment."""
    b = rs.Build("ha_cell")
    labels = {}

    def dust(x, y, z, lab):
        b.stone(x, y - 1, z)
        b.put(x, y, z, rs.DUST)
        labels[(x, y, z)] = lab

    # a lane (z0):  in .. C1(sub: a-b) .. partial out
    for x in (0, 1, 2):
        dust(x, 1, 0, "a")
    b.stone(3, 0, 0)
    b.put(3, 1, 0, COMP % ("west", "subtract"))
    dust(4, 1, 0, "xor")

    # b lane (z4):  in .. C2(sub: b-a) .. partial out
    for x in (0, 1, 2, 3, 4):
        dust(x, 1, 4, "b")
    b.stone(5, 0, 4)
    b.put(5, 1, 4, COMP % ("west", "subtract"))
    dust(6, 1, 4, "xor")

    # C1 side: repeater fed from a short inner b-branch (verified clean)
    b.stone(3, 0, 1)
    b.put(3, 1, 1, rs.repeater("south"))     # feeds C1 side with b
    dust(3, 1, 2, "b")

    # a-row (z6), SOUTH of everything: one branch of `a` serves both C2's
    # side (repeater at z5 reads it) and C3's back input.  Keeping it on the
    # outer flank leaves the cell centre free for the xor merge.
    b.stone(5, 0, 5)
    b.put(5, 1, 5, rs.repeater("south"))     # feeds C2 side with a
    for x in (5, 6, 7):
        dust(x, 1, 6, "a")
    b.stone(8, 0, 6)
    b.put(8, 1, 6, COMP % ("west", "subtract"))   # C3: carry = a - xor
    dust(9, 1, 6, "carry")
    b.stone(8, 0, 5)
    b.put(8, 1, 5, rs.repeater("north"))     # feeds C3 side with xor
    dust(8, 1, 4, "xor")
    dust(9, 1, 4, "xor")                      # xor output port

    # Designed bridge: `a` crosses the b lane at x=1 on a y3 flyover whose
    # middle support CAPS the b lane -- legal because b there is a flat run
    # with no diagonals to cut.  Leaf cells are hand-crafted; the router is
    # for composition above this level.
    dust(1, 2, 1, "a")
    dust(1, 3, 2, "a")
    for z in (3, 4, 5):
        dust(1, 3, z, "a")
    dust(1, 2, 6, "a")
    dust(2, 1, 6, "a")
    # drive margin: `a` has crossed ~10 repeater-free cells by here -- refresh
    # it before it feeds C2's side and C3's back, or the cell has no fanout
    b.stone(3, 0, 6)
    b.put(3, 1, 6, rs.repeater("west"))
    dust(4, 1, 6, "a")
    b.cells[(1, 2, 4)] = b.cells[(1, 2, 4)]        # (cap laid by dust support)

    r = router_mod.Router(b, labels, bounds=(0, 11, 1, 5, -1, 8))
    r.route((2, 1, 4), (3, 1, 2), "b")            # b -> C1 side feed
    r.route((4, 1, 0), (6, 1, 4), "xor")          # merge partials (centre)
    r.route((6, 1, 4), (8, 1, 4), "xor")          # xor -> C3 side + port

    frag = Fragment()
    frag.cells, frag.labels = dict(b.cells), dict(labels)
    frag.ports = {"a": (0, 1, 0), "b": (0, 1, 4),
                  "xor": (9, 1, 4), "carry": (9, 1, 6)}
    return frag


def verify_fragment(frag, inputs, outputs, truth, name):
    """Exhaustively truth-table a fragment in mc-tick."""
    b = rs.Build("v_" + name)
    labels = {}
    ports = frag.stamp(b, labels, 2, 0, 0)
    lever_at = {}
    for i, sig in enumerate(inputs):
        x, y, z = ports[sig]
        dx, dz = (0, -1) if sig.startswith("cin") else (-1, 0)
        b.stone(x + 2 * dx, y - 1, z + 2 * dz)
        b.force(x + 2 * dx, y, z + 2 * dz, rs.LEVER_OFF)
        b.stone(x + dx, y - 1, z + dz)
        b.put(x + dx, y, z + dz, rs.DUST)
        lever_at[sig] = (x + 2 * dx, y, z + 2 * dz)
    import nets
    shorts = nets.check(b.cells, labels)
    if shorts:
        for s in shorts[:4]:
            print("   SHORT", s)
        return False
    sim = b.sim()
    lv = rs.Levers(sim, [lever_at[s] for s in inputs])
    ok = True
    for row in truth:
        ins, outs = row[:len(inputs)], row[len(inputs):]
        lv.set(list(ins))
        got = tuple(int(sim.on(*ports[o])) for o in outputs)
        if got != tuple(outs):
            print("   %s: %s -> %s, want %s" % (name, ins, got, outs))
            ok = False
    print("%s truth table: %s" % (name, "PASS" if ok else "FAIL"))
    return ok


PITCH = 13          # bit pitch: FA content spans z-1..10, so 13 leaves
                    # one clearance row between cells -- only the carry
                    # port column may touch the cell boundary
CIN_X = 21          # the carry chain column: cout south, next bit's cin north


def build_full_adder(ha):
    """FA from two HA stamps: sum = (a^b)^cin, cout = maj via carry-OR merge.

    Port contract (datapath style): a, b enter the WEST face (z0, z4); sum
    leaves the EAST face; cin enters the NORTH face and cout leaves the SOUTH
    face at the same x -- so stacking bits at PITCH in z connects the carry
    chain by abutment alone.
    """
    b = rs.Build("fa_cell")
    labels = {}
    p1 = ha.stamp(b, labels, 0, 0, 0,
                  rename=lambda s: {"xor": "h1x", "carry": "cout"}.get(s, s))
    p2 = ha.stamp(b, labels, 10, 0, 4,
                  rename=lambda s: {"a": "h1x", "b": "cin", "xor": "sum",
                                    "carry": "cout"}.get(s, "h2" + s))
    # Port stubs run ALONG the carry column, one-way southbound.  The cell
    # boundary rows must hold nothing but this x=CIN_X chain: two cells'
    # east-west port approaches once formed a repeater RING across the seam --
    # aliased into one net, invisible to the short checker, and it latched.
    for z, blk in ((-1, rs.DUST), (0, rs.DUST), (1, rs.repeater("north")),
                   (2, rs.DUST)):
        b.stone(CIN_X, 0, z)
        b.put(CIN_X, 1, z, blk)
        if blk == rs.DUST:
            labels[(CIN_X, 1, z)] = "cin"
    for z, blk in ((8, rs.DUST), (9, rs.repeater("north")), (10, rs.DUST),
                   (PITCH - 2, rs.DUST)):
        b.stone(CIN_X, 0, z)
        b.put(CIN_X, 1, z, blk)
        if blk == rs.DUST:
            labels[(CIN_X, 1, z)] = "cout"

    # tightest nets first; cin has the most routing freedom, so it goes last.
    # Routes are confined to the INTERIOR (z 0..PITCH-3): the boundary rows
    # z=-1 and z=PITCH-2 belong to the port column ALONE.  Two adjacent
    # stamps once routed in their facing boundary rows -- bit k's cout join
    # (repeater east into the port) touched bit k+1's cin route (repeater
    # west out of the port) across the seam.  Same aliased net, so the short
    # checker was silent, but the pair of opposite repeaters closed a
    # SELF-SUSTAINING RING that latched on the first sum transient.
    r = router_mod.Router(b, labels, bounds=(0, 23, 1, 5, 0, PITCH - 3))
    # OR-merge targets: the designed port stub cells on the interior side.
    # The carry sources are themselves labelled cout, so a naive route-to-net
    # would "succeed" instantly without laying a single block -- exclude them.
    stub = [(CIN_X, 1, 8), (CIN_X, 1, 10)]
    r.route(p2["carry"], stub, "cout")
    r.route(p1["xor"], p2["a"], "h1x")            # HA1.xor -> HA2.a abutment gap
    cout_net = [c for c, l in labels.items()
                if l == "cout" and c[0] >= 19 and c[2] <= PITCH - 3]
    r.route(p1["carry"], cout_net, "cout")
    r.route((CIN_X, 1, 2), p2["b"], "cin")        # from the stub's interior end

    frag = Fragment()
    frag.cells, frag.labels = dict(b.cells), dict(labels)
    frag.ports = {"a": (0, 1, 0), "b": (0, 1, 4), "cin": (CIN_X, 1, -1),
                  "sum": p2["xor"], "cout": (CIN_X, 1, PITCH - 2)}
    return frag


if __name__ == "__main__":
    ha = build_half_adder()
    xs = [p[0] for p in ha.cells]
    zs = [p[2] for p in ha.cells]
    print("HA cell bbox: %dx2x%d, %d blocks"
          % (max(xs) - min(xs) + 1, max(zs) - min(zs) + 1, len(ha.cells)))
    verify_fragment(ha, ["a", "b"], ["xor", "carry"],
                    [(0, 0, 0, 0), (1, 0, 1, 0), (0, 1, 1, 0), (1, 1, 0, 1)],
                    "half_adder")
    fa = build_full_adder(ha)
    xs = [p[0] for p in fa.cells]; zs = [p[2] for p in fa.cells]
    print("FA cell bbox: %dx%dx%d, %d blocks" % (max(xs)-min(xs)+1,
          max(p[1] for p in fa.cells)+1, max(zs)-min(zs)+1, len(fa.cells)))
    rows = []
    for a in (0,1):
        for bb in (0,1):
            for c in (0,1):
                s = a ^ bb ^ c
                co = (a & bb) | (a & c) | (bb & c)
                rows.append((a, bb, c, s, co))
    verify_fragment(fa, ["a", "b", "cin"], ["sum", "cout"], rows, "full_adder")
