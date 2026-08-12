"""Design-rule checks every builder runs: shorts, support, and diode rings.

`nets.check` proves no two signals touch and `audit.audit` proves nothing
floats.  Neither can see the third defect class, which has now cost this
project three circuits:

    a directed cycle through a repeater/comparator is a STORAGE ELEMENT.

Unless the latch was built on purpose it is a bug, and it is the worst kind of
bug this toolchain produces, because every check we had stayed green:

  * `nets.check` is blind -- every cell on the ring carries the SAME label, so
    there is no short to find;
  * `audit.audit` is blind -- the ring is perfectly supported;
  * simulation is blind -- the ring latches at 15 and the world is then
    QUIESCENT, so `run_until_quiescent` sees nothing left to do and the
    exhaustive proof just reports a wrong answer somewhere.

The three occurrences:

  1. rca_cells -- two cells' port approaches formed opposite-facing repeaters
     in a ring on an aliased net; it latched the placement transient.
  2. the same shape again on the Rust side, which is why
     `crates/nucleation-routing/src/drc.rs::repeater_cycles` exists.  This
     module is the Python port of that check -- the parity gap that let (3)
     through.
  3. mult4's `m1B2` -- `REFRESH` 5 -> 14 (commit 5076520c) re-pathed the route
     along the rail it was supposed to DRIVE.  `dust_ok` allowed it (same net
     label, and own-net reuse is normally fine), but a compiled rail is fed
     THROUGH a repeater, so the reused cells sat downstream of a diode the
     route then fed from the other end:

         rail driver -> rail cell -> route tail -> drive cell -> rail driver

     Output bit 3 stuck at 1; the multiplier scored 64/256.
     `router.downstream_rail` refuses that specific contact now, and
     `test_diode_ring` guards it -- but only for `router.py`, and only for a
     route joining a rail.  THIS check is the general rule: it does not care
     how the ring was formed or which builder formed it.

The rule, stated over geometry alone (no simulation, no netlist):

    build the conduction graph -- dust components are single nodes (union-find
    over `nets.neighbours`, so all of a net's dust collapses), each
    repeater/comparator is a DIRECTED edge from the node at its input to the
    node at its output -- then report every directed cycle.

Two conduction paths beyond bare dust-to-dust are modelled, because the router
emits them and a ring through one latches just as hard:

  * block-sandwich stations (`router.station_ok`): trunk dust -> entry block ->
    repeater -> exit block -> dust.  Dust only weak-powers the block it points
    INTO, so the input is read collinearly through the entry block; the exit
    block is STRONGLY powered and re-emits 15 to every adjacent dust, so the
    output fans out to the whole 6-neighbourhood (probe_station S_exit_*).
  * diode -> block -> diode with no dust between them, via the same
    strongly-powered exit block.

Torches are deliberately NOT diodes here: an inverter ring oscillates rather
than latching, and the NOR-latch gadgets in `rca_cells` / `seq_cells` are built
from torches on purpose.  Keeping torches out matches the Rust check and keeps
this rule free of false positives, so it can run on every build.

Cost is one pass over `cells` plus an iterative DFS over dust components --
geometry only, no simulation.  It is cheap enough to be unconditional (mult4's
20k-block design: tens of milliseconds), which is the whole point: a check that
is expensive is a check that gets skipped, and this defect class is exactly the
one that survives every check you skip.

Usage -- one line next to the audit + nets pair the builders already spell out:

    rings_ok = drc.check_rings("mult4", b.cells)
    if any(problems.values()) or shorts or not rings_ok:
        return 1

`expect` accepts a deliberate latch count for a design that really does store
state in a ring (`seq_counter` declares one per bit); it must be given
explicitly, per build, with a reason.  `drc.check` returns all three findings at
once for a caller that would rather inspect than print.
"""
import audit
import nets

DUST_KEY = nets.DUST_KEY

FACING_VEC = {"north": (0, 0, -1), "south": (0, 0, 1),
              "east": (1, 0, 0), "west": (-1, 0, 0)}


def is_dust(block):
    return block is not None and DUST_KEY in block


def is_diode(block):
    """Repeater or comparator: a one-way gate that can hold a ring high."""
    return block is not None and ("minecraft:repeater" in block
                                 or "minecraft:comparator" in block)


def facing_of(block):
    if block is None or "facing=" not in block:
        return None
    return block.split("facing=")[1].split(",")[0].rstrip("]")


def diode_io(pos, block):
    """(input cell, output cell) of the diode at `pos`.

    `facing` names the INPUT side: `repeater[facing=west]` reads -X and
    conducts toward +X (verified, rs.repeater's docstring).
    """
    f = facing_of(block)
    v = FACING_VEC.get(f)
    if v is None:
        return None
    x, y, z = pos
    dx, dy, dz = v
    return (x + dx, y + dy, z + dz), (x - dx, y - dy, z - dz)


def _components(cells):
    """Union-find over dust: pos -> component root."""
    parent = {}

    def find(a):
        parent.setdefault(a, a)
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    dust = [p for p, b in cells.items() if DUST_KEY in b]
    for p in dust:
        find(p)
        for q in nets.neighbours(cells, p):
            ra, rb = find(p), find(q)
            if ra != rb:
                parent[ra] = rb
    return find


def _adjacent(pos):
    x, y, z = pos
    return ((x + 1, y, z), (x - 1, y, z), (x, y, z + 1), (x, y, z - 1),
            (x, y + 1, z), (x, y - 1, z))


def conduction_graph(cells):
    """Directed graph whose cycles are latches.

    Nodes are ``("d", root)`` for a dust component and ``("b", pos)`` for a
    solid block that some diode STRONGLY powers.  Edges out of a diode are
    labelled with that diode's position; edges out of a block node are plain
    conduction (strong power re-emitted to adjacent dust).  A block node's only
    incoming edges come from diodes, and dust-to-dust conduction is collapsed
    into the component, so every cycle in this graph contains at least one
    diode -- which is exactly the thing that makes it latch.

    Returns ``{node: [(node, diode_pos_or_None), ...]}``.
    """
    find = _components(cells)
    diodes = [(p, b) for p, b in cells.items() if is_diode(b)]

    # Blocks a diode strongly powers become nodes: they re-emit 15.
    strong = set()
    for p, b in diodes:
        io = diode_io(p, b)
        if io is None:
            continue
        out = io[1]
        if not is_dust(cells.get(out)) and nets.is_solid(cells, out):
            strong.add(out)

    edges = {}

    def add(src, dst, diode):
        # self-loops are kept on purpose: src == dst is the mult4 shape, a
        # diode whose input and output landed in the SAME dust component
        edges.setdefault(src, []).append((dst, diode))

    def node_at(pos, collinear_probe=None):
        """The conduction node a diode reads/drives at `pos`."""
        if is_dust(cells.get(pos)):
            return ("d", find(pos))
        if pos in strong:
            return ("b", pos)
        if collinear_probe is not None and nets.is_solid(cells, pos):
            # station entry block: only the dust POINTING INTO it powers it
            if is_dust(cells.get(collinear_probe)):
                return ("d", find(collinear_probe))
        return None

    for p, b in diodes:
        io = diode_io(p, b)
        if io is None:
            continue
        inp, out = io
        v = FACING_VEC[facing_of(b)]
        beyond = (inp[0] + v[0], inp[1] + v[1], inp[2] + v[2])
        src = node_at(inp, collinear_probe=beyond)
        dst = node_at(out)
        if src is None or dst is None:
            continue
        add(src, dst, p)

    # Strongly-powered blocks re-emit to every adjacent dust cell.
    for blk in strong:
        for q in _adjacent(blk):
            if is_dust(cells.get(q)):
                add(("b", blk), ("d", find(q)), None)

    return edges


def repeater_cycles(cells):
    """Every directed cycle through a repeater/comparator.

    Returns a list of tuples of diode positions, sorted and de-duplicated, so
    the result is deterministic and a caller can print it verbatim.
    """
    edges = conduction_graph(cells)
    nodes = set(edges)
    for succs in edges.values():
        for to, _d in succs:
            nodes.add(to)

    WHITE, GREY, BLACK = 0, 1, 2
    color = {n: WHITE for n in nodes}
    found, reported = [], set()

    for root in sorted(nodes):
        if color[root] != WHITE:
            continue
        color[root] = GREY
        path = [root]
        entry = [None]                      # diode each path node was entered by
        stack = [iter(edges.get(root, ()))]
        while stack:
            nxt = next(stack[-1], None)
            if nxt is None:
                color[path[-1]] = BLACK
                path.pop()
                entry.pop()
                stack.pop()
                continue
            to, diode = nxt
            c = color.get(to, WHITE)
            if c == GREY:
                # the cycle runs from `to` forward along the path and closes
                # through this edge; `to`'s own entry edge came from outside
                i = path.index(to)
                ring = [d for d in entry[i + 1:] if d is not None]
                if diode is not None:
                    ring.append(diode)
                key = tuple(sorted(ring))
                if key and key not in reported:
                    reported.add(key)
                    found.append(key)
            elif c == WHITE:
                color[to] = GREY
                path.append(to)
                entry.append(diode)
                stack.append(iter(edges.get(to, ())))
    return found


def check_rings(name, cells, expect=0, quiet=False):
    """The ring rule alone, for a builder that already prints its own
    audit/nets report.  Returns True when the geometry holds no latch.

    `expect` is a deliberate latch count -- a design that really does store
    state states it here, with a reason, at the call site.  Never raise it to
    silence a surprise: an unexplained ring means the build is broken in the
    one way nothing else can see.
    """
    cyc = repeater_cycles(cells)
    if len(cyc) == expect:
        if not quiet:
            print("diode rings: %d%s" % (len(cyc), " (declared)" if expect else ""))
        return True
    if not quiet:
        print("DIODE RING x%d (expected %d) in %s -- a ring holding a repeater "
              "LATCHES at 15: the net reads high forever, no input can clear "
              "it, and the world is quiescent so simulation cannot see it"
              % (len(cyc), expect, name))
        for ring in cyc[:6]:
            print("   diodes:", ", ".join(str(p) for p in ring))
    return False


def check(cells, labels, aliases=()):
    """All three static rules at once, for a caller that wants the findings
    rather than a printed report: shorts, support, rings."""
    return {
        "problems": audit.audit(cells),
        "shorts": nets.check(cells, labels, aliases),
        "cycles": repeater_cycles(cells),
    }


if __name__ == "__main__":
    # Ring report for the builds that are cheap to construct.  The real proof
    # lives in test_diode_ring.py, which also rebuilds the PRE-FIX mult4 tree
    # and confirms the rule fires there.
    import time

    def report(label, cells):
        t0 = time.time()
        rings = repeater_cycles(cells)
        print("%-14s %7d cells  %5.0f ms  rings=%d%s"
              % (label, len(cells), 1000 * (time.time() - t0), len(rings),
                 "" if not rings else "  %s" % (rings[:2],)))

    import build_adder as ad
    report("adder4", ad.build(4)[0].cells)
    import mult4
    report("mult4", mult4.build()["b"].cells)
