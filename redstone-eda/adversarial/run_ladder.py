#!/usr/bin/env python3
"""SOLVE + VERIFY RUNNER — walk the difficulty ladder, record where it breaks.

For every generated problem: hand it to the Rust bus router (via the
`adv-harness` binary, which owns no files in the main tree), then prove the
result IN SIMULATION across many words and bit patterns -- walking ones, the
all-on word, the alternating pairs and seeded pseudorandom words, with a
crosstalk phase where one bus is hot and its neighbours must stay at zero. One
quiet value validates a leaking pitch, so a single vector is never enough.

Each run appends one JSON record to results.jsonl. Nothing here is adversarial
yet; this is the measurement the critic argues with.

Usage:
  python3 run_ladder.py --problems problems --out results.jsonl [--tiers 1-8]
"""

import argparse
import json
import os
import subprocess
import sys
import time

HARNESS = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "harness", "target", "release", "adv-harness")

# Wall-clock ceiling per problem: a big obstacle field costs the occupancy
# index real time, and a hung solve must not stall the ladder.
TIMEOUT = {1: 120, 2: 120, 3: 180, 4: 300, 5: 420, 6: 600, 7: 900, 8: 900}


def solve_one(path, work_dir, timeout=None, tag=""):
    """Run the harness on one problem file. Never raises: a crash is a result."""
    spec = json.load(open(path))
    t = timeout or TIMEOUT.get(spec.get("tier", 1), 300)
    t0 = time.time()
    try:
        p = subprocess.run([HARNESS, path, "--work-dir", work_dir],
                           capture_output=True, text=True, timeout=t)
    except subprocess.TimeoutExpired:
        return {"id": spec["id"] + tag, "tier": spec["tier"], "family": spec.get("family"),
                "solved": False, "error": "timeout", "wall_s": t}
    wall = round(time.time() - t0, 2)
    if p.returncode != 0 or not p.stdout.strip():
        return {"id": spec["id"] + tag, "tier": spec["tier"], "family": spec.get("family"),
                "solved": False, "error": f"harness exit {p.returncode}",
                "stderr": (p.stderr or "")[-800:], "wall_s": wall}
    try:
        rec = json.loads(p.stdout.strip().splitlines()[-1])
    except Exception as e:
        return {"id": spec["id"] + tag, "tier": spec["tier"], "solved": False,
                "error": f"unparseable harness output: {e}",
                "stdout": p.stdout[-800:], "wall_s": wall}
    rec["wall_s"] = wall
    rec["problem_file"] = path
    rec["axes"] = spec.get("axes", {})
    return rec


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--problems", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--work-dir", default="work")
    ap.add_argument("--tiers", default="1-8")
    ap.add_argument("--timeout", type=int, default=0)
    a = ap.parse_args()
    lo, _, hi = a.tiers.partition("-")
    tiers = set(range(int(lo), int(hi or lo) + 1))
    os.makedirs(a.work_dir, exist_ok=True)
    index = json.load(open(os.path.join(a.problems, "index.json")))
    # tier 0 == the per-axis probes; they are never excluded by a tier range.
    todo = [e for e in index if e["tier"] in tiers or e["tier"] == 0]
    todo.sort(key=lambda e: (e["tier"], e["id"]))
    with open(a.out, "a") as out:
        for e in todo:
            rec = solve_one(e["path"], a.work_dir, a.timeout or None)
            out.write(json.dumps(rec) + "\n")
            out.flush()
            flag = "OK " if rec.get("solved") else "FAIL"
            note = rec.get("error") or rec.get("unsupported") or ""
            if not note and not rec.get("routed", True):
                note = next((b.get("reason", "") for b in rec.get("buses", [])
                             if b.get("state") != "Routed"), "")
            if not note and rec.get("sim", {}) and isinstance(rec.get("sim"), dict) \
                    and not rec["sim"].get("pass", True):
                note = f"sim {rec['sim'].get('failures')} failures of {rec['sim'].get('checks')}"
            print(f"[{flag}] t{rec.get('tier')} {rec.get('id')} "
                  f"{rec.get('wall_s')}s {note[:120]}", flush=True)
    print("done", file=sys.stderr)


if __name__ == "__main__":
    main()
