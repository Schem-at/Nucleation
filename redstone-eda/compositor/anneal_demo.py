"""Placement annealing demo (C2): scatter cell instances, anneal placements
on the ABSTRACT view (HPWL + hull overlap -- CellAbstract style, no voxels
touched), then make the winning placement REAL: stamp it, route every net
with the bridge's negotiated route_all, and DRC/LVS-clean the result.

5 half-adder cells, chained: c[i].xor -> c[i+1].a, c[i].carry -> c[i+1].b.
"""
import json
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)

import cells                       # noqa: E402
from compositor import Compositor  # noqa: E402

N = 5
AREA = ((0, 0), (46, 46))          # x/z placement window
NETS = [("c%d" % i, "xor", "c%d" % (i + 1), "a") for i in range(N - 1)] + \
       [("c%d" % i, "carry", "c%d" % (i + 1), "b") for i in range(N - 1)]


def hull(frag):
    xs = [p[0] for p in frag.cells]
    zs = [p[2] for p in frag.cells]
    return (min(xs), min(zs), max(xs), max(zs))


def ports_at(frag, place):
    px, pz = place
    return {name: (x + px, y, z + pz) for name, (x, y, z) in frag.ports.items()}


def hpwl(frag, places):
    pts = [ports_at(frag, p) for p in places]
    total = 0
    for si, sp, di, dp in NETS:
        a = pts[int(si[1:])][sp]
        b = pts[int(di[1:])][dp]
        total += abs(a[0] - b[0]) + abs(a[2] - b[2])
    return total


def cost(frag, places):
    h = hull(frag)
    total = float(hpwl(frag, places))
    # inflated-hull overlap penalty (clearance margin 4: the routing halo
    # eats one cell off each face, and parallel nets need a 3-wide corridor)
    m = 4
    for i in range(len(places)):
        for j in range(i + 1, len(places)):
            (xi, zi), (xj, zj) = places[i], places[j]
            ox = min(xi + h[2], xj + h[2]) - max(xi + h[0], xj + h[0]) + 1 + m
            oz = min(zi + h[3], zj + h[3]) - max(zi + h[1], zj + h[1]) + 1 + m
            if ox > 0 and oz > 0:
                total += 1000 * ox * oz
    return total


def anneal(frag, places, seed=7, iters=8000, t0=25.0, t1=0.4):
    rng = random.Random(seed)
    cur = list(places)
    c = cost(frag, cur)
    best, bc = list(cur), c
    h = hull(frag)
    for it in range(iters):
        t = t0 * (t1 / t0) ** (it / iters)
        i = rng.randrange(len(cur))
        dx, dz = rng.randint(-5, 5), rng.randint(-5, 5)
        x = min(max(cur[i][0] + dx, AREA[0][0] - h[0]), AREA[1][0] - h[2])
        z = min(max(cur[i][1] + dz, AREA[0][1] - h[1]), AREA[1][1] - h[3])
        old = cur[i]
        cur[i] = (x, z)
        nc = cost(frag, cur)
        if nc <= c or rng.random() < math.exp((c - nc) / t):
            c = nc
            if c < bc:
                best, bc = list(cur), c
        else:
            cur[i] = old
    return best, bc


def touches(c):
    """Actual touching dust pairs between different alias-rooted nets."""
    import nets as nm
    root = {}

    def find(a):
        root.setdefault(a, a)
        while root[a] != a:
            root[a] = root[root[a]]
            a = root[a]
        return a

    for x, y in c.aliases:
        root[find(x)] = find(y)
    out = []
    for p, blk in c.b.cells.items():
        if "redstone_wire" not in blk:
            continue
        la = c.labels.get(p)
        if la is None:
            continue
        for q in nm.neighbours(c.b.cells, p):
            lb = c.labels.get(q)
            if lb is not None and find(la) != find(lb):
                out.append((la, p, lb, q))
    return out


def realize(name, frag, places):
    """Stamp a placement, route all queued nets via the bridge, analyse.
    Rebuilds from scratch per attempt: a failed negotiation may leave
    partial geometry in the schematic."""
    lo = (AREA[0][0] - 6, 0, AREA[0][1] - 6)
    hi = (AREA[1][0] + 6, 5, AREA[1][1] + 6)
    for rounds in (200, 400, 800):     # negotiation is order-sensitive
        try:
            return _realize_once(name, frag, places, lo, hi, rounds)
        except Exception as e:
            err = e
    raise RuntimeError("route_all did not converge for %s: %s" % (name, err))


def _realize_once(name, frag, places, lo, hi, rounds):
    c = Compositor(name)
    insts = [c.add("c%d" % i, frag, (px, 0, pz))
             for i, (px, pz) in enumerate(places)]
    for si, sp, di, dp in NETS:
        kind = c.connect(insts[int(si[1:])].ref(sp), insts[int(di[1:])].ref(dp),
                         src_off=(1, 0, 0), dst_off=(-1, 0, 0))  # east-out, west-in
        assert kind == "net"
    # ---- clearance halo: the bridge workspace cannot see the stamped
    # cells' net labels (documented limitation), so on its own it may lay
    # dust adjacent to a foreign port lane.  Occupy every free cell that
    # is electrically connectable to a stamped dust cell (same-level plus
    # up/down diagonals) with a stone -- except the nets' own endpoint
    # cells -- and the label-blind router is FORCED to keep clearance.
    skip = set()
    for pa, pb, la, lb in c.pending:
        skip.update((pa, pb))
    nhalo = 0
    for (x, y, z), blk in list(c.b.cells.items()):
        if "redstone_wire" not in blk:
            continue
        for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            for dy in (-1, 0, 1):
                q = (x + dx, y + dy, z + dz)
                below = (q[0], q[1] - 1, q[2])
                if (q not in skip and q not in c.b.cells
                        and "redstone_wire" not in c.b.cells.get(below, "")):
                    c.b.stone(*q, role="route")  # never caps a dust diagonal
                    nhalo += 1

    rep = c.route_bridge(bounds=(lo, hi),
                         congestion={"max_rounds": rounds,
                                     "history_increment": 4,
                                     "present_penalty": 6})
    paths = {r["label"]: [tuple(p) for p in r["path"]]
             for r in rep.get("routes", [])}
    delays = {r["label"]: r.get("delay_rt", 0) for r in rep.get("routes", [])}
    clean, _, shorts = c.check(verbose=False)
    routes = paths
    cells_total = sum(len(v) for v in paths.values())
    delay = sum(delays.values())
    repaired = ["halo:%d" % nhalo]
    drc = c.drc()
    hard = [v for v in drc if v["kind"] in
            ("short", "floating", "unattached_wall_torch")]
    lvs = c.lvs()
    return c, {"routed_nets": len(routes), "route_cells": cells_total,
               "repaired_nets": sorted(repaired),
               "route_delay_rt": delay, "violations": rep.get("violations", []),
               "nets_check_shorts": len(shorts), "audit_clean": clean,
               "drc_hard": len(hard), "drc_repeater_cycles":
                   sum(1 for v in drc if v["kind"] == "repeater_cycle"),
               "lvs_opens": len(lvs.get("opens", []))}


def scatter(frag, seed=13):
    """Random non-overlapping (but badly spread) initial placement."""
    rng = random.Random(seed)
    h = hull(frag)
    places = []
    while len(places) < N:
        x = rng.randint(AREA[0][0] - h[0], AREA[1][0] - h[2])
        z = rng.randint(AREA[0][1] - h[1], AREA[1][1] - h[3])
        trial = places + [(x, z)]
        m = 2
        ok = all(not (min(a[0] + h[2], b[0] + h[2]) - max(a[0] + h[0], b[0] + h[0]) + 1 + m > 0
                      and min(a[1] + h[3], b[1] + h[3]) - max(a[1] + h[1], b[1] + h[1]) + 1 + m > 0)
                 for a in places for b in [(x, z)])
        if ok:
            places = trial
    return places


def main():
    ha = cells.build_half_adder()
    before = scatter(ha)
    c_before = cost(ha, before)
    after, c_after = anneal(ha, before)
    h0, h1 = hpwl(ha, before), hpwl(ha, after)
    print("anneal: cost %d -> %d, pure HPWL %d -> %d (%.0f%% shorter)"
          % (c_before, c_after, h0, h1, 100 * (1 - h1 / h0)))

    _, rb = realize("anneal_before", ha, before)
    ca, ra = realize("anneal_after", ha, after)
    for tag, r in (("before", rb), ("after", ra)):
        print("%s: %d/%d nets routed (%s), %d route cells, %d rt delay, "
              "drc_hard=%d, nets.check shorts=%d, lvs opens=%d, viol=%s"
              % (tag, r["routed_nets"], len(NETS), r["repaired_nets"][0],
                 r["route_cells"], r["route_delay_rt"], r["drc_hard"],
                 r["nets_check_shorts"], r["lvs_opens"],
                 r["violations"] or "none"))
    improved = ra["route_cells"] < rb["route_cells"]
    print("routed wirelength: %d -> %d cells (%s)"
          % (rb["route_cells"], ra["route_cells"],
             "IMPROVED" if improved else "no gain"))
    with open(os.path.join(HERE, "anneal_report.json"), "w") as f:
        json.dump({"cost_before": c_before, "cost_after": c_after,
                   "places_before": before, "places_after": after,
                   "route_before": rb, "route_after": ra}, f, indent=1)
    ok = (ra["routed_nets"] == len(NETS) and ra["drc_hard"] == 0
          and ra["nets_check_shorts"] == 0 and ra["lvs_opens"] == 0
          and improved)
    return ok


if __name__ == "__main__":
    print("anneal_demo:", "ALL PASS" if main() else "FAILURES")
