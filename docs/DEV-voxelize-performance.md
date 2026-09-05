# Voxelize performance: design (0.10.17)

Mesh voxelization (GLB and OBJ to blocks) is slow at useful sizes and
grows as N^6 in the target size N. This pass makes it N^3, fixes the same
pattern in the neighbouring code paths, and adds the bridge calls the
schemat.io tools need so they stop round-tripping every block as JSON.

Measured before (native, macOS arm64, 5,001-triangle sphere):
size 32 fill 0.36 s, 64 fill 15.9 s, 128 fill 1578 s, 256 extrapolated
28 h. The mask itself takes milliseconds at every size. An analytic
sphere of the same volume fills in 0.004 s.

## 1. Cause

`BuildingTool::fill` and `fill_enum_masked` (src/building/mod.rs:25-37,
47-72) call `shape.normal_at(x, y, z)` for every solid voxel, whatever the
brush does with it. `MeshShape::normal_at` (src/voxelize/shape.rs:535)
calls `nearest_triangle` (src/voxelize/shape.rs:215-264), which allocates
`vec![false; triangles]` per call and walks an expanding ring over the
grid, so an interior voxel at depth r scans r^3 cells. The textured path
(`voxelize_textured`, src/voxelize/mod.rs:34) does the same search per
voxel for colour, converts through Lab and Oklch per voxel
(src/blockpedia/color/mod.rs:32-52), and scans the palette linearly with
a `String` clone per voxel, with no cache.

## 2. Changes in nucleation

### 2.1 Brushes declare whether they need a normal

`Brush` gets `fn uses_normal(&self) -> bool { true }` with the default
kept conservative. `SolidBrush`, `ColorBrush`, the gradient brushes
(`LinearGradientBrush`, `MultiPointGradientBrush`,
`BilinearGradientBrush`, `PointGradientBrush`, and the brush at
brushes.rs:1102) override it to `false`; `ShadedBrush`, `SpotlightBrush`
and `CurveGradientBrush` keep `true`. `BrushEnum` forwards it. `fill`,
`fill_enum`, `fill_enum_masked` and `fill_sdf_function` compute the normal
only when the brush asks; otherwise they pass `(0.0, 0.0, 0.0)`.

### 2.2 Mesh shape precomputes its surface fields

`MeshShape` grows an optional `SurfaceField` built once by the same rayon
rasterisation that builds the mask: for every voxel in the mask, the id of
the triangle that claimed it (`Vec<u32>` over the bounding volume,
`u32::MAX` for none). Interior voxels of a solid fill inherit the id of
the nearest surface voxel by a 6-neighbour flood from the shell (one BFS
over the volume, O(N^3)). `normal_at` and `surface_color` become O(1)
lookups into the field; the ring search stays only as a fallback when the
field is absent (shapes built without the mesh, if any). The per-call
`vec![false; n]` goes; the fallback uses an epoch-stamped visit buffer
stored in `MeshData` behind a `RefCell` or a thread-local, so it allocates
once.

### 2.3 Textured path

`voxelize_textured` samples colour through the surface field, quantises
each sampled colour to 6 bits per channel and memoises the palette result
in a `HashMap<u32, usize>` keyed by the quantised colour, and stores
palette indices, resolving to `BlockState` once per distinct index at the
end. The per-voxel loop over the volume is a rayon parallel iterator on
native (single-threaded on wasm, as now) writing into a preallocated
`Vec<u16>` of palette indices. `find_closest` stops cloning `String`s per
call; it returns an index.

### 2.4 Neighbours with the same pattern

- `Shape::for_each_point` callers in src/building that call `normal_at`
  unconditionally (grep for `normal_at(` in src/building): switched to the
  `uses_normal` gate.
- `CurveShape::normal_at` and `BezierShape::normal_at` (nearest point on
  a curve per voxel): if they are O(segments) per call, they get a cached
  nearest-segment grid built once from the shape's bounds; if they are
  already O(1) or cheap, leave them and say so in the plan.
- `HollowShape` and `CompositeShape` forward `uses_normal` decisions
  through to their inner shapes' costs by not calling `normal_at` unless
  asked.

### 2.5 Bridge and export APIs

New Diplomat bridge methods on `Schematic` (regenerated bindings for
PHP, JS, Python, Kotlin through tools/gen-bindings.sh, with the CI
determinism gate):

- `count_blocks_json() -> String`: `{ "minecraft:stone": 123, ... }` over
  non-air blocks, one pass, no per-block allocation.
- `replace_blocks_json(map_json: &str) -> u64`: applies a from-id to
  to-id map in place and returns the number of blocks changed.
- `non_air_blocks_packed() -> Box<[u8]>` (or the Diplomat slice type
  the repo uses for bytes): a compact export, little-endian `u32 count`,
  then per block `i32 x, i32 y, i32 z, u16 palette_index`, followed by
  the palette as JSON length-prefixed. Documented in the API reference.
- `get_all_blocks_json` keeps its shape but is documented as including
  air and discouraged in favour of `get_non_air_blocks_json` and the two
  above.

### 2.6 Progress

`Voxelizer::shape_from_glb`, `shape_from_obj` and
`schematic_from_glb_textured` accept an optional progress sink in the
Rust API (a `&mut dyn FnMut(f32)` or the crate's existing progress
trait); the bridge exposes a `Voxelizer::*_with_progress` variant only if
Diplomat supports callbacks in this repo already (check `ProgressSink` or
similar in src/bridge); otherwise this item is dropped and stated.

## 3. Benchmarks and tests

- `benches/voxelize_bench.rs` (criterion): the 5,001-triangle sphere at
  32, 64, 128 for shell and solid fill with `SolidBrush`, and the textured
  BoxTextured GLB (find it under tests or examples; otherwise a generated
  textured cube) at 64 and 128. Reported in the plan and the release
  notes before and after.
- A regression test in src/voxelize (not criterion) asserting the
  size-128 solid fill of the sphere completes under 2 s in release mode
  (skipped in debug builds via `cfg!(debug_assertions)`), and that the
  block set is byte-identical to the pre-change implementation on a
  size-32 case (golden block list hashed with sha256, generated before the
  change from v0.10.16 and committed as a test fixture).
- Unit tests: `uses_normal` gating (a counting shape proves `normal_at`
  is not called for a solid brush and is called for a shaded brush);
  surface field ids match the ring-search result on a small mesh for every
  surface voxel; the textured memo returns the same block as the uncached
  path; `count_blocks_json`, `replace_blocks_json` and the packed export
  round-trip on a small schematic; bindings smoke tests for the new
  methods in the JS and Python bridge tests that exist.
- Everything the CI gates run: cargo test, gen-bindings diff, bridge
  build, wasm32 check, mkdocs strict.

## 4. Release

Version 0.10.16 to 0.10.17 in Cargo.toml, Cargo.lock,
bindings/python/pyproject.toml and RELEASE_NOTES.md, tag v0.10.17, pushed
to master; CI publishes. Then schemat.io (separate work): pin 0.10.17,
use `count_blocks_json` in the worker and PHP, `replace_blocks_json` in
the replace pass, drop the JSON tally, raise the size cap back to 256.

## 5. Build and bench host

Compilation and benchmarks run on root@schematio0 (16 cores, 125 GB),
synced from the local worktree by rsync (no git on the server side);
commits happen locally. Wasm builds use the same box with the
wasm32-unknown-unknown target.

## 6. Out of scope

Multithreaded wasm; a BVH for arbitrary point queries outside voxelization;
LOD or decimation of input meshes; changes to the mesher or renderer.
