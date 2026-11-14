# Simulation performance resumption

Work resumed 15 November 2026 from `ee89008d` (current upstream master),
which includes the published v0.12.2 parity fixes. Development is isolated in
`codex/simulation-performance`; the original local checkout has unrelated edits.
Rust compilation and native benchmarks run on the user's `harrison-work` laptop.

The 14 November handoff describes unpushed optimizations, but their patches,
source snapshots, measurements and graphs were not found in the accessible
local checkout. No connection to the old managed cloud disk is available in
this session; this does **not** establish that its contents were deleted.
The optimizations are being reconstructed, with fresh tests and measurements.
Historical figures in the handoff are not fresh results on this machine.

Planned checkpoints:

1. Reconstruct diagnostics caching, dense state tables, stack dust ordering,
   indexed wire shapes, cached world layout, pure power query simplifications,
   and recording control; validate and push.
2. Add a shared differential/benchmark harness and measure identical binaries
   and workloads on the same host, with randomized paired sampling.
3. Evaluate conservative derived strong-power caching, preserving live
   comparator outputs and immediate invalidation within a tick. Keep explicit
   uncached and shadow modes; retain only if parity and timing justify it.

The existing Minecraft 1.21.11 golden trace is the starting parity reference.
Its hipster fixture retains save data version 2730. A fresh reference-server
run requires the reference archive; it is not a prerequisite for reconstruction.

## Reconstruction checkpoint

Reconstructed all seven retained changes (A–G), with five new regression tests:
sparse state tables, all 20,736 wire shape indices, Java dust ordering over 3,500
coordinate/origin combinations, world layout at negative/extreme coordinates,
and recording disable/restart without altering state or update recording.
No update queue experiments are included. No derived-power cache is included yet.

Fresh validation on harrison-work, Rust 1.98.0 (88d9e12ae):

- Released engine: 410 tests passed, 7 ignored (including doctests).
- Reconstructed engine: 415 tests passed, 7 ignored.
- WebAssembly check: passed (`wasm32-unknown-unknown`).
- Bridge: 30 tests passed.

Full logs are in `validation/`. These counts reflect this reconstructed test
suite, not the historical 446-pass suite whose additional tests were not recovered.

## Workloads and timing boundaries

All simulations use the shared `mc_test::build_sim` loader and RNG seed 0.
Fixtures are committed under `crates/mc-tick/tests/corpus/structures`.

| Case | Setup outside tick timer | Timed work |
|---|---|---|
| hipster | InWorld, save version 2730 | 320 ticks; lever (12,7,13) at 10 and 170 |
| hipster_unrecorded | same, discard ordinary recording | same inputs/ticks |
| hipster_updates | same, ordinary + update recording | same inputs/ticks |
| hipster_timeline | same, timeline recording | same inputs/ticks |
| door6 | InWorld | 100 ticks, lever (10,4,1) at 0 |
| flyer | Quiet; run 2, redstone at (2,1,1), run 2, remove, run 40; assert x>5 | 200 ticks |
| bb | InWorld; remove (31,7,13), run 30; assert activity | 200 ticks |
| adder_idle | Placement; run 200; assert quiescence | 500 ticks |
| hipster_load | embedded fixture text; no file I/O | parse, build, validate; destruction excluded |

Each timed iteration includes inputs, stepping, and the selected recording.
Checkpoint restoration, RNG reset, recording setup, verification, and export are
outside the tick timer. Each subprocess performs one untimed warmup sample with
the same iteration count as its measured sample. Loading is measured separately.

The runner calibrates identical iterations per case from a baseline probe toward
150 ms CPU time, randomizes variant order within each case/round with seed
20261114, and retains all samples. Analysis uses ratios of medians with 10,000
paired bootstrap resamples over rounds. CPU and elapsed time are both retained;
CPU time removes descheduling but does not remove frequency/cache/host effects.
Builds and profiles are excluded from final native timing periods.

## Reproduction

Use a clean checkout of this branch and Rust/Cargo 1.98.0, matching the fresh
measurements. Production runtime dependencies are unchanged; `cpu-time = 1.0.0`
is a development dependency. The lockfile adds only that dependency.

```sh
python3 tools/prepare_tick_variants.py --output /tmp/tick-patches
python3 tools/build_tick_variants.py \
  --worktree /tmp/nucleation-tick-stages \
  --patches /tmp/tick-patches --output /tmp/tick-binaries \
  --target /tmp/nucleation-tick-stage-target
```

The worktree must not exist. Use a distinct Cargo target directory for each
independently changing checkout: sharing one across concurrent worktree builds
can leave Cargo's output executable belonging to another checkout, even when
its source-specific fingerprints are current. The stage builder builds and
copies each variant sequentially and checks its complete differential fingerprint.
It never resets an existing user checkout. Stage patches are cumulative against
`ee89008d`; apply one selected patch, not all sequentially.

```sh
python3 tools/benchmark_tick.py \
  --variant baseline=/tmp/tick-binaries/baseline \
  --variant power=/tmp/tick-binaries/power \
  --cpu 0 --rounds 21 --output /tmp/tick-timings.jsonl
python3 tools/analyze_tick_followup.py /tmp/tick-timings.jsonl \
  --output /tmp/tick-summary.json
python3 tools/plot_tick_benchmarks.py /tmp/tick-summary.json \
  --output /tmp/tick-speedups
```

Choose an actually allowed CPU. The full run adds diagnostics, tables, dust,
wire, world, and cache variants with repeated `--variant` arguments. Cache
experiment patches, bases, and hashes are recorded in `experiments/manifest.json`.
Apply a cache patch in its own clean checkout, use its own target directory,
and pass `--mode cache_on=on` for the named enabled-cache variant. Shadow mode is
for verification only (`MC_TICK_POWER_CACHE=shadow binary --verify`).
