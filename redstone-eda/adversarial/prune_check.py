#!/usr/bin/env python3
"""CONSTRUCT-BY-DEMOLITION: the one verification the solver's API cannot give.

codex's most interesting recurring critique is "the router refreshes too often":
it places a repeater roughly every 8 cells on a straight run, where dust reaches
15. There is no repeater-pitch knob to turn, so verify.py can only mark that
UNVERIFIED. This script settles it EMPIRICALLY instead, in mc-tick, using the
`rs.py` conventions (DUST / EXTRA_STATES / lever toggling / lamp reads):

  1. open the flattened .schem the harness already exported;
  2. drive the driver's levers and read the sink's lamps, and confirm the
     BASELINE passes every vector (walking ones, all-on, alternating pairs,
     seeded pseudorandom) -- if the baseline does not pass here, the check is
     void and says so;
  3. DELETE repeaters, replacing each with plain dust, until each lane's
     refresh pitch is as wide as the claim says is enough;
  4. re-run every vector.

Still passing => the removed repeaters were provably redundant, and the critique
is ACCEPTED with a construction. Failing => REJECTED with the exact word and bit
that broke, which is worth more than the critique was: it is the empirical proof
that the conservative pitch is load-bearing.

Usage:
  python3 prune_check.py --results results.jsonl [--ids p_span_6,...] \
      [--pitch 15] --out prune.jsonl
"""

import argparse
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))          # redstone-eda/, for rs.py

import rs                                           # noqa: E402
import nucleation as n                              # noqa: E402


def words_for(width, seed):
    """Byte-for-byte the harness's vector set, so a pass here means the same
    thing as a pass there."""
    mask = 0xFFFFFFFF if width >= 32 else (1 << width) - 1
    v = [1 << i for i in range(min(width, 32))]
    v += [w & mask for w in (0, mask, 0xAAAAAAAA, 0x55555555, 0x33333333, 0x0F0F0F0F)]
    x = seed | 1
    for _ in range(4):
        x ^= (x << 13) & 0xFFFFFFFFFFFFFFFF
        x ^= x >> 7
        x ^= (x << 17) & 0xFFFFFFFFFFFFFFFF
        v.append((x & 0xFFFFFFFF) & mask)
    out = []
    for w in v:
        if w not in out:
            out.append(w)
    return out


def all_blocks(schem):
    return {(b["x"], b["y"], b["z"]): b["name"]
            for b in json.loads(schem.get_all_blocks_json())}


def find_offset(blocks, expected):
    """The .schem save normalises to non-negative bounds, so everything is
    shifted by a constant. Recover it from the lever bank instead of trusting
    the design coordinates (the lesson from the parametric-adder work)."""
    got = sorted(p for p, b in blocks.items() if "lever" in b)
    exp = sorted(expected)
    if len(got) != len(exp) or not got:
        return None
    off = tuple(got[0][i] - exp[0][i] for i in range(3))
    for g, e in zip(got, exp):
        if tuple(g[i] - e[i] for i in range(3)) != off:
            return None
    return off


class Driver:
    """Levers driven the way a player drives them: one at a time, settling
    between, because flipping several inside one tick injects transients a
    ripple can latch."""

    def __init__(self, sim, levers, lamps, settle=800):
        self.sim, self.levers, self.lamps, self.settle_n = sim, levers, lamps, settle
        self.state = [self._powered(p) for p in levers]

    def _powered(self, p):
        return "powered=true" in self.sim.get_block(*p)

    def set(self, word):
        for i, p in enumerate(self.levers):
            want = bool((word >> i) & 1)
            if self.state[i] != want:
                self.sim.use_block(*p)
                self.state[i] = want
                self.sim.run_until_quiescent(self.settle_n)
        self.sim.run_until_quiescent(self.settle_n)

    def read(self):
        v = 0
        for i, p in enumerate(self.lamps):
            if "lit=true" in self.sim.get_block(*p):
                v |= 1 << i
        return v


def build_sim(schem, settle=4000):
    sim = n.TickSimulation.from_schematic(schem, n.TickSettleMode.Placement,
                                          0, 0, 0, rs.EXTRA_STATES)
    sim.run_until_quiescent(settle)
    return sim


def check(schem, levers, lamps, words):
    sim = build_sim(schem)
    d = Driver(sim, levers, lamps)
    fails = []
    for w in words:
        d.set(w)
        got = d.read()
        if got != w:
            fails.append({"word": w, "got": got})
    return fails


def run_one(rec, spec, pitch):
    """One problem: baseline, then pruned. Single-bus only -- with several buses
    the lever bank cannot be attributed by scanning alone."""
    out = {"id": rec["id"], "pitch": pitch}
    if len(spec["buses"]) != 1 or len(spec["buses"][0]["sinks"]) != 1:
        return {**out, "verdict": "SKIPPED", "why": "multi-bus / fanout"}
    schem_path = rec.get("schem_file")
    if not schem_path or not os.path.exists(schem_path):
        return {**out, "verdict": "SKIPPED", "why": "no exported schem"}

    ports = {p["name"]: p for p in spec["ports"]}
    drv = ports[spec["buses"][0]["driver"]]
    snk = ports[spec["buses"][0]["sinks"][0]]

    def cells(p):
        a, st, w = p["anchor"], p["step"], p["width"]
        return [(a[0] + st[0] * k, a[1] + st[1] * k, a[2] + st[2] * k) for k in range(w)]

    exp_levers = [(c[0] + drv["out"][0], c[1] + drv["out"][1], c[2] + drv["out"][2])
                  for c in cells(drv)]
    exp_lamps = [(c[0], c[1] - 1, c[2]) for c in cells(snk)]

    schem = n.Schematic.open(schem_path)
    blocks = all_blocks(schem)
    off = find_offset(blocks, exp_levers)
    if off is None:
        return {**out, "verdict": "SKIPPED", "why": "could not recover the schem offset"}
    sh = lambda p: (p[0] + off[0], p[1] + off[1], p[2] + off[2])   # noqa: E731
    levers = [sh(p) for p in exp_levers]
    lamps = [sh(p) for p in exp_lamps]
    words = words_for(drv["width"], rec.get("seed", 1))

    base_fails = check(schem, levers, lamps, words)
    out["baseline_vectors"] = len(words)
    out["baseline_failures"] = len(base_fails)
    if base_fails:
        return {**out, "verdict": "VOID",
                "why": f"the baseline itself fails {len(base_fails)} vector(s) through "
                       f"this independent Python path: {base_fails[:3]}"}

    # ---- prune ----
    reps = sorted([p for p, b in blocks.items() if b.endswith("repeater")])
    lanes = {}
    for p in reps:
        lanes.setdefault((p[1], p[2]), []).append(p)
    removed = []
    for lane, ps in lanes.items():
        ps.sort()
        last = None
        for p in ps:
            if last is not None and p[0] - last < pitch - 1:
                removed.append(p)
            else:
                last = p[0]
    if not removed:
        return {**out, "verdict": "N/A", "why": f"no repeater is closer than {pitch} "
                                                f"cells to its neighbour already",
                "repeaters": len(reps)}
    pruned = n.Schematic.open(schem_path)
    for p in removed:
        pruned.set_block_from_string(p[0], p[1], p[2], rs.DUST)
    fails = check(pruned, levers, lamps, words)
    out.update({"repeaters": len(reps), "removed": len(removed),
                "kept": len(reps) - len(removed), "pruned_failures": len(fails),
                "first_failures": fails[:3]})
    if not fails:
        out["verdict"] = "ACCEPTED"
        out["why"] = (f"{len(removed)} of {len(reps)} repeaters removed (refresh pitch "
                      f"widened to >= {pitch}) and all {len(words)} vectors still "
                      f"arrive: those repeaters were provably redundant")
    else:
        out["verdict"] = "REJECTED"
        out["why"] = (f"widening the refresh pitch to {pitch} breaks "
                      f"{len(fails)}/{len(words)} vectors — the conservative pitch is "
                      f"load-bearing, and the critique is wrong")
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--results", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--ids", default="")
    ap.add_argument("--pitch", type=int, default=15)
    a = ap.parse_args()
    want = set(x for x in a.ids.split(",") if x)
    recs = [json.loads(l) for l in open(a.results)]
    with open(a.out, "a") as out:
        for r in recs:
            if ".var_" in r.get("id", "") or not r.get("solved"):
                continue
            if want and r["id"] not in want:
                continue
            spec = json.load(open(r["problem_file"]))
            try:
                row = run_one(r, spec, a.pitch)
            except Exception as e:
                row = {"id": r["id"], "verdict": "ERROR", "why": repr(e)}
            out.write(json.dumps(row) + "\n")
            out.flush()
            print(f"{row['id']}: {row['verdict']} — {row.get('why', '')[:150]}",
                  flush=True)


if __name__ == "__main__":
    main()
