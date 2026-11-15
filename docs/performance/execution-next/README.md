# Dynamic execution experiments

Started 15 November 2026 from `dbbfd534` on
`codex/simulation-execution-next`. All Rust compilation and native measurement
run on harrison-work; the original dirty Mac checkout is preserved. Every new
candidate is default-off. This report distinguishes unfinished hypotheses from
validated results.

## Flying-machine regression investigation

A fresh paired pilot reuses the previous four final binaries, plus the same
selected PGO binary with wire plans disabled at process start. Five variants,
three cases and nine rounds produce 135 measured samples. No builds or profilers
ran during that native comparison.

The selected PGO flyer measures 0.944× [0.927, 0.985] against the PGO reference.
Disabling wire plans in that exact binary measures 1.008× [0.974, 1.028] relative
to plans enabled: no resolved improvement. The counters-only flyer command
produces no plan-cache report, consistent with no wire-plan cache initialization.
Thus wire-plan evaluation is not supported as the cause. The experiment does not
yet distinguish always-on bookkeeping, changed layout or PGO code generation.

For contrast, turning plans off on hipster measures 0.682× [0.672, 0.701]
relative to enabled plans in the same binary. Do not use the flyer result to
infer that caching contributes nothing to wire-heavy builds.

[Raw pilot](results/regression-timings.jsonl) and
[paired analysis](results/regression-summary.json) retain all samples.

## Candidates under test

- `experiment-lazy-journal`: defer mutation history until a plan consumer exists.
- `experiment-execution-view`: bounded plan address records with live state reads
  and invalidation on geometry changes.
- `experiment-update-arena`: eager notification payloads in reusable storage,
  preserving callback-visible queues, nested order and generic custom behaviour.
- `experiment-piston-plan`: structural piston power inputs with live comparator
  and dust emissions, including quasi-connectivity.

The previous selected configuration remains available as
`experiment-directional,experiment-update-pool,experiment-world-plan-cache`.
The feature-free reference and the prior checkpoint branch are also preserved.

## Randomized differential verification

`bench_circuits --fuzz 8` applies 96 deterministic operations for each of eight
seeds across seven real fixtures: hipster, small flyer, comparator drain,
falling-item hopper collection, sorter, storage cart and 2×3 trapdoor.
Operations interleave lever use, inventory edits, live comparator output writes
across the u8 domain, block placement, world growth, and checkpoint cloning and
restoration. A separate deterministic generator chooses operations; simulation
RNG seeding is explicit.

Each operation hashes world/tick/queue state, live comparator outputs, motion,
inventories and entities, and every newly delivered ordered record. Every
sequence is replayed and compared both by trace fingerprint and complete final
checkpoint equality. Candidate JSON is compared with the feature-free binary.
These are differential regression checks, not newly captured vanilla behavior.
