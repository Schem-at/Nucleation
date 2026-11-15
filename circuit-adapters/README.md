# Executable circuit library

Each admitted build gets a source-pinned adapter and an independent test suite.
The adapter runs the **actual mc-tick world**; the arithmetic in the suite is
only an expected-answer oracle. The same operations are usable as tests and
benchmarks. Original archive files are retained unchanged outside Git.

| Adapter | Named inputs | Named outputs | Fixed response window |
| --- | --- | --- | --- |
| `add005` | `a: u8`, `b: u8`, `cin: bool` | `sum: u8`, `cout: bool` | 32 game ticks |
| `othercomb017` | `a: u8`, `b: u8` | `equal`, `less`, `greater`: bool | 24 game ticks |
| `plot-lamp2x2` | `closed: bool` | `air_mask: u4` (15=open, 0=closed) | 160 game ticks |
| `add003` | `a: u8`, `b: u8`, `cin: bool` | `sum: u8`, `cout: bool` | 32 game ticks |
| `add008` | `a`, `b`, `cin`: bool | `sum`, `cout`: bool | 32 game ticks |
| `alu001` | `a: u8`, `b: u8`, `operation: u11` (one-hot) | `out: u8` | 64 game ticks |
| `othercomb001` | `value: u4` | `result: u4` (decrement) | 32 game ticks |
| `othercomb003` | `value: u4` | `result: u4` (negate) | 32 game ticks |
| `othercomb004` | `value: u4` | `result: u4` (increment) | 32 game ticks |
| `encode001`, `encode003` | `selected: u4` | `code: u4` (saved matrix) | 32 game ticks |
| `decode004` | `data: u4` | `matches: u4`, `input_echo: u4` | 32 game ticks |
| `lff006` | `data: bool`, `capture: bool` (button press) | `stored: bool` | 64 game ticks |


These conservative observation windows are not minimal latency measurements.
The adder has 328 deterministic vectors (both carry values, boundaries, bit
walks, carry chains and seeded pairs); the comparator has 40. Neither is an
exhaustive truth table. Adapter execution alone is not a vanilla parity proof.

## Run on harrison-work

```sh
cargo build --locked --release --features mc-tick --example run_tick_adapter
python3 tools/run_tick_adapters.py --archive /path/to/archive \
  --binary target/release/examples/run_tick_adapter --output /path/to/results
# With no other builds, profilers or simulations running:
python3 tools/run_tick_adapters.py --archive /path/to/archive \
  --binary target/release/examples/run_tick_adapter --output /path/to/benchmark \
  --samples 21 --iterations 3
# Calibrated, randomized 21-round sampling on a permitted Linux CPU:
python3 tools/benchmark_tick_adapters.py --archive /path/to/archive \
  --binary target/release/examples/run_tick_adapter --output /path/to/fresh-run
```

The runner resolves the original by SHA-256 from the archive manifest. It
validates that hash and the source save DataVersion, imports using the production
reader, and uses InWorld initialization without block substitutions. Adapter
positions are relative to the production export's bounding-box minimum; its
`hash_origin` preserves source-world dust ordering. Record source/global origin explicitly when adding
an order-sensitive spatial reference; do not reuse a translated vanilla capture
without checking that origin.

`adapter.json` embeds Nucleation's existing `io_contract::CellContract`, adding:

- Ordered input ports and LSB-first bit positions. Actual lever pulls happen in
  this order before the next game tick, with no settling between individual bits.
  Optional `inputs` drivers add `button_press`: true presses once, false does
  nothing; the engine releases it. Pressing an already powered button is rejected
  before any input mutation. Omitted drivers retain lever-level behavior.
- Strict boolean/unsigned input validation. Out-of-range, missing or extra values
  fail before actions. Missing levers and output lamps fail explicitly.
- Setup ticks, response ticks, quiescence requirement, seed and reset behavior.
  Random block ticks are disabled in v1. Setup and checkpoint restore are outside
  timing. Entity/inventory state is carried by the engine checkpoint.
- Explicit output observations. v1 supports lamps and exact air occupancy bits;
  it does not infer a door aperture, auto-fill inventories or guess controls.
- Source identity and evidence explaining the mapping and its limits.

`suite.json` contains inputs and independent expected outputs. Regenerate with
`python3 tools/generate_tick_adapter_suites.py`; `--check` detects drift. Never
generate expected values from the simulator under test. For sequential builds,
preserve operation order and author reset/clock semantics rather than treating
each input as a combinational call. The latch explicitly settles data before
pressing its real button. Inventory operations and moving reference frames still
need explicit adapter extensions before admission. Per-build `oracle.py` and
`mapping.md` explain expected-answer generation and physical port evidence.

The saved encoder matrices have every crosspoint enabled: their output is 15 for
any nonzero selection, otherwise zero. All four decoder rows match 10. ALU
right-shift coverage is restricted to `b=0`; other selectors use the documented
sampled operands. These contracts preserve the original builds, including their
programming, and do not imply arbitrary programming or exhaustive 8-bit coverage.

The native runner checks all vectors, resets, and requires the repeated outputs
**and full engine checkpoint** to match. Benchmark samples include typed input
handling, actual ticks, named output reads and result collection. Parsing,
construction, setup, restore, assertions, result destruction and rendering are
excluded. Every measured run is checked outside its timer. Timing is elapsed
monotonic time, not thread CPU time. Use randomized interleaving for engine
comparisons; a standalone sample does not establish a speedup.

Circuit Atlas exposes the contract, suite, implementation and recorded result
beside matching schematics. Entries without an adapter remain unvalidated.
Raw plot smoke tests and exact vanilla captures remain separate evidence; a
mechanism that merely moves does not acquire a successful door contract.

Implementation: `src/tick_adapter.rs`; native carrier:
`examples/run_tick_adapter.rs`. The reusable API is `TickAdapter::execute`,
`read` and `reset`. It is enabled by `mc-tick`, without the MCHPRS backend or
rendering features.
