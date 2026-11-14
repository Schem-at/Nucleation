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
