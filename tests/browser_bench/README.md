# TickSimulation in the browser — import recipe

Verified in headless Chromium (main thread + module Web Worker), 2026-07-28.

## Build the package

```sh
# best-performing artifact (+6% evals/sec, -0.5 MB):
RUSTFLAGS='-C codegen-units=1' NUCLEATION_WASM_FEATURES=bridge,mc-tick \
    tools/package-npm.sh dist/npm-mctick
```

`tools/package-npm.sh` now takes `NUCLEATION_WASM_FEATURES` (default is the
published set `bridge,simulation,meshing`; add `mc-tick` for TickSimulation)
and emits an isomorphic `diplomat.config.mjs` (`new URL("./nucleation.wasm",
import.meta.url)`) that works in Node, the browser main thread, and workers.
The `.wasm` is ~11 MB — serve it with `Content-Type: application/wasm` so
`instantiateStreaming` works (Vite does this out of the box for public/ files).

## Vite frontends

Copy (or symlink) `dist/npm-mctick/` into the app's `public/engine/` and:

```js
const { TickSimulation, TickSettleMode } = await import("/engine/index.mjs");
const sim = TickSimulation.fromSnbt(snbt, TickSettleMode.Quiet, 0, 0, 0, "");
sim.setRngSeed(12345n);          // BigInt
sim.step();                       // or sim.run(80)
sim.nonAirMinX();                 // scalar queries — no JSON in hot loops
```

Don't `import` the package through Vite's dependency pipeline — the runtime
`fetch` of the wasm wants a stable URL; `public/` gives it one.

## Web Workers (the GA's parallelism)

Each worker instantiates its own wasm module (~1 s cold, then full speed —
4.9k flying-machine evals/sec/worker in Chromium). Use a **dynamic import
inside try/catch** in the worker:

```js
self.onmessage = async ({ data }) => {
  try {
    const { TickSimulation, TickSettleMode } = await import("/engine/index.mjs");
    // ... evaluate, postMessage results
  } catch (e) { self.postMessage({ error: String(e) }); }
};
```

A top-level `import` that fails in a module worker fires **neither**
`onmessage` nor `onerror` — the page just hangs. The dynamic form reports.

## Scalar queries (added for the GA)

`nonAirCount(): number` · `nonAirCenterX(): number` · `nonAirMinX(): number` ·
`nonAirMaxX(): number` · `changesCount(): number` — displacement metrics
without `worldSnapshotJson()` round-trips (~19% faster evals).

## Benchmarks (flying-machine eval: construct + quiet settle + kick + 80 ticks)

| where | evals/sec |
|---|---|
| Node wasm, JSON query | 4,072 |
| Node wasm, scalar query | 4,864 |
| Node wasm, scalar + cgu=1 build | 5,185 |
| Chromium main thread | 4,298 |
| Chromium module worker | 4,934 |

Construction ~35% / stepping ~62% of an eval; both dominated by the engine
itself, not the FFI. Native Python pool comparison: ~15k/sec across 6 procs.
