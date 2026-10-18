# Performance benchmarks

Nucleation keeps three complementary benchmark layers. They deliberately do
not share one headline number because they answer different questions.

## Public Python API

The wheel targets CPython 3.12+ (stable ABI) and builds with CMake and
nanobind. A self-contained environment, independent of the system Python:

```bash
uv venv -p 3.12 .venv-bench          # uv downloads 3.12 if it is missing
uv pip install -p .venv-bench/bin/python build mcschematic
.venv-bench/bin/python -m build --wheel bindings/python --outdir target/python-wheels
uv pip install -p .venv-bench/bin/python target/python-wheels/nucleation-*.whl
```

Comparable operations against `mcschematic`:

```bash
python benches/bench_python.py --warmups 3 --iterations 21
```

Nucleation-specific editor and simulation workflows:

```bash
python benches/bench_realistic_python.py --warmups 3 --iterations 21
python benches/bench_realistic_python.py --filter redstone
```

The realistic suite covers deterministic sparse placement, named regions,
barrel signal and jukebox record shorthands, content-bearing block
replacement, `{simulate=true}` placement, and schematic-to-tick-engine piston
simulation. It also compares repeated convenience placement with
`set_blocks_simulated`, which selects one local active component and amortizes
its construction across an ordered sequence. Simple deterministic placement
components may be handled by the generic resolver pipeline without constructing
the tick engine; interactive neighbours force the correctness fallback. Every
scenario performs a correctness smoke check before timing;
an invalid or inert workload fails instead of publishing a fast number.

Both scripts accept `--json PATH` for machine-readable results. Run them from
an otherwise idle host against a freshly built wheel. Do not compare results
from different machines or power modes as if they were a regression series.
On virtual machines, pin both runs to one CPU (`taskset -c 3 python ...`):
migrations between vCPUs make whole runs bimodal (2-3x apart), which swamps
the differences being measured.

## Rust schematic core

```bash
cargo bench --bench block_placement
cargo bench --bench block_placement -- realistic_sparse_placement
cargo bench --bench block_placement -- content_shorthands
cargo bench --bench block_placement -- chest_batch_phases
```

These Criterion benchmarks remove the Python boundary and isolate palette,
region, block-state parsing, block-entity, fill, transformation, copy, and
export costs. `chest_batch_phases` compares the batch API
(`UniversalSchematic::set_blocks_from_string`) with the per-call loop in
`set_block_chest_nbt`.

## Mesh voxelization

```bash
cargo bench --features voxelize --bench voxelize_bench
NUCLEATION_BENCH_LARGE=1 cargo bench --features voxelize --bench voxelize_bench
```

Solid and shell fills of a generated 5,000 triangle UV sphere at 32 and 64
voxels, plus the textured `BoxTextured.glb` cube at the same sizes. Size 128
is behind `NUCLEATION_BENCH_LARGE=1`. Every case rebuilds the mask and the
surface field per iteration (`MeshShape::clone_uncached`), so the numbers are
cold fill times, not cache hit times.

## Baselines and regression checks

Criterion compares against a named baseline recorded on the same machine:

```bash
git switch main && tools/bench.sh save main
git switch my-branch && tools/bench.sh compare main
tools/bench.sh compare main block_placement region_bench   # a subset
```

`tools/bench.sh` runs every root bench with the features they need
(`voxelize`, `scripting`) and passes `--save-baseline` / `--baseline` through.
Criterion then reports each case as improved, regressed or unchanged with a
confidence interval. Results are only comparable for the same machine, power
mode and toolchain; never compare numbers across machines as a regression
series.

`block_placement` runs 60 samples in an 8 s window, so every case gets a
stable estimate without Criterion extending the window. Cases that would only
read a cached value (`Region::count_blocks`, `is_empty`) are deliberately not
benchmarked; `region_bench/rescan_content` times the scan that maintains
those caches instead.

## Redstone tick engine

```bash
cargo bench -p mc-tick --bench tick
cargo bench -p mc-tick --bench tick -- --test
```

The engine suite uses real fixtures: a 32-bit adder, a 6x6 piston door, a
flying machine, and BB. It separates construction/settling, idle and active
ticks, complete solve time, search-style batches, and timeline-recording
overhead. Each active fixture asserts that it really moved or computed the
expected result before Criterion records timings.
