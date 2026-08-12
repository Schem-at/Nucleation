#!/usr/bin/env python
"""Run the bus-solver corpus: one scenario per file, one subprocess each.

    python run_corpus.py                 # every scenario
    python run_corpus.py X01 P02 ...     # only these (prefix match)
    python run_corpus.py --list
    python run_corpus.py --one <id>      # in-process (used by the runner)

Writes ``results/<id>.json`` and ``artifacts/<id>.schem`` per scenario.
A scenario that crashes the router or the engine must not take the corpus
down with it, hence the subprocess per entry.

Needs the wheel built WITH routing:
    NUCLEATION_FEATURES=bridge-full,routing,hdl \\
        <venv>/bin/pip install ./bindings/python
"""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
SCN = os.path.join(HERE, "scenarios")
RESULTS = os.path.join(HERE, "results")


def load_all():
    out = {}
    if SCN not in sys.path:
        sys.path.insert(0, SCN)   # so a scenario can `import _shared`
    for f in sorted(os.listdir(SCN)):
        if not f.endswith(".py") or f.startswith("_"):
            continue
        path = os.path.join(SCN, f)
        spec = importlib.util.spec_from_file_location(f[:-3], path)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        s = mod.SCENARIO
        s.setdefault("id", f[:-3])
        s["_file"] = os.path.relpath(path, HERE)
        out[s["id"]] = s
    return out


def run_one(sid):
    import harness
    scn = load_all()[sid]
    res = harness.run(scn, HERE)
    os.makedirs(RESULTS, exist_ok=True)
    with open(os.path.join(RESULTS, sid + ".json"), "w") as fh:
        json.dump(res, fh, indent=1, sort_keys=True, default=str)
    tag = "SOLVED" if res["solved"] else "UNSOLVED"
    v = res.get("verification") or {}
    print("%-9s %-26s %s  %s" % (
        tag, sid,
        "%d/%d cases" % (v.get("passed", 0), v.get("total", 0)),
        "" if res["solved"] else "<- " + str(res.get("blocked_by", ""))[:110]))
    return 0


def main(argv):
    sys.path.insert(0, HERE)
    if argv[:1] == ["--one"]:
        return run_one(argv[1])
    scns = load_all()
    if "--list" in argv:
        for sid, s in scns.items():
            print("%-26s %s" % (sid, s["title"]))
        return 0
    picks = [s for s in argv if not s.startswith("-")]
    names = [sid for sid in scns
             if not picks or any(sid.startswith(p) for p in picks)]
    ok, bad, t0 = [], [], time.perf_counter()
    for sid in names:
        r = subprocess.run([sys.executable, __file__, "--one", sid],
                           capture_output=True, text=True, cwd=HERE)
        if r.returncode == 0:
            print(r.stdout.rstrip())
            ok.append(sid)
        else:
            tail = (r.stderr.strip().splitlines() or ["<no stderr>"])[-1]
            print("CRASHED   %-26s %s" % (sid, tail[:150]))
            bad.append(sid)
            os.makedirs(RESULTS, exist_ok=True)
            with open(os.path.join(RESULTS, sid + ".json"), "w") as fh:
                json.dump({"id": sid, "title": scns[sid]["title"],
                           "question": scns[sid].get("question", ""),
                           "expect": scns[sid].get("expect", "solved"),
                           "notes": scns[sid].get("notes", ""),
                           "scenario": scns[sid], "solved": False,
                           "harness_crash": r.stderr[-2000:],
                           "blocked_by": "harness crashed: " + tail[:200]},
                          fh, indent=1, sort_keys=True, default=str)
    print("\n%d ran, %d crashed, %.1fs" % (len(ok), len(bad),
                                           time.perf_counter() - t0))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
