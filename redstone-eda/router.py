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

import rs
import nets

H_MOVES = ((1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1))


class Router:
    def __init__(self, build, labels, bounds=None):
        self.b = build
        self.labels = labels
        self.bounds = bounds        # (x0,x1, y0,y1, z0,z1): cells must not
                                    # route outside their own footprint

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
        sup = (x, y - 1, z)
        s = self.b.cells.get(sup)
        if s is not None and not self.b.solid_at(*sup):
            return False                            # support cell blocked
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

    def col_free(self, x, z, y0, y1):
        return all((x, y, z) not in self.b.cells for y in range(y0, y1 + 1))

    # -- search -------------------------------------------------------------
    def find(self, src, dst, label, friendly=None, max_iter=1200000):
        friendly = (friendly or set()) | {label}
        # route-to-net: dst may be one cell or a set -- reaching ANY cell of
        # the target net completes the route (multi-terminal joining)
        dsts = {dst} if isinstance(dst, tuple) else set(dst)

        def h(p):
            # weighted A* (eps=1.3): slightly suboptimal paths, much less
            # exploration -- the state space carries a stair counter now
            return 1.3 * min(abs(p[0] - q[0]) + abs(p[2] - q[2]) + abs(p[1] - q[1])
                             for q in dsts)

        # state = (cell, consecutive-stair count, previous stair direction).
        # Stairs cannot carry repeaters (chain capped at 4), and a stair may
        # not exactly REVERSE the previous stair: a switchback's support block
        # lands on the cell that cuts the previous diagonal.
        start = (src, 0, None)
        openq = [(h(src), 0, start)]
        came, gbest = {start: (None, None)}, {start: 0}
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
                if mv == "up" and self.b.solid_at(x, y + 1, z):
                    continue        # solid corner above the lower dust cuts the diagonal
                if mv == "down" and self.b.solid_at(q[0], y, q[2]):
                    continue        # same rule, descending
                if isinstance(mv, tuple):
                    dx, dz = mv[1], mv[2]
                    entry = (x + dx, y, z + dz)
                    if not (self.dust_ok(entry, label, friendly)
                            and self.col_free(q[0], q[2], y, y + 4)):
                        continue    # entry cell + ladder column must be free
                stair = mv in ("up", "down")
                if stair and pdir is not None:
                    if (q[0] - x, q[2] - z) == (-pdir[0], -pdir[1]):
                        continue                  # switchback cuts own diagonal
                nd = d + 1 if stair else 0
                nq = (q, nd, (q[0] - x, q[2] - z) if stair else None)
                ng = g + cost
                if ng < gbest.get(nq, 1e18):
                    gbest[nq] = ng
                    came[nq] = (s, mv)
                    heapq.heappush(openq, (ng + h(q), ng, nq))
        return None

    # -- emission -----------------------------------------------------------
    REFRESH = 5

    def emit(self, path, label):
        """Lay the routed path into the build."""
        since = 0
        for i, (p, mv) in enumerate(path):
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
                # verified template: base, torch, block, torch, cap, exit dust
                for k in range(3):
                    self.b.stone(x, py + 2 * k, z, "route")
                    if k < 2:
                        self.b.put(x, py + 2 * k + 1, z, rs.TORCH)
                self.labels[(x, y, z)] = label
                self.b.put(x, y, z, rs.DUST)
                since = 0
                continue
            if "redstone_wire" in self.b.cells.get(p, ""):
                since = self.REFRESH                    # reused own-net trunk
                continue
            self.b.stone(x, y - 1, z, "route")
            prev = path[i - 1][0] if i else None
            nxt = path[i + 1][0] if i + 1 < len(path) else None
            straight = (prev is not None and nxt is not None
                        and prev[1] == y == nxt[1]
                        and (prev[0] == x == nxt[0] or prev[2] == z == nxt[2])
                        and not isinstance(path[i + 1][1], tuple))
            since += 1
            if straight and since >= self.REFRESH:
                d = {(-1, 0): "west", (1, 0): "east",
                     (0, -1): "north", (0, 1): "south"}[(prev[0] - x, prev[2] - z)]
                self.b.put(x, y, z, rs.repeater(d))
                since = 0
            else:
                self.labels[p] = label
                self.b.put(x, y, z, rs.DUST)

    def route(self, src, dst, label, friendly=None):
        path = self.find(src, dst, label, friendly)
        if path is None:
            raise RuntimeError("router: no path for %s: %s -> %s" % (label, src, dst))
        self.emit(path, label)
        return len(path)
