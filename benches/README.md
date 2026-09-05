# Performance benchmarks

Nucleation keeps three complementary benchmark layers. They deliberately do
not share one headline number because they answer different questions.

## Public Python API

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

## Rust schematic core

```bash
cargo bench --bench block_placement
cargo bench --bench block_placement -- realistic_sparse_placement
cargo bench --bench block_placement -- content_shorthands
```

These Criterion benchmarks remove the Python boundary and isolate palette,
region, block-state parsing, block-entity, fill, transformation, copy, and
export costs.

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
