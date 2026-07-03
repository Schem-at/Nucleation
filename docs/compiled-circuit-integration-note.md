# Compiled-circuit container — integration note for hardwired (written 2026-07-04)

## API (mavenLocal 0.2.17, NOT in the released/vendored 0.2.17 jar)
- `TypedCircuitExecutor.serialize(): byte[]` — versioned container (magic `NCCK`, u32 LE version, currently 1)
- `TypedCircuitExecutor.fromCompiled(byte[])` — restores; throws on magic/version mismatch
- Rust: `to_compiled_bytes()/from_compiled_bytes()`; constants `COMPILED_MAGIC`/`COMPILED_VERSION` re-exported from `typed_executor`

## Measured verdict: DO NOT integrate a compiled-blob cache yet
The container caches the *front end* (schematic parse + insign extraction + IO layout).
Measured on release builds (`bench_compiled_vs_schematic_start[_large]` in
`src/simulation/typed_executor/compiled.rs`):

| circuit | full start | schem parse | insign+layout+redpiler | compiled-path start |
|---|---|---|---|---|
| full adder (~40 components) | 233 µs | 35 µs | 198 µs | 238 µs |
| 48×48 field (~2.3k components) | 770 ms | ~0.2 ms | ~770 ms | 783 ms |

At every scale the redpiler compile dominates and is NOT captured by the container,
so restoring saves nothing today (container even loses by container-decode overhead
at small scale). Adding a `kernels.compiled` BLOB column would cache the wrong thing.

## What IS worth doing (the real feature, needs MCHPRS fork surgery)
At kernel scale the redpiler compile is ~hundreds of ms. Serializing the
post-optimization `CompileGraph` (petgraph `StableGraph<CompileNode, CompileLink>`,
no serde today) in Nano112/MCHPRS + a `Compiler::compile_from_graph` entry, then
bumping nucleation's mchprs rev, slots into this same container as version 2 —
the JVM API surface does not change. THEN add the kernels BLOB column
(compile-on-first-start, invalidate on header version mismatch).

## Performance landmine found while measuring (check in hardwired NOW)
`TypedCircuitExecutor.reset()` — and every `Stateless`-mode `execute()` —
recreates the MchprsWorld, i.e. RE-RUNS the full redpiler compile
(~770 ms at 48×48 scale). Hardwired's KernelRuntime/SimulationManager should be
audited: any per-run or periodic reset() on a large kernel will stall a worker
thread for the full compile time. Manual/Stateful modes avoid it.
