# adversarial/ — procedural problem generation, solve+verify, and a critic that gets fact-checked

Read [`REPORT.md`](REPORT.md) first; it is the output. This file explains the parts.

The loop exists because "we just need smarter bussing" is not measurable. So:
invent problems the solver has never seen, at escalating difficulty; make it
solve them; prove the solutions in the real engine rather than trusting the
router's own "Routed"; then attack the solutions — first with a search the
solver's own API can run, then with an LLM critic whose every claim is
mechanically checked before it is allowed to count.

| file | what it does |
|---|---|
| `gen_problems.py` | Seeded, deterministic problem generator. Twelve difficulty axes, documented in its docstring. Two modes: `ladder` (compound tiers 1-8) and `probe` (one axis at a time, so a failure names the capability instead of the tier). Problems are valid by construction — no overlapping port hardware, no walled-in anchor. |
| `harness/` | A small Rust binary, OUTSIDE the nucleation workspace so it owns nothing in the main tree. Builds the hardware and obstacle field, calls `Design::route_bus`, measures the result (cost vector, bbox, block kinds), runs DRC/LVS, and proves the route in the vanilla-accurate engine across many words with an isolation/crosstalk phase. |
| `run_ladder.py` | Walks the problem set, one subprocess per problem with a timeout, appends `results.jsonl`. A crash or hang is a result, not an outage. |
| `verify.py` | The adjudicator. Hard geometric floors (per-bit manhattan, mandatory refreshes), direct measurements (detour overshoot, repeater count), and a CONSTRUCTION search that re-solves with the knobs the solver itself exposes (routing order, gate waypoints, sink order). Useful with no critic at all. |
| `critic.py` | Hands codex the problem, the solution and the transport rules, and asks for something strictly better. Effort-tiered (low by default), budget-capped, every call logged with its token cost. |
| `prune_check.py` | The verifier channel the API could not provide: deletes repeaters from the router's own output and re-simulates through an independent Python path (`rs.py` conventions). This is what turned the best critique from a hypothesis into a finding — and what refuted it on form conversions. |
| `probe_unsolved.py` | Attacks the walls instead of the solutions: gives codex the refusal plus an ASCII map, asks for gate waypoints, then executes them against the real solver to separate "impossible" from "planner gap". |
| `readjudicate.py` | Folds the demolition evidence back into verdicts that were UNVERIFIED only because no knob existed. Never overturns a decided verdict. |
| `make_report.py` | Pure aggregation into `REPORT.md`. Every number traces to a run. |

Evidence files: `results.jsonl` (every solve + sim), `variants.jsonl` (every
construction attempt), `critiques.jsonl` / `critiques_final.jsonl` (every claim
and its verdict), `prune.jsonl`, `unsolved_probes.jsonl`, `codex_log.jsonl`
(every call, its effort and its token cost).

## Reproducing

```sh
cd harness && CARGO_TARGET_DIR=$PWD/target cargo build --release && cd ..
python3 gen_problems.py --out problems --mode both --per-tier 6 --seed 20260810
python3 run_ladder.py  --problems problems --out results.jsonl --work-dir work
python3 verify.py      --results results.jsonl --out variants.jsonl
python3 prune_check.py --results results.jsonl --out prune.jsonl --pitch 15
python3 critic.py      --results results.jsonl --out critiques.jsonl \
                       --log codex_log.jsonl --work-dir work2 --max-calls 37 --pass low
python3 probe_unsolved.py --results results.jsonl --out unsolved_probes.jsonl \
                       --log codex_log.jsonl --effort high --max-calls 3
python3 readjudicate.py --critiques critiques.jsonl --prune prune.jsonl \
                       --out critiques_final.jsonl
python3 make_report.py
```

The generator is a pure function of its seed, so every id in the report
regenerates byte-identically.

## The one discipline that matters here

A critique is a hypothesis. Plausible-but-wrong is the expected output of an LLM
critic, and in this run it was the majority output — 24 of 61 claims could not be
checked at all and 5 were checked and false. Nothing is counted until it is
proven impossible, measured, or constructed and re-simulated. The value of the
loop is in the filter, not in the critic.
