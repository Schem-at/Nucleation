#!/usr/bin/env python3
"""Fold the demolition evidence back into the verdicts.

verify.py can only build variants out of knobs the solver exposes, so its
strongest possible verdict on "you refresh too often" is UNVERIFIED — there is no
repeater-pitch knob to turn. prune_check.py answers that question empirically
instead, by deleting repeaters from the router's own output and re-simulating.

This pass re-reads every UNVERIFIED refresh-related claim and adjudicates it
against that evidence, so the final tally reflects everything we actually know.
Verdicts that were already decided are never overturned, and every upgraded row
records which evidence moved it and any inference involved.

  python3 readjudicate.py --critiques critiques.jsonl --prune prune.jsonl \\
      --out critiques_final.jsonl
"""

import argparse
import json
import math


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--critiques", required=True)
    ap.add_argument("--prune", required=True)
    ap.add_argument("--results", default="results.jsonl")
    ap.add_argument("--out", required=True)
    a = ap.parse_args()

    prune = {}
    for line in open(a.prune):
        r = json.loads(line)
        prune[r["id"]] = r
    widths = {}
    for line in open(a.results):
        r = json.loads(line)
        if ".var_" not in r.get("id", ""):
            widths[r["id"]] = (r.get("axes") or {}).get("width", 8)

    moved = 0
    with open(a.out, "w") as out:
        for line in open(a.critiques):
            row = json.loads(line)
            p = prune.get(row.get("id"))
            for v in row.get("verdicts", []) or []:
                if v["verdict"] != "UNVERIFIED" or not p:
                    continue
                kind = v["claim"].get("kind")
                val = v["claim"].get("claim_value")
                if kind not in ("repeaters", "delay_rt") or val is None:
                    continue
                if p["verdict"] == "REJECTED":
                    v["verdict"] = "REJECTED"
                    v["why"] = ("empirically refuted by demolition: " + p["why"]
                                + f" (baseline passed {p.get('baseline_vectors')} "
                                  f"vectors first, so the test was sound)")
                    v["evidence"] = "prune-construction"
                    moved += 1
                elif p["verdict"] == "ACCEPTED":
                    kept = p["kept"]
                    if kind == "repeaters" and float(val) >= kept:
                        v["verdict"] = "ACCEPTED"
                        v["why"] = (f"constructed by demolition: {p['removed']} of "
                                    f"{p['repeaters']} repeaters deleted, {kept} kept, "
                                    f"and all {p['baseline_vectors']} vectors still "
                                    f"arrive in mc-tick")
                        v["evidence"] = "prune-construction"
                        moved += 1
                    elif kind == "delay_rt":
                        w = max(1, widths.get(row["id"], 8))
                        per_bit = math.ceil(kept / w)
                        if float(val) >= 2 * per_bit:
                            v["verdict"] = "ACCEPTED"
                            v["why"] = (f"the refresh count this delay implies was "
                                        f"constructed and verified: {kept} repeaters "
                                        f"kept (~{per_bit}/bit, 2 rt each). NOTE: "
                                        f"inferred from the verified refresh count, "
                                        f"not measured on the pruned build")
                            v["evidence"] = "prune-construction (inferred delay)"
                            moved += 1
            out.write(json.dumps(row) + "\n")
    print(f"re-adjudicated {moved} claim(s) using demolition evidence -> {a.out}")


if __name__ == "__main__":
    main()
