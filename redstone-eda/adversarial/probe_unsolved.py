#!/usr/bin/env python3
"""THE OTHER HALF OF THE ADVERSARY: attack the walls, not the solutions.

When the router says "no corridor" or "every placement of the level-shift tile
was blocked", there are two possibilities and they need very different fixes:
the problem really is impossible, or the planner simply will not leave its lane
to get past the obstacle. Only a construction can tell them apart.

So we hand codex the failure -- the reason string, the port geometry and an
ASCII map of the obstacle field on the two planes that matter -- and ask for
concrete GATE WAYPOINTS that would make it routable. Then we run the solver
again with exactly those waypoints. If it routes and passes sim, the problem was
never impossible and the finding is "the planner does not stage its own detours".
If it still fails, we have learned that too, cheaply.

These are the only calls that justify high/xhigh effort: the cheap passes have
nothing to work with when there is no solution to criticise.

Usage:
  python3 probe_unsolved.py --results results.jsonl --out unsolved_probes.jsonl \
      --log codex_log.jsonl --effort high --max-calls 3
"""

import argparse
import copy
import json
import os
import time

import critic
from run_ladder import solve_one

HERE = os.path.dirname(os.path.abspath(__file__))


def ascii_map(obs, fixed_axis, fixed_val, u_axis, v_axis, ulo, uhi, vlo, vhi,
              marks, width=118):
    """A downsampled slice of the obstacle field. `marks` is {(u,v): char}."""
    du = max(1, (uhi - ulo + 1) // width)
    dv = 1
    rows = []
    header = f"slice {'xyz'[fixed_axis]}={fixed_val}  " \
             f"{'xyz'[u_axis]} {ulo}..{uhi} (1 char = {du} cells), " \
             f"{'xyz'[v_axis]} {vlo}..{vhi}"
    grid = {}
    for o in obs:
        if o[fixed_axis] != fixed_val:
            continue
        u, v = o[u_axis], o[v_axis]
        if not (ulo <= u <= uhi and vlo <= v <= vhi):
            continue
        grid[((u - ulo) // du, v)] = "#"
    for (u, v), ch in marks.items():
        if ulo <= u <= uhi and vlo <= v <= vhi:
            grid[((u - ulo) // du, v)] = ch
    for v in range(vhi, vlo - 1, -dv):
        row = "".join(grid.get((i, v), ".") for i in range((uhi - ulo) // du + 1))
        rows.append(f"{v:4d} |{row}")
    return header + "\n" + "\n".join(rows)


PROMPT = """A redstone BUS ROUTER refused this problem. Decide whether the problem is
actually impossible, or whether the planner merely failed to stage a detour.

Read: {brief}
Transport rules: {rules}

The router's own failure reason is in the brief. The router accepts GATES:
waypoint columns (bit-0 anchor + the bus's step) that split the route into legs
it plans independently. A gate is the only way a caller can tell it "go through
here". Gate columns must sit in FREE space, on the bus's own step, and each leg
must have room for what it needs (a level shift needs ~8+ cells of straight run
on one horizontal axis, per level).

Propose up to 3 candidate gate sets, best first. Each will be executed verbatim
against the real solver and simulated; guessing costs you nothing but a rejected
row, so prefer waypoints you can justify from the map.

Reply with ONLY this JSON:
{{"assessment": "impossible" | "planner_gap",
  "why": "<= 60 words",
  "candidates": [
    {{"label": "...", "gates": [[x,y,z], ...]}}
  ]}}
"""


def brief_unsolved(spec, rec, path):
    b0 = spec["buses"][0]
    ports = {p["name"]: p for p in spec["ports"]}
    drv, snk = ports[b0["driver"]], ports[b0["sinks"][0]]
    reason = ""
    for b in rec.get("buses", []):
        if b.get("state") != "Routed":
            reason = f"{b.get('state')}: {b.get('reason')}"
            break
    reason = reason or rec.get("unsupported") or rec.get("error") or "?"
    L = ["# A PROBLEM THE ROUTER REFUSED", f"id: {spec['id']}  family: {spec['family']}",
         f"axes: {json.dumps(spec['axes'])}", "",
         "## the router's reason", "```", str(reason), "```", "", "## ports"]
    for p in spec["ports"]:
        L.append(f"* {p['name']} ({p['dir']}) anchor={p['anchor']} step={p['step']} "
                 f"width={p['width']}")
    L += ["", "## buses (routed in this order)"]
    for b in spec["buses"]:
        L.append(f"* {b['name']}: {b['driver']} -> {b['sinks']} "
                 f"gates={[g['anchor'] for g in b['gates']]}")
    obs = spec["obstacles"]
    L += ["", f"## obstacle field: {len(obs)} solid cells", "",
          "`#` solid, `D` driver bit 0, `S` sink bit 0, `.` free.", "```"]
    xs = [drv["anchor"][0], snk["anchor"][0]]
    xlo, xhi = min(xs) - 2, max(xs) + 2
    ylo = min(o[1] for o in obs) if obs else 0
    yhi = max(o[1] for o in obs) if obs else 8
    zlo = min(o[2] for o in obs) if obs else 0
    zhi = max(o[2] for o in obs) if obs else 8
    # plan view at the driver's level
    L.append(ascii_map(obs, 1, drv["anchor"][1], 0, 2, xlo, xhi, zlo, zhi,
                       {(drv["anchor"][0], drv["anchor"][2]): "D",
                        (snk["anchor"][0], snk["anchor"][2]): "S"}))
    L.append("")
    # side view along the driver's lane
    L.append(ascii_map(obs, 2, drv["anchor"][2], 0, 1, xlo, xhi, ylo, yhi,
                       {(drv["anchor"][0], drv["anchor"][1]): "D",
                        (snk["anchor"][0], snk["anchor"][1]): "S"}))
    L += ["```", ""]
    with open(path, "w") as f:
        f.write("\n".join(L) + "\n")
    return path


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--results", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--log", required=True)
    ap.add_argument("--work-dir", default="work3")
    ap.add_argument("--effort", default="high")
    ap.add_argument("--max-calls", type=int, default=3)
    ap.add_argument("--ids", default="")
    a = ap.parse_args()
    os.makedirs(a.work_dir, exist_ok=True)
    rules = critic.extract_rules()
    recs = [json.loads(l) for l in open(a.results)]
    want = set(x for x in a.ids.split(",") if x)

    # One representative per WALL kind: the same reason on twelve problems needs
    # one construction attempt, not twelve.
    def wall_of(r):
        why = (json.dumps(r.get("buses", [])) + str(r.get("unsupported"))
               + str(r.get("error"))).lower()
        for key, tag in (("no corridor", "no_corridor"),
                         ("level-shift tile", "level_shift_blocked"),
                         ("unsupported bus form", "unsupported_form"),
                         ("no encoding knob", "no_hex_carrier"),
                         ("no trunk run aligns", "fanout_trunk"),
                         ("walled in", "endpoint_walled_in"),
                         ("timeout", "timeout")):
            if key in why:
                return tag
        return "other"

    cands = {}
    for r in recs:
        if r.get("solved") or ".var_" in r.get("id", ""):
            continue
        if want and r["id"] not in want:
            continue
        w = wall_of(r)
        # constructions can only help where gates are even meaningful
        if w in ("unsupported_form", "no_hex_carrier"):
            continue
        if w not in cands or (r.get("tier") or 0) < (cands[w].get("tier") or 0):
            cands[w] = r
    order = ["no_corridor", "level_shift_blocked", "fanout_trunk",
             "endpoint_walled_in", "other", "timeout"]
    picks = [cands[k] for k in order if k in cands][:a.max_calls]
    print(f"walls found: {sorted(cands)}; attacking {[p['id'] for p in picks]}",
          flush=True)

    logf, outf = open(a.log, "a"), open(a.out, "a")
    for rec in picks:
        spec = json.load(open(rec["problem_file"]))
        brief = brief_unsolved(spec, rec,
                              os.path.join(a.work_dir, f"{spec['id']}.wall.md"))
        prompt = PROMPT.format(brief=os.path.abspath(brief),
                              rules=os.path.abspath(rules))
        res = critic.call_codex(prompt, a.effort, timeout=1800)
        logf.write(json.dumps({"id": rec["id"], "effort": a.effort, "ok": res["ok"],
                               "tokens": res["tokens"], "wall_s": res["wall_s"],
                               "error": res["error"], "purpose": "unsolved_wall",
                               "justification": "no solution exists to criticise; "
                               "only a construction can separate `impossible` from "
                               "`planner gap`", "ts": time.time()}) + "\n")
        logf.flush()
        if not res["ok"]:
            outf.write(json.dumps({"id": rec["id"], "wall": wall_of(rec),
                                   "codex_error": res["error"]}) + "\n")
            outf.flush()
            print(f"{rec['id']}: CODEX FAILED {res['error']}", flush=True)
            continue
        reply = res["reply"] or {}
        tried = []
        for c in (reply.get("candidates") or [])[:3]:
            s = copy.deepcopy(spec)
            drv = next(p for p in s["ports"] if p["name"] == s["buses"][0]["driver"])
            s["buses"][0]["gates"] = [
                {"name": f"cg{i}", "anchor": list(g), "step": list(drv["step"])}
                for i, g in enumerate(c.get("gates") or [])]
            label = str(c.get("label", "cand"))[:24].replace(" ", "_")
            s["id"] = f"{spec['id']}.wall_{label}"
            path = os.path.join(a.work_dir, s["id"] + ".json")
            with open(path, "w") as f:
                json.dump(s, f)
            r2 = solve_one(path, a.work_dir)
            ok = (r2.get("solved") and r2.get("drc_lvs_clean")
                  and isinstance(r2.get("sim"), dict) and r2["sim"].get("pass"))
            tried.append({"label": label, "gates": c.get("gates"),
                          "routed": bool(r2.get("routed")), "solved_and_verified": bool(ok),
                          "reason": next((b.get("reason") for b in r2.get("buses", [])
                                          if b.get("state") != "Routed"), None)})
            print(f"  {rec['id']} candidate {label}: "
                  f"{'ROUTED+VERIFIED' if ok else ('routed' if r2.get('routed') else 'failed')}",
                  flush=True)
        won = any(t["solved_and_verified"] for t in tried)
        outf.write(json.dumps({"id": rec["id"], "wall": wall_of(rec),
                               "effort": a.effort, "tokens": res["tokens"],
                               "assessment": reply.get("assessment"),
                               "why": reply.get("why"), "candidates": tried,
                               "verdict": "PLANNER_GAP_PROVEN" if won else "UNVERIFIED"})
                   + "\n")
        outf.flush()
        print(f"{rec['id']}: {'PLANNER GAP PROVEN' if won else 'unverified'}", flush=True)


if __name__ == "__main__":
    main()
