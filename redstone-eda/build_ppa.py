"""A 32-bit Kogge-Stone parallel-prefix adder in redstone.

Same verified primitive as the ripple-carry build: a lidded rail, a dust spur that
dead-ends into a block, a torch on top injecting NOT(rail) into a collector, and a
torch on the collector's dead-end inverting the OR -- so one column computes
AND(its rails).  Everything here is that column, repeated.

    stage 0   p_i = A_i^B_i  (2 columns),  g_i = A_i&B_i  (1 column)
    stage k   Kogge-Stone prefix, stride s = 2^(k-1):
                G_i = G_i + P_i&G_{i-s}    (2 columns, OR-ed)
                P_i = P_i&P_{i-s}          (1 column)
              every term is a POSITIVE literal, so prefix stages need no
              complement rails at all -- only stage 0 and the sum stage do.
    carry     c_i = G_{i-1} + P_{i-1}&cin  (2 columns),  c_0 = cin
    sum       s_i = p_i ^ c_i              (2 columns)

Layout: X is the bit slice, Z is the stage band, Y is detail (rails y=0..2,
collector y=3..4, gate y=5..6, output lane y=7).  Rails get packed into shared Z
slots by interval-colouring their X spans, which is what keeps a 32-bit build
inside the engine's 8M-cell volume cap.
"""
import argparse
import itertools
import random
import rs
from rs import DUST, LEVER_OFF, STONE, repeater

# ---- slice geometry (offsets from a slice's x0) --------------------------
SLICE_W = 17
CH = (0, 2, 4)        # rail west end / wiring channel, one per outgoing route
WIRE_DZ = (0, 2, 4)   # each route's own Z lane leaving the output lane
INV_X = 6             # inverter spur; stairs at 7,8,9; its output rail starts at 9
INV_RAIL_X = 9
COL_A = (11, 13)      # group A columns (OR-ed onto one lane)
COL_B = (15,)         # group B column
EXTRA_COLS = ()       # further gi=0 columns (the ALU's 4-term mux nodes)
RAIL_E = 15           # every rail must reach the last column
Y_COLL, Y_GATE, Y_LANE = 4, 5, 7
RAIL_REPEAT = 12      # a rail loses a level per block; refresh at least this often
STAGE_PAD = 1


def sx(i):
    return i * SLICE_W


def rail_repeater_xs(x0, x1, drive):
    """Where to refresh a rail.  The FIRST one sits right at the drive point: a
    route arrives having decayed on the way, so without it the rail starts at
    whatever trickle survived the journey and dies before the first column."""
    bad = (INV_X,) + COL_A + COL_B + EXTRA_COLS   # never on a tap or the inverter
    out, want = set(), drive + 1
    while want <= x1:
        x = want
        while x <= x1 and (x % SLICE_W) in bad:
            x += 1
        if x > x1 or x <= x0:
            break
        out.add(x)
        want = x + RAIL_REPEAT
    return out


# ---- netlist --------------------------------------------------------------
def netlist(N):
    """Stages of (slice, [(output signal, [product term, ...]), ...])."""
    L = N.bit_length() - 1
    stages = []

    nodes = [(i, [("p%d" % i, [["A%d" % i, "nB%d" % i], ["nA%d" % i, "B%d" % i]]),
                  ("g%d" % i, [["A%d" % i, "B%d" % i]])]) for i in range(N)]
    stages.append(dict(name="pg", nodes=nodes, local=True,
                       inverters=[(i, "A%d" % i, "nA%d" % i) for i in range(N)]
                                 + [(i, "B%d" % i, "nB%d" % i) for i in range(N)],
                       levers=[(i, "A%d" % i, 0) for i in range(N)]
                              + [(i, "B%d" % i, 1) for i in range(N)]))

    gp = lambda k, i: ("g%d" % i, "p%d" % i) if k == 0 else ("G%d_%d" % (k, i), "P%d_%d" % (k, i))
    for k in range(1, L + 1):
        s = 1 << (k - 1)
        nodes = []
        for i in range(N):
            gh, ph = gp(k - 1, i)
            go, po = gp(k, i)
            if i >= s:
                gl, pl = gp(k - 1, i - s)
                nodes.append((i, [(go, [[gh], [ph, gl]]), (po, [[ph, pl]])]))
            else:                                    # nothing to merge: buffer
                nodes.append((i, [(go, [[gh]]), (po, [[ph]])]))
        stages.append(dict(name="prefix%d" % k, nodes=nodes, local=False,
                           inverters=[], levers=[], stride=s, width=N))

    gN, pN = gp(L, 0)[0], gp(L, 0)[1]
    nodes = [(0, [("c0", [["cin"]])])]
    for i in range(1, N + 1):                        # slice N carries cout
        g, p = gp(L, i - 1)
        out = "cout" if i == N else "c%d" % i
        nodes.append((i, [(out, [[g], [p, "cin"]])]))
    stages.append(dict(name="carry", nodes=nodes, local=False, inverters=[],
                       levers=[(0, "cin", 2)], lever_span=N + 1))

    nodes = [(i, [("s%d" % i, [["p%d" % i, "nc%d" % i], ["np%d" % i, "c%d" % i]])])
             for i in range(N)]
    stages.append(dict(name="sum", nodes=nodes, local=True,
                       inverters=[(i, "p%d" % i, "np%d" % i) for i in range(N)]
                                 + [(i, "c%d" % i, "nc%d" % i) for i in range(N)],
                       levers=[]))
    return stages


def plan(N, make=None):
    """Work out, for every signal: who makes it, who taps it, and its rail slot."""
    stages = (make or netlist)(N)
    produced = {}                     # signal -> (stage, slice, group index)
    for si, st in enumerate(stages):
        for sl, groups in st["nodes"]:
            for gi, (out, _terms) in enumerate(groups):
                produced[out] = (si, sl, gi)

    for si, st in enumerate(stages):
        taps = {}                     # signal -> set of consumer slices
        for sl, groups in st["nodes"]:
            for _out, terms in groups:
                for term in terms:
                    for sig in term:
                        taps.setdefault(sig, set()).add(sl)
        for sl, src, dst in st["inverters"]:
            taps.setdefault(src, set()).add(sl)
            taps.setdefault(dst, set()).add(sl)
        st["taps"] = taps

        # rail X span: from its drive point to the last slice that taps it
        inv_dst = {d: sl for sl, _s, d in st["inverters"]}
        inv_src = {s for _sl, s, _d in st["inverters"]}
        spans, drive = {}, {}
        for sig, slices in taps.items():
            hi = max(slices)
            if sig in inv_dst:                       # made in-stage by a torch
                drive[sig] = sx(inv_dst[sig]) + INV_RAIL_X
            elif sig in produced and produced[sig][0] < si:
                _, psl, _ = produced[sig]
                drive[sig] = sx(psl) + CH[route_channel(stages, sig, si)]
            else:
                # lever-driven, or "ext": an input rail a router will later
                # drive by landing a wire on its west-end dust cell
                lsl, lch = [(sl, c) for sl, s, c in
                            list(st["levers"]) + list(st.get("ext", []))
                            if s == sig][0]
                drive[sig] = sx(lsl) + CH[lch]
            west = drive[sig]
            if sig in inv_src:                       # must reach the inverter spur
                west = min(west, sx(min(slices)) + INV_X)
            spans[sig] = (west, sx(hi) + RAIL_E)
        st["spans"], st["drive"] = spans, drive
        st["slot"] = assign_slots(st, spans, N)
    return stages, produced


def route_channel(stages, sig, dst_stage):
    """Which of a slice's channels this signal uses to reach dst_stage."""
    src = None
    for si, st in enumerate(stages):
        for sl, groups in st["nodes"]:
            for gi, (out, _t) in enumerate(groups):
                if out == sig:
                    src = (si, gi)
    if src is None:
        return 2
    # Channels are per-slice resources: two different signals may not share a
    # corridor with overlapping z-ranges.  Next-stage hops recycle channels 0/1
    # (their z-ranges are sequential and disjoint); far hops get an exclusive
    # channel per group index (2 for gi=0, 3 for gi=1), because far corridors
    # span many bands and would collide with everything.
    return src[1] if dst_stage == src[0] + 1 else 2 + src[1]


def assign_slots(st, spans, N):
    """Give every rail a Z slot, letting rails with disjoint X spans share one."""
    if st["local"]:
        # stage 0 / sum: every rail is slice-local, so all slices reuse the same
        # four slots.  Order them X, ~X so an inverter's output rail always sits
        # exactly one slot above its input -- which is what the stair expects.
        slot, per = {}, {}
        for sl, src, dst in st["inverters"]:
            per.setdefault(sl, []).append((src, dst))
        for sl, pairs in per.items():
            for j, (src, dst) in enumerate(pairs):
                slot[src], slot[dst] = 2 * j, 2 * j + 1
        # A "local" stage may still carry rails that are not inverter pairs
        # (the ALU's op rails, a lever-driven cin).  Interval-colour those into
        # slots above the pinned pairs.
        leftovers = {s: spans[s] for s in spans if s not in slot}
        if leftovers:
            base = max(slot.values()) + 1 if slot else 0
            extra = _interval_colour(leftovers)
            for s, j in extra.items():
                slot[s] = base + j
        return slot

    if st.get("stride"):
        # Placement beats routing.  Slotting rails by producer index puts a
        # node's two inputs (producers i and i-s) 6s apart in Z, so its collector
        # has to stretch ~6s cells of dust to reach both -- ~100 cells at stride
        # 16, nearly all of it empty span.  Order instead so that i and i-s land
        # in ADJACENT slots, and every node's four taps sit inside four slots.
        # Assign slot by producer index mod (s+1).  Two things fall out at once:
        #   * (i-s) mod (s+1) == (i+1) mod (s+1), so a node's two producers land
        #     in ADJACENT slot groups -- its four taps span 4 slots, not 6s.
        #   * producers s+1 apart share a track, and their X spans are disjoint
        #     by construction, so the stage still packs into 2(s+1) slots.
        s = st["stride"]

        def key(sig):
            # trailing digits = producer bit index; works with any name prefix
            import re
            j = int(re.search(r"(\d+)$", sig).group(1))
            return 2 * (j % (s + 1)) + (0 if ("g" in sig or "G" in sig) else 1)

        return {sig: key(sig) for sig in spans}

    return _interval_colour(spans)


def _interval_colour(spans):
    """Left-edge channel assignment: rails whose X spans are 4+ apart share a slot."""
    GAP = 4
    slot, ends = {}, []
    for sig in sorted(spans, key=lambda s: (spans[s][0], s)):
        x0, x1 = spans[sig]
        for j, end in enumerate(ends):
            if x0 > end + GAP:
                slot[sig] = j
                ends[j] = x1
                break
        else:
            slot[sig] = len(ends)
            ends.append(x1)
    return slot


# ---- geometry -------------------------------------------------------------
def layout(N, make=None):
    """Attach Z offsets to every node group, then place the stage bands."""
    stages, produced = plan(N, make)
    for st in stages:
        st["geom"] = {}
        for sl, groups in st["nodes"]:
            for gi, (out, terms) in enumerate(groups):
                taps = [3 * st["slot"][s] + 2 for term in terms for s in term]
                gate = max(taps) + 2
                st["geom"][out] = dict(slice=sl, gi=gi, terms=terms,
                                       gate=gate, lane=gate + 1)

    routes = []
    for si, st in enumerate(stages):
        for sig in st["taps"]:
            if sig in produced and produced[sig][0] < si:
                routes.append(dict(sig=sig, src=produced[sig][0], dst=si,
                                   slice=produced[sig][1],
                                   ch=route_channel(stages, sig, si)))

    # Every route leaving a slice gets its OWN Z corridor at y=7, so no route's
    # support block can ever land on another route's wire.  Corridors are handed
    # out west-to-east by riser column: a run heading west then only ever passes
    # risers that finished climbing at a LOWER z than the run itself.
    # The riser must start ON the producing node's own lane: a gi=0 lane spans
    # COL_A, a gi=1 lane only COL_B -- so the riser column follows the group,
    # not just the channel.  Same-signal routes to different stages share the
    # same riser x deliberately: the overlap becomes a common trunk (a free
    # Steiner point) and run() treats re-laying the same dust as a no-op.
    per_slice = {}
    for r in routes:
        gi = produced[r["sig"]][2]
        nterms = len(stages[r["src"]]["geom"][r["sig"]]["terms"])
        # last actual column of the producing node == the east end of its lane
        r["riser"] = COL_B[0] if gi else (COL_A + COL_B + EXTRA_COLS)[nterms - 1]
        per_slice.setdefault((r["src"], r["slice"]), []).append(r)
    for (si, sl), rs_ in per_slice.items():
        st = stages[si]
        base = max(g["lane"] for g in st["geom"].values() if g["slice"] == sl) + 2
        for idx, r in enumerate(sorted(rs_, key=lambda r: r["riser"])):
            r["t"] = base + 4 * idx

    z = [0]
    for k in range(len(stages) - 1):
        st = stages[k]
        ext = max([3 * s for s in st["slot"].values()] + [g["lane"] for g in st["geom"].values()]
                  + [r["t"] for r in routes if r["src"] == k])
        need = max([r["t"] + 8 - 3 * stages[r["dst"]]["slot"][r["sig"]]
                    for r in routes if r["src"] == k and r["dst"] == k + 1] or [0])
        z.append(z[k] + max(ext, need) + STAGE_PAD)
    for st, zb in zip(stages, z):
        st["z"] = zb
    for r in routes:                                 # the plan must be physical
        wz = stages[r["src"]]["z"] + r["t"]
        dz = stages[r["dst"]]["z"] + 3 * stages[r["dst"]]["slot"][r["sig"]]
        assert dz >= wz + 8, "no room to route %s: %d -> %d" % (r["sig"], wz, dz)
    return stages, produced, routes


class PPA:
    def __init__(self, N, make=None, name=None):
        self.N = N
        self.b = rs.Build(name or "kogge_stone_%dbit" % N)
        self.labels = {}
        self.rail_dust = set()
        self.stages, self.produced, self.routes = layout(N, make)
        self.levers = []
        self.probe = {}

    def dust(self, x, y, z, label):
        self.b.put(x, y, z, DUST)
        self.labels[(x, y, z)] = label

    def rail_z(self, st, sig):
        return st["z"] + 3 * st["slot"][sig]

    # -- rails -------------------------------------------------------------
    def build_rails(self):
        for st in self.stages:
            for sig, (x0, x1) in st["spans"].items():
                z = self.rail_z(st, sig)
                drive = st["drive"][sig]
                reps = rail_repeater_xs(x0, x1, drive)
                for x in range(x0, x1 + 1):
                    self.b.stone(x, 0, z, "rail_floor")
                    self.rail_dust.add((x, z))                  # lid decided later
                    if x in reps:
                        self.b.put(x, 1, z, repeater("west"))
                    else:
                        self.dust(x, 1, z, sig)
                # read a DUST cell -- a repeater has no power= property and would
                # silently read as 0 forever
                px = next((x for x in range(drive + 1, x1 + 1) if x not in reps), drive)
                self.probe[sig] = (px, 1, z)
            for sl, sig, ch in st["levers"]:
                p = (sx(sl) + CH[ch], 1, self.rail_z(st, sig))
                self.b.force(*p, LEVER_OFF)
                self.labels.pop(p, None)
                self.levers.append((sig, p))

    # -- inverter: spur off a rail, torch, stair down onto the rail above ----
    def build_inverters(self):
        for st in self.stages:
            for sl, src, dst in st["inverters"]:
                x, zs = sx(sl) + INV_X, self.rail_z(st, src)
                assert self.rail_z(st, dst) == zs + 3
                self.b.stone(x, 0, zs + 1, "inv"); self.b.stone(x, 0, zs + 2, "inv")
                self.dust(x, 1, zs + 1, src)
                self.b.stone(x, 1, zs + 2)
                self.b.put(x, 2, zs + 2, rs.TORCH)
                self.b.stone(x, 3, zs + 2, "inv")
                self.dust(x, 4, zs + 2, dst)
                for k, y in ((1, 3), (2, 2), (3, 1)):
                    self.b.stone(x + k, y - 1, zs + 2, "inv")
                    self.dust(x + k, y, zs + 2, dst)

    # -- one product term: taps + collector + inverting gate ----------------
    def build_column(self, st, x, term, gate_off, label):
        zb = st["z"]
        for sig in term:
            rz = self.rail_z(st, sig)
            self.b.stone(x, 0, rz + 1, "tap"); self.b.stone(x, 0, rz + 2, "tap")
            self.dust(x, 1, rz + 1, sig)
            self.b.stone(x, 1, rz + 2, "tap")
            self.b.put(x, 2, rz + 2, rs.TORCH)
        lo = min(3 * st["slot"][s] + 2 for s in term)
        since = 0
        for off in range(lo, gate_off):
            self.b.stone(x, 3, zb + off, "coll_floor")
            since += 1
            if since >= 10 and off % 3 == 0 and off > lo:
                self.b.put(x, Y_COLL, zb + off, repeater("north"))
                since = 0
            else:
                self.dust(x, Y_COLL, zb + off, label)
        self.b.stone(x, Y_COLL, zb + gate_off, "gate")
        self.b.put(x, Y_GATE, zb + gate_off, rs.TORCH)
        self.b.stone(x, 6, zb + gate_off, "gate")

    def build_nodes(self):
        for st in self.stages:
            for out, g in st["geom"].items():
                # a gi=0 group may hold up to 3 (or with EXTRA_COLS, 4) terms by
                # borrowing COL_B's column -- the slice then has one wide node
                cols = ((COL_A + COL_B + EXTRA_COLS) if g["gi"] == 0 else COL_B)[:len(g["terms"])]
                assert len(cols) == len(g["terms"]), \
                    "node %s needs %d columns, layout has %d" % (out, len(g["terms"]), len(cols))
                xs = [sx(g["slice"]) + c for c in cols]
                for x, term in zip(xs, g["terms"]):
                    self.build_column(st, x, term, g["gate"], "%s#%d" % (out, x))
                    self.dust(x, Y_LANE, st["z"] + g["gate"], out)
                for x in range(min(xs), max(xs) + 1):
                    self.b.stone(x, 6, st["z"] + g["lane"], "lane")
                    self.dust(x, Y_LANE, st["z"] + g["lane"], out)
                self.probe[out] = (min(xs), Y_LANE, st["z"] + g["lane"])

    # -- routing ------------------------------------------------------------
    def build_routes(self):
        for r in self.routes:
            sig = r["sig"]
            src, dst = self.stages[r["src"]], self.stages[r["dst"]]
            g = src["geom"][sig]
            cx = sx(g["slice"]) + CH[r["ch"]]
            rx = sx(g["slice"]) + r["riser"]
            lane_z, wz = src["z"] + g["lane"], src["z"] + r["t"]
            dz = self.rail_z(dst, sig)
            cells = [(rx, Y_LANE, zz) for zz in range(lane_z + 1, wz + 1)]
            cells += [(x, Y_LANE, wz) for x in range(rx - 1, cx - 1, -1)]
            cells += [(cx, 6, wz + 1), (cx, Y_GATE, wz + 2), (cx, Y_COLL, wz + 3)]
            cells += [(cx, Y_COLL, zz) for zz in range(wz + 4, dz - 3)]
            # Land on the rail HORIZONTALLY.  Dropping the last step diagonally
            # from y=2 cannot work: the block above the rail dust is the rail's
            # own lid, and blocking that diagonal is the entire point of the lid.
            cells += [(cx, 3, dz - 3), (cx, 2, dz - 2), (cx, 1, dz - 1), (cx, 1, dz)]
            self.run(cells, sig)

    def run(self, cells, label, repeat_every=6):
        since = 0
        for i, (x, y, z) in enumerate(cells):
            # Shared trunk: a second route of the SAME signal may re-walk cells
            # the first already laid (same riser, longer climb).  Existing dust
            # is reused as-is; `since` is primed so a repeater lands at the
            # first legal cell after the shared prefix ends, because we cannot
            # know how much signal level the trunk had left at the fork.
            if "redstone_wire" in self.b.cells.get((x, y, z), ""):
                if self.labels.get((x, y, z)) != label:
                    raise AssertionError("route %s crosses net %s at %s"
                                         % (label, self.labels.get((x, y, z)), (x, y, z)))
                since = repeat_every
                continue
            self.b.stone(x, y - 1, z, "route")
            prev = cells[i - 1] if i else None
            nxt = cells[i + 1] if i + 1 < len(cells) else None
            straight = (prev and nxt and prev[1] == y == nxt[1]
                        and (prev[0] == x == nxt[0] or prev[2] == z == nxt[2]))
            since += 1
            if straight and since >= repeat_every:
                self.b.put(x, y, z, repeater(facing_toward((x, y, z), prev)))
                since = 0
            else:
                self.dust(x, y, z, label)

    def place_lids(self):
        """A rail lid is only load-bearing where a wire could actually drop into
        the rail.  Blanketing every rail with one costs a third of the rail
        budget for nothing; place them where the connection rule says they
        matter -- any dust in the four y=2 neighbours of a rail cell."""
        n = 0
        for (x, z) in self.rail_dust:
            if (x, 2, z) in self.b.cells:
                continue
            for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nb = self.b.cells.get((x + dx, 2, z + dz), "")
                if "redstone_wire" in nb:
                    self.b.stone(x, 2, z, "lid")
                    n += 1
                    break
        return n

    def build(self):
        self.build_rails()
        self.build_inverters()
        self.build_nodes()
        self.build_routes()
        self.lids = self.place_lids()
        return self.b


def facing_toward(frm, to):
    dx, dz = to[0] - frm[0], to[2] - frm[2]
    return {(-1, 0): "west", (1, 0): "east", (0, -1): "north", (0, 1): "south"}[(dx, dz)]


# ---- reference model ------------------------------------------------------
def expect(N, A, B, cin):
    v = {"cin": cin}
    for i in range(N):
        a, b = (A >> i) & 1, (B >> i) & 1
        v["A%d" % i], v["B%d" % i] = a, b
        v["nA%d" % i], v["nB%d" % i] = 1 - a, 1 - b
        v["p%d" % i], v["g%d" % i] = a ^ b, a & b
    G = {i: v["g%d" % i] for i in range(N)}
    P = {i: v["p%d" % i] for i in range(N)}
    for k in range(1, N.bit_length()):
        s, nG, nP = 1 << (k - 1), {}, {}
        for i in range(N):
            nG[i] = G[i] | (P[i] & G[i - s]) if i >= s else G[i]
            nP[i] = P[i] & P[i - s] if i >= s else P[i]
            v["G%d_%d" % (k, i)], v["P%d_%d" % (k, i)] = nG[i], nP[i]
        G, P = nG, nP
    v["c0"] = cin
    for i in range(1, N):
        v["c%d" % i] = G[i - 1] | (P[i - 1] & cin)
    v["cout"] = G[N - 1] | (P[N - 1] & cin)
    for i in range(N):
        v["s%d" % i] = v["p%d" % i] ^ v["c%d" % i]
        v["np%d" % i], v["nc%d" % i] = 1 - v["p%d" % i], 1 - v["c%d" % i]
    return v


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--width", type=int, default=32)
    ap.add_argument("--cases", type=int, default=0, help="0 = exhaustive if small")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--out", default="")
    ap.add_argument("--no-sim", action="store_true")
    args = ap.parse_args()
    N = args.width

    ppa = PPA(N)
    b = ppa.build()
    (x0, y0, z0), (x1, y1, z1) = b.bounds()
    vol = (x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)
    print("width %d: %d blocks, extent x %d..%d y %d..%d z %d..%d"
          % (N, len(b.cells), x0, x1, y0, y1, z0, z1))
    print("volume %.2fM cells (engine cap %.1fM)" % (vol / 1e6, 8.0))

    import audit, nets
    problems = audit.audit(b.cells)
    for kind, items in problems.items():
        if items:
            print("STRUCTURAL %s x%d e.g. %s" % (kind, len(items), items[0]))
    shorts = nets.check(b.cells, ppa.labels)
    print("lids placed: %d (was: one per rail cell)" % ppa.lids)
    print("net check: %d shorted nets" % len(shorts))
    for la, lb, pa, pb in shorts[:10]:
        print("   SHORT %s <-> %s  at %s / %s" % (la, lb, pa, pb))
    if any(problems.values()) or shorts:
        return 1
    if args.no_sim:
        return 0
    if vol > 8_000_000:
        print("too big to simulate")
        return 1

    sim = b.sim(settle=4000)
    print("placed; quiescent:", sim.sim.is_quiescent(), "non-air:", sim.sim.non_air_count())
    order = [p for _s, p in ppa.levers]
    names = [s for s, _p in ppa.levers]
    lv = rs.Levers(sim, order)

    if args.cases:
        rnd = random.Random(args.seed)
        cases = [(rnd.getrandbits(N), rnd.getrandbits(N), rnd.getrandbits(1))
                 for _ in range(args.cases)]
        edge = [(0, 0, 0), (0, 0, 1), ((1 << N) - 1, 1, 0), ((1 << N) - 1, 1, 1),
                ((1 << N) - 1, (1 << N) - 1, 1), (0, (1 << N) - 1, 1),
                (0x55555555 & ((1 << N) - 1), 0xAAAAAAAA & ((1 << N) - 1), 0)]
        cases = edge + cases
    else:
        cases = [(a, bb, c) for a in range(1 << N) for bb in range(1 << N) for c in (0, 1)]
    print("checking %d cases" % len(cases))

    bad, sig_bad = 0, {}
    for A, B, cin in cases:
        bits = [((A >> i) & 1) if n.startswith("A") else
                ((B >> i) & 1) if n.startswith("B") else cin
                for n, i in ((n, int(n[1:]) if n[1:].isdigit() else 0) for n in names)]
        lv.set(bits)
        want = expect(N, A, B, cin)
        got = {s: int(sim.on(*p)) for s, p in ppa.probe.items() if s in want}
        wrong = [s for s in got if got[s] != want[s]]
        total = sum(got["s%d" % i] << i for i in range(N)) + (got["cout"] << N)
        if total != A + B + cin:
            bad += 1
        for s in wrong:
            sig_bad[s] = sig_bad.get(s, 0) + 1
    print("sums correct: %d/%d" % (len(cases) - bad, len(cases)))
    if sig_bad:
        worst = sorted(sig_bad.items(), key=lambda kv: -kv[1])[:12]
        print("signals that ever disagreed with the model (%d distinct):" % len(sig_bad))
        for s, c in worst:
            print("   %-10s wrong in %d/%d cases" % (s, c, len(cases)))
    if bad or sig_bad:
        return 1
    if args.out:
        lv.set([0] * len(order))
        print("baked %d states" % bake(b, sim))
        b.s.save_to_file(args.out)
        print("saved", args.out)
    return 0


def bake(b, sim):
    changed = 0
    for pos in sorted(b.cells):
        state = sim.block(*pos)
        if state != b.cells[pos] and "air" not in state:
            b.force(*pos, state)
            changed += 1
    return changed


if __name__ == "__main__":
    raise SystemExit(main())
