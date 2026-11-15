# Adapter expansion checkpoint — 15 November 2026

Ten additional source-pinned builds now have executable contracts, independent
expected-answer code, physical port mappings and native verification receipts.
The library contains thirteen adapters. Originals remain unchanged in the local
schematic archive; download provenance and hashes are recorded in each profile.

## Fresh validation

All thirteen passed on **harrison-work** with the same release runner binary:
`e9ab2662d0401d49f56fc8a525457db3f1983ff99c080cdad30a4e09ce356b02`.

There are 5,201 operations and 6,725 named output assertions per run. Every suite
was repeated after checkpoint reset, requiring identical outputs and full engine
checkpoint: **13,450 output assertions** across both runs. Five focused adapter
unit tests also passed. Rust compilation and native simulation ran only on the
laptop; local work handled documentation, indexing and web checks.

| New adapter | Operations | Output assertions per run |
| --- | ---: | ---: |
| `add003` | 372 | 744 |
| `add008` | 136 | 272 |
| `alu001` | 1017 | 1017 |
| `decode004` | 608 | 1216 |
| `encode001` | 513 | 513 |
| `encode003` | 513 | 513 |
| `lff006` | 131 | 131 |
| `othercomb001` | 513 | 513 |
| `othercomb003` | 513 | 513 |
| `othercomb004` | 513 | 513 |

`results.json` summarizes hashed receipts; complete outputs are committed as
`circuit-adapters/<id>/result.json`. Raw command output is retained under
`validation/adapter-expansion-final-v1` in the local archive and
`/home/harrison/nucleation-perf/adapter-expansion-final-v1` on harrison-work.

The optional `button_press` driver uses the real engine interaction and scheduled
release. True presses once; false does nothing. An already powered button is
rejected before input mutation. Existing profiles retain lever-level semantics.
The latch explicitly settles its data before capture and checks holds and both
stored polarities.

## Scope and limits

These are native endpoint functionality checks, not new vanilla captures or a
proof of arbitrary input timing. Fixed response windows are conservative, not
latency measurements. Expected answers come from independently authored arithmetic,
logic or sequential state rules, never from the simulator being tested.

The four-bit arithmetic and encoder suites cover every input and every ordered
settled transition. Both encoder templates retain all-enabled matrices (15 for
any nonzero selection, otherwise 0). All four decoder rows are programmed for 10.
The one-bit adder covers every truth-table state and directed transition. The
8-bit adder and ALU operand pairs are sampled, not exhaustive. The ALU uses eleven
one-hot operation masks; right-shift coverage is restricted to b=0 and includes
all 256 A values. The generic profile currently does not enforce that semantic
one-hot/domain restriction. Consult each `mapping.md` before creating new suites.

No fresh benchmark timing or speedup is claimed. New adapters can use the existing
runner and randomized benchmark tooling when work resumes. Prior vanilla evidence,
unsupported cases and failure receipts remain separate from these passes.

## Reproduce

```sh
CARGO_BUILD_JOBS=4 cargo test --locked --features mc-tick --lib tick_adapter::tests
CARGO_BUILD_JOBS=4 cargo build --locked --release --features mc-tick --example run_tick_adapter
python3 tools/generate_tick_adapter_suites.py --check
python3 tools/run_tick_adapters.py --archive /path/to/archive \
  --binary target/release/examples/run_tick_adapter --output /path/to/new-results
```

Circuit Atlas exposes per-build expected-answer code and mapping notes alongside
its profile, suite and result. `atlas-api.json` records the final read-only API
check (20/20 passed, 111 HTTP requests). The desktop/mobile UI check also
passed nine checks with no uncaught browser errors. The user requested a pause after this batch; no next batch, vanilla capture
or benchmark campaign was started. All workers stopped after their assigned builds.
