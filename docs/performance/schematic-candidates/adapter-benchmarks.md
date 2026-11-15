# Reusable build adapters and fresh timings

15 November 2026. These measurements ran on **harrison-work**, AMD Ryzen AI 9
HX 370, Linux x86-64, Rust 1.98.0. The native thread was pinned to permitted CPU
0. No builds, profilers, reference servers or thumbnail jobs ran concurrently.
These are new workload baselines, **not a comparison against master** and not
a claim of an optimization speedup.

Each source-pinned [adapter](https://github.com/Schem-at/Nucleation/blob/v0.12.3/circuit-adapters/README.md) uses real lever
interactions and the actual mc-tick world. Its profile embeds the existing
`CellContract`. Expected arithmetic is computed independently, not by replacing
the circuit. The door's aperture was independently checked in Minecraft 1.21.11.

| Workload | Work per suite | Median elapsed time | Operations/s |
| --- | --- | ---: | ---: |
| ADD005 8-bit adder | 328 operand/carry vectors, 10,496 game ticks | 201.098 ms | 1,631 |
| OTHERCOMB017 comparator | 40 operand pairs, 960 game ticks | 2.107 ms | 18,988 |
| Plot lamp-panel 2×2 | 4 close/open commands, 640 game ticks | 0.130 ms | 30,820 |

An operation is one input vector or door command followed by its declared fixed
response window. Those windows deliberately include settling/idle time: 32,
24 and 160 game ticks respectively. Consequently simulated tick rates
(52,194; 455,703; 4,931,240 TPS) are **workload-specific**, especially the tiny
mostly-idle door. They do not measure Minecraft server TPS, graphical frame
rate, minimal circuit latency, or performance of the hipster benchmark.

There are 21 rounds per case, randomized/interleaved with seed 20261115, for 63
samples. Calibration targets 150 ms/sample, capped at 1,000 iterations. Actual
iterations/sample were 1, 71 and 1,000. No outliers were removed. Each process
performs full functional and exact checkpoint replay checks, then an untimed
warmup with the same iteration count. Every measured result is checked outside
its timer. Clock is monotonic elapsed time, not CPU time.

The timed section includes typed input validation/encoding, real lever actions,
ticks, named output reads and result collection. Parsing, construction, setup,
checkpoint restore, assertions, result destruction and rendering are excluded.
Recording is disabled. Random block ticks are disabled, seed is 0, and each
profile explicitly records the Java dust hash origin.

All process records were audited to have identical source, profile, suite and
binary hashes within each workload. Raw per-process receipts remain in the local
archive `validation/adapter-benchmark-final/runs/`. Git retains [all samples](validation/adapter-benchmark/records.json),
[host/compiler/binary metadata](validation/adapter-benchmark/metadata.json) and
[the summary](validation/adapter-benchmark/summary.json). The single binary hash
is recorded in those files; no unmeasured compiler or build variant is mixed in.

The adder passes 656 output assertions per run; comparator passes 120, and door
passes four 4-bit aperture masks. Each repeats with identical outputs and full
engine checkpoint after reset. The adder is not yet covered by a fresh vanilla
trace; its evidence is arithmetic endpoint functionality. The comparator and
door additionally have separately retained vanilla capture evidence. None of
these finite suites establish exhaustive correctness for all inputs.

All 803 plot candidates also underwent a separate native survival/activity
assessment after the lamp/trapdoor observer fix. Its results are not substituted
for these adapter assertions. See [plot assessment](plot-simulation-method.md)
and [vanilla checks](plot-validation/ORACLES.md) for the four remaining sampled
engine mismatches and the hollow-door functional rejection.
