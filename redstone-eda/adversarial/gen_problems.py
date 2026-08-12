#!/usr/bin/env python3
"""PROCEDURAL BUS-ROUTING PROBLEM GENERATOR — seeded, deterministic, escalating.

Every problem is a pure function of (mode, axis/tier, level/index, master seed):
no unseeded randomness anywhere, so a failure is reproducible from its filename.

Two modes, and they answer different questions:

  --mode ladder   COMPOUND difficulty, tiers 1..8. Tier 1 is near-trivial; tier
                  8 is deliberately beyond the current solver. Answers "how far
                  up can it go".
  --mode probe    ONE AXIS AT A TIME, graded levels, everything else held at an
                  easy baseline. Answers "WHICH capability breaks, and at what
                  level" — a compound tier cannot tell you that, because the
                  first wall it hits masks every axis behind it.

------------------------------------------------------------------------------
THE DIFFICULTY AXES
------------------------------------------------------------------------------
  A1  span            manhattan distance to cross (16 -> 256). Past 15 dust
                      cells a repeater is mandatory, so span also forces
                      stations and buys delay.
  A2  width           bits in the word (2 -> 32). Width multiplies every other
                      axis: a level shift costs cells PER BIT.
  A3  form            cross-section shape per endpoint: `vert` (2y stack),
                      `flat_z` (2-pitch across travel), `flat_x` (2-pitch ALONG
                      travel — awkward, no rotation fixes it), `vert3` (pitch
                      3) and `diag` (step (1,2,0)). Mismatched ends force a
                      form conversion, charged as `coherence`.
  A4  dy              level change between driver bit 0 and sink bit 0 (0->12).
                      Needs the verified level-shift tile, per level per bit.
  A5  dogleg          off-axis offset while travelling in x (0 -> 32).
  A6  obstacles       density AND SHAPE: pillars, a wall with ONE window, a
                      slab with ONE hole, diagonal ribs, a staggered maze.
                      Shape matters more than density: a wall with a window is
                      a topology problem, 400 pillars are a nuisance.
  A7  exclusions      solid keepout cuboids planted in the natural corridor.
  A8  congestion      N buses routed IN SEQUENCE into one corridor; each later
                      bus must survive what the earlier ones took. This is
                      where first-come-first-served order becomes a quality
                      bug.
  A9  permutation     sink bit order reversed (negative step) or rotated by one
                      pitch, so lanes must cross without shorting.
  A10 gates           mandatory waypoint columns to thread in order.
  A11 fanout          one driver, several sinks in different places.
  A12 carrier         `binary` (one bit per wire) vs `hex` (the measured analog
                      signal-strength carrier of notes-hex-transport.md, which
                      needs a VALUE-PRESERVING route: a repeater refresh
                      destroys the value).

------------------------------------------------------------------------------
VALIDITY (why a failure means something)
------------------------------------------------------------------------------
A generated problem is worthless if it is unsolvable for a silly reason, so the
generator enforces, by construction:
  * no two ports share a hardware cell (dust / lamp / lever / support) — the
    lanes are packed with a gap that accounts for the dogleg swing;
  * every port keeps a clear approach: a Chebyshev-2 keepout around every bit
    cell plus a 5-cell clear tube outward along the travel axis, so no anchor is
    ever walled in;
  * `flat_x` spans are long enough that the two banks cannot overlap.

Usage:
  python3 gen_problems.py --out DIR [--mode ladder|probe|both]
                          [--per-tier 6] [--seed 20260810] [--tiers 1-8]
"""

import argparse
import json
import os
import random

STONE = "minecraft:stone"
DEEPSLATE = "minecraft:deepslate"
BRICK = "minecraft:bricks"

FORMS = {
    "vert": (0, 2, 0),
    "flat_z": (0, 0, 2),
    "flat_x": (2, 0, 0),
    "vert3": (0, 3, 0),
    "diag": (1, 2, 0),
}


def extent(form, width):
    s = FORMS[form]
    return (abs(s[0]) * (width - 1), abs(s[1]) * (width - 1), abs(s[2]) * (width - 1))


def port(name, direction, anchor, step, width, out):
    return {"name": name, "dir": direction, "anchor": list(anchor),
            "step": list(step), "width": width, "out": list(out)}


def port_cells(p):
    """Every cell a port's hardware claims: dust, support, lever (+its support)."""
    a, st, w, out = p["anchor"], p["step"], p["width"], p["out"]
    cells = []
    for k in range(w):
        c = (a[0] + st[0] * k, a[1] + st[1] * k, a[2] + st[2] * k)
        cells.append(c)                                    # dust
        cells.append((c[0], c[1] - 1, c[2]))               # support / lamp
        if p["dir"] == "in":
            lv = (c[0] + out[0], c[1] + out[1], c[2] + out[2])
            cells.append(lv)
            cells.append((lv[0], lv[1] - 1, lv[2]))
    return cells


def dust_cells(p):
    a, st, w = p["anchor"], p["step"], p["width"]
    return [(a[0] + st[0] * k, a[1] + st[1] * k, a[2] + st[2] * k) for k in range(w)]


# ---------------------------------------------------------------------------
# obstacle shapes (A6, A7)
# ---------------------------------------------------------------------------
def pillars(rng, x0, x1, y0, y1, z0, z1, n, block=STONE):
    out = []
    for _ in range(n):
        x, z = rng.randint(x0, x1), rng.randint(z0, z1)
        h = rng.randint(2, max(2, y1 - y0))
        for y in range(y0, min(y1, y0 + h) + 1):
            out.append([x, y, z, block])
    return out


def wall_with_window(rng, x, y0, y1, z0, z1, win_h, win_w, block=BRICK):
    wy = rng.randint(y0, max(y0, y1 - win_h))
    wz = rng.randint(z0, max(z0, z1 - win_w))
    return [[x, y, z, block]
            for y in range(y0, y1 + 1) for z in range(z0, z1 + 1)
            if not (wy <= y < wy + win_h and wz <= z < wz + win_w)]


def slab_with_hole(rng, y, x0, x1, z0, z1, hole, block=DEEPSLATE):
    hx = rng.randint(x0, max(x0, x1 - hole))
    hz = rng.randint(z0, max(z0, z1 - hole))
    return [[x, y, z, block]
            for x in range(x0, x1 + 1) for z in range(z0, z1 + 1)
            if not (hx <= x < hx + hole and hz <= z < hz + hole)]


def ribs(rng, x0, x1, y0, y1, z0, z1, n, block=STONE):
    out = []
    for _ in range(n):
        sx = rng.randint(x0, x1)
        for k in range((z1 - z0) + 1):
            x, z = sx + k, z0 + k
            if x > x1:
                break
            for y in range(y0, y1 + 1, 2):
                out.append([x, y, z, block])
    return out


def maze(rng, x0, x1, y0, y1, z0, z1, walls, block=BRICK):
    out, side = [], rng.randint(0, 1)
    if walls <= 0:
        return out
    step = max(4, (x1 - x0) // walls)
    for i in range(walls):
        x = x0 + i * step
        if x > x1:
            break
        za, zb = (z0, (z0 + z1) // 2) if side == 0 else ((z0 + z1) // 2, z1)
        for y in range(y0, y1 + 1):
            for z in range(za, zb + 1):
                out.append([x, y, z, block])
        side ^= 1
    return out


def box(lo, hi, block=DEEPSLATE):
    return [[x, y, z, block]
            for x in range(lo[0], hi[0] + 1)
            for y in range(lo[1], hi[1] + 1)
            for z in range(lo[2], hi[2] + 1)]


# ---------------------------------------------------------------------------
# the core builder: one cfg -> one problem
# ---------------------------------------------------------------------------
BASE = dict(span=32, width=8, src="vert", dst="vert", dy=0, dogleg=0,
            obstacles=(), excl=0, competitors=0, perm="none", gates=0,
            fanout=1, carrier="binary")


def build(pid, cfg, seed):
    rng = random.Random(seed)
    span, width = cfg["span"], cfg["width"]
    src_form, dst_form = cfg["src"], cfg["dst"]
    dy, dogleg = cfg["dy"], cfg["dogleg"]
    nbus = 1 + cfg["competitors"]

    # `flat_x` banks advance along travel: keep them from overlapping.
    need = 2 * max(extent(src_form, width)[0], extent(dst_form, width)[0]) + 16
    span = max(span, need)

    # Lane pitch: the widest cross-section either endpoint form needs, plus the
    # dogleg swing, plus a gap. Generous on purpose — a lane collision would
    # make the problem invalid, and invalid problems teach nothing.
    vertical_lanes = src_form in ("vert", "vert3", "diag")

    def cross(form):
        """Extent on the axis the LANES are stacked along — using max(ey,ez)
        here instead would stack a 24-bit vertical bus 700 cells wide in z."""
        ex, ey, ez = extent(form, width)
        return ez if vertical_lanes else ey
    pitch = cross(src_form) + cross(dst_form) + 2 * dogleg + 10

    y0, z0 = 2, 8
    ports, buses = [], []
    for b in range(nbus):
        if vertical_lanes:
            sy, sz = y0, z0 + pitch * b
        else:
            sy, sz = y0 + pitch * b, z0
        src_anchor = (0, sy, sz)
        src_step = list(FORMS[src_form])
        dst_form_b = dst_form if b == 0 else src_form
        dst_step = list(FORMS[dst_form_b])
        dz = dogleg if b % 2 == 0 else -dogleg
        dst_anchor = [span, sy + dy, sz + dz]

        if cfg["perm"] == "reverse" and b == 0:
            ex, ey, ez = extent(dst_form_b, width)
            dst_anchor = [dst_anchor[0] + ex, dst_anchor[1] + ey, dst_anchor[2] + ez]
            dst_step = [-v for v in dst_step]
        elif cfg["perm"] == "rotate" and b == 0:
            dst_anchor = [dst_anchor[i] + dst_step[i] for i in range(3)]

        name = "b0" if b == 0 else f"c{b}"
        din, dout = f"{name}_in", f"{name}_out"
        ports.append(port(din, "in", src_anchor, src_step, width, (-1, 0, 0)))
        ports.append(port(dout, "out", dst_anchor, dst_step, width, (1, 0, 0)))
        sinks = [dout]
        buses.append({"name": name, "driver": din, "sinks": sinks, "gates": []})

    # A11 fanout: extra sinks for bus 0, in their own band past every lane, so
    # they can never collide with a competitor.
    if cfg["fanout"] > 1:
        band = (z0 + pitch * nbus + 8) if vertical_lanes else (y0 + pitch * nbus + 8)
        for f in range(1, cfg["fanout"]):
            fname = f"b0_out{f}"
            if vertical_lanes:
                fa = [span - 12 * f, y0 + dy, band + pitch * (f - 1)]
            else:
                fa = [span - 12 * f, band + pitch * (f - 1), z0]
            ports.append(port(fname, "out", fa, list(FORMS[src_form]), width, (1, 0, 0)))
            buses[0]["sinks"].append(fname)

    # A10 gates: waypoint columns on bus 0, on its own form and level.
    for g in range(cfg["gates"]):
        gx = span * (g + 1) // (cfg["gates"] + 1)
        buses[0]["gates"].append({"name": f"g{g}", "anchor": [gx, y0, z0],
                                  "step": list(FORMS[src_form])})

    # ---- validity: no two ports may share a hardware cell ----
    claimed, dupes = {}, []
    for p in ports:
        for c in port_cells(p):
            if c in claimed and claimed[c] != p["name"]:
                dupes.append((c, claimed[c], p["name"]))
            claimed[c] = p["name"]
    if dupes:
        raise AssertionError(f"{pid}: port hardware collision, e.g. {dupes[0]}")

    # ---- obstacle field, sized from the ACTUAL bank extents ----
    allcells = [c for p in ports for c in port_cells(p)]
    ylo_p, yhi_p = min(c[1] for c in allcells), max(c[1] for c in allcells)
    zlo_p, zhi_p = min(c[2] for c in allcells), max(c[2] for c in allcells)
    margin = max(4, dogleg // 2 + 3)
    ylo, yhi = max(0, ylo_p - 1), yhi_p + margin
    zlo, zhi = zlo_p - margin, zhi_p + margin
    ey, ez = yhi_p - ylo_p, zhi_p - zlo_p

    obs = []
    for kind in cfg["obstacles"]:
        if kind == "pillars_sparse":
            obs += pillars(rng, 8, span - 8, ylo, yhi, zlo, zhi, max(4, span // 8))
        elif kind == "pillars_dense":
            obs += pillars(rng, 8, span - 8, ylo, yhi, zlo, zhi, max(12, span // 3))
        elif kind == "wall_window":
            wx = rng.randint(span // 3, 2 * span // 3)
            obs += wall_with_window(rng, wx, ylo, yhi, zlo, zhi,
                                    win_h=max(4, ey // 2), win_w=max(4, ez // 2 + 3))
        elif kind == "slab_hole":
            sy2 = rng.randint(ylo + 2, max(ylo + 2, yhi - 1))
            obs += slab_with_hole(rng, sy2, 10, span - 10, zlo, zhi,
                                  hole=max(5, width // 2))
        elif kind == "ribs":
            obs += ribs(rng, 12, span - 12, ylo, yhi, zlo, min(zhi, zlo + 14), n=2)
        elif kind == "maze":
            obs += maze(rng, 12, span - 12, ylo, yhi, zlo, zhi, walls=max(2, span // 30))
    for _ in range(cfg["excl"]):
        lx = rng.randint(span // 4, 3 * span // 4)
        ly = rng.randint(ylo, max(ylo, yhi - 3))
        lz = rng.randint(zlo, max(zlo, zhi - 5))
        obs += box((lx, ly, lz),
                   (lx + rng.randint(3, 8), ly + rng.randint(2, 6), lz + rng.randint(3, 8)))

    # ---- keepout: Chebyshev-2 around every port cell + a 5-cell approach tube ----
    protected = set()
    for p in ports:
        outward = 1 if p["dir"] == "in" else -1
        for c in dust_cells(p):
            for dx in range(-2, 3):
                for dyy in range(-2, 3):
                    for dz2 in range(-2, 3):
                        protected.add((c[0] + dx, c[1] + dyy, c[2] + dz2))
            for t in range(1, 6):
                for dyy in (-1, 0, 1):
                    for dz2 in (-1, 0, 1):
                        protected.add((c[0] + outward * t, c[1] + dyy, c[2] + dz2))
        for c in port_cells(p):
            protected.add(c)
    for g in buses[0]["gates"]:
        a, st = g["anchor"], g["step"]
        for k in range(width):
            c = (a[0] + st[0] * k, a[1] + st[1] * k, a[2] + st[2] * k)
            for dx in range(-2, 3):
                for dyy in range(-2, 3):
                    for dz2 in range(-2, 3):
                        protected.add((c[0] + dx, c[1] + dyy, c[2] + dz2))

    seen, clean = set(), []
    for o in obs:
        k = (o[0], o[1], o[2])
        if k in protected or k in seen:
            continue
        seen.add(k)
        clean.append(o)

    family = "-".join([f"w{width}", f"s{span}", f"{src_form}2{dst_form}", f"dy{dy}",
                       f"dl{dogleg}", f"n{nbus}", cfg["perm"], f"g{cfg['gates']}",
                       f"f{cfg['fanout']}", cfg["carrier"]])
    return {
        "id": pid, "seed": seed, "family": family, "carrier": cfg["carrier"],
        "tier": cfg.get("tier", 0), "axis": cfg.get("axis"), "level": cfg.get("level"),
        "axes": {"span": span, "width": width, "src_form": src_form,
                 "dst_form": dst_form, "dy": dy, "dogleg": dogleg,
                 "obstacles": list(cfg["obstacles"]), "exclusions": cfg["excl"],
                 "competitors": cfg["competitors"], "perm": cfg["perm"],
                 "gates": cfg["gates"], "fanout": cfg["fanout"],
                 "carrier": cfg["carrier"], "obstacle_cells": len(clean)},
        "style": {"bus_block": "minecraft:gray_concrete",
                  "transparent_block": "minecraft:glass"},
        "ports": ports, "buses": buses, "obstacles": clean, "settle": 4000,
    }


# ---------------------------------------------------------------------------
# mode: ladder (compound)
# ---------------------------------------------------------------------------
TIERS = {
    1: dict(span=(12, 16), width=(4, 4), src="vert", dst="vert", dy=(0, 0),
            dogleg=(0, 0), obstacles=[], excl=0, competitors=0, perm="none",
            gates=0, fanout=1, carrier="binary"),
    2: dict(span=(24, 32), width=(8, 8), src="vert", dst="vert", dy=(1, 2),
            dogleg=(0, 2), obstacles=["pillars_sparse"], excl=0, competitors=0,
            perm="none", gates=0, fanout=1, carrier="binary"),
    3: dict(span=(36, 48), width=(8, 8), src="vert", dst="flat_z", dy=(0, 2),
            dogleg=(2, 6), obstacles=["pillars_sparse"], excl=0, competitors=0,
            perm="none", gates=0, fanout=1, carrier="binary"),
    4: dict(span=(48, 64), width=(8, 12), src="vert", dst="vert", dy=(1, 3),
            dogleg=(2, 8), obstacles=["wall_window"], excl=0, competitors=1,
            perm="none", gates=0, fanout=1, carrier="binary"),
    5: dict(span=(64, 88), width=(12, 16), src="vert", dst="flat_z", dy=(1, 3),
            dogleg=(4, 10), obstacles=["wall_window", "pillars_dense"], excl=1,
            competitors=2, perm="none", gates=1, fanout=1, carrier="binary"),
    6: dict(span=(88, 120), width=(16, 16), src="vert", dst="vert", dy=(2, 4),
            dogleg=(6, 14), obstacles=["maze", "slab_hole"], excl=1,
            competitors=3, perm="none", gates=1, fanout=2, carrier="binary"),
    7: dict(span=(120, 168), width=(16, 24), src="vert", dst="flat_x",
            dy=(4, 6), dogleg=(8, 20), obstacles=["maze", "ribs", "wall_window"],
            excl=2, competitors=4, perm="reverse", gates=2, fanout=2,
            carrier="binary"),
    8: dict(span=(160, 220), width=(16, 24), src="diag", dst="flat_x",
            dy=(6, 9), dogleg=(8, 20), obstacles=["maze", "ribs", "wall_window"],
            excl=2, competitors=4, perm="rotate", gates=2, fanout=2,
            carrier="hex"),
}


def ladder(tier, idx, master):
    t = TIERS[tier]
    seed = (master * 1000003 + tier * 9176 + idx * 131) & 0x7FFFFFFF
    rng = random.Random(seed ^ 0xA5A5)
    cfg = dict(BASE)
    cfg.update(span=rng.randint(*t["span"]), width=rng.randint(*t["width"]),
               src=t["src"], dst=t["dst"], dy=rng.randint(*t["dy"]),
               dogleg=rng.randint(*t["dogleg"]), obstacles=tuple(t["obstacles"]),
               excl=t["excl"], competitors=t["competitors"], perm=t["perm"],
               gates=t["gates"], fanout=t["fanout"], carrier=t["carrier"],
               tier=tier)
    return build(f"t{tier}_{idx:02d}", cfg, seed)


# ---------------------------------------------------------------------------
# mode: probe (one axis at a time)
# ---------------------------------------------------------------------------
PROBES = {
    "span":     [dict(span=v) for v in (16, 32, 64, 96, 128, 192, 256)],
    "width":    [dict(width=v) for v in (2, 4, 8, 12, 16, 24, 32)],
    "dy":       [dict(dy=v) for v in (0, 1, 2, 3, 5, 8, 12)],
    "dogleg":   [dict(dogleg=v) for v in (0, 2, 4, 8, 16, 32)],
    "form":     [dict(src=a, dst=b) for a, b in (
                    ("vert", "vert"), ("vert", "flat_z"), ("flat_z", "vert"),
                    ("flat_z", "flat_z"), ("vert", "flat_x"), ("flat_x", "flat_x"),
                    ("vert", "vert3"), ("vert3", "vert3"), ("vert", "diag"))],
    "obstacle": [dict(obstacles=v) for v in (
                    (), ("pillars_sparse",), ("pillars_dense",), ("wall_window",),
                    ("slab_hole",), ("ribs",), ("maze",),
                    ("maze", "wall_window", "ribs"))],
    "excl":     [dict(excl=v) for v in (0, 1, 2, 4)],
    "congest":  [dict(competitors=v) for v in (0, 1, 2, 3, 5, 7)],
    "perm":     [dict(perm=v) for v in ("none", "reverse", "rotate")],
    "gates":    [dict(gates=v) for v in (0, 1, 2, 3)],
    "fanout":   [dict(fanout=v) for v in (1, 2, 3, 4)],
    "carrier":  [dict(carrier=v) for v in ("binary", "hex")],
}


def probe(axis, level, master):
    seed = (master * 7919 + hash(axis) % 100003 * 37 + level * 17) & 0x7FFFFFFF
    cfg = dict(BASE)
    cfg.update(PROBES[axis][level])
    cfg.update(axis=axis, level=level, tier=0)
    # A probe that varies nothing but its axis still needs enough span for a
    # wide word to have somewhere to go.
    if axis in ("width", "form") and cfg["span"] < 48:
        cfg["span"] = 48
    return build(f"p_{axis}_{level}", cfg, seed)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--mode", default="both", choices=["ladder", "probe", "both"])
    ap.add_argument("--per-tier", type=int, default=6)
    ap.add_argument("--seed", type=int, default=20260810)
    ap.add_argument("--tiers", default="1-8")
    a = ap.parse_args()
    lo, _, hi = a.tiers.partition("-")
    tiers = range(int(lo), int(hi or lo) + 1)
    os.makedirs(a.out, exist_ok=True)
    index = []

    def emit(p):
        path = os.path.join(a.out, f"{p['id']}.json")
        with open(path, "w") as f:
            json.dump(p, f)
        index.append({"id": p["id"], "tier": p["tier"], "axis": p.get("axis"),
                      "level": p.get("level"), "path": path,
                      "family": p["family"], "axes": p["axes"]})

    if a.mode in ("ladder", "both"):
        for t in tiers:
            for i in range(a.per_tier):
                emit(ladder(t, i, a.seed))
    if a.mode in ("probe", "both"):
        for axis, levels in PROBES.items():
            for lv in range(len(levels)):
                emit(probe(axis, lv, a.seed))

    with open(os.path.join(a.out, "index.json"), "w") as f:
        json.dump(index, f, indent=1)
    print(f"{len(index)} problems -> {a.out}")


if __name__ == "__main__":
    main()
