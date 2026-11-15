# Optimization experiments, 15 November 2026

All Rust builds and native timings ran on `harrison-work`. No production cache
remains. Patches preserve the tested alternatives and their regression tests.
See `manifest.json` for hashes and the engine base. Apply a selected cumulative
patch to that base; do not apply all the patches sequentially.

| Patch | Scope | Decision |
| --- | --- | --- |
| `packed-power.patch` | Four-byte immutable emission/direction metadata per state | Reject: hipster 1.003×, CI 0.984–1.016; flyer and loading regressed |
| `neighbours.patch` | Share coordinates and bounds checks for six adjacent world reads | Retain; final 21-round comparison is in the parent report |
| `inert-dispatch.patch` | Explicit no-op neighbour/shape callback flag; recording unchanged | Reject: no convincing incremental broad benefit; this patch alone layers on the neighbours stage |
| `fused-wire.patch` | Shared six-state snapshot between non-wire power and incoming-wire queries | Reject as a standalone default: pilot item-sorter regression; cumulative with neighbours |
| `local-cache-mutex.patch` | Whole wire target structure, live comparator dependencies, local invalidation, bounded mutex table | Reject: small hipster gain, regressions elsewhere; cumulative with neighbours and fused query |
| `local-cache-once.patch` | Same query with `OnceLock` slots and collision fallback | Reject: smaller hipster gain and door regression; cumulative |
| `local-cache-constant.patch` | Pre-resolve entries with no comparator dependencies | Reject: still small and inconsistent; cumulative |
| `wire-cache-counters.patch` | Deliberately instrumented counters | Apply only after `local-cache-once.patch`; never time this build |

The packed and no-op dispatch pilots used the original nine-case harness for
the reference stages. Subsequent pilots used the expanded common harness.
The final retained comparison rebuilt all three variants with exactly the same
expanded harness, including the RNG reset after setup. `results/` retains all
observations; pilot and final sampling periods must not be mixed.

## Local cache dependency proof

This is a cache of `wire_target`, **not** a cache of unrestricted emitted power.
With wire emission muted, its world-state dependency footprint is exactly the
25 integer cells at Manhattan distance at most two from the queried position:

- The six immediate neighbours supply their own non-wire emission.
- A conducting neighbour reads strong emission from its six neighbours.
- Incoming dust reads the four horizontal neighbours and their up/down
  diagonals, plus the conductor above the queried position.
- Muted wire emission never calls the dynamic connection-shape calculation.

The cached payload contains six fixed non-comparator contributions, six masks
of comparator dependencies and the incoming-wire strength. Every cache hit
reads comparator outputs from the caller's current map. Powered state, emission
direction, excluded direction and strong-output direction determine eligibility.
A comparator whose current output is zero is still a dependency. Missing map
entries are zero; insertion, removal, `entry` edits and map replacement are live.

The six contributions remain separate because the existing implementation stops
when its running non-wire maximum is **exactly** 15. Combining everything into
one maximum or assuming all outputs are at most 15 would change arbitrary-u8
semantics. Tests include strengths 16, 127 and 255.

Every attempted `World::set` invalidates affected entries immediately, including
multiple writes inside one tick. Successful growth clears the entire cache.
This design has no generation counter, so rollover is inapplicable. Full signed
positions are compared after hashing. Storage is bounded to 1,024 entries.
Collisions overwrite with key checks in the mutex version; the `OnceLock`
version computes uncached on a different key instead of replacing through shared
access. An invalidation may clear only the matching key in that version.

An owned `Arc` identity ties entries to one immutable rules table and prevents
allocator-address reuse. The mutex version clears on rules replacement; the
`OnceLock` version falls back for different rules until cleared. Cloning a world
starts empty. Checkpoint restoration starts empty and preserves the receiving
world's chosen configuration. Cache state is excluded from semantic equality
and debug serialization. `World` remains `Send + Sync`; no unsafe code, `RefCell`
or runtime dependency was introduced.

`Off`, `On` and `Shadow` are explicit experimental modes. The environment default
is sampled once per process from `MC_TICK_POWER_CACHE`; a public setter can
change an existing world's mode. Shadow mode recomputes the structure and final
answer and asserts equality. It is disabled during timing.

## Evidence and rejection

The `OnceLock` experiment passed **421 tests, 0 failures, 7 ignored**, with shadow
checking enabled, plus the wasm check. The tests include every dependency offset,
collisions, cloning, growth, restoration, different rules with overlapping IDs,
within-tick conductor changes, live comparator map edits and 2,000 randomized
neighbourhoods. The expanded 15-case differential verification matched the
retained baseline's ordered fingerprints for all three cache variants. Existing
goldens cover pistons, inventory and item transitions, and flying machines.

Nine paired rounds, cache-on versus cache-off in the **same binary**:

| Case | Mutex cache | OnceLock cache |
| --- | --- | --- |
| Hipster | 1.049× [1.029, 1.071] | 1.027× [1.007, 1.058] |
| 6×6 door | 0.972× [0.965, 0.990] | 0.964× [0.948, 0.975] |
| Item sorter | 1.006× [0.989, 1.103] | 1.020× [0.993, 1.032] |
| 2×3 trapdoor | 0.952× [0.889, 1.003] | 0.972× [0.959, 1.012] |

The constant-result fast path likewise failed to establish a broad improvement.
These small gains and regressions did not justify adding this cache to production.
Correctness success alone is not a performance justification.

An **instrumented, untimed** OnceLock run (`hipster 1 1`) produced identical
counts in its warmup and measured-work cycle:

- 65,430 queries;
- 21,081 cold/mutated entry initializations;
- 3,100 collision fallbacks;
- 41,249 hits (63.0%);
- 20,997 effective entry invalidations.

The hit rate is an operation count, not CPU time saved. Cache lookup, payload
resolution, initialization and invalidation costs consume much of the benefit.
The instrumented binary's elapsed/CPU timings are deliberately not used.

## Compiler experiments

`tools/build_tick_pgo.py` follows the
[official Rust PGO workflow](https://doc.rust-lang.org/rustc/profile-guided-optimization.html):
instrument, train, merge with the matching toolchain's `llvm-profdata`, rebuild
without instrumentation, then verify. Training spans doors, both flying machines,
the idle adder, sorting and a hopper/cart interaction. The community trapdoor,
hopper transfer-speed and comparator update-collection cases are held out.

Portable PGO is retained as an **opt-in build recipe**, not a package release or
an automatic compiler flag. `target-cpu=native` was also tested independently.
Its item-sorter pilot was 0.911× [0.891, 0.923] relative to the previous engine,
so it is not a default. Final results use generic x86-64 for all variants.
