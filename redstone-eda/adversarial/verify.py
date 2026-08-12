#!/usr/bin/env python3
"""MECHANICAL VERIFIER — turn a critique into a verdict, or refuse to count it.

A critique is a HYPOTHESIS. Plausible-but-wrong is the expected failure mode of
any LLM critic, so nothing is counted until it has been CHECKED here, by one of
three mechanisms:

  1. PROVEN IMPOSSIBLE — the claim is below a hard lower bound derived from the
     problem geometry (per-bit manhattan distance, mandatory repeater count).
     Verdict REJECTED, no solver run needed.
  2. MEASURED — the claim is about the solution we already have ("it detours N
     cells", "it spends N repeaters"). Measure the fragment. Verdict ACCEPTED if
     the measurement supports the claim, REJECTED if it contradicts it.
  3. CONSTRUCTED — the claim is "a better solution exists". We try to BUILD one
     using only knobs the solver itself exposes (routing order, gate waypoints,
     sink order) plus any concrete hint the critic gave, and we re-verify the
     result in simulation. Verdict ACCEPTED only if a variant is strictly better
     AND still passes sim + DRC. Otherwise UNVERIFIED — never counted as a win.

The variant search is useful on its own, with no critic at all: anything it
finds is a case where the solver's default choice is beaten by its own API.

Usage (standalone variant sweep):
  python3 verify.py --results results.jsonl --out variants.jsonl
"""

import argparse
import copy
import json
import os

from run_ladder import solve_one

# The design's default BusCost weights (src/design.rs BusCost::default).
W = {"length": 1.0, "delay_rt": 4.0, "skew_rt": 8.0, "coherence": 6.0,
     "footprint": 0.5}


def total(cost):
    return sum(W[k] * float(cost.get(k, 0)) for k in W)


def total_all(rec):
    return sum(total(b.get("cost", {})) for b in rec.get("buses", [])
               if b.get("state") == "Routed")


def bus_of(rec, name):
    for b in rec.get("buses", []):
        if b.get("name") == name:
            return b
    return None


def dominates(a, b):
    """a is at least as good on every term and strictly better on one."""
    keys = list(W)
    if any(float(a.get(k, 0)) > float(b.get(k, 0)) for k in keys):
        return False
    return any(float(a.get(k, 0)) < float(b.get(k, 0)) for k in keys)


# ---------------------------------------------------------------------------
# lower bounds (mechanism 1)
# ---------------------------------------------------------------------------
def cells_of(p):
    a, st, w = p["anchor"], p["step"], p["width"]
    return [(a[0] + st[0] * k, a[1] + st[1] * k, a[2] + st[2] * k) for k in range(w)]


def lower_bounds(spec, bus_name):
    """Hard floors. Nothing below these is constructible, whatever anyone says.

    * A bit's wire must occupy at least (manhattan - 1) cells between its two
      connection cells: every step of a monotone path is one cell.
    * Dust decays 1 per cell over 15 cells, so a bit whose path is longer than
      15 needs at least floor(len/15) refreshes, each a repeater cell.
    * A repeater at delay=1 costs 2 redstone ticks.
    """
    ports = {p["name"]: p for p in spec["ports"]}
    bus = next(b for b in spec["buses"] if b["name"] == bus_name)
    drv = ports[bus["driver"]]
    per_bit = []
    for sink_name in bus["sinks"]:
        snk = ports[sink_name]
        dc, sc = cells_of(drv), cells_of(snk)
        for k in range(min(len(dc), len(sc))):
            per_bit.append(sum(abs(dc[k][i] - sc[k][i]) for i in range(3)))
    wire = sum(max(0, d - 1) for d in per_bit)
    reps = sum(max(0, (d - 1) // 15) for d in per_bit)
    longest = max(per_bit) if per_bit else 0
    return {
        "min_wire_cells": wire,
        "min_fragment_cells": wire,      # supports may be shared/pre-existing
        "min_repeaters": reps,
        "min_delay_rt": 2 * max(0, (longest - 1) // 15),
        "longest_bit_manhattan": longest,
        "per_bit_manhattan": per_bit,
    }


# ---------------------------------------------------------------------------
# measurements (mechanism 2)
# ---------------------------------------------------------------------------
def endpoint_bbox(spec, bus_name):
    ports = {p["name"]: p for p in spec["ports"]}
    bus = next(b for b in spec["buses"] if b["name"] == bus_name)
    cs = cells_of(ports[bus["driver"]])
    for s in bus["sinks"]:
        cs += cells_of(ports[s])
    lo = tuple(min(c[i] for c in cs) for i in range(3))
    hi = tuple(max(c[i] for c in cs) for i in range(3))
    return lo, hi


def overshoot(spec, rec, bus_name):
    """How far the route strays outside the span its own endpoints occupy,
    per face: [-x, +x, -y, +y, -z, +z]. The topology metric from
    tests/design_bus_topology.rs."""
    b = bus_of(rec, bus_name)
    if not b or "bbox" not in b:
        return None
    (blo, bhi) = b["bbox"]
    elo, ehi = endpoint_bbox(spec, bus_name)
    return [elo[0] - blo[0], bhi[0] - ehi[0],
            elo[1] - blo[1], bhi[1] - ehi[1],
            elo[2] - blo[2], bhi[2] - ehi[2]]


def repeater_count(rec, bus_name):
    b = bus_of(rec, bus_name)
    if not b:
        return None
    return int(b.get("block_kinds", {}).get("minecraft:repeater", 0))


# ---------------------------------------------------------------------------
# construction (mechanism 3)
# ---------------------------------------------------------------------------
def variant_specs(spec, hints=None):
    """Modified problems that keep the PROBLEM identical (same ports, same
    obstacles, same widths) and change only what the solver's own API lets a
    caller choose: routing order, gate waypoints, sink order."""
    out = []
    nbus = len(spec["buses"])
    if nbus > 1:
        s = copy.deepcopy(spec)
        s["buses"] = list(reversed(s["buses"]))
        out.append(("order_rev", s))
        s = copy.deepcopy(spec)
        s["buses"] = s["buses"][1:] + s["buses"][:1]
        out.append(("order_b0_last", s))
    if nbus > 2:
        s = copy.deepcopy(spec)
        s["buses"] = s["buses"][2:] + s["buses"][:2]
        out.append(("order_rot2", s))
    # GATES are the solver's main remaining freedom once the problem is fixed:
    # a waypoint splits the route into legs it plans independently. Placements
    # are computed from the ACTUAL anchors (not the nominal span), and both
    # levels are tried, because "shift early on the long leg" and "shift late"
    # are genuinely different plans.
    b0 = spec["buses"][0]
    drv = next(p for p in spec["ports"] if p["name"] == b0["driver"])
    snk = next(p for p in spec["ports"] if p["name"] == b0["sinks"][0])
    x0, x1 = drv["anchor"][0], snk["anchor"][0]
    src_lv, dst_lv = drv["anchor"][1], snk["anchor"][1]
    zlane = drv["anchor"][2]

    def g(name, frac, y):
        return {"name": name, "anchor": [int(x0 + (x1 - x0) * frac), y, zlane],
                "step": list(drv["step"])}

    cands = [("gate_mid_src", [g("vg", 0.5, src_lv)]),
             ("gate_early_src", [g("vg", 0.3, src_lv)]),
             ("gate_late_src", [g("vg", 0.7, src_lv)])]
    if dst_lv != src_lv:
        cands += [("gate_mid_dst", [g("vg", 0.5, dst_lv)]),
                  ("gate_early_dst", [g("vg", 0.3, dst_lv)]),
                  ("gate_pair", [g("vg0", 0.35, src_lv), g("vg1", 0.7, dst_lv)])]
    for label, gates in cands:
        s = copy.deepcopy(spec)
        s["buses"][0]["gates"] = gates
        out.append((label, s))
    if len(b0["sinks"]) > 1:
        s = copy.deepcopy(spec)
        s["buses"][0]["sinks"] = list(reversed(s["buses"][0]["sinks"]))
        out.append(("sinks_rev", s))
    for h in hints or []:
        s = copy.deepcopy(spec)
        label = "hint"
        if h.get("gate_at"):
            drv = next(p for p in s["ports"] if p["name"] == s["buses"][0]["driver"])
            s["buses"][0]["gates"] = [{"name": "hg", "anchor": list(h["gate_at"]),
                                       "step": list(drv["step"])}]
            label = "hint_gate_%d_%d_%d" % tuple(h["gate_at"])
        if h.get("route_order"):
            order = h["route_order"]
            keyed = {b["name"]: b for b in s["buses"]}
            if set(order) == set(keyed):
                s["buses"] = [keyed[n] for n in order]
                label += "_order"
        if label != "hint":
            out.append((label, s))
    return out


def search_variants(spec, baseline, work_dir, hints=None, limit=12):
    """Run the variants and return the ones that are better AND still correct."""
    base_total = total_all(baseline)
    base_b0 = (bus_of(baseline, spec["buses"][0]["name"]) or {}).get("cost", {})
    found = []
    tried = []
    for label, s in variant_specs(spec, hints)[:limit]:
        path = os.path.join(work_dir, f"{spec['id']}.var_{label}.json")
        s["id"] = f"{spec['id']}.var_{label}"
        with open(path, "w") as f:
            json.dump(s, f)
        rec = solve_one(path, work_dir)
        ok = (rec.get("solved") and rec.get("drc_lvs_clean")
              and isinstance(rec.get("sim"), dict) and rec["sim"].get("pass"))
        t = total_all(rec) if ok else None
        b0 = (bus_of(rec, spec["buses"][0]["name"]) or {}).get("cost", {})
        entry = {"label": label, "correct": bool(ok), "total_all": t,
                 "b0_cost": b0, "cells": (bus_of(rec, spec["buses"][0]["name"]) or {}).get("cells")}
        tried.append(entry)
        if ok and (t < base_total - 1e-9 or dominates(b0, base_b0)):
            found.append({**entry, "base_total": base_total, "base_b0": base_b0,
                          "delta_total": round(t - base_total, 2)})
    return {"base_total": base_total, "tried": tried, "better": found}


# ---------------------------------------------------------------------------
# claim adjudication
# ---------------------------------------------------------------------------
NUMERIC = {"cells", "length", "delay_rt", "skew_rt", "coherence", "footprint",
           "repeaters"}


def judge(spec, baseline, claim, sweep, lb):
    """One claim -> ACCEPTED / REJECTED / UNVERIFIED, with the reason recorded."""
    kind = claim.get("kind")
    bus_name = claim.get("bus") or spec["buses"][0]["name"]
    b = bus_of(baseline, bus_name) or {}
    cost = b.get("cost", {})
    val = claim.get("claim_value")

    if kind == "detour":
        os_ = overshoot(spec, baseline, bus_name)
        if os_ is None:
            return "UNVERIFIED", "no route bbox to measure"
        worst = max(os_)
        if val is None:
            return "UNVERIFIED", "no value to check"
        if worst >= float(val):
            return "ACCEPTED", f"measured overshoot {os_} (worst {worst}) >= claimed {val}"
        return "REJECTED", f"measured overshoot {os_} (worst {worst}) < claimed {val}"

    if kind == "repeaters":
        have = repeater_count(baseline, bus_name)
        if have is None or val is None:
            return "UNVERIFIED", "nothing to measure"
        if float(val) < lb["min_repeaters"]:
            return "REJECTED", (f"claim {val} is below the mandatory minimum "
                                f"{lb['min_repeaters']} (dust decays over 15 cells)")
        if have <= float(val):
            return "REJECTED", f"the route already uses {have} <= claimed {val}"
        for v in sweep["better"]:
            return "ACCEPTED", (f"variant `{v['label']}` is better overall "
                                f"({v['delta_total']} on the weighted total)")
        return "UNVERIFIED", (f"route uses {have}, claim {val} is above the floor "
                              f"{lb['min_repeaters']}, but no constructible variant "
                              f"reached it")

    if kind in NUMERIC:
        key = "length" if kind == "cells" else kind
        have = float(cost.get(key, b.get("cells", 0) or 0))
        if val is None:
            return "UNVERIFIED", "no value to check"
        val = float(val)
        if val >= have:
            return "REJECTED", f"claim {val} is not better than the actual {have}"
        floor = {"length": lb["min_fragment_cells"], "delay_rt": lb["min_delay_rt"]}.get(key)
        if floor is not None and val < floor:
            return "REJECTED", (f"claim {val} is below the geometric floor {floor} "
                                f"(per-bit manhattan sum / mandatory refreshes)")
        for v in sweep["better"]:
            got = float(v["b0_cost"].get(key, 1e18))
            if got <= val:
                return "ACCEPTED", (f"variant `{v['label']}` reaches {key}={got} "
                                    f"<= claimed {val}, sim PASS, DRC clean")
        if sweep["better"]:
            v = sweep["better"][0]
            return "ACCEPTED", (f"claimed {key}<={val} not reached, but variant "
                                f"`{v['label']}` is strictly better overall "
                                f"({v['delta_total']} weighted) — the criticism "
                                f"that this solution is beatable holds")
        return "UNVERIFIED", (f"claim {key}<={val} is above the floor but no "
                              f"constructible variant achieved it")

    if kind == "strategy":
        for v in sweep["better"]:
            if v["label"].startswith("hint"):
                return "ACCEPTED", (f"the critic's own hint routed better: "
                                    f"{v['label']} {v['delta_total']} weighted")
        for v in sweep["better"]:
            return "ACCEPTED", (f"a constructible variant beats the default: "
                                f"{v['label']} {v['delta_total']} weighted")
        return "UNVERIFIED", "no variant, including the critic's hint, improved it"

    return "UNVERIFIED", f"unknown claim kind `{kind}`"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--results", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--work-dir", default="work")
    a = ap.parse_args()
    with open(a.out, "w") as out:
        for line in open(a.results):
            rec = json.loads(line)
            if not rec.get("solved") or ".var_" in rec.get("id", ""):
                continue
            spec = json.load(open(rec["problem_file"]))
            sweep = search_variants(spec, rec, a.work_dir)
            lb = lower_bounds(spec, spec["buses"][0]["name"])
            out.write(json.dumps({"id": rec["id"], "tier": rec.get("tier"),
                                  "axis": spec.get("axis"), "level": spec.get("level"),
                                  "lower_bounds": lb, "sweep": sweep}) + "\n")
            out.flush()
            n = len(sweep["better"])
            print(f"{rec['id']}: {n} better variant(s) of {len(sweep['tried'])}"
                  + (f"  best={sweep['better'][0]['label']} "
                     f"{sweep['better'][0]['delta_total']}" if n else ""), flush=True)


if __name__ == "__main__":
    main()
