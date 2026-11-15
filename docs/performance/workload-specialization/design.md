# Workload-specialization design and invariants

These are independent, default-off experiments. They preserve the general
simulation path, including live comparator outputs, edits, movement and custom
callbacks. None removes, combines or reorders a notification delivery.
The main report records which configurations pass the performance gate.

## Dense wire-plan storage

`experiment-logic-plan-storage` changes storage in
[`wire_plan.rs`](../../../crates/mc-tick/src/wire_plan.rs), not the computed wire
target. The previous 1,024-slot direct map can repeatedly evict plans for fixed
wire positions whose hashes collide. The experiment assigns distinct sparse
plan slots through a dense position index.

- The position index is at most 262,144 `u32` cells: **1 MiB**.
- At most 32,768 additional sparse plan slots supplement the 1,024 fallback
  slots. Plans are allocated as positions are queried, rather than for every
  cell. Entry size depends on enabled features; built-in plans retain at most
  eight incoming wires and 25 unique comparator positions.
- Worlds above the cell limit, out-of-bounds queries and exhausted sparse
  budgets use the original direct-map fallback. Full position keys distinguish
  collisions there.
- Invalidating a position does not allocate a new slot. Its assigned slot is
  retained for reuse, so a moving workload can eventually consume the sparse
  budget and use the fallback for new positions.

The cache still synchronizes the World mutation journal before evaluating a
query. Structural changes invalidate affected plans immediately, including
changes within a tick; wire values and comparator outputs remain live inputs.
Invalidation uses the same dense position mapping as lookup. A colliding
fallback key cannot invalidate a different resident position.

World growth, lost journal history and identity changes clear the mapping and
plans. World clones/checkpoint restoration start with empty per-World caches;
rules identity is checked independently, so overlapping StateIds from other
rules cannot reuse plans. Resetting an eligible cache may allocate/initialize
the dense index again, including during the first query after restoration.
Storage is safe for dynamic builds too; stationary logic merely offers longer
reuse of fixed positions.

Tests cover colliding keys staying warm, immediate independent invalidation,
growth and cloning, negative/extreme coordinates, both memory limits, and
end-to-end colliding fallback queries after budget exhaustion. Existing
rules-isolation, checkpoint, concurrency and randomized mutation tests exercise
the new storage path. `MC_TICK_WIRE_PLAN=shadow` compares complete planned
answers with the reference; `off` bypasses plan queries.

## Compiled transition signatures

`experiment-transition-signatures` replaces repeated rule-table comparisons in
[`VanillaRules::wire_plan_transition`](../../../crates/mc-tick/src/vanilla.rs)
with two indexed signature reads. It compiles immutable rule metadata, not
World state or power-query results.

The table covers all 65,536 possible StateIds and stores one `u64` per ID:
**512 KiB per initialized rules instance**. Its low bits encode exactly the
reference classifier's structural comparisons: powered membership, optional
analog emission, emission direction/exclusion, strong-output direction,
conductor/comparator membership and wire presence. Upper bits encode wire power.
Presence bits distinguish `None` from `Some(0)`, and an absent direction from
`Dir::Down`. All `u8` values are preserved. Stored wire shape remains excluded,
exactly as in the original classifier.

A `OnceLock` initializes the complete table on first classification. Subsequent
queries sharing the immutable rules instance reuse it across Worlds. Rule
cloning starts a fresh, cold table; cloning a World does not clone its shared
rules. `VanillaRules` exposes no public rule mutator, and registration completes
the private tables before publishing them through an `Arc`. Internal code that
edits a rules clone does so before first use. Any future API permitting edits
to warmed rules would also need to invalidate derived metadata.

`MC_TICK_TRANSITION_SIGNATURES=off` uses the original classifier without
initializing the table. `shadow` computes both classifiers and asserts equality;
`on` uses the compiled table. These diagnostic modes, like the wire-plan mode,
are sampled once per process. Build without the feature for the original path.
No shadow instrumentation belongs in native timing runs.

Tests compare every pair in a registered mixed-logic palette; all 256 wire and
analog values; every unassigned StateId and the highest ID; absent/zero and all
direction options; and warmed-rule cloning followed by custom metadata edits.
They also check the cold off path, Send/Sync and semantic Debug stability.

### First-use measurement

The separately executed, ignored release-mode test
`vanilla::transition_signature_tests::measure_transition_signature_cold_and_warm`
uses a registered mixed-logic palette, three warmup rounds and 21 measured
samples. Rule cloning, destruction and JSON formatting are outside the timers.

Median first classification, including table initialization, measured
**197,740 ns CPU / 198,272 ns elapsed**. In the same narrow microbenchmark,
warmed signature classification measured **1.16356 ns/call**, versus
**2.29913 ns/call** for the original classifier. Those per-call CPU measurements
include loop and `black_box` overhead and exclude environment-mode lookup,
World access, journal processing and circuit work. They are not overall
simulation speedups.

Initialization can occur during placement settling, or at first activity after
an untouched InWorld load. Warmup amortizes this for steady-state measurements;
it does not eliminate the cold allocation and initialization cost.

## Moving-block phase

`experiment-moving-phase` changes temporary work in
[`sim.rs`](../../../crates/mc-tick/src/sim.rs):

1. Extract due moves into reusable storage with the same `resolve_on <= tick`
   and ticking-bounds predicate. Stable extraction followed by the original
   stable due-tick sort preserves insertion order among equal deadlines.
2. Replace the per-landing one-element allocation with a one-element array.
3. Skip displacement geometry work only when every live entity list and the
   pending piston-probe list are empty. Per-tick piston deltas are still cleared
   before that guard, and the guard runs after block-entity callbacks.

Both implementations remove the complete due-move snapshot from the public
queue before any landing callback. A callback can inspect, clear or append to
that queue; it cannot cancel or extend the already extracted phase snapshot.
Frozen moves remain pending. Each landing still completes its own shape
updates, onPlace, neighbor updates, neighbor-shape updates and final self
notification before the next landing begins.

The guard reads current lists each pass, so an entity introduced earlier in the
tick disables the fast path. Pending probes also disable it. It does not infer
that a build will remain entity-free from its initial schematic.

The due buffer is empty after a completed phase and retains capacity for reuse;
it has no separate fixed capacity cap. Checkpoints contain semantic pending
moves, not this scratch buffer. Restoring a checkpoint reuses only empty
allocation capacity. Tests cover out-of-order deadlines, frozen moves, callbacks
that replace pending moves and introduce items, restore/replay, and newly added
carts/probes being observed by the guard.

## Validation scope

The core/scenario/doc suite passes **481 tests**, with **8 ignored**; the bridge
suite passes another **30**, for **511 passing tests**. The manual initialization
benchmark is one of the ignored tests and was run separately. Candidate traces
and randomized replay checks retain the same
notification ordering and checkpoint comparisons used by the common harness.
These checks complement the retained vanilla captures; they do not constitute
a new complete vanilla notification trace or a universal performance claim.
