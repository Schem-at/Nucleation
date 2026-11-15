# Independently reversible simulation experiments

Prepared 15 November 2026. Base: `48765c09`, the previous optimized engine.
Branch: `codex/simulation-execution-experiments`. All Rust compilation and native
measurement use `harrison-work`; the original dirty Mac checkout is untouched.
No package release or default feature change is made by this research branch.

## Final result

The recommended opt-in combination is **directional queries + eager buffer reuse
+ per-world input plans**. Cursor, typed dispatch and certificates remain disabled
experiments. The reference remains the default; no package was published.

Fresh hipster results (320 ticks, ordinary recording) from the same 21-round run:

| Build | CPU ms/cycle | Elapsed ms/cycle | Elapsed simulated TPS |
| --- | ---: | ---: | ---: |
| Reference | 13.7556 | 13.8024 | 23,184 |
| Recommended source | 10.3834 | 10.4217 | 30,705 |
| Reference + PGO | 12.4839 | 12.5104 | 25,579 |
| Recommended source + PGO | 9.1880 | 9.2169 | 34,719 |

The source gain is **1.325× [1.285, 1.346]**. With PGO applied to both reference
and candidate, the gain is **1.359× [1.347, 1.397]**. These are paired CPU-time
ratios; elapsed throughput is shown above. This is headless circuit simulation,
not rendering or Minecraft server TPS. **The requested further 10× is not achieved.**

The gain is not universal. The standard candidate reduces throughput on the
explicitly idle adder by about 1.8%. Against the PGO reference, candidate PGO
throughput is lower on the small flyer by about 6.6%, loading by 1.3%, falling-item
collection by 1.5%, and hopper/storage-cart interaction by 2.3%; their intervals
are below 1.0. Keep the
reference build for workloads where these tradeoffs matter. We do not enable the
experimental features globally or claim that passing mechanics tests establishes
a performance improvement for every circuit.

## Candidates and rollback

Every candidate is disabled in the reference build and can be selected separately:

| Feature | Candidate |
| --- | --- |
| `experiment-directional` | Ask only the horizontal connection question needed by emission |
| `experiment-cursor` | Share index/stride calculation inside a pure non-wire power query |
| `experiment-update-pool` | Recycle existing eager notification vectors, preserving queue layout |
| `experiment-wire-dispatch` | Direct neighbour dispatch for explicitly registered vanilla wire |
| `experiment-wire-plan` | Retain input relationships across structurally equivalent world writes |
| `experiment-wire-certificate` | Reuse a target while its actual inputs and live comparator values agree; implies plans |

The complete reference path remains available without deleting or resetting any
source:

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --locked --release --no-default-features \
  -p mc-tick --example bench_circuits
```

Add `--features experiment-directional` (or a comma-separated list) for an
experiment. `MC_TICK_WIRE_PLAN=off` bypasses plans/certificates at runtime, while
`shadow` recomputes the ordinary answer and asserts equality. Runtime off still
pays for compiled-in world journaling, so the feature-free binary is the clean
performance reference. The environment mode is sampled once per process.

The previous production checkpoint is preserved on `codex/simulation-performance`.
Switching build features or checking out that branch in a separate worktree
provides rollback without touching user edits. Feature commits and raw results
remain available even if a candidate is rejected. Experimental features are not
published API promises: notably, enabling the pool adds a scratch field to
`TickCtx` literals.

## Correctness boundaries

Notifications retain their delivery order, sequence counts and recording. Pools
retain the existing `Vec`-owning entries and eagerly captured coordinates. Typed
dispatch is installed by trusted registration, never inferred from a block name;
ordinary registration replaces it, including custom behaviours.

The cursor borrows an immutable world for one query and cannot survive a write or
growth. Its radius-two interior guard falls back to the existing boundary path.
The directional predicate specializes only concrete vanilla rules; the public
generic `WireWorld` connection calculation is unchanged. Internal side-query
diagnostics may emit fewer lines because fewer pure queries are needed.

Plans preserve six ordered non-wire groups, including the exact-strength-15 early
exit and arbitrary-u8 output semantics. Comparator map values are read live;
zero outputs, removal, insertion and map replacement cannot make a dependency
disappear. Initially unpowered comparator activation is a structural transition.
Dust levels are read live; stored dust shape is ignored only where the muted
target query's dependency proof permits it.

A world-owned 512-write journal is updated synchronously. Each query consumes
changes before reuse. Structural changes invalidate radius-two entries; the
certificate additionally tracks changes to actual incoming-wire inputs. History
loss, growth, cloning/restoration and revision rollover force fresh validity.
Owned `Arc` identities prevent address-reuse mistakes. Full position keys guard
the bounded 1,024-slot cache. Rules clones begin empty. Cache/journal state is
excluded from semantic snapshots. The existing safe Rust and `Send + Sync`
constraints remain intact.

Cross-review caught an extreme-coordinate edge: an eager plan could visit an
overflowing neighbour that the reference skips after finding strength 15. Plans
now fall back near signed-coordinate limits, with a regression exercising actual
notification delivery there.

Certificates never suppress delivery or bypass effects when the target differs
from current power. In particular, the existing notifications still occur when
a requested wire sibling state is absent.

## Measurement and interpretation

`tools/run_tick_experiments.py` records compiler, feature switches, source hashes,
binary hashes, test output and expanded differential traces. Final timings must
run after all builds and profilers have stopped, with shadow/counter builds
excluded. The randomized runner preserves every sample and pairs variants within
case/round. Confidence intervals use the existing paired bootstrap analysis.

The counter feature `experiment-wire-plan-counters` is solely diagnostic. Its
Drop reports cover a cache's complete lifetime, including setup and warmup;
these are not reset-per-cycle counts or performance measurements.

One limitation is deliberate and measured by a regression test: the initial
rules-owned cache retains one world's plans. Sixteen forced alternating-world
queries rebuild sixteen times; sixteen subsequent same-world queries rebuild
once. Concurrent queries remain correct under its mutex, but shared-rules
multi-world throughput needs separate consideration before adopting this storage
as a general default.

The previous PGO `TickCtx::drain` disassembly still contains an indirect vtable
callback (`call *%rax`); its body is 2,069 bytes. This establishes that the typed
dispatch experiment is not simply duplicating an existing guarded direct wire
call. It does not establish that the experiment is faster.

## Validation before the ownership fix

The corrected combined source passes **439 engine/scenario/doc tests** (7 ignored),
**30 bridge tests**, and the wasm check: **469 passing tests** in total. All eight
native variants (reference, six individual candidates, and combined) match all
15 accepted tick-workload records and fingerprints, including the earlier
optimized engine's records. The hipster golden still covers 640 tick-end states
against Minecraft 1.21.11. New community cases remain functional/differential
checks rather than newly captured complete vanilla traces.

The typed table entry is 24 bytes versus 16 bytes for the previous dynamic entry.
That footprint cost is included in the native measurements; the direct-call
hypothesis does not assume it away.

An untimed counters-only `hipster 1 1` command reports 130,860 plan queries,
3,828 plan builds, 2,502 structural invalidations, 18,062 value dirty markings,
and 109,724 certificate hits. Classified writes comprise 1,862 structural,
11,228 wire-value and 4,876 irrelevant-to-this-query transitions. There are two
world resets. These are whole-command operation counts (warmup and measured-work
cycle plus setup), not CPU-time shares or timing results.

## Individual candidates: nine-round pilot

Eight binaries ×16 workloads ×9 paired rounds =1,152 measured observations.
The source is the corrected implementation for plan/certificate/combined and
identical feature-off code for the other binaries; per-build source hashes are
retained in metadata. All use the same compiler, generic release flags and harness.

Hipster CPU-time ratios against the reference, measured together:

| Candidate | Speedup | Paired 95% CI |
| --- | ---: | --- |
| Directional predicate | 1.019× | 0.954–1.045 |
| Indexed cursor | 0.960× | 0.940–0.981 |
| Buffer pool | 1.082× | 1.054–1.108 |
| Typed dispatch | 0.929× | 0.900–0.941 |
| Input plans | 1.173× | 1.144–1.183 |
| Plans + certificates | 1.163× | 1.114–1.194 |
| All six combined | 1.316× | 1.261–1.337 |

The cursor and typed dispatch are not justified as standalone defaults. The
combined result does not establish that each constituent helps; subsequent
combination comparisons remove components to test that directly. The directional
predicate's standalone hipster gain is unresolved. Input plans and pooling show
clear hipster improvements; other workloads and fresh confirmation remain part
of the selection decision.

![Every pilot workload and uncertainty](results/pilot-speedups.png)

[Raw pilot samples](results/pilot-timings.jsonl) and [analysis](results/pilot-summary.json)
retain all observations. The graph includes regressions; no outliers or losing
candidates were removed. Final selection and confirmation follow below.


## Removing components from the combined build

Nine variants ×6 dynamic workloads ×9 paired rounds =486 measured observations.
The simpler **plan + pool + directional** combination measured 1.340×
[1.300, 1.353] on hipster versus the reference in that run. Adding the directional
predicate to plan + pool measured 1.047× [1.014, 1.080] directly.

Adding certificates to that selected combination measured 0.983× [0.954, 0.994]
on hipster, 0.975× [0.945, 0.996] on the 6×6 door, and 0.935× [0.921, 0.974] on
item sorting. Certificates are therefore retained as a disabled experiment,
not part of the recommended combination, despite their high hit count.

Adding both cursor and typed dispatch to the selected combination (without
certificates) did not resolve an additional hipster gain: 1.000× [0.972, 1.020].
It slowed the 6×6 door to 0.942× [0.926, 0.975] and BB to 0.971× [0.963, 0.982]
relative to the simpler combination. Both remain disabled experiments.

These incremental ratios come from paired observations in this combination run;
they are not ratios assembled from different pilot periods. [Raw combination
observations](results/combinations-timings.jsonl), [comparison](results/combinations-summary.json)
and [direct incremental analysis](results/combinations-increment.json) retain the evidence.


## Fixing shared-world cache ownership

`experiment-world-plan-cache` gives each World its own boxed, bounded cache and
checks a fresh-on-clone rules identity before reuse. Shared rules no longer
serialize different worlds through one cache or evict their plans on alternation.
This optional storage fix preserves the earlier rules-owned variant for comparison.
Three actual-rules tests cover alternating/concurrent worlds, empty cloned-world
caches, and changed rules with overlapping StateIds.

The complete branch now passes **442 engine/scenario/doc tests** (7 ignored) plus
**30 bridge tests**, with all six candidates and the ownership fix enabled:
**472 passing tests**. Wasm check passes. The selected directional + pool +
world-owned-plan configuration separately passes **437 engine/scenario/doc tests**.
These overlapping configurations must not be added together as unique tests.
All owner variants match the same 15 accepted differential records.

A separate 3-variant, 6-case, 9-round ownership comparison found a hipster ratio
of 0.968× [0.943, 1.002] for per-world storage versus rules-owned storage.
Its point estimate is slower; the difference is unresolved in this pilot.
Other ownership comparisons likewise span 1.0. The final confirmation uses
per-world storage to remove demonstrated cross-world cache thrashing, with
fresh standard and PGO references. [Ownership samples](results/ownership-timings.jsonl)
and [direct comparison](results/ownership-increment.json) preserve the tradeoff.


## Final suite and reproducibility

The confirmation run contains **4 variants ×16 workloads ×21 rounds =1,344
observations**, with seed 20261114, CPU0 affinity, 150ms target samples and identical
per-case iteration counts across variants. Each process runs an equal untimed
warmup. No samples are discarded. CPU uses ThreadTime and elapsed uses Instant;
construction, checkpoint restore, recording setup, fingerprints, validation,
destruction and rendering remain outside tick timers. Loading is separate.

All four use Rust 1.98.0/LLVM 22.1.8, locked dependencies, release opt-level 3, LTO,
debug info level 1 and generic x86-64 on harrison-work (Ryzen AI 9 HX 370). Both PGO builds
were freshly trained with the same existing multi-circuit training recipe;
trapdoor, hopper transfer speed and comparator collection remain held out.
No builds or profilers ran concurrently with the final native timings.

| Workload | Standard candidate vs standard reference (95% CI) | PGO candidate vs PGO reference (95% CI) |
| --- | --- | --- |
| hipster | 1.325× [1.285, 1.346] | 1.359× [1.347, 1.397] |
| hipster_unrecorded | 1.331× [1.299, 1.370] | 1.389× [1.363, 1.416] |
| hipster_updates | 1.305× [1.285, 1.323] | 1.329× [1.300, 1.348] |
| hipster_timeline | 1.336× [1.317, 1.354] | 1.378× [1.360, 1.393] |
| door6 | 1.110× [1.092, 1.128] | 1.108× [1.092, 1.127] |
| flyer | 1.020× [0.991, 1.035] | 0.934× [0.922, 0.966] |
| bb | 1.020× [1.006, 1.036] | 1.001× [0.982, 1.011] |
| adder_idle | 0.982× [0.962, 0.988] | 0.994× [0.957, 1.025] |
| hipster_load | 1.020× [1.007, 1.028] | 0.987× [0.977, 0.991] |
| comparator_drain | 1.002× [0.989, 1.029] | 1.000× [0.984, 1.013] |
| item_into_hopper | 0.990× [0.975, 1.004] | 0.985× [0.967, 0.993] |
| item_sorter | 1.052× [1.032, 1.067] | 1.041× [1.010, 1.056] |
| hopper_transfer_speed | 1.011× [0.995, 1.030] | 0.986× [0.966, 1.000] |
| hopper_storagecart_interaction | 0.988× [0.976, 1.010] | 0.977× [0.962, 0.999] |
| comparator_update_collection | 1.120× [1.086, 1.156] | 1.114× [1.101, 1.130] |
| trapdoor2x3 | 1.099× [1.076, 1.120] | 1.112× [1.074, 1.136] |

![Final paired comparisons](results/final-speedups.png)

The figure and table compare each candidate with its corresponding reference: standard
against standard, and PGO against PGO. [Final raw samples](results/final-timings.jsonl),
[analysis](results/final-summary.json), [PGO-to-PGO comparison](results/final-pgo-increment.json)
and [compiler/source/binary metadata](metadata/final-builds.json) retain the exact evidence.

Build the recommended standard configuration with:

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --locked --release --no-default-features \
  -p mc-tick --example bench_circuits \
  --features experiment-directional,experiment-update-pool,experiment-world-plan-cache
```

For optional PGO, use the same features with the reusable builder:

```sh
python3 tools/build_tick_pgo.py --output /absolute/new-output \
  --target-dir /absolute/pgo-target --fixtures "$MC_TICK_BENCH_FIXTURES" \
  --skip-native \
  --features experiment-directional,experiment-update-pool,experiment-world-plan-cache
```

Rollback is the feature-free build command above; binaries can also be preserved
side by side before switching. `MC_TICK_WIRE_PLAN=off` is an immediate process-start
fallback for plan queries, and `shadow` is available for compatibility investigation.
The fixture preparation workflow is in [the expanded fixture report](../hipster-next/fixtures.md).

After preserving the four binaries at the paths in the build metadata, rerun
the paired comparison on harrison-work with:

```sh
MC_TICK_BENCH_FIXTURES=/home/harrison/nucleation-perf/expanded-fixtures MC_TICK_WIRE_PLAN=on \
python3 tools/benchmark_tick.py \
  --variant reference=/home/harrison/nucleation-perf/execution-all/reference \
  --variant selected=/home/harrison/nucleation-perf/execution-owner/owner_selected \
  --variant reference_pgo=/home/harrison/nucleation-perf/execution-pgo-reference/pgo \
  --variant selected_pgo=/home/harrison/nucleation-perf/execution-pgo-selected/pgo \
  --cases hipster hipster_unrecorded hipster_updates hipster_timeline door6 flyer bb \
    adder_idle hipster_load comparator_drain item_into_hopper item_sorter \
    hopper_transfer_speed hopper_storagecart_interaction comparator_update_collection trapdoor2x3 \
  --rounds 21 --seed 20261114 --cpu 0 --target-ms 150 \
  --build-info docs/performance/execution-plans/metadata/final-builds.json \
  --output /tmp/execution-final.jsonl
python3 tools/analyze_tick_followup.py /tmp/execution-final.jsonl \
  --baseline reference --compare selected=reference \
  --compare selected_pgo=reference_pgo --output /tmp/execution-final-summary.json
```

Select a permitted CPU if reproducing on another host. The [artifact audit](validation/audit.json)
checks every expected sample cell and all 20 saved verification outputs, including
counter-only builds. Each output contains the same 15 workload records. The final
32 source, harness and dependency hashes match the validated owner build. Build
metadata's `base` field records the remote checkout's then-current Git HEAD;
the per-file hashes identify the synced source actually compiled. That source is
committed as `421dd00d` before the final report checkpoint.


## Final instruction and diagnostic checks

Separate Callgrind runs after native timing counted **858,124,694 →587,598,018
instructions**, a **31.5% reduction**, comparing the reference PGO and selected
PGO binaries with the identical `hipster 1 2` command (one load, four cycles
including warmup). This is not a wall-clock speedup estimate. Raw compressed
profiles and annotations are in [profiles/](https://github.com/Schem-at/Nucleation/tree/v0.12.3/docs/performance/execution-plans/profiles).

The standard reference and selected build also produced **17,172 byte-identical
stderr lines** for `hipster 1 1` with `MC_TICK_TRACE_EVENTS=1`,
`MC_TICK_TRACE_WRITE=12,7,13`, `MC_TICK_TRACE_NOTIFY=12,7,13`,
`MC_TICK_TRACE_SCHED=1`, and `MC_TICK_TRACE_POWER=12,7,13`.
SHA-256: `4614c50446f79ae5c7b9933b419bc35ae5753ca6968b0905b3ea3783cab7b2c0`.
This comparison does not enable every internal diagnostic option.
