"""A* maze router for redstone, with design rules and vertical vias.

This replaces hand-allocated channels for irregular interconnect: give it a
built world (cells + net labels), a source dust cell and a target dust cell,
and it finds a legal path through free space, laying dust, supports,
repeaters, and torch-ladder climbs.

Moves and costs:
    horizontal step        1     dust + support block
    stair (dy = +-1)       3     dust + support, diagonal connection
                                 (max 4 in a row: stairs cannot host
                                 repeaters, so long runs decay to nothing --
                                 long verticals must use ladders, whose cap
                                 emits a fresh 15)
    ladder climb (+5 y)    8     1x1 torch ladder, 2 torches, non-inverting

Design rules enforced per placed dust cell:
    * the cell and its support must be free (or the support already solid);
    * no FOREIGN dust electrically adjacent (nets.py connectivity, including
      the cut rules for blocked diagonals);
    * supports are never placed directly above existing dust -- a block above
      dust cuts that dust's up-diagonals, changing someone else's circuit.

Signal budget: a repeater is inserted after 6 straight cells; stairs and
climbs refresh or preserve level by construction (the climb's output is a
strongly powered cap block: a fresh 15).
"""
import heapq
import re

import rs
import nets
import materials as _mt

_FACING = re.compile(r"facing=(north|south|east|west)")

H_MOVES = ((1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1))
H_MOVES_XZ = ((1, 0), (-1, 0), (0, 1), (0, -1))


class Router:
    def __init__(self, build, labels, bounds=None):
        self.b = build
        self.labels = labels
        self.bounds = bounds        # (x0,x1, y0,y1, z0,z1): cells must not
                                    # route outside their own footprint
        self.strong = {}            # station exit blocks: strongly powered
                                    # solids that re-emit 15 into ANY adjacent
                                    # dust -- later routes must keep clear
        self.soft = {}              # pos -> (label, penalty): extra A* cost
                                    # for FOREIGN nets entering the cell.
                                    # Soft, not legal: port pockets stay
                                    # crossable, but trunks stop parking in
                                    # front of them (a legally-dead-ended
                                    # foreign trunk can seal a pocket)
        self.ss = {}                # dust pos -> estimated signal strength
                                    # as emitted.  Lets a multi-source route
                                    # resume the refresh budget mid-trunk --
                                    # a branch leaving a trunk one cell
                                    # before its repeater starts at ss1 and
                                    # a corner dust there is DEAD (found by
                                    # genlib seg7: n32's branch never fired)

    def in_bounds(self, p):
        if self.bounds is None:
            return True
        x0, x1, y0, y1, z0, z1 = self.bounds
        return x0 <= p[0] <= x1 and y0 <= p[1] <= y1 and z0 <= p[2] <= z1

    # -- design rules -------------------------------------------------------
    def dust_ok(self, p, label, friendly=None):
        x, y, z = p
        ok = friendly or {label}
        if p in self.b.cells:
            return self.labels.get(p) in ok         # reuse own net only
        # a station's exit block strong-powers every adjacent dust cell
        # (probe_station S_exit_*): the electrical model below cannot see
        # that, so keep foreign dust out of its whole 6-neighbourhood
        for nb in ((x + 1, y, z), (x - 1, y, z), (x, y, z + 1), (x, y, z - 1),
                   (x, y + 1, z), (x, y - 1, z)):
            if nb in self.strong and self.strong[nb] not in ok:
                return False
        # a repeater/comparator drives the cell it faces and reads the cell
        # behind it -- neither is in nets.py's dust-only model, so new dust
        # on either axis cell is an invisible short (an annealed seg7 laid a
        # ladder entry in a foreign repeater's muzzle: injected 15 AND bent
        # the entry's line off the base block)
        for dx, dz in H_MOVES_XZ:
            nb = (x + dx, y, z + dz)
            cell = self.b.cells.get(nb)
            if cell and ("repeater" in cell or "comparator" in cell):
                m = _FACING.search(cell)
                if m:
                    vx, vz = {"north": (0, -1), "south": (0, 1),
                              "east": (1, 0), "west": (-1, 0)}[m.group(1)]
                    if p in ((nb[0] + vx, y, nb[2] + vz),
                             (nb[0] - vx, y, nb[2] - vz)):
                        return False
        sup = (x, y - 1, z)
        s = self.b.cells.get(sup)
        if s is not None and not self.b.solid_at(*sup) and not _mt.sturdy(s):
            return False        # support cell blocked (probed: transparent
            #                     and slab-top supports carry dust legally;
            #                     the router still PLACES only solid supports.
            #                     TODO: exploit glass supports/diode-aware
            #                     costs for denser routing)
        if s is None:
            below = (x, y - 2, z)
            if "redstone_wire" in self.b.cells.get(below, ""):
                # Capping dust only breaks it if it is USING a diagonal --
                # a flat-run dust caps harmlessly, which is what makes y+1
                # bridges over existing lanes legal.
                for q in nets.neighbours(self.b.cells, below):
                    if q[1] != y - 2:
                        return False
        # electrical clearance: simulate the cell and ask who it touches
        self.b.cells[p] = rs.DUST
        try:
            for q in nets.neighbours(self.b.cells, p):
                # exact-label match only: "sig#13" (a pre-gate collector) is a
                # DIFFERENT electrical net from "sig", touching it is a short
                if self.labels.get(q) is not None and self.labels.get(q) not in ok:
                    return False
            return True
        finally:
            del self.b.cells[p]

    def move_ok(self, p, q, label, friendly=None):
        """Subclass hook: may the path step p -> q?  Base router: always.

        CountingRouter (genlib fabric) uses this to ban own-net GRAZING:
        a branch touching its own trunk on both sides of a repeater closes
        a self-sustaining ring (the FA seam latch, rediscovered at fabric
        scale: a d[1] corridor ring stayed lit after the lever went off)."""
        return True

    def col_free(self, x, z, y0, y1):
        return all((x, y, z) not in self.b.cells for y in range(y0, y1 + 1))

    def climb_entry_ok(self, prev, entry, base):
        """May `entry` serve as a torch ladder's entry dust?

        The entry's SHAPE is load-bearing: it must be a FRESH dead end whose
        single connection is the path predecessor, so its line points into
        the ladder's base block and weak-powers it.

        Two ways this fails, both found in-sim:
          * reusing an EXISTING own-net dust cell -- `dust_ok` legally allows
            own-net reuse, but an already-wired cell may carry other
            connections.  An annealed seg7 lost seg[0] here: the entry
            landed on a port-corridor dust cell that already had the
            corridor's repeater on its west face, so the entry rendered as a
            corner and never fired the ladder.
          * any connectable neighbour on a PERPENDICULAR face (dust,
            repeater, comparator, torch, lever -- own net included) bends
            the line off the base.  Same physics as `station_ok`'s trunk
            check: only solids and air may flank it.
        """
        if entry in self.b.cells:
            return False
        ex, ey, ez = entry
        for dx, dz in H_MOVES_XZ:
            q = (ex + dx, ey, ez + dz)
            if q == prev or q == base:
                continue                    # on-axis: keeps the line
            cell = self.b.cells.get(q)
            if cell is not None and not self.b.solid_at(*q):
                return False
            for dy in (-1, 1):               # up/down diagonals connect too
                if "redstone_wire" in self.b.cells.get((q[0], ey + dy, q[2]), ""):
                    return False
        return True

    def ladder_clear(self, x, z, y0, label, friendly=None):
        """May a torch ladder occupy column (x, *, z) from base y0?

        The ladder's torches power every horizontally adjacent dust and its
        interior blocks are STRONG (that is how it climbs), so a ladder may
        not stand beside a foreign dust, repeater or comparator -- in either
        build order (emit marks its cells strong for LATER routes; this
        guards against EXISTING neighbours)."""
        ok = (friendly or set()) | {label}
        for y in range(y0, y0 + 6):
            for dx, dz in H_MOVES_XZ:
                q = (x + dx, y, z + dz)
                cell = self.b.cells.get(q)
                if cell is None or self.b.solid_at(*q):
                    continue
                if "redstone_wire" in cell and self.labels.get(q) in ok:
                    continue            # own net: a 15 re-emit is harmless
                if ("redstone_wire" in cell or "repeater" in cell
                        or "comparator" in cell or "torch" in cell
                        or "lever" in cell):
                    return False
        return True

    # -- search -------------------------------------------------------------
    def find(self, src, dst, label, friendly=None, max_iter=1200000):
        friendly = (friendly or set()) | {label}
        # route-to-net: dst may be one cell or a set -- reaching ANY cell of
        # the target net completes the route (multi-terminal joining).
        # route-FROM-net: src may likewise be a set of cells that are already
        # electrically one net (a driver trunk); the search starts from all
        # of them at g=0, so a branch may leave the trunk anywhere.  Emission
        # is direction-correct because the path runs trunk -> sink.
        dsts = {dst} if isinstance(dst, tuple) else set(dst)
        srcs = [src] if isinstance(src, tuple) else list(src)
        self._dsts = dsts           # for move_ok subclass hooks

        def h(p):
            # weighted A* (eps=1.3): slightly suboptimal paths, much less
            # exploration -- the state space carries a stair counter now
            return 1.3 * min(abs(p[0] - q[0]) + abs(p[2] - q[2]) + abs(p[1] - q[1])
                             for q in dsts)

        # state = (cell, consecutive-stair count, previous stair direction).
        # Stairs cannot carry repeaters (chain capped at 4), and a stair may
        # not exactly REVERSE the previous stair: a switchback's support block
        # lands on the cell that cuts the previous diagonal.
        openq, came, gbest = [], {}, {}
        for s0 in srcs:
            start = (s0, 0, None)
            openq.append((h(s0), 0, start))
            came[start] = (None, None)
            gbest[start] = 0
        heapq.heapify(openq)
        it = 0
        while openq:
            it += 1
            if it > max_iter:
                break
            _, g, s = heapq.heappop(openq)
            p, d, pdir = s
            if p in dsts:
                path = []
                while s is not None:
                    prev, mv = came[s]
                    path.append((s[0], mv))
                    s = prev
                return list(reversed(path))
            if g > gbest.get(s, 1e18):
                continue
            x, y, z = p
            cand = []
            for dx, dz in ((m[0], m[2]) for m in H_MOVES):
                cand.append(((x + dx, y, z + dz), 1, "h"))
                if d < 4:
                    cand.append(((x + dx, y + 1, z + dz), 3, "up"))
                    cand.append(((x + dx, y - 1, z + dz), 3, "down"))
            # torch-ladder climb.  The climb lays its OWN entry cell one step
            # ahead: a fresh dead-end dust with a single neighbour behind it is
            # always a straight line pointing into the base block.  Using the
            # current path cell as entry does NOT work -- its shape may run
            # perpendicular (a lane cell), and dust only powers blocks it
            # points into.
            for dx, dz in ((m[0], m[2]) for m in H_MOVES):
                exit_p = (x + 2 * dx, y + 5, z + 2 * dz)
                cand.append((exit_p, 9, ("climb", dx, dz)))
            for q, cost, mv in cand:
                if not self.in_bounds(q):
                    continue
                if not self.dust_ok(q, label, friendly):
                    continue
                if not self.move_ok(p, q, label, friendly):
                    continue
                if mv == "up" and self.b.solid_at(x, y + 1, z):
                    continue        # solid corner above the lower dust cuts the diagonal
                if mv == "down" and self.b.solid_at(q[0], y, q[2]):
                    continue        # same rule, descending
                # NOTE (probed diode, probe_materials.py): a step whose upper
                # dust sits on a TRANSPARENT support conducts up only.  The
                # router never places transparent supports and today's builds
                # never route over them, so descent stays legal; when glass
                # exploitation lands, "down" moves must check the current
                # cell's support for conductivity (a naive guard here changes
                # A* paths in ways the emitter was never verified for).
                if isinstance(mv, tuple):
                    dx, dz = mv[1], mv[2]
                    entry = (x + dx, y, z + dz)
                    if not (self.dust_ok(entry, label, friendly)
                            and self.move_ok(p, entry, label, friendly)
                            and self.climb_entry_ok(p, entry, (q[0], y, q[2]))
                            and self.col_free(q[0], q[2], y, y + 4)
                            and self.ladder_clear(q[0], q[2], y, label,
                                                  friendly)):
                        continue    # entry cell + ladder column must be free
                stair = mv in ("up", "down")
                if stair and pdir is not None:
                    if (q[0] - x, q[2] - z) == (-pdir[0], -pdir[1]):
                        continue                  # switchback cuts own diagonal
                nd = d + 1 if stair else 0
                nq = (q, nd, (q[0] - x, q[2] - z) if stair else None)
                ng = g + cost
                if q in self.soft:
                    slab, spen = self.soft[q]
                    if slab not in friendly:
                        ng += spen
                if ng < gbest.get(nq, 1e18):
                    gbest[nq] = ng
                    came[nq] = (s, mv)
                    heapq.heappush(openq, (ng + h(q), ng, nq))
        return None

    # -- emission -----------------------------------------------------------
    # Max-pitch refresh, probe-verified (probe_station A15/I15): dust arriving
    # at ss1 still fires a repeater or a station's entry block, so the true
    # max is 15 dust cells between refreshes (REFRESH=16).  Leave 2 levels of
    # margin: the last dust before a refresh sits at ss3.  (Was 5, a
    # debugging-era safety margin.)  Routes are tap-free, so unlike rails
    # (build_ppa.RAIL_REPEAT) there is no ss>=2 tap floor to respect.
    REFRESH = 14
    # A path's SOURCE strength is unknown (a cell port may arrive already
    # decayed), so the first refresh keeps the old conservative spacing; only
    # refresh-to-refresh spans are trusted at full pitch.
    FIRST = 6
    # `strong` sentinel that equals no net label: claims a cell against
    # EVERY net, own included.
    LADDER_ENTRY = "#ladder_entry"

    def station_ok(self, path, i, label, friendly=None):
        """May a block-sandwich station (block/repeater/block) replace path
        cells i, i+1, i+2?  Probe-verified constraints (probe_station S*):
          * the trunk dust, both blocks, the repeater and the next dust must
            be collinear at one y -- dust only weak-powers blocks it points
            into, and the exit block must strong-power the following dust;
          * the exit block re-emits at 15 to EVERY adjacent dust, and any
            dust atop either block joins the net diagonally -- so no foreign
            component may touch the station body anywhere but from below.
        """
        if i < 1 or i + 3 >= len(path):
            return False
        seg = path[i - 1:i + 4]
        if any(isinstance(mv, tuple) for _p, mv in seg[1:]):
            return False
        pts = [p for p, _mv in seg]
        y = pts[0][1]
        if any(p[1] != y for p in pts):
            return False
        if not (all(p[0] == pts[0][0] for p in pts[:4])
                or all(p[2] == pts[0][2] for p in pts[:4])):
            return False
        # The trunk dust must stay a STRAIGHT LINE pointing into the entry
        # block -- dust only weak-powers blocks along its line.  Any other
        # connectable neighbour (dust, repeater, comparator, torch -- own net
        # included: dust_ok legally allows own-net grazing, and a port stub's
        # repeater is exactly what broke the FA cell here) bends its shape
        # off the block face and the station goes dead.  Same physics as the
        # climb's dead-end entry dust: only solids may flank the trunk.
        x0, y0, z0 = pts[0]
        ax = (pts[1][0] - x0, pts[1][2] - z0)
        for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if (dx, dz) in (ax, (-ax[0], -ax[1])):
                continue                        # on-axis keeps the line
            for dy in (-1, 0, 1):
                q = (x0 + dx, y0 + dy, z0 + dz)
                if q in self.b.cells and not self.b.solid_at(*q):
                    return False
        ok = (friendly or set()) | {label}
        body = set(pts[1:4])
        for (x, _y, z) in body:
            if (x, y, z) in self.b.cells:
                return False              # never overwrite (trunk reuse etc.)
            for nb in ((x + 1, y, z), (x - 1, y, z), (x, y, z + 1),
                       (x, y, z - 1), (x, y + 1, z)):
                if nb in body or nb == pts[0] or nb == pts[4]:
                    continue
                cell = self.b.cells.get(nb)
                if cell is None or self.b.solid_at(*nb):
                    continue
                if "redstone_wire" in cell and self.labels.get(nb) in ok:
                    continue              # own net: a 15 re-emit is harmless
                return False              # foreign dust/repeater/torch/lever
        return True

    def emit(self, path, label, friendly=None):
        """Lay the routed path into the build."""
        since, self.stations = self.REFRESH - self.FIRST, 0
        i = 0
        while i < len(path):
            p, mv = path[i]
            x, y, z = p
            if isinstance(mv, tuple) and mv[0] == "climb":
                px, py, pz = path[i - 1][0]
                dx, dz = mv[1], mv[2]
                # fresh dead-end entry dust: single neighbour behind -> straight
                # line into the base, so the base is reliably weak-powered
                ex, ez = px + dx, pz + dz
                self.b.stone(ex, py - 1, ez, "route")
                self.labels[(ex, py, ez)] = label
                self.b.put(ex, py, ez, rs.DUST)
                self.ss[(ex, py, ez)] = max(1, 15 - since - 1)
                # The entry's SHAPE is load-bearing: it must stay a dead end
                # on the climb axis so its line points into the base block.
                # Anything connectable landing on a perpendicular face bends
                # it into a corner and the ladder never fires -- own net
                # INCLUDED (an annealed seg7 lost seg[0] when a later route
                # of the SAME net put a repeater in the entry's west face).
                # A never-matching sentinel claims the entry's whole
                # neighbourhood, so no cell of any net may be placed beside
                # it after the fact.
                self.strong[(ex, py, ez)] = self.LADDER_ENTRY
                # verified template: base, torch, block, torch, cap, exit dust
                for k in range(3):
                    self.b.stone(x, py + 2 * k, z, "route")
                    if k < 2:
                        self.b.put(x, py + 2 * k + 1, z, rs.TORCH)
                        # a ladder torch powers any adjacent dust, and the
                        # block above it is STRONG-powered (that is how the
                        # ladder climbs) -- both inject 15 into any foreign
                        # dust that later snuggles up (genlib seg7: a d[0]
                        # flyover read 15 off a neighbouring ladder block)
                        self.strong[(x, py + 2 * k + 1, z)] = label
                        self.strong[(x, py + 2 * k + 2, z)] = label
                self.labels[(x, y, z)] = label
                self.b.put(x, y, z, rs.DUST)
                self.ss[(x, y, z)] = 15         # climb cap re-emits fresh
                since = 0
                i += 1
                continue
            if "redstone_wire" in self.b.cells.get(p, ""):
                # reused own-net trunk: resume the budget from this cell's
                # recorded strength (unknown cells assume worst case, which
                # forces a refresh at the first straight cell after leaving)
                since = 15 - self.ss.get(p, 1)
                i += 1
                continue
            prev = path[i - 1][0] if i else None
            nxt = path[i + 1][0] if i + 1 < len(path) else None
            straight = (prev is not None and nxt is not None
                        and prev[1] == y == nxt[1]
                        and (prev[0] == x == nxt[0] or prev[2] == z == nxt[2])
                        and not isinstance(path[i + 1][1], tuple))
            # bank a refresh at the last straight cell before a stair/climb
            # tail: the tail cannot host repeaters and must not start a long
            # descent on a nearly-spent budget
            nxt2 = path[i + 2][0] if i + 2 < len(path) else None
            cont = (nxt2 is not None and nxt[1] == y == nxt2[1]
                    and (nxt[0] == x == nxt2[0] or nxt[2] == z == nxt2[2])
                    and not isinstance(path[i + 2][1], tuple))
            since += 1
            if straight and since >= (self.REFRESH if cont else self.REFRESH - 6):
                d = {(-1, 0): "west", (1, 0): "east",
                     (0, -1): "north", (0, 1): "south"}[(prev[0] - x, prev[2] - z)]
                if self.station_ok(path, i, label, friendly):
                    # block sandwich: the blocks conduct for free, so the
                    # station spans 3 cells but restarts the budget at 15
                    # (probe_station B) -- 18 cells of reach per repeater
                    # instead of 16, and no support needed under the blocks
                    p2, p3 = path[i + 1][0], path[i + 2][0]
                    self.b.put(x, y, z, rs.PALETTE["route"])
                    self.b.stone(p2[0], p2[1] - 1, p2[2], "route")
                    self.b.put(p2[0], p2[1], p2[2], rs.repeater(d))
                    self.b.put(p3[0], p3[1], p3[2], rs.PALETTE["route"])
                    self.strong[p3] = label
                    # the ENTRY block is an injection port: any later foreign
                    # dust pointing into it fires the station's repeater and
                    # writes a hard 15 onto this net (seen in genlib seg7:
                    # a dead trunk went 0 -> 15 across a station).  Claim its
                    # neighbourhood exactly like the exit's.
                    self.strong[p] = label
                    self.stations += 1
                    since = 0
                    i += 3
                    continue
                self.b.stone(x, y - 1, z, "route")
                self.b.put(x, y, z, rs.repeater(d))
                since = 0
            else:
                self.b.stone(x, y - 1, z, "route")
                self.labels[p] = label
                self.b.put(x, y, z, rs.DUST)
                self.ss[p] = max(1, 15 - since)
            i += 1

    def route(self, src, dst, label, friendly=None):
        path = self.find(src, dst, label, friendly)
        if path is None:
            raise RuntimeError("router: no path for %s: %s -> %s" % (label, src, dst))
        self.emit(path, label, friendly)
        return len(path)
