# Stationary logic, flying machines, and current master

Started 15 November 2026 from `fac555fb` on
`codex/simulation-workload-specialization`. All Rust builds, checks and native
measurements run on harrison-work. The original dirty Mac checkout is untouched.

The freshly fetched default branch is `master`, commit
`ee89008d085a8fb0088851dcac7221ab94d2b91c`. An isolated detached checkout receives
only the common benchmark harness and cpu-time 1.0.0 dev dependency/lock entries.
Its production engine source stays unchanged. `bench_master` excludes the
unsupported unrecorded case; no recording API or optimization is backported.

Two independent, default-off experiments are under measurement:

- `experiment-logic-plan-storage`: dense position indices and sparse cached wire
  plans avoid conflict misses in fixed circuitry. Live inputs, synchronous
  invalidation and the general dynamic simulation path remain intact. The index
  is bounded to262,144 u32 cells (1MiB); at most32,768 sparse plan entries supplement
  the1,024 fallback slots. Larger worlds/exhausted budgets use the prior cache.
- `experiment-moving-phase`: reuse due-move storage, remove per-landing singleton
  allocation and skip empty entity-displacement work. Selection, stable ordering,
  callbacks and live probes/entities remain authoritative.

New active logic cases use the existing32-bit adder (eight input vectors across
768ticks) and an authored four-lane comparator/repeater network (16Gray-code
phases across384ticks). Exact arithmetic/signal outputs are checked from recorded
changes outside the timer. These augment idle logic, two flying machines and the
existing doors/inventory/item corpus. They are not fresh vanilla captures.

Final measurements and selection will be recorded here after the controlled run.
