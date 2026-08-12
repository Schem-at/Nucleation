#!/usr/bin/env python3
"""ADVERSARIAL CRITIC — codex proposes, the verifier disposes.

For each SOLVED problem we hand codex the problem, the solver's actual solution
(metrics + geometry summary + the full cell list on disk) and the transport rules
that constrain it, and ask for a STRICTLY BETTER solution or a concrete argument
that this one is needlessly bad.

Every claim then goes through verify.py, which either proves it impossible,
measures it, or tries to CONSTRUCT it with the solver's own knobs. Nothing is
counted as a finding until that succeeds. Plausible-but-wrong is the expected
failure mode of an LLM critic; filtering it is the whole point of the loop.

EFFORT TIERING (and why): the verifier — not the critic's reasoning effort — is
what guarantees quality, so a cheap critic that emits more hypotheses is better
value than an expensive one that emits few. Default is `low`; `medium` is the
second look at problems a cheap pass found nothing on; `high` only for the
hardest; `xhigh` essentially never. Every call logs its effort and token cost so
the report can say empirically whether effort bought accepted critiques.

Usage:
  python3 critic.py --results results.jsonl --problems problems \
      --out critiques.jsonl --log codex_log.jsonl --max-calls 60
"""

import argparse
import json
import os
import re
import subprocess
import time

import verify

HERE = os.path.dirname(os.path.abspath(__file__))
TRANSPORT = os.path.join(os.path.dirname(HERE), "TRANSPORT_MODEL.md")

SCHEMA = """{
  "verdict": "beatable" | "reasonable",
  "reasoning_summary": "<= 60 words",
  "claims": [
    {"kind": "cells"|"length"|"delay_rt"|"skew_rt"|"coherence"|"footprint"
             |"repeaters"|"detour"|"strategy",
     "bus": "<bus name>",
     "claim_value": <number>,
     "argument": "<= 40 words, the mechanism, not an adjective",
     "gate_at": [x,y,z],              // optional, only for kind=strategy
     "route_order": ["b0","c1",...]   // optional, only for kind=strategy
    }
  ]
}"""

PROMPT = """You are auditing a Minecraft-redstone BUS ROUTER's output, adversarially.

Read these two files (they are small):
  {brief}
  {rules}
The full cell list of the routed geometry is at:
  {geom}

Your job: find a STRICTLY BETTER solution to the SAME problem, or argue that
this solution is needlessly bad and say concretely what to do instead.

Rules of engagement:
* The problem is FIXED. Port anchors, steps, widths, obstacles and the set of
  buses may not change. You may only criticise the ROUTE.
* Every claim you make will be mechanically checked: a numeric claim below the
  stated geometric floor is thrown out, and a "better solution exists" claim is
  only counted if it can actually be built. Vague claims score nothing.
* If the solution is already close to the floor, say so: "reasonable" with no
  claims is a valid and useful answer. Do not invent a critique to look useful.
* The most valuable claim kind is `strategy` WITH a concrete `gate_at` waypoint
  or a `route_order`, because those can be executed directly.

Reply with ONLY this JSON, no prose around it:
{schema}
"""


# ---------------------------------------------------------------------------
def extract_rules():
    """A curated excerpt of TRANSPORT_MODEL.md: the mechanism table and the
    rules it separates. Passing the path to the whole file makes codex read a
    lot of tokens it does not need."""
    out = os.path.join(HERE, "work", "rules.md")
    if os.path.exists(out):
        return out
    text = open(TRANSPORT).read().splitlines()
    keep, on = [], False
    for ln in text:
        if ln.startswith("## THE MECHANISM TABLE"):
            on = True
        elif ln.startswith("## ") and on and "MECHANISM" not in ln and "CONFLATED" not in ln:
            if len(keep) > 40:
                break
        if on:
            keep.append(ln)
        if len(keep) > 220:
            break
    body = "\n".join(keep) if keep else "(transport model unavailable)"
    with open(out, "w") as f:
        f.write("# TRANSPORT RULES (excerpt of redstone-eda/TRANSPORT_MODEL.md)\n\n")
        f.write(body)
        f.write("""

# COST TERMS (src/design.rs BusCostVector)
* length     realized cells.
* delay_rt   worst-bit arrival, redstone ticks. A repeater at delay=1 is 2 rt.
* skew_rt    max-min arrival across bits. Matched skew is a requirement.
* coherence  bundle dispersion: cross-section area ABOVE the canonical form's,
             summed per slice, plus a fixed charge per form conversion. 0 means
             the bits travelled together the whole way as ONE object.
* footprint  occupied volume including a one-cell clearance shell.
Default weights when the router compares two routes:
  length 1, delay 4, skew 8, coherence 6, footprint 0.5.
""")
    return out


def write_brief(spec, rec, lb, path):
    b0 = spec["buses"][0]["name"]
    ov = verify.overshoot(spec, rec, b0)
    lines = ["# PROBLEM", f"id: {spec['id']}  family: {spec['family']}",
             f"axes: {json.dumps(spec['axes'])}", "", "## ports"]
    for p in spec["ports"]:
        lines.append(f"* {p['name']} ({p['dir']}) anchor={p['anchor']} "
                     f"step={p['step']} width={p['width']}")
    lines += ["", "## buses (routed in THIS order; each later bus must avoid "
              "what the earlier ones took)"]
    for b in spec["buses"]:
        lines.append(f"* {b['name']}: {b['driver']} -> {b['sinks']} "
                     f"gates={[g['anchor'] for g in b['gates']]}")
    obs = spec["obstacles"]
    lines += ["", "## obstacle field",
              f"{len(obs)} solid cells, kinds={sorted(set(o[3] for o in obs))}"]
    if obs:
        lo = [min(o[i] for o in obs) for i in range(3)]
        hi = [max(o[i] for o in obs) for i in range(3)]
        lines.append(f"bbox {lo} .. {hi}; shapes generated: {spec['axes']['obstacles']}")
        xs = {}
        for o in obs:
            xs[o[0]] = xs.get(o[0], 0) + 1
        dense = sorted(xs.items(), key=lambda kv: -kv[1])[:8]
        lines.append(f"densest x slices (x, cells): {dense}")
    lines += ["", "# THE SOLVER'S SOLUTION"]
    for b in rec.get("buses", []):
        lines.append(f"* {b['name']}: {b.get('state')} cells={b.get('cells')} "
                     f"cost={json.dumps(b.get('cost', {}))} bbox={b.get('bbox')} "
                     f"runs={b.get('runs')} segments={b.get('segments')}")
        lines.append(f"  blocks: {json.dumps(b.get('block_kinds', {}))}")
    lines += [f"DRC/LVS clean: {rec.get('drc_lvs_clean')}  "
              f"sim: {json.dumps(rec.get('sim'))}",
              f"solve time: {rec.get('solve_ms')} ms",
              f"overshoot of `{b0}` beyond its own endpoint bbox "
              f"[-x,+x,-y,+y,-z,+z]: {ov}", "",
              "# HARD FLOORS for bus " + b0 + " (nothing below these is constructible)",
              json.dumps(lb, indent=1)]
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")
    return path


LAST_JSON = re.compile(r"\{.*\}", re.S)


def parse_reply(text, keys=("verdict", "assessment")):
    """Last balanced JSON object in codex's output that carries one of `keys`.

    codex echoes its final message after a `tokens used` line, and may wrap it in
    prose; scanning for the last balanced object that looks like our schema is
    more robust than trusting the layout."""
    depth, start, best = 0, None, None
    for i, ch in enumerate(text):
        if ch == "{":
            if depth == 0:
                start = i
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0 and start is not None:
                cand = text[start:i + 1]
                try:
                    obj = json.loads(cand)
                    if isinstance(obj, dict) and any(k in obj for k in keys):
                        best = obj
                except Exception:
                    pass
    return best


TOKENS = re.compile(r"tokens used[\s:]*\n?\s*([\d,]+)")


def call_codex(prompt, effort, timeout=900):
    t0 = time.time()
    try:
        p = subprocess.run(
            ["codex", "exec", "--sandbox", "read-only",
             "-c", f'model_reasoning_effort="{effort}"', prompt],
            capture_output=True, text=True, timeout=timeout, stdin=subprocess.DEVNULL)
        out = (p.stdout or "") + "\n" + (p.stderr or "")
        rc = p.returncode
    except subprocess.TimeoutExpired:
        return {"ok": False, "error": "timeout", "effort": effort,
                "wall_s": round(time.time() - t0, 1), "tokens": None, "reply": None}
    except Exception as e:                                    # codex missing, etc.
        return {"ok": False, "error": str(e), "effort": effort,
                "wall_s": round(time.time() - t0, 1), "tokens": None, "reply": None}
    m = TOKENS.search(out)
    tokens = int(m.group(1).replace(",", "")) if m else None
    reply = parse_reply(out)
    return {"ok": reply is not None, "error": None if reply else f"unparseable (rc={rc})",
            "effort": effort, "wall_s": round(time.time() - t0, 1),
            "tokens": tokens, "reply": reply, "raw_tail": out[-600:] if reply is None else None}


# ---------------------------------------------------------------------------
def coarse_key(spec):
    """Family key for de-duplication: spending six calls on six near-identical
    tier-1 problems buys nothing."""
    a = spec["axes"]
    return (spec.get("tier"), spec.get("axis"), a["src_form"], a["dst_form"],
            tuple(a["obstacles"]), a["competitors"], a["gates"], a["fanout"],
            a["perm"], a["carrier"], a["dy"] > 0, a["dogleg"] > 0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--results", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--log", required=True)
    ap.add_argument("--work-dir", default="work")
    ap.add_argument("--max-calls", type=int, default=60)
    ap.add_argument("--pass", dest="passes", default="low,medium,high")
    a = ap.parse_args()

    rules = extract_rules()
    recs = [json.loads(l) for l in open(a.results)]
    solved = [r for r in recs if r.get("solved") and ".var_" not in r.get("id", "")]

    # one representative per coarse family, the biggest span in the group
    groups = {}
    for r in solved:
        spec = json.load(open(r["problem_file"]))
        k = coarse_key(spec)
        cur = groups.get(k)
        if cur is None or spec["axes"]["span"] > cur[1]["axes"]["span"]:
            groups[k] = (r, spec)
    reps = sorted(groups.values(), key=lambda rs: (-(rs[1]["axes"]["span"]),
                                                   rs[0]["id"]))
    print(f"{len(solved)} solved, {len(reps)} representative families", flush=True)

    calls = 0
    if os.path.exists(a.log):
        calls = sum(1 for _ in open(a.log))
    logf = open(a.log, "a")
    outf = open(a.out, "a")
    # Resume-safe: never pay twice for the same (problem, effort), and let a
    # later pass see what an earlier INVOCATION already found.
    done, prior = set(), {}
    if os.path.exists(a.out):
        for l in open(a.out):
            try:
                row = json.loads(l)
            except Exception:
                continue
            done.add((row.get("id"), row.get("effort")))
            if row.get("verdicts"):
                prior.setdefault(row["id"], []).extend(row["verdicts"])

    def critique(rec, spec, effort):
        nonlocal calls
        if calls >= a.max_calls:
            return None
        if (rec["id"], effort) in done:
            return None
        b0 = spec["buses"][0]["name"]
        lb = verify.lower_bounds(spec, b0)
        brief = write_brief(spec, rec, lb,
                            os.path.join(a.work_dir, f"{spec['id']}.brief.md"))
        geom = (verify.bus_of(rec, b0) or {}).get("geom_file", "(none)")
        prompt = PROMPT.format(brief=os.path.abspath(brief),
                               rules=os.path.abspath(rules),
                               geom=os.path.abspath(geom) if geom != "(none)" else geom,
                               schema=SCHEMA)
        res = call_codex(prompt, effort)
        calls += 1
        logf.write(json.dumps({"id": rec["id"], "effort": effort, "ok": res["ok"],
                               "tokens": res["tokens"], "wall_s": res["wall_s"],
                               "error": res["error"], "call_index": calls,
                               "ts": time.time()}) + "\n")
        logf.flush()
        if not res["ok"]:
            outf.write(json.dumps({"id": rec["id"], "effort": effort,
                                   "codex_error": res["error"],
                                   "raw_tail": res.get("raw_tail")}) + "\n")
            outf.flush()
            print(f"  [{effort}] {rec['id']}: CODEX FAILED ({res['error']})", flush=True)
            return None

        reply = res["reply"]
        claims = reply.get("claims") or []
        hints = [c for c in claims if c.get("gate_at") or c.get("route_order")]
        sweep = verify.search_variants(spec, rec, a.work_dir, hints=hints)
        verdicts = []
        for c in claims:
            v, why = verify.judge(spec, rec, c, sweep, lb)
            verdicts.append({"claim": c, "verdict": v, "why": why})
        row = {"id": rec["id"], "tier": rec.get("tier"), "axis": spec.get("axis"),
               "level": spec.get("level"), "family": spec["family"],
               "effort": effort, "tokens": res["tokens"], "wall_s": res["wall_s"],
               "codex_verdict": reply.get("verdict"),
               "summary": reply.get("reasoning_summary"),
               "verdicts": verdicts,
               "variant_sweep": {"base_total": sweep["base_total"],
                                 "better": sweep["better"],
                                 "tried": [t["label"] for t in sweep["tried"]]}}
        outf.write(json.dumps(row) + "\n")
        outf.flush()
        acc = sum(1 for v in verdicts if v["verdict"] == "ACCEPTED")
        rej = sum(1 for v in verdicts if v["verdict"] == "REJECTED")
        unv = sum(1 for v in verdicts if v["verdict"] == "UNVERIFIED")
        print(f"  [{effort}] {rec['id']}: {reply.get('verdict')} "
              f"A{acc}/R{rej}/U{unv} tok={res['tokens']} {res['wall_s']}s", flush=True)
        return row

    passes = a.passes.split(",")

    # PASS A -- low effort on every representative family.
    rows = {}
    if "low" in passes:
        for rec, spec in reps:
            r = critique(rec, spec, "low")
            if r:
                rows[rec["id"]] = r

    # PASS B -- medium, only where the cheap pass found nothing verifiable and
    # the problem is not trivial.
    if "medium" in passes:
        for rec, spec in reps:
            vs = (rows.get(rec["id"], {}).get("verdicts") or []) \
                + prior.get(rec["id"], [])
            got = any(v["verdict"] == "ACCEPTED" for v in vs)
            hard = (spec.get("tier") or 0) >= 3 or spec["axes"]["competitors"] >= 2 \
                or spec["axes"]["span"] >= 96 or spec["axes"]["width"] >= 16
            if not got and hard:
                critique(rec, spec, "medium")

    # PASS C -- high, only the hardest handful.
    if "high" in passes:
        hardest = sorted(reps, key=lambda rs: -(
            (rs[1].get("tier") or 0) * 100 + rs[1]["axes"]["competitors"] * 20
            + rs[1]["axes"]["span"] // 10 + rs[1]["axes"]["width"]))[:5]
        for rec, spec in hardest:
            critique(rec, spec, "high")

    print(f"codex calls used: {calls}", flush=True)


if __name__ == "__main__":
    main()
