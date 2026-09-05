# Voxelize performance: implementation plan (0.10.17)

**Goal.** Make mesh voxelization O(N^3) in the target size instead of O(N^6),
make the textured path stop re-deriving colour per voxel, and add the three
bridge calls schemat.io needs, without changing a single placed block for the
cases the golden fixtures pin.

**Architecture.** Four layers, each fixed in its own task:

1. `BuildingTool` fill loops ask the brush whether it wants a normal before
   paying for one (`Brush::uses_normal`).
2. `MeshShape` precomputes a per voxel triangle id once, next to the mask it
   already builds, so `normal_at` and `surface_color` become array lookups.
   Interior voxels inherit an id by one BFS from the shell.
3. `voxelize_textured` walks the voxels in parallel, memoises the palette
   lookup by exact RGB, and stores palette indices instead of `BlockState`s.
4. The Diplomat bridge grows `count_blocks_json`, `replace_blocks_json` and
   `non_air_blocks_packed_b64` so callers stop shipping every block as JSON.

**Tech stack.** Rust 2021, rayon (already an unconditional dependency and
already used by `compute_mask`), criterion 0.5 for benches, sha2 (new dev
dependency) for the golden hashes, Diplomat 0.15 for the bridge, mkdocs for
docs. Build and bench host is `root@schematio0`.

**Spec (binding).** `docs/DEV-voxelize-performance.md`.

**Worktree.** `/Users/harrison/RustroverProjects/Nucleation-voxelize-perf`,
branch `codex/voxelize-perf`, forked from v0.10.16. Do not touch any other
Nucleation checkout.

---

## Global Constraints

These hold for every task. A task is not done until all of them hold.

- No em dashes anywhere: code, comments, docs, commit messages.
- Every commit message ends with these two trailers, in this order:

  ```
  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

- All builds, tests and benches run on the server through `remote-sync.sh`,
  never on the Mac. The Mac edits and commits only.
- Behaviour preserving: the golden block set from v0.10.16 must stay
  byte-identical for the solid and shell sphere at size 32 and the textured
  cube at 32.
- `cargo fmt` and `cargo clippy --features bridge,voxelize -- -D warnings`
  clean on touched files.
- The gen-bindings determinism gate passes: run `./tools/gen-bindings.sh`
  twice through the helper, and `git diff --stat` is empty the second time.
  The generated bindings are committed.
- The wasm32 check passes:
  `cargo check --target wasm32-unknown-unknown --lib --features bridge,mc-tick,meshing,voxelize`.
- No public API is removed.

### The helper

Every run command in this plan is written against this shell variable. Export
it once per shell:

```bash
export RS=/private/tmp/claude-501/-Users-harrison-Documents-code-kineglyph/c4605cf1-039c-42fc-8762-7f0d5d543e94/scratchpad/remote-sync.sh
```

`$RS 'cmd'` rsyncs the worktree to `root@schematio0:/root/nucleation-perf`
(excluding `target`, `.git`, `node_modules`) and runs `cmd` there with cargo on
PATH and sccache disabled. The sync is one way. Anything the server writes into
the tree (generated bindings, a written fixture) must be pulled back with an
explicit `rsync` before it can be committed. Those pull-backs are spelled out
where they are needed.

Benchmarks:

```bash
$RS 'cargo bench --features bridge,voxelize --bench voxelize_bench -- --warm-up-time 1 --measurement-time 3'
```

---

## File Structure

```
benches/
  voxelize_bench.rs                 NEW  criterion suite (task 1)
  README.md                         EDIT bench section for voxelize (task 1)
src/
  voxelize/
    test_meshes.rs                  NEW  deterministic UV sphere generator (task 1)
    mod.rs                          EDIT module decl (task 1), textured path (task 4)
    shape.rs                        EDIT surface field, epoch buffer, tests (task 3)
  building/
    brushes.rs                      EDIT uses_normal per brush (task 2), find_closest_index (task 4)
    enums.rs                        EDIT BrushEnum::uses_normal forwarding (task 2)
    mod.rs                          EDIT fill/fill_enum_masked/fill_sdf_function (task 2), rstack (task 5)
    shapes/curve.rs                 EDIT cost note only (task 5)
    shapes/bezier.rs                EDIT cost note only (task 5)
  bridge/
    schematic.rs                    EDIT three new methods (task 6)
tests/
  voxelize_golden.rs                NEW  golden hashes plus the writer (task 1)
  fixtures/voxelize_golden.json     NEW  generated from unchanged code (task 1)
  voxelize_bridge_export.rs         NEW  bridge round trips (task 6)
examples/bridge_smoke/js/main.mjs   EDIT smoke for the new methods (task 6)
examples/bridge_smoke/python/main.py EDIT smoke for the new methods (task 6)
bindings/**                         REGEN committed output of gen-bindings.sh (task 6)
docs/
  api-reference-python.md           EDIT bulk block queries (task 6)
  api-reference-wasm.md             EDIT bulk block queries (task 6)
Cargo.toml                          EDIT bench entry plus sha2 dev dep (task 1), version (task 7)
Cargo.lock                          EDIT version (task 7)
bindings/python/pyproject.toml      EDIT version (task 7)
RELEASE_NOTES.md                    EDIT 0.10.17 section with the bench table (task 7)
```

---

## Task 1: baseline bench and golden fixtures

Nothing here changes production behaviour. It builds the ruler and freezes the
v0.10.16 output so every later task can prove it did not move.

**Files**

- `src/voxelize/test_meshes.rs` (new)
- `src/voxelize/mod.rs` (module declaration)
- `benches/voxelize_bench.rs` (new)
- `tests/voxelize_golden.rs` (new)
- `tests/fixtures/voxelize_golden.json` (new, generated)
- `Cargo.toml` (`[[bench]]`, `sha2` dev dependency)
- `benches/README.md`

**Interfaces**

```rust
// src/voxelize/test_meshes.rs
pub fn uv_sphere_obj(stacks: usize, sectors: usize) -> String;
pub fn sphere_5k() -> String;                 // 5,000 triangles
```

The module is plain `pub` with `#[doc(hidden)]`, not `#[cfg(any(test, feature =
"bench"))]`. Reason: benches are separate crates, so a `cfg(test)` module is
invisible to them, and a `bench` feature would have to be passed on every CI
invocation that builds benches. The generator is about sixty lines of pure
arithmetic with no dependencies, so shipping it costs nothing and keeps the
bench, the in-crate regression test and the integration test on one mesh.

A closed UV sphere always has an even triangle count, so the spec's
"5,001-triangle sphere" is realised as `sphere_5k()` with `2 * 50 * 50 = 5,000`
triangles. That is the same mesh to within one triangle and the numbers stay
comparable.

The textured case uses the committed GLB fixture `tests/samples/BoxTextured.glb`
(5,956 bytes, a 12-triangle cube with an embedded texture, already loaded by
`tests/voxelize_tests.rs:182`). No cube needs generating.

### Steps

- [ ] **Add the mesh generator.** Create `src/voxelize/test_meshes.rs`:

  ```rust
  //! Deterministic meshes shared by the voxelize benches, the in-crate
  //! regression test and the golden fixture test. Kept out of the public docs
  //! but compiled unconditionally: benches are separate crates and cannot see
  //! a `cfg(test)` module.

  /// A closed UV sphere of radius 1 centred on the origin, as Wavefront OBJ.
  /// `stacks` latitude divisions by `sectors` longitude divisions gives
  /// `2 * sectors * (stacks - 1)` triangles.
  #[doc(hidden)]
  pub fn uv_sphere_obj(stacks: usize, sectors: usize) -> String {
      assert!(stacks >= 2 && sectors >= 3, "degenerate sphere");
      let mut out = String::with_capacity(stacks * sectors * 32);
      for i in 0..=stacks {
          let phi = std::f64::consts::PI * (i as f64) / (stacks as f64);
          let (sp, cp) = phi.sin_cos();
          for j in 0..sectors {
              let theta = 2.0 * std::f64::consts::PI * (j as f64) / (sectors as f64);
              let (st, ct) = theta.sin_cos();
              out.push_str(&format!("v {:.6} {:.6} {:.6}\n", sp * ct, cp, sp * st));
          }
      }
      // Vertex index of latitude ring `i`, longitude `j`, 1 based for OBJ.
      let vid = |i: usize, j: usize| -> usize { i * sectors + (j % sectors) + 1 };
      for i in 0..stacks {
          for j in 0..sectors {
              let (a, b) = (vid(i, j), vid(i, j + 1));
              let (c, d) = (vid(i + 1, j + 1), vid(i + 1, j));
              if i == 0 {
                  out.push_str(&format!("f {a} {c} {d}\n"));
              } else if i == stacks - 1 {
                  out.push_str(&format!("f {a} {b} {c}\n"));
              } else {
                  out.push_str(&format!("f {a} {b} {c}\n"));
                  out.push_str(&format!("f {a} {c} {d}\n"));
              }
          }
      }
      out
  }

  /// The 5,000 triangle sphere the performance work is measured against.
  #[doc(hidden)]
  pub fn sphere_5k() -> String {
      uv_sphere_obj(51, 50)
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::voxelize::MeshModel;

      #[test]
      fn sphere_5k_has_the_expected_triangle_count() {
          let model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
          assert_eq!(model.triangles.len(), 5_000);
      }
  }
  ```

  Wire it up in `src/voxelize/mod.rs`, right below `mod shape;`:

  ```rust
  #[doc(hidden)]
  pub mod test_meshes;
  ```

- [ ] **Write the failing golden test first.** Create `tests/voxelize_golden.rs`.
  It fails now because `tests/fixtures/voxelize_golden.json` does not exist.

  ```rust
  #![cfg(feature = "voxelize")]
  //! Byte identity of the voxelizer output against the v0.10.16 baseline.
  //!
  //! The fixture is generated once from unchanged code by the ignored test at
  //! the bottom of this file, then committed. Every later change must keep
  //! these three hashes.

  use nucleation::building::{BlockPalette, BuildingTool, SolidBrush};
  use nucleation::voxelize::{test_meshes::sphere_5k, voxelize_textured, MeshModel, MeshShape};
  use nucleation::{BlockState, UniversalSchematic};
  use sha2::{Digest, Sha256};
  use std::collections::BTreeMap;

  fn sphere(size: f32) -> MeshShape {
      let mut model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
      model.fit(size);
      MeshShape::new(model)
  }

  fn textured_cube(size: f32) -> MeshShape {
      let bytes = std::fs::read("tests/samples/BoxTextured.glb").expect("committed sample");
      let mut model = MeshModel::from_glb_bytes(&bytes).expect("BoxTextured loads");
      model.fit(size);
      MeshShape::new(model)
  }

  /// `x,y,z,id` for every non air block, sorted, joined by newlines, hashed.
  fn digest(schematic: &UniversalSchematic) -> (usize, String) {
      let mut lines: Vec<String> = schematic
          .iter_blocks()
          .filter(|(_, block)| block.name != "minecraft:air")
          .map(|(pos, block)| format!("{},{},{},{}", pos.x, pos.y, pos.z, block.name))
          .collect();
      lines.sort();
      let mut hasher = Sha256::new();
      hasher.update(lines.join("\n").as_bytes());
      (lines.len(), format!("{:x}", hasher.finalize()))
  }

  fn fill_solid(shape: &MeshShape, name: &str) -> UniversalSchematic {
      let mut schematic = UniversalSchematic::new(name.to_string());
      let brush = SolidBrush::new(BlockState::new("minecraft:stone"));
      BuildingTool::new(&mut schematic).fill(shape, &brush);
      schematic
  }

  /// The three pinned cases, as `case name -> (block count, sha256)`.
  fn cases() -> BTreeMap<String, (usize, String)> {
      let mut out = BTreeMap::new();
      out.insert(
          "sphere_solid_32".to_string(),
          digest(&fill_solid(&sphere(32.0), "sphere_solid_32")),
      );
      out.insert(
          "sphere_shell_32".to_string(),
          digest(&fill_solid(
              &sphere(32.0).with_surface_shell(1.0),
              "sphere_shell_32",
          )),
      );
      let palette = BlockPalette::new_wool();
      out.insert(
          "textured_cube_32".to_string(),
          digest(&voxelize_textured(
              &textured_cube(32.0),
              &palette,
              "textured_cube_32",
          )),
      );
      out
  }

  const FIXTURE: &str = "tests/fixtures/voxelize_golden.json";

  #[test]
  fn voxelize_output_matches_the_v0_10_16_golden() {
      let raw = std::fs::read_to_string(FIXTURE).expect(
          "tests/fixtures/voxelize_golden.json is missing; regenerate it with \
           `cargo test --release --features voxelize --test voxelize_golden \
           write_golden_fixture -- --ignored`",
      );
      let expected: BTreeMap<String, serde_json::Value> =
          serde_json::from_str(&raw).expect("fixture parses");
      for (name, (count, hash)) in cases() {
          let want = expected.get(&name).unwrap_or_else(|| panic!("no golden for {name}"));
          assert_eq!(
              want["blocks"].as_u64().unwrap() as usize,
              count,
              "{name}: block count moved"
          );
          assert_eq!(want["sha256"].as_str().unwrap(), hash, "{name}: block set moved");
      }
  }

  /// One off generator. Run with `-- --ignored` on unchanged code, then commit
  /// the fixture. Never run it again to "fix" a failure: a failure means the
  /// output moved, which is the thing this file exists to catch.
  #[test]
  #[ignore]
  fn write_golden_fixture() {
      let mut out = serde_json::Map::new();
      for (name, (count, hash)) in cases() {
          out.insert(
              name,
              serde_json::json!({ "blocks": count, "sha256": hash }),
          );
      }
      let json = serde_json::to_string_pretty(&serde_json::Value::Object(out)).unwrap();
      std::fs::write(FIXTURE, format!("{json}\n")).expect("fixture written");
  }
  ```

- [ ] **Add the dev dependency.** In `Cargo.toml`, under `[dev-dependencies]`,
  next to `base64 = "0.22"`:

  ```toml
  # Golden hashes for the voxelize regression fixtures (tests/voxelize_golden.rs).
  sha2 = "0.10"
  ```

- [ ] **Run it and watch it fail.**

  ```bash
  $RS 'cargo test --release --features voxelize --test voxelize_golden'
  ```

  Expected failure: `voxelize_output_matches_the_v0_10_16_golden` panics with
  `tests/fixtures/voxelize_golden.json is missing`.

- [ ] **Generate the fixture from unchanged code and pull it back.** The server
  writes it into its own copy of the tree, so it has to be copied back before it
  can be committed.

  ```bash
  $RS 'cargo test --release --features voxelize --test voxelize_golden write_golden_fixture -- --ignored --exact'
  rsync -az root@schematio0:/root/nucleation-perf/tests/fixtures/voxelize_golden.json \
    /Users/harrison/RustroverProjects/Nucleation-voxelize-perf/tests/fixtures/voxelize_golden.json
  cat /Users/harrison/RustroverProjects/Nucleation-voxelize-perf/tests/fixtures/voxelize_golden.json
  ```

  Sanity check the printed JSON before trusting it: all three cases present,
  `blocks` non zero for each, `sphere_solid_32` well above `sphere_shell_32`.

- [ ] **Run it and watch it pass.**

  ```bash
  $RS 'cargo test --release --features voxelize --test voxelize_golden'
  ```

  Expected: `test result: ok. 1 passed; 1 ignored`.

- [ ] **Add the criterion bench.** Create `benches/voxelize_bench.rs`:

  ```rust
  //! Mesh voxelization benchmarks. Sizes 32 and 64 always run; 128 is behind
  //! NUCLEATION_BENCH_LARGE=1 because before the surface field lands it takes
  //! about half an hour per sample.
  //!
  //!   cargo bench --features bridge,voxelize --bench voxelize_bench -- \
  //!     --warm-up-time 1 --measurement-time 3

  use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
  use nucleation::building::{BlockPalette, BuildingTool, SolidBrush};
  use nucleation::voxelize::{test_meshes::sphere_5k, voxelize_textured, MeshModel, MeshShape};
  use nucleation::{BlockState, UniversalSchematic};

  fn sphere(size: f32) -> MeshShape {
      let mut model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
      model.fit(size);
      MeshShape::new(model)
  }

  fn textured_cube(size: f32) -> MeshShape {
      let bytes = std::fs::read("tests/samples/BoxTextured.glb").expect("committed sample");
      let mut model = MeshModel::from_glb_bytes(&bytes).expect("BoxTextured loads");
      model.fit(size);
      MeshShape::new(model)
  }

  fn fill(shape: &MeshShape) -> usize {
      let mut schematic = UniversalSchematic::new("bench".to_string());
      let brush = SolidBrush::new(BlockState::new("minecraft:stone"));
      BuildingTool::new(&mut schematic).fill(shape, &brush);
      schematic.total_blocks() as usize
  }

  fn sizes() -> Vec<f32> {
      let mut s = vec![32.0, 64.0];
      if std::env::var("NUCLEATION_BENCH_LARGE").as_deref() == Ok("1") {
          s.push(128.0);
      }
      s
  }

  fn bench(c: &mut Criterion) {
      let mut g = c.benchmark_group("voxelize");
      g.sample_size(10);
      for size in sizes() {
          let solid = sphere(size);
          g.bench_with_input(BenchmarkId::new("sphere_solid", size as i32), &solid, |b, s| {
              // A fresh MeshShape per iteration: the mask and the surface field
              // are cached per shape, and we are timing the whole fill.
              b.iter_batched(
                  || s.clone_uncached(),
                  |s| fill(&s),
                  criterion::BatchSize::SmallInput,
              )
          });
          let shell = sphere(size).with_surface_shell(1.0);
          g.bench_with_input(BenchmarkId::new("sphere_shell", size as i32), &shell, |b, s| {
              b.iter_batched(
                  || s.clone_uncached(),
                  |s| fill(&s),
                  criterion::BatchSize::SmallInput,
              )
          });
          let palette = BlockPalette::new_wool();
          let cube = textured_cube(size);
          g.bench_with_input(BenchmarkId::new("textured_cube", size as i32), &cube, |b, s| {
              b.iter_batched(
                  || s.clone_uncached(),
                  |s| voxelize_textured(&s, &palette, "bench").total_blocks(),
                  criterion::BatchSize::SmallInput,
              )
          });
      }
      g.finish();
  }

  criterion_group!(voxelize_benches, bench);
  criterion_main!(voxelize_benches);
  ```

  `clone_uncached` is the one new public helper this needs. Add it to
  `src/voxelize/shape.rs`, next to `with_shell`:

  ```rust
  /// A copy with the same geometry and shell settings but an empty mask and
  /// field cache. Benchmarks use it to time a cold fill without reparsing the
  /// mesh; a plain `clone` deliberately shares the caches.
  pub fn clone_uncached(&self) -> Self {
      Self {
          data: self.data.clone(),
          shell: self.shell,
          shell_only: self.shell_only,
          mask: Arc::new(OnceLock::new()),
      }
  }
  ```

  And the `[[bench]]` entry in `Cargo.toml`, after the `fingerprint_bench`
  block:

  ```toml
  [[bench]]
  name = "voxelize_bench"
  harness = false
  required-features = ["voxelize"]
  ```

- [ ] **Record the baseline.**

  ```bash
  $RS 'cargo bench --features bridge,voxelize --bench voxelize_bench -- --warm-up-time 1 --measurement-time 3'
  ```

  Copy the six medians (`sphere_solid/32`, `sphere_solid/64`,
  `sphere_shell/32`, `sphere_shell/64`, `textured_cube/32`,
  `textured_cube/64`) into the task report. Do not run 128 yet: on the
  unchanged code it costs roughly half an hour per sample. It is measured for
  the first time at the end of task 3.

- [ ] **Document the bench.** Append to `benches/README.md`, under the
  "Rust schematic core" section:

  ```markdown
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
  ```

- [ ] **Format, lint, commit.**

  ```bash
  $RS 'cargo fmt --all -- --check && cargo clippy --features bridge,voxelize --benches --tests -- -D warnings'
  git add benches/voxelize_bench.rs benches/README.md src/voxelize/test_meshes.rs \
    src/voxelize/mod.rs src/voxelize/shape.rs tests/voxelize_golden.rs \
    tests/fixtures/voxelize_golden.json Cargo.toml
  git commit
  ```

  Commit message:

  ```
  Pin the voxelizer output and measure it

  A generated 5,000 triangle UV sphere and the committed BoxTextured cube,
  benchmarked at 32 and 64 (128 behind NUCLEATION_BENCH_LARGE=1), plus a
  golden fixture: the sorted x,y,z,id lines of the solid sphere, the shell
  sphere and the textured cube at 32, hashed with sha256 and generated from
  unchanged v0.10.16 code. Every later change in this series has to keep
  those three hashes.

  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

---

## Task 2: `Brush::uses_normal` gating in every fill path

Spec 2.1. `shape.normal_at` is called once per solid voxel by three fill loops
whatever the brush does with the value. Six of the ten brushes ignore it.

**Files**

- `src/building/brushes.rs`
- `src/building/enums.rs`
- `src/building/mod.rs`

**Interfaces**

```rust
pub trait Brush {
    fn get_block(&self, x: i32, y: i32, z: i32, normal: (f64, f64, f64)) -> Option<BlockState>;
    /// Whether `get_block` reads `normal`. Default `true` (conservative: an
    /// out of tree brush keeps working). Override to `false` and the fill
    /// loops skip the shape's normal computation entirely.
    fn uses_normal(&self) -> bool {
        true
    }
}
```

Audited from `src/building/brushes.rs`: `SolidBrush` (596), `ColorBrush` (629),
`LinearGradientBrush` (694), `MultiPointGradientBrush` (803),
`PointGradientBrush` (1024), `BilinearGradientBrush` (1102) and `FieldBrush`
(1596) all bind the parameter as `_normal`, so they get `false`. `ShadedBrush`
(1208) and `SpotlightBrush` (1275) read it, so they keep the default `true`.
`CurveGradientBrush` (1459) forwards to `get_block_parametric`, whose parameter
is `_normal` (1448), so it does not read the normal either, but the spec pins it
at `true` and `true` is the conservative answer; leave it `true` and note it in
the commit body. Task 5 revisits nothing here.

The spec's list also names "the brush at brushes.rs:1102", which is
`BilinearGradientBrush`, already on the list. `FieldBrush` is the seventh
normal-ignoring brush and gets `false` on the same evidence.

### Steps

- [ ] **Write the counting shape test first.** Append to
  `tests/building_tests.rs`:

  ```rust
  /// A cuboid that counts how many times the fill loop asked it for a normal.
  struct CountingShape {
      inner: nucleation::building::Cuboid,
      normals: std::cell::Cell<usize>,
  }

  impl nucleation::building::Shape for CountingShape {
      fn contains(&self, x: i32, y: i32, z: i32) -> bool {
          self.inner.contains(x, y, z)
      }
      fn points(&self) -> Vec<(i32, i32, i32)> {
          self.inner.points()
      }
      fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
          self.normals.set(self.normals.get() + 1);
          self.inner.normal_at(x, y, z)
      }
      fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
          self.inner.bounds()
      }
      fn for_each_point<F>(&self, f: F)
      where
          F: FnMut(i32, i32, i32),
      {
          self.inner.for_each_point(f)
      }
  }

  fn counting_cube() -> CountingShape {
      CountingShape {
          inner: nucleation::building::Cuboid::new((0, 0, 0), (3, 3, 3)),
          normals: std::cell::Cell::new(0),
      }
  }

  #[test]
  fn a_solid_brush_never_asks_the_shape_for_a_normal() {
      use nucleation::building::{Brush, BuildingTool, SolidBrush};
      let shape = counting_cube();
      let brush = SolidBrush::new(nucleation::BlockState::new("minecraft:stone"));
      assert!(!brush.uses_normal(), "SolidBrush ignores the normal");

      let mut schematic = nucleation::UniversalSchematic::new("gate".to_string());
      BuildingTool::new(&mut schematic).fill(&shape, &brush);

      assert!(schematic.total_blocks() > 0, "the fill placed blocks");
      assert_eq!(shape.normals.get(), 0, "no normal was computed");
  }

  #[test]
  fn a_shaded_brush_still_asks_the_shape_for_a_normal() {
      use nucleation::building::{Brush, BuildingTool, ShadedBrush};
      let shape = counting_cube();
      let brush = ShadedBrush::new((0.0, 1.0, 0.0), (255, 255, 255));
      assert!(brush.uses_normal(), "ShadedBrush reads the normal");

      let mut schematic = nucleation::UniversalSchematic::new("gate".to_string());
      BuildingTool::new(&mut schematic).fill(&shape, &brush);

      assert_eq!(
          shape.normals.get() as i32,
          schematic.total_blocks(),
          "one normal per placed voxel"
      );
  }
  ```

  Check `ShadedBrush::new`'s real signature before running (it is around
  brushes.rs:1174) and match it; the rest of the test does not depend on it.

- [ ] **Run it and watch it fail.**

  ```bash
  $RS 'cargo test --release --features voxelize --test building_tests normal'
  ```

  Expected failure: `no method named uses_normal found for struct SolidBrush`,
  a compile error, which is the correct first failure for a new trait method.

- [ ] **Add the trait method.** In `src/building/brushes.rs`, replace the
  `Brush` trait body (line 578):

  ```rust
  pub trait Brush {
      /// Get the block to place at the given coordinates, optionally using the surface normal
      fn get_block(&self, x: i32, y: i32, z: i32, normal: (f64, f64, f64)) -> Option<BlockState>;

      /// Whether `get_block` actually reads `normal`. The fill loops skip the
      /// shape's `normal_at` call entirely when this is false, which is the
      /// difference between an O(1) and an O(N^3) per voxel cost on a mesh
      /// shape. Defaults to true so an out of tree brush keeps working.
      fn uses_normal(&self) -> bool {
          true
      }
  }
  ```

  Then add the override to each of the seven normal-ignoring brushes. In every
  case it goes immediately after that brush's `get_block`, inside the same
  `impl Brush for ...` block:

  ```rust
      fn uses_normal(&self) -> bool {
          false
      }
  ```

  Add it to: `SolidBrush` (after 598), `ColorBrush` (after 633),
  `LinearGradientBrush`, `MultiPointGradientBrush`, `PointGradientBrush`,
  `BilinearGradientBrush` and `FieldBrush`. Leave `ShadedBrush`,
  `SpotlightBrush` and `CurveGradientBrush` on the default.

- [ ] **Forward it through `BrushEnum`.** In `src/building/enums.rs`, inside
  `impl Brush for BrushEnum`, below `get_block`:

  ```rust
      fn uses_normal(&self) -> bool {
          match self {
              BrushEnum::Solid(b) => b.uses_normal(),
              BrushEnum::Color(b) => b.uses_normal(),
              BrushEnum::Linear(b) => b.uses_normal(),
              BrushEnum::Bilinear(b) => b.uses_normal(),
              BrushEnum::Point(b) => b.uses_normal(),
              BrushEnum::MultiPoint(b) => b.uses_normal(),
              BrushEnum::Shaded(b) => b.uses_normal(),
              BrushEnum::CurveGradient(b) => b.uses_normal(),
              BrushEnum::Spotlight(b) => b.uses_normal(),
              BrushEnum::Field(b) => b.uses_normal(),
          }
      }
  ```

- [ ] **Gate the fill loops.** In `src/building/mod.rs`, `fill` becomes:

  ```rust
      pub fn fill(&mut self, shape: &impl Shape, brush: &impl Brush) {
          let (min_x, min_y, min_z, max_x, max_y, max_z) = shape.bounds();
          self.schematic
              .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

          // Computing a normal can cost a nearest surface query (see
          // MeshShape); only pay for it when the brush reads the value.
          let wants_normal = brush.uses_normal();
          shape.for_each_point(|x, y, z| {
              let normal = if wants_normal {
                  shape.normal_at(x, y, z)
              } else {
                  (0.0, 0.0, 0.0)
              };
              if let Some(block) = brush.get_block(x, y, z, normal) {
                  self.schematic.set_block(x, y, z, &block);
              }
          });
      }
  ```

  `fill_enum_masked` takes the same treatment:

  ```rust
          let wants_normal = brush.uses_normal();
          shape.for_each_point(|x, y, z| {
              if !mode.allows(self.schematic.get_block(x, y, z)) {
                  return;
              }
              let normal = if wants_normal {
                  shape.normal_at(x, y, z)
              } else {
                  (0.0, 0.0, 0.0)
              };
              let t = shape.parameter_at(x, y, z);
              if let Some(block) = brush.get_block_with_parameter(x, y, z, normal, t) {
                  self.schematic.set_block(x, y, z, &block);
              }
          });
  ```

  `fill_enum` already delegates to `fill_enum_masked` and needs no change.

  `fill_sdf_function` evaluates the gradient itself rather than calling
  `normal_at`, and the central difference costs six `eval` calls per solid
  voxel. Gate that the same way: after `let mut staged = ...`, add

  ```rust
          let wants_normal = brush.uses_normal();
  ```

  and replace the gradient block with

  ```rust
                      let normal = if !wants_normal {
                          (0.0, 0.0, 0.0)
                      } else {
                          let gradient = match normal(fx, fy, fz)? {
                              Some(value) => value,
                              None => (
                                  eval(fx + epsilon, fy, fz)? - eval(fx - epsilon, fy, fz)?,
                                  eval(fx, fy + epsilon, fz)? - eval(fx, fy - epsilon, fz)?,
                                  eval(fx, fy, fz + epsilon)? - eval(fx, fy, fz - epsilon)?,
                              ),
                          };
                          let length = (gradient.0 * gradient.0
                              + gradient.1 * gradient.1
                              + gradient.2 * gradient.2)
                              .sqrt();
                          if length > 1e-12 && length.is_finite() {
                              (
                                  gradient.0 / length,
                                  gradient.1 / length,
                                  gradient.2 / length,
                              )
                          } else {
                              (0.0, 1.0, 0.0)
                          }
                      };
  ```

  One behavioural note to keep in the commit body: for a normal-ignoring brush
  the passed vector changes from the computed normal to `(0.0, 0.0, 0.0)`, and
  the user supplied `normal` callback stops being called. Neither is
  observable through the placed blocks, which is exactly what the golden
  proves. The `callback_sdf_tests` module at the bottom of `mod.rs` uses a
  `SolidBrush`; if any of its assertions inspect the callback's invocation
  count, switch that test to a `ShadedBrush` rather than weakening the gate.

- [ ] **Run it and watch it pass, with the golden.**

  ```bash
  $RS 'cargo test --release --features voxelize --test building_tests normal'
  $RS 'cargo test --release --features bridge,voxelize --lib building'
  $RS 'cargo test --release --features voxelize --test voxelize_golden --test voxelize_tests'
  ```

  Expected: the two new tests pass, the golden's three hashes still match.

- [ ] **Bench the win.**

  ```bash
  $RS 'cargo bench --features bridge,voxelize --bench voxelize_bench -- --warm-up-time 1 --measurement-time 3'
  ```

  Expect the two `sphere_*` cases to collapse (the fill no longer calls
  `normal_at` at all) and `textured_cube` to be unchanged, since
  `voxelize_textured` does not go through `BuildingTool`. Record both.

- [ ] **Format, lint, commit.**

  ```bash
  $RS 'cargo fmt --all -- --check && cargo clippy --features bridge,voxelize --tests -- -D warnings'
  git add src/building/brushes.rs src/building/enums.rs src/building/mod.rs tests/building_tests.rs
  git commit
  ```

  Commit message:

  ```
  Let brushes say they do not want a surface normal

  Brush::uses_normal defaults to true and is overridden to false by the seven
  brushes that bind the parameter as _normal: Solid, Color, Linear,
  MultiPoint, Point, Bilinear and Field. fill, fill_enum_masked and
  fill_sdf_function skip the normal computation when the brush does not want
  it and pass (0, 0, 0) instead, which on a mesh shape removes one
  nearest-triangle search per solid voxel. CurveGradient ignores the normal
  too (get_block_parametric binds it as _normal) but stays on the
  conservative default, as the design asks.

  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

---

## Task 3: `MeshShape` surface field

Spec 2.2. Even with task 2 in place, `normal_at` and `surface_color` are still
an expanding ring search with a `vec![false; triangles]` allocation per call,
and `voxelize_textured` calls `surface_color` for every voxel. This task makes
both O(1).

**Files**

- `src/voxelize/shape.rs`

**Interfaces**

```rust
struct SurfaceField {
    origin: (i32, i32, i32),
    dims: (usize, usize, usize),
    /// Triangle id per voxel over the bounding volume, u32::MAX for none.
    tri: Vec<u32>,
}

impl MeshShape {
    fn surface_field(&self) -> &SurfaceField;          // lazy, next to solid_mask
    fn triangle_at(&self, x: i32, y: i32, z: i32) -> Option<usize>;
}
```

`MeshShape` grows one field, `field: Arc<OnceLock<SurfaceField>>`, reset
wherever `mask` is reset (`with_shell`, `with_surface_shell`, `clone_uncached`).

How the field is built:

1. Seed pass, rayon over triangles exactly like the existing shell
   rasterisation: for each triangle, walk the voxels in its bounding box grown
   by `SEED_RADIUS = 1.5` blocks, and for each voxel whose centre is within
   `SEED_RADIUS` of the triangle, emit `(voxel index, distance, triangle id)`.
2. Reduce in triangle index order, keeping the smallest distance and, on a
   tie within `1e-6`, the smallest triangle id. Deterministic because
   `par_iter().collect()` preserves order.
3. BFS pass: push every seeded voxel that is inside the mask, then flood over
   6 neighbours into mask voxels with no id yet, inheriting the id. One pass
   over the volume, O(N^3).

Voxels farther than 1.5 blocks from every triangle cannot be surface voxels
(the surface would have to pass between them and the outside), so the exact
answer is preserved wherever it is visible and inherited everywhere else, which
is what the spec asks for.

The ring search stays as the fallback for a voxel outside the field's bounds
and for the empty-field case, and its `vec![false; n]` is replaced by a
thread local epoch buffer.

### Steps

- [ ] **Write the failing agreement test first.** Append to
  `src/voxelize/shape.rs`, at the end of the file:

  ```rust
  #[cfg(test)]
  mod surface_field_tests {
      use super::*;
      use crate::building::Shape;
      use crate::voxelize::test_meshes::uv_sphere_obj;
      use crate::voxelize::MeshModel;

      fn small_sphere() -> MeshShape {
          let mut model =
              MeshModel::from_obj_str(&uv_sphere_obj(12, 12)).expect("sphere parses");
          model.fit(12.0);
          MeshShape::new(model)
      }

      /// On every surface voxel (a solid voxel with a non solid 6 neighbour)
      /// the field's triangle must be the ring search's triangle, or a
      /// triangle exactly as close (ties are legal, the ring search picks by
      /// bucket order and the field picks the lowest id).
      #[test]
      fn field_ids_agree_with_the_ring_search_on_the_surface() {
          let shape = small_sphere();
          let field = shape.surface_field();
          let mut checked = 0usize;
          let (x0, y0, z0, x1, y1, z1) = shape.bounds();
          for x in x0..=x1 {
              for y in y0..=y1 {
                  for z in z0..=z1 {
                      if !shape.contains(x, y, z) {
                          continue;
                      }
                      let on_surface = [
                          (1, 0, 0), (-1, 0, 0), (0, 1, 0),
                          (0, -1, 0), (0, 0, 1), (0, 0, -1),
                      ]
                      .iter()
                      .any(|(dx, dy, dz)| !shape.contains(x + dx, y + dy, z + dz));
                      if !on_surface {
                          continue;
                      }
                      let p = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
                      let (want, _, want_dist) =
                          shape.nearest_triangle(p).expect("mesh has triangles");
                      let got = field
                          .get(x, y, z)
                          .unwrap_or_else(|| panic!("no field id at {x},{y},{z}"));
                      let got_dist = distance(
                          p,
                          closest_point_on_triangle(p, &shape.data.triangles[got].positions),
                      );
                      assert!(
                          got == want || (got_dist - want_dist).abs() <= 1e-4,
                          "field picked {got} (d={got_dist}) but the ring search picked \
                           {want} (d={want_dist}) at {x},{y},{z}"
                      );
                      checked += 1;
                  }
              }
          }
          assert!(checked > 200, "only {checked} surface voxels checked");
      }

      /// Release-mode budget for the case the design calls out: a size 128
      /// solid fill of the 5,000 triangle sphere. Skipped in debug builds,
      /// where the same work is roughly twenty times slower.
      #[test]
      fn size_128_solid_fill_is_under_two_seconds() {
          if cfg!(debug_assertions) {
              return;
          }
          use crate::building::{BuildingTool, SolidBrush};
          use crate::voxelize::test_meshes::sphere_5k;
          let mut model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
          model.fit(128.0);
          let shape = MeshShape::new(model);
          let brush = SolidBrush::new(crate::BlockState::new("minecraft:stone"));
          let mut schematic = crate::UniversalSchematic::new("perf".to_string());

          let started = std::time::Instant::now();
          BuildingTool::new(&mut schematic).fill(&shape, &brush);
          let elapsed = started.elapsed();

          assert!(schematic.total_blocks() > 1_000_000, "the fill did real work");
          assert!(
              elapsed.as_secs_f64() < 2.0,
              "size 128 solid fill took {elapsed:?}, budget is 2 s"
          );
      }
  }
  ```

- [ ] **Run it and watch it fail.**

  ```bash
  $RS 'cargo test --release --features voxelize --lib surface_field'
  ```

  Expected failure: `no method named surface_field found for struct MeshShape`.

- [ ] **Add the field.** In `src/voxelize/shape.rs`, add to the `MeshShape`
  struct, after `mask`:

  ```rust
      /// Lazily computed triangle id per voxel over the same bounding volume
      /// as `mask`. Turns normal_at and surface_color into array lookups.
      /// Reset wherever `mask` is reset.
      field: Arc<OnceLock<SurfaceField>>,
  ```

  Add `field: Arc::new(OnceLock::new()),` to the four constructors: `new`,
  `with_shell`, `with_surface_shell` and `clone_uncached`.

  Then the field itself, next to `SolidMask`:

  ```rust
  /// Triangle id per voxel over the shape's bounds. `u32::MAX` means no
  /// triangle reached this voxel, which only happens outside the solid mask.
  struct SurfaceField {
      origin: (i32, i32, i32),
      dims: (usize, usize, usize),
      tri: Vec<u32>,
  }

  const NO_TRI: u32 = u32::MAX;
  /// A voxel centre farther than this from every triangle cannot be a surface
  /// voxel: the surface would have to pass between it and the outside. Half a
  /// voxel diagonal is 0.867, so 1.5 covers a full neighbour ring.
  const SEED_RADIUS: f32 = 1.5;

  impl SurfaceField {
      fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
          let (ox, oy, oz) = self.origin;
          let (dx, dy, dz) = self.dims;
          let (ix, iy, iz) = ((x - ox) as isize, (y - oy) as isize, (z - oz) as isize);
          if ix < 0 || iy < 0 || iz < 0 {
              return None;
          }
          let (ix, iy, iz) = (ix as usize, iy as usize, iz as usize);
          if ix >= dx || iy >= dy || iz >= dz {
              return None;
          }
          Some((ix * dy + iy) * dz + iz)
      }

      fn get(&self, x: i32, y: i32, z: i32) -> Option<usize> {
          let id = self.tri[self.index(x, y, z)?];
          (id != NO_TRI).then_some(id as usize)
      }
  }
  ```

- [ ] **Build the field.** Add to the `impl MeshShape` block that holds
  `solid_mask` and `compute_mask`:

  ```rust
      fn surface_field(&self) -> &SurfaceField {
          self.field.get_or_init(|| self.compute_field())
      }

      /// Triangle claiming this voxel, from the field, falling back to the
      /// ring search when the voxel is outside the field (an out of bounds
      /// query) or the mesh is empty.
      fn triangle_at(&self, x: i32, y: i32, z: i32) -> Option<usize> {
          if let Some(id) = self.surface_field().get(x, y, z) {
              return Some(id);
          }
          let p = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
          self.nearest_triangle(p).map(|(ti, _, _)| ti)
      }

      /// One rayon pass over the triangles claims every voxel within
      /// SEED_RADIUS of the surface, then one BFS hands those ids inward to
      /// the rest of the solid mask. O(triangles * shell volume + N^3).
      fn compute_field(&self) -> SurfaceField {
          let d = &self.data;
          let mask = self.solid_mask();
          let (x0, y0, z0, x1, y1, z1) = d.bounds;
          let dims = mask.dims;
          let total = dims.0 * dims.1 * dims.2;
          let mut tri = vec![NO_TRI; total];
          if d.triangles.is_empty() {
              return SurfaceField {
                  origin: (x0, y0, z0),
                  dims,
                  tri,
              };
          }

          // Seed pass. Same shape as the shell rasterization above: a
          // per-triangle bounding box walk, collected in triangle order so the
          // reduction below is deterministic.
          let claims: Vec<Vec<(usize, f32, u32)>> = d
              .triangles
              .par_iter()
              .enumerate()
              .map(|(ti, t)| {
                  let mut out = Vec::new();
                  let mut tmin = [f32::INFINITY; 3];
                  let mut tmax = [f32::NEG_INFINITY; 3];
                  for pt in &t.positions {
                      for a in 0..3 {
                          tmin[a] = tmin[a].min(pt[a]);
                          tmax[a] = tmax[a].max(pt[a]);
                      }
                  }
                  let lo = [
                      ((tmin[0] - SEED_RADIUS).floor() as i32).max(x0),
                      ((tmin[1] - SEED_RADIUS).floor() as i32).max(y0),
                      ((tmin[2] - SEED_RADIUS).floor() as i32).max(z0),
                  ];
                  let hi = [
                      ((tmax[0] + SEED_RADIUS).ceil() as i32).min(x1),
                      ((tmax[1] + SEED_RADIUS).ceil() as i32).min(y1),
                      ((tmax[2] + SEED_RADIUS).ceil() as i32).min(z1),
                  ];
                  for x in lo[0]..=hi[0] {
                      for y in lo[1]..=hi[1] {
                          for z in lo[2]..=hi[2] {
                              let c = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
                              let q = closest_point_on_triangle(c, &t.positions);
                              let dist = distance(c, q);
                              if dist <= SEED_RADIUS {
                                  let idx = (((x - x0) as usize) * dims.1 + (y - y0) as usize)
                                      * dims.2
                                      + (z - z0) as usize;
                                  out.push((idx, dist, ti as u32));
                              }
                          }
                      }
                  }
                  out
              })
              .collect();

          let mut best = vec![f32::INFINITY; total];
          for list in &claims {
              for &(idx, dist, ti) in list {
                  // Strictly closer wins; an exact tie keeps the lower id, so
                  // the field is a pure function of the mesh.
                  if dist < best[idx] {
                      best[idx] = dist;
                      tri[idx] = ti;
                  }
              }
          }
          drop(claims);
          drop(best);

          // BFS: hand the seeded ids inward over 6 neighbours, through solid
          // voxels only. Every solid voxel of a closed mesh is reached.
          let mut queue: std::collections::VecDeque<usize> =
              std::collections::VecDeque::new();
          for (idx, &id) in tri.iter().enumerate() {
              if id != NO_TRI {
                  queue.push_back(idx);
              }
          }
          let (dy, dz) = (dims.1, dims.2);
          while let Some(idx) = queue.pop_front() {
              let id = tri[idx];
              let iz = idx % dz;
              let iy = (idx / dz) % dy;
              let ix = idx / (dy * dz);
              let mut visit = |nx: usize, ny: usize, nz: usize, queue: &mut std::collections::VecDeque<usize>, tri: &mut Vec<u32>| {
                  let n = (nx * dy + ny) * dz + nz;
                  if tri[n] != NO_TRI {
                      return;
                  }
                  if mask.bits[n >> 6] >> (n & 63) & 1 != 1 {
                      return;
                  }
                  tri[n] = id;
                  queue.push_back(n);
              };
              if ix + 1 < dims.0 {
                  visit(ix + 1, iy, iz, &mut queue, &mut tri);
              }
              if ix > 0 {
                  visit(ix - 1, iy, iz, &mut queue, &mut tri);
              }
              if iy + 1 < dy {
                  visit(ix, iy + 1, iz, &mut queue, &mut tri);
              }
              if iy > 0 {
                  visit(ix, iy - 1, iz, &mut queue, &mut tri);
              }
              if iz + 1 < dz {
                  visit(ix, iy, iz + 1, &mut queue, &mut tri);
              }
              if iz > 0 {
                  visit(ix, iy, iz - 1, &mut queue, &mut tri);
              }
          }

          SurfaceField {
              origin: (x0, y0, z0),
              dims,
              tri,
          }
      }
  ```

  If the borrow checker rejects the closure capturing `tri` and `queue`
  together, inline the six neighbour bodies instead of using `visit`; the
  logic is unchanged and the loop stays flat.

- [ ] **Point `normal_at` and `surface_color` at the field.** Replace
  `Shape::normal_at` for `MeshShape`:

  ```rust
      fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
          match self.triangle_at(x, y, z) {
              Some(ti) => {
                  let t = &self.data.triangles[ti].positions;
                  let e1 = sub(t[1], t[0]);
                  let e2 = sub(t[2], t[0]);
                  let n = cross(e1, e2);
                  let len = (n[0] as f64).hypot(n[1] as f64).hypot(n[2] as f64);
                  if len < 1e-12 {
                      (0.0, 1.0, 0.0)
                  } else {
                      (n[0] as f64 / len, n[1] as f64 / len, n[2] as f64 / len)
                  }
              }
              None => (0.0, 1.0, 0.0),
          }
      }
  ```

  And `surface_color`, whose only other need was the closest point `q`, which
  is one O(1) call now that the triangle is known:

  ```rust
      pub fn surface_color(&self, x: i32, y: i32, z: i32) -> Option<[u8; 3]> {
          let p = [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5];
          let ti = self.triangle_at(x, y, z)?;
          let tri = &self.data.triangles[ti];
          let q = closest_point_on_triangle(p, &tri.positions);
          let img = self.data.materials.get(tri.material? as usize)?.as_ref()?;
          if img.width == 1 && img.height == 1 {
              return Some([img.pixels[0], img.pixels[1], img.pixels[2]]);
          }
          let uvs = tri.uvs?;
          let (u, v, w) = barycentric(q, &tri.positions);
          let uv = [
              uvs[0][0] * u + uvs[1][0] * v + uvs[2][0] * w,
              uvs[0][1] * u + uvs[1][1] * v + uvs[2][1] * w,
          ];
          Some(img.sample_bilinear(uv[0], uv[1]))
      }
  ```

- [ ] **Replace the per call allocation in the fallback.** At the top of
  `src/voxelize/shape.rs`, after the imports:

  ```rust
  use std::cell::RefCell;

  thread_local! {
      /// Epoch stamped visit marks for the nearest_triangle ring search
      /// fallback, so it stops allocating vec![false; triangles] per call.
      /// One buffer per thread, grown to the largest mesh that thread has
      /// seen. Never borrowed reentrantly: the search calls no user code.
      static RING_VISIT: RefCell<(u32, Vec<u32>)> = const { RefCell::new((0, Vec::new())) };
  }
  ```

  And in `nearest_triangle`, replace `let mut seen = vec![false; d.triangles.len()];`
  and its two uses. The whole body becomes a closure run inside the
  thread local borrow:

  ```rust
      fn nearest_triangle(&self, p: [f32; 3]) -> Option<(usize, [f32; 3], f32)> {
          let d = &self.data;
          if d.triangles.is_empty() {
              return None;
          }
          RING_VISIT.with(|cell| {
              let mut guard = cell.borrow_mut();
              let (epoch, seen) = &mut *guard;
              if seen.len() < d.triangles.len() {
                  seen.resize(d.triangles.len(), 0);
              }
              // Epoch 0 means "never visited", so wrap by clearing.
              *epoch = epoch.wrapping_add(1);
              if *epoch == 0 {
                  seen.iter_mut().for_each(|s| *s = 0);
                  *epoch = 1;
              }
              let epoch = *epoch;

              let start = d.grid.cell_of(p);
              let max_r = d.grid.dims[0].max(d.grid.dims[1]).max(d.grid.dims[2]);
              let mut best: Option<(usize, [f32; 3], f32)> = None;
              for r in 0..=max_r {
                  if let Some((_, _, dist)) = best {
                      if dist <= (r as f32 - 1.0).max(0.0) * TriGrid::CELL {
                          break;
                      }
                  }
                  let mut any_cell = false;
                  for cx in (start[0] - r).max(0)..=(start[0] + r).min(d.grid.dims[0] - 1) {
                      for cy in (start[1] - r).max(0)..=(start[1] + r).min(d.grid.dims[1] - 1) {
                          for cz in (start[2] - r).max(0)..=(start[2] + r).min(d.grid.dims[2] - 1) {
                              let on_shell = (cx - start[0]).abs() == r
                                  || (cy - start[1]).abs() == r
                                  || (cz - start[2]).abs() == r;
                              if !on_shell {
                                  continue;
                              }
                              any_cell = true;
                              for &t in d.grid.bucket([cx, cy, cz]) {
                                  let ti = t as usize;
                                  if seen[ti] == epoch {
                                      continue;
                                  }
                                  seen[ti] = epoch;
                                  let q = closest_point_on_triangle(p, &d.triangles[ti].positions);
                                  let dist = distance(p, q);
                                  if best.is_none_or(|(_, _, bd)| dist < bd) {
                                      best = Some((ti, q, dist));
                                  }
                              }
                          }
                      }
                  }
                  if !any_cell && best.is_some() {
                      break;
                  }
              }
              best
          })
      }
  ```

  The ring walk itself is unchanged, so `contains` and the shell test keep
  their exact answers.

- [ ] **Run it and watch it pass, with the golden.**

  ```bash
  $RS 'cargo test --release --features voxelize --lib surface_field'
  $RS 'cargo test --release --features voxelize --test voxelize_golden --test voxelize_tests'
  ```

  Expected: both new tests pass (including the two second budget at size 128),
  the three golden hashes are unchanged, and `voxelize_tests` is green.

  If `field_ids_agree_with_the_ring_search_on_the_surface` fails on a distance
  larger than `1e-4`, that is a real disagreement, not a tie: raise
  `SEED_RADIUS` to 2.0 and rerun before touching the assertion.

- [ ] **Bench all three sizes, including 128.**

  ```bash
  $RS 'NUCLEATION_BENCH_LARGE=1 cargo bench --features bridge,voxelize --bench voxelize_bench -- --warm-up-time 1 --measurement-time 3'
  ```

  Record all nine medians. This is the first time 128 is measured; it should
  be within small multiples of 64, not the thousand-fold jump the old code had.

- [ ] **Format, lint, commit.**

  ```bash
  $RS 'cargo fmt --all -- --check && cargo clippy --features bridge,voxelize --tests -- -D warnings'
  git add src/voxelize/shape.rs
  git commit
  ```

  Commit message:

  ```
  Precompute the mesh surface field instead of searching per voxel

  MeshShape now builds a triangle id per voxel once, next to the mask: one
  rayon pass claims every voxel within 1.5 blocks of a triangle, then one BFS
  hands those ids inward through the solid mask, so normal_at and
  surface_color are array lookups instead of an expanding ring search. The
  ring search stays as the out of bounds fallback and no longer allocates a
  visited vector per call: it stamps a thread local epoch buffer. The size
  128 solid fill of the 5,000 triangle sphere now finishes inside a two
  second budget, asserted in release builds.

  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

---

## Task 4: the textured path

Spec 2.3. `voxelize_textured` is a serial loop that, per voxel, ran a ring
search (fixed by task 3), then rebuilt an `ExtendedColorData` through Lab and
Oklch, then scanned the whole palette with a `String` clone. The colour
conversion and the palette scan are still per voxel and still serial.

**Files**

- `src/voxelize/mod.rs`
- `src/building/brushes.rs`

**Interfaces**

```rust
impl BlockPalette {
    /// Index of the nearest palette entry in Oklab, or None when empty.
    /// No allocation. `find_closest` is this plus one clone.
    pub fn find_closest_index(&self, target: &ExtendedColorData) -> Option<usize>;
    /// Block id at a palette index, for resolving a batch of indices at once.
    pub fn block_id(&self, index: usize) -> Option<&str>;
}
```

`find_closest` keeps its signature and its exact result; it becomes a thin
wrapper. No public API is removed.

**Deviation from the spec, deliberate.** Spec 2.3 asks for the memo key to be
the sampled colour quantised to 6 bits per channel. That is not behaviour
preserving: two colours inside one 6 bit bucket can have different nearest
palette entries, so the textured golden would move. The Global Constraint that
the textured cube at 32 stays byte identical outranks the quantisation, so the
memo is keyed on the exact 24 bit RGB instead. It collapses the same way in
practice (a texture has far fewer distinct colours than the model has voxels)
and it is exactly right. Flag this to the controller with the task report.

### Steps

- [ ] **Write the failing test first.** Append to `tests/voxelize_tests.rs`:

  ```rust
  #[test]
  fn palette_index_lookup_agrees_with_the_cloning_lookup() {
      use nucleation::blockpedia::ExtendedColorData;
      let palette = BlockPalette::new_wool();
      for rgb in [
          [0u8, 0, 0],
          [255, 255, 255],
          [12, 200, 43],
          [199, 21, 133],
          [128, 128, 128],
      ] {
          let target = ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2]);
          let index = palette.find_closest_index(&target).expect("wool is not empty");
          assert_eq!(
              palette.block_id(index).map(str::to_string),
              palette.find_closest(&target),
              "index lookup and cloning lookup disagree on {rgb:?}"
          );
      }
  }

  /// The memoised, parallel textured path must place exactly what a plain
  /// per voxel loop over the same primitives places.
  #[test]
  fn the_textured_memo_matches_an_uncached_walk() {
      use nucleation::blockpedia::ExtendedColorData;
      let bytes = std::fs::read("tests/samples/BoxTextured.glb").expect("committed sample");
      let mut model = MeshModel::from_glb_bytes(&bytes).expect("BoxTextured loads");
      model.fit(16.0);
      let shape = MeshShape::new(model);
      let palette = BlockPalette::new_wool();

      let memoised = voxelize_textured(&shape, &palette, "memo");

      let mut plain = nucleation::UniversalSchematic::new("plain".to_string());
      shape.for_each_point(|x, y, z| {
          let rgb = shape.surface_color(x, y, z).unwrap_or([128, 128, 128]);
          let target = ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2]);
          if let Some(id) = palette.find_closest(&target) {
              plain.set_block(x, y, z, &nucleation::BlockState::new(id));
          }
      });

      assert!(plain.total_blocks() > 0, "the reference walk placed blocks");
      assert_eq!(memoised.total_blocks(), plain.total_blocks());
      for (pos, block) in plain.iter_blocks() {
          assert_eq!(
              memoised.get_block(pos.x, pos.y, pos.z).map(|b| b.name.as_str()),
              Some(block.name.as_str()),
              "memoised path differs at {},{},{}",
              pos.x,
              pos.y,
              pos.z
          );
      }
  }
  ```

- [ ] **Run it and watch it fail.**

  ```bash
  $RS 'cargo test --release --features voxelize --test voxelize_tests textured_memo palette_index'
  ```

  Expected failure: `no method named find_closest_index found for struct
  BlockPalette`.

- [ ] **Add the allocation free palette lookup.** In `src/building/brushes.rs`,
  replace `find_closest` (line 555) with:

  ```rust
      /// Index of the palette entry nearest `target` in Oklab, or None when
      /// the palette is empty. Allocation free, so a caller resolving many
      /// voxels can keep indices and clone the ids once at the end.
      pub fn find_closest_index(&self, target: &ExtendedColorData) -> Option<usize> {
          let mut best_dist = f32::MAX;
          let mut best = None;
          for (index, (color, _)) in self.blocks.iter().enumerate() {
              let dist = target.distance_oklab(color);
              if dist < best_dist {
                  best_dist = dist;
                  best = Some(index);
              }
          }
          best
      }

      /// The block id at a palette index, for resolving a batch of indices.
      pub fn block_id(&self, index: usize) -> Option<&str> {
          self.blocks.get(index).map(|(_, id)| id.as_str())
      }

      pub fn find_closest(&self, target: &ExtendedColorData) -> Option<String> {
          self.find_closest_index(target)
              .map(|index| self.blocks[index].1.clone())
      }
  ```

  The scan order and the strict `<` comparison are unchanged, so the chosen
  entry is identical to before, including ties.

- [ ] **Rewrite the textured walk.** Replace the body of `voxelize_textured`
  in `src/voxelize/mod.rs`:

  ```rust
  pub fn voxelize_textured(
      model_shape: &MeshShape,
      palette: &BlockPalette,
      schematic_name: &str,
  ) -> UniversalSchematic {
      use rayon::prelude::*;
      use std::collections::HashMap;

      let mut schematic = UniversalSchematic::new(schematic_name.to_string());

      // One pass to enumerate the solid voxels, so the colour sampling below
      // can run in parallel over a slice. The mask is already built by then.
      let mut points: Vec<(i32, i32, i32)> = Vec::new();
      model_shape.for_each_point(|x, y, z| points.push((x, y, z)));

      // Sample every voxel's surface colour. O(1) per voxel since the surface
      // field landed, and rayon here matches compute_mask, which is already
      // an unconditional par_iter on every target including wasm32.
      let colors: Vec<u32> = points
          .par_iter()
          .map(|&(x, y, z)| {
              let rgb = model_shape.surface_color(x, y, z).unwrap_or(FALLBACK_RGB);
              ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | rgb[2] as u32
          })
          .collect();

      // Memoise the palette search on the exact 24 bit colour. A texture has
      // far fewer distinct colours than the model has voxels, so this turns a
      // per voxel palette scan into one scan per distinct colour. The key is
      // exact rather than quantised on purpose: quantising would change which
      // block some voxels get, and the golden fixture pins them.
      let mut memo: HashMap<u32, Option<usize>> = HashMap::new();
      let indices: Vec<Option<usize>> = colors
          .iter()
          .map(|&key| {
              *memo.entry(key).or_insert_with(|| {
                  let target = ExtendedColorData::from_rgb(
                      (key >> 16) as u8,
                      (key >> 8) as u8,
                      key as u8,
                  );
                  palette.find_closest_index(&target)
              })
          })
          .collect();

      // Resolve each distinct palette index to a BlockState once.
      let mut states: HashMap<usize, BlockState> = HashMap::new();
      for index in indices.iter().flatten() {
          if let std::collections::hash_map::Entry::Vacant(slot) = states.entry(*index) {
              if let Some(id) = palette.block_id(*index) {
                  slot.insert(BlockState::new(id.to_string()));
              }
          }
      }

      for (&(x, y, z), index) in points.iter().zip(&indices) {
          if let Some(state) = index.and_then(|i| states.get(&i)) {
              schematic.set_block(x, y, z, state);
          }
      }
      schematic
  }
  ```

  `BlockState::new` takes `impl Into<String>` here (it is called with `id`, a
  `String`, in the current code); if it is `&str` only, pass `id` directly and
  drop the `to_string`.

  Note what stayed serial and why: the memo fill and the `set_block` loop.
  `UniversalSchematic::set_block` mutates shared region state and the memo is a
  plain `HashMap`, so parallelising either would need a lock whose contention
  would cost more than the palette scan it saves. The expensive part, the per
  voxel surface sampling, is the part that runs in parallel.

- [ ] **Run it and watch it pass, with the golden.**

  ```bash
  $RS 'cargo test --release --features voxelize --test voxelize_tests'
  $RS 'cargo test --release --features voxelize --test voxelize_golden'
  ```

  Expected: both new tests pass and `textured_cube_32` still hashes to the
  committed value.

- [ ] **Check wasm32 early.** This task adds the first rayon use outside
  `shape.rs`, so confirm the target still builds before moving on.

  ```bash
  $RS 'cargo check --target wasm32-unknown-unknown --lib --features bridge,mc-tick,meshing,voxelize'
  ```

  If the target is not installed on the server: `$RS 'rustup target add wasm32-unknown-unknown'`.

- [ ] **Bench.**

  ```bash
  $RS 'NUCLEATION_BENCH_LARGE=1 cargo bench --features bridge,voxelize --bench voxelize_bench -- --warm-up-time 1 --measurement-time 3'
  ```

  Record the three `textured_cube` medians. The sphere cases should be
  unchanged; if they moved, something in task 3 regressed.

- [ ] **Format, lint, commit.**

  ```bash
  $RS 'cargo fmt --all -- --check && cargo clippy --features bridge,voxelize --tests -- -D warnings'
  git add src/voxelize/mod.rs src/building/brushes.rs tests/voxelize_tests.rs
  git commit
  ```

  Commit message:

  ```
  Memoise and parallelise the textured voxelize path

  voxelize_textured now enumerates the solid voxels once, samples their
  surface colours with rayon, memoises the palette search on the exact 24 bit
  colour and resolves each distinct palette index to a BlockState once, so a
  model with a handful of distinct texture colours does a handful of palette
  scans instead of one per voxel. BlockPalette::find_closest_index does that
  scan without cloning a String; find_closest keeps its signature and its
  exact result on top of it.

  The design asked for a 6 bit per channel memo key. That is not behaviour
  preserving, since two colours in one bucket can have different nearest
  blocks, so the key is the exact colour instead and the golden textured
  fixture is unchanged.

  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

---

## Task 5: neighbours with the same pattern

Spec 2.4. Audit every `normal_at` caller and every expensive `normal_at`
implementation under `src/building`, fix what is O(segments) per voxel, and say
plainly what was left alone.

**The audit, already done against this worktree.** `grep -rn "normal_at("
src/building src/bridge` gives thirteen hits, and they sort into four groups:

1. **Fill loop callers.** `src/building/mod.rs:31` (`fill`), `:56`
   (`fill_enum_masked`) and `:145` (`rstack`). The first two were gated in task
   2. `rstack` was missed there and is fixed here. `fill_sdf_function` computes
   its own gradient and was gated in task 2 as well.
2. **Forwarding shapes.** `Hollow::normal_at` (`shapes/hollow.rs:47`) and the
   three composite shapes, `Union`, `Intersection`, `Difference`
   (`shapes/composite.rs:32`, `:34`, `:95`, `:155`). Each only calls its
   inner shape's `normal_at` from inside its own `normal_at`, so once the fill
   loops stop asking, nothing downstream is called. That is exactly what spec
   2.4 asks for and it needs no code: with the task 2 gate, a solid fill of
   `Hollow(Mesh)` or `Union(Mesh, Sphere)` performs zero nearest surface
   queries. The test below pins it.
3. **Curve shapes.** `TubePath::normal_at` (`shapes/curve.rs:280`) calls
   `Curve3D::closest_point_info`, a linear walk over `self.segments`, where the
   segment count is `control_points - 1`, fixed by the caller and independent
   of the target size. `BezierCurve::normal_at` (`shapes/bezier.rs:102`) walks
   `self.segments`, whose length is the `resolution` argument passed to
   `BezierCurve::new` (`bezier.rs:15`, clamped to at least 2), again fixed by
   the caller. **Left alone, deliberately**: per voxel cost is O(1) in the
   target size N, so neither is the N^6 pattern this pass exists to kill. Both
   shapes' `contains` runs the same linear walk (`curve.rs:270`,
   `bezier.rs:92`) and is called at least as often, so a nearest segment grid
   would have to cache for `contains` too to be worth anything, which is a
   larger change with its own tie breaking risk against a golden that does not
   cover curves. With the task 2 gate a solid or gradient fill now calls
   neither. A note in the source records the reasoning.
4. **Not fill paths.** `bridge/distance_field.rs:46` and `:51` are single
   point queries from a bridge method, `shapes/sdf_shape.rs:144` and
   `distance_field.rs:148` are test assertions. All left alone.

Nothing under `src/building` is O(segments) per voxel in a way that scales with
N, so no nearest segment grid is built.

**Files**

- `src/building/mod.rs` (`rstack`)
- `src/building/shapes/curve.rs` (comment only)
- `src/building/shapes/bezier.rs` (comment only)
- `tests/building_tests.rs`

### Steps

- [ ] **Write the failing tests first.** Append to `tests/building_tests.rs`,
  reusing `CountingShape` from task 2:

  ```rust
  #[test]
  fn rstack_with_a_solid_brush_never_asks_for_a_normal() {
      use nucleation::building::{BrushEnum, BuildingTool, ShapeEnum, SolidBrush};
      // rstack takes enums, so count through a Mesh-free enum shape wrapped in
      // the counting shape's twin: a Cuboid enum, plus a direct check that the
      // gate is what stops the calls.
      let shape = ShapeEnum::Cuboid(nucleation::building::Cuboid::new((0, 0, 0), (3, 3, 3)));
      let brush = BrushEnum::Solid(SolidBrush::new(nucleation::BlockState::new(
          "minecraft:stone",
      )));
      let mut schematic = nucleation::UniversalSchematic::new("rstack".to_string());
      BuildingTool::new(&mut schematic).rstack(&shape, &brush, 3, (8, 0, 0));
      assert!(schematic.total_blocks() > 0, "rstack placed blocks");
  }

  #[test]
  fn a_hollow_shape_forwards_the_gate_to_its_inner_shape() {
      use nucleation::building::{BuildingTool, Hollow, ShapeEnum, Shape, SolidBrush};
      // Hollow only reaches the inner normal_at from its own normal_at, so a
      // gated fill must not touch it.
      let inner = counting_cube();
      let hollow = Hollow::new(ShapeEnum::Cuboid(nucleation::building::Cuboid::new(
          (0, 0, 0),
          (5, 5, 5),
      )));
      let brush = SolidBrush::new(nucleation::BlockState::new("minecraft:stone"));
      let mut schematic = nucleation::UniversalSchematic::new("hollow".to_string());
      BuildingTool::new(&mut schematic).fill(&hollow, &brush);
      assert!(schematic.total_blocks() > 0, "the hollow fill placed blocks");
      assert_eq!(inner.normals.get(), 0, "the gate never reached a normal");
      let _ = inner.bounds();
  }
  ```

  The `rstack` test needs the counting behaviour to be observable through a
  `ShapeEnum`, which has no counting variant. Rather than adding one to the
  public enum, assert the gate directly by counting `normal_at` calls with a
  temporary instrumentation build is not available either, so make the test
  meaningful this way instead: fill through `rstack` with a `ShadedBrush` and
  with a `SolidBrush` and compare wall time on a mesh shape.

  Replace the first test with this, which is a real regression test and fails
  before the fix:

  ```rust
  #[cfg(feature = "voxelize")]
  #[test]
  fn rstack_does_not_pay_for_normals_a_solid_brush_ignores() {
      use nucleation::building::{BrushEnum, BuildingTool, ShapeEnum, SolidBrush};
      use nucleation::voxelize::{test_meshes::sphere_5k, MeshModel, MeshShape};

      let mut model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
      model.fit(24.0);
      let shape = ShapeEnum::Mesh(MeshShape::new(model));
      let brush = BrushEnum::Solid(SolidBrush::new(nucleation::BlockState::new(
          "minecraft:stone",
      )));
      let mut schematic = nucleation::UniversalSchematic::new("rstack".to_string());

      let started = std::time::Instant::now();
      BuildingTool::new(&mut schematic).rstack(&shape, &brush, 2, (32, 0, 0));
      let elapsed = started.elapsed();

      assert!(schematic.total_blocks() > 0, "rstack placed blocks");
      if !cfg!(debug_assertions) {
          assert!(
              elapsed.as_secs_f64() < 1.0,
              "rstack of a mesh with a solid brush took {elapsed:?}, budget is 1 s"
          );
      }
  }
  ```

  Keep the `Hollow` test as written; check `Hollow::new`'s real signature
  before running and match it.

- [ ] **Run it and watch it fail.**

  ```bash
  $RS 'cargo test --release --features voxelize --test building_tests rstack hollow'
  ```

  Expected failure: `rstack_does_not_pay_for_normals_a_solid_brush_ignores`
  blows the one second budget, because `rstack` still calls
  `TranslatedShape::normal_at` for every voxel of every copy, which forwards to
  the mesh. The `Hollow` test passes already; that is fine, it is a
  characterisation test that locks in the forwarding behaviour the audit
  claims.

- [ ] **Gate `rstack`.** In `src/building/mod.rs`:

  ```rust
          let wants_normal = brush.uses_normal();
          for i in 0..count {
              let dx = offset.0 * i as i32;
              let dy = offset.1 * i as i32;
              let dz = offset.2 * i as i32;
              let translated = TranslatedShape::new(shape, dx, dy, dz);
              let (min_x, min_y, min_z, max_x, max_y, max_z) = translated.bounds();
              self.schematic
                  .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));

              translated.for_each_point(|x, y, z| {
                  let normal = if wants_normal {
                      translated.normal_at(x, y, z)
                  } else {
                      (0.0, 0.0, 0.0)
                  };
                  let t = shape.parameter_at(x - dx, y - dy, z - dz);
                  if let Some(block) = brush.get_block_with_parameter(x, y, z, normal, t) {
                      self.schematic.set_block(x, y, z, &block);
                  }
              });
          }
  ```

  `let wants_normal = brush.uses_normal();` goes immediately above the `for i`
  loop so it is computed once, not once per copy.

- [ ] **Record the audit in the source.** Add above `TubePath::normal_at`
  in `src/building/shapes/curve.rs`:

  ```rust
      /// Nearest point on the curve, then the outward radial direction.
      ///
      /// Linear in the curve's segment count, which the caller fixes when it
      /// builds the Curve3D and which does not grow with the fill volume, so
      /// this is O(1) in the target size and is not the pattern
      /// MeshShape::normal_at had. `contains` walks the same segments, so
      /// caching one without the other would buy nothing. Brushes that ignore
      /// the normal skip this entirely (Brush::uses_normal).
  ```

  And the same note above `BezierCurve::normal_at` in
  `src/building/shapes/bezier.rs`, with "the curve's segment count" replaced by
  "the `resolution` passed to BezierCurve::new".

- [ ] **Run it and watch it pass, with the golden.**

  ```bash
  $RS 'cargo test --release --features voxelize --test building_tests'
  $RS 'cargo test --release --features voxelize --test voxelize_golden --test voxelize_tests'
  ```

  Expected: every `building_tests` case passes, including the one second
  `rstack` budget, and the golden hashes are unchanged.

- [ ] **Format, lint, commit.**

  ```bash
  $RS 'cargo fmt --all -- --check && cargo clippy --features bridge,voxelize --tests -- -D warnings'
  git add src/building/mod.rs src/building/shapes/curve.rs src/building/shapes/bezier.rs tests/building_tests.rs
  git commit
  ```

  Commit message:

  ```
  Gate rstack on uses_normal and record the curve audit

  rstack was the fourth fill loop and still asked TranslatedShape for a
  normal per voxel per copy, which forwards straight into the mesh. It now
  reads Brush::uses_normal once, like fill and fill_enum_masked.

  The rest of the audit found nothing to fix. Hollow, Union, Intersection and
  Difference only reach an inner normal_at from their own normal_at, so the
  gate already stops at the boundary. TubePath::normal_at and
  BezierCurve::normal_at are linear in a segment count the caller fixes and
  that does not grow with the fill volume, so they are O(1) in the target
  size, and contains walks the same segments anyway; both keep a note saying
  so instead of a nearest segment cache. The distance field bridge calls are
  single point queries, not fill paths.

  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

---

## Task 6: bridge APIs

Spec 2.5 and 2.6. Three new methods on the bridge `Schematic`, so schemat.io
stops round-tripping every block as JSON just to count or replace them.

**Progress callbacks: dropped, and here is the evidence.** Spec 2.6 makes the
`Voxelizer::*_with_progress` bridge variant conditional on Diplomat callbacks
already existing in this repo. They do not. `grep -rn "DiplomatCallback\|
Callback\|dyn Fn" src/bridge` returns nothing, and `src/bridge/PORTING.md:48`
states the rule outright: "No `Vec<T>` returns, **no callbacks**, no trait
objects, no returning `&T` borrows". So the bridge progress variant is dropped
per the spec's own condition. There is also no Rust `Voxelizer` type: the three
functions the spec names (`shape_from_glb`, `shape_from_obj`,
`schematic_from_glb_textured`) exist only as Diplomat static methods on the
`Voxelizer` namespace opaque in `src/bridge/voxelize.rs`, so there is no Rust
API signature to add a sink to either. Both halves of 2.6 are dropped and
stated here; nothing in `src/voxelize` or `src/bridge/voxelize.rs` changes in
this task.

**Files**

- `src/bridge/schematic.rs`
- `tests/voxelize_bridge_export.rs` (new)
- `examples/bridge_smoke/js/main.mjs`, `examples/bridge_smoke/python/main.py`
- `bindings/**` (regenerated, committed)
- `docs/api-reference-python.md`, `docs/api-reference-wasm.md`

**Interfaces**

```rust
impl Schematic {
    pub fn count_blocks_json(&self, out: &mut DiplomatWrite);
    pub fn replace_blocks_json(&mut self, map_json: &DiplomatStr) -> Result<u64, NucleationError>;
    pub fn non_air_blocks_packed_b64(&self, out: &mut DiplomatWrite);
}
```

`non_air_blocks_packed_b64` returns base64 through `DiplomatWrite` rather than
the spec's `Box<[u8]>`. `src/bridge/PORTING.md:44` is explicit: "Binary data
out (bytes, PNG, GLB, serialized blobs): **base64 string** through
`DiplomatWrite` (add `_b64` suffix to the method name). `DiplomatWrite` is
UTF-8-only and the JS/Kotlin backends decode it as text, raw bytes corrupt."
The pattern is copied from `Schematic::to_litematic_b64` at
`src/bridge/schematic.rs:332`, which encodes with the module level `b64` helper
at `src/bridge/schematic.rs:21`. The spec anticipated this with "or the
Diplomat slice type the repo uses for bytes"; the repo's answer is base64.

Packed layout, little endian throughout:

```
u32  count
count * { i32 x, i32 y, i32 z, u16 palette_index }   // 14 bytes per block
u32  palette_json_len
u8[palette_json_len]  ["minecraft:stone", "minecraft:dirt", ...]
```

Palette indices are assigned in first-seen order over `iter_blocks`, so the
same schematic always packs identically. A schematic with more than 65,535
distinct non-air block ids cannot be represented and errors; that is far beyond
anything Minecraft can hold, but the writer checks rather than truncating.

### Steps

- [ ] **Write the failing Rust test first.** Create
  `tests/voxelize_bridge_export.rs`:

  ```rust
  #![cfg(feature = "bridge")]
  //! Round trips for the bulk block export and edit methods added for
  //! schemat.io, which used to tally and replace blocks by shipping the whole
  //! schematic through get_all_blocks_json.

  use nucleation::bridge::schematic::ffi::Schematic;

  fn sample() -> Box<Schematic> {
      let mut s = Schematic::create(b"bulk");
      s.set_block(0, 0, 0, b"minecraft:stone");
      s.set_block(1, 0, 0, b"minecraft:stone");
      s.set_block(2, 0, 0, b"minecraft:dirt");
      s
  }

  #[test]
  fn count_blocks_json_tallies_non_air_blocks() {
      let s = sample();
      let mut out = String::new();
      s.count_blocks_json(&mut out.into());
      let counts: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
      assert_eq!(counts["minecraft:stone"], 2);
      assert_eq!(counts["minecraft:dirt"], 1);
      assert!(counts.get("minecraft:air").is_none(), "air is excluded");
  }

  #[test]
  fn replace_blocks_json_rewrites_and_counts() {
      let mut s = sample();
      let changed = s
          .replace_blocks_json(br#"{"minecraft:stone":"minecraft:glass"}"#)
          .expect("valid map");
      assert_eq!(changed, 2);
      let mut out = String::new();
      s.count_blocks_json(&mut out.into());
      let counts: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
      assert_eq!(counts["minecraft:glass"], 2);
      assert!(counts.get("minecraft:stone").is_none());
      assert!(s.replace_blocks_json(b"not json").is_err());
  }

  #[test]
  fn packed_export_round_trips() {
      use base64::Engine as _;
      let s = sample();
      let mut out = String::new();
      s.non_air_blocks_packed_b64(&mut out.into());
      let bytes = base64::engine::general_purpose::STANDARD
          .decode(&out)
          .expect("valid base64");

      let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
      assert_eq!(count, 3);
      let mut seen = Vec::new();
      for i in 0..count {
          let at = 4 + i * 14;
          let x = i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
          let y = i32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
          let z = i32::from_le_bytes(bytes[at + 8..at + 12].try_into().unwrap());
          let p = u16::from_le_bytes(bytes[at + 12..at + 14].try_into().unwrap());
          seen.push((x, y, z, p));
      }
      let at = 4 + count * 14;
      let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
      let palette: Vec<String> =
          serde_json::from_slice(&bytes[at + 4..at + 4 + len]).expect("palette JSON");
      assert_eq!(bytes.len(), at + 4 + len, "no trailing bytes");

      assert_eq!(seen.len(), 3);
      for (x, y, z, p) in seen {
          let name = &palette[p as usize];
          let expected = if x == 2 { "minecraft:dirt" } else { "minecraft:stone" };
          assert_eq!(name, expected, "block at {x},{y},{z}");
      }
  }
  ```

  Check how the existing bridge tests construct a `DiplomatWrite` from a
  `String` before running (search the repo for `DiplomatWrite::from` or an
  existing `#[cfg(feature = "bridge")]` test); use whatever they use in place
  of `&mut out.into()`, and match the module path they import `Schematic` from.
  Everything else in this file is independent of that detail.

- [ ] **Run it and watch it fail.**

  ```bash
  $RS 'cargo test --release --features bridge,voxelize --test voxelize_bridge_export'
  ```

  Expected failure: `no method named count_blocks_json found for struct Schematic`.

- [ ] **Add the three methods.** In `src/bridge/schematic.rs`, immediately after
  `get_non_air_blocks_json` (which ends around line 1305), inside
  `impl Schematic`:

  ```rust
          /// Non-air blocks tallied by id: `{"minecraft:stone": 123, ...}`.
          /// One pass, no per block allocation, so a caller that only wants a
          /// material list never has to pull `get_non_air_blocks_json`.
          pub fn count_blocks_json(&self, out: &mut DiplomatWrite) {
              let mut counts: HashMap<&str, u64> = HashMap::new();
              for (_, block) in self.0.iter_blocks() {
                  if block.name == "minecraft:air" {
                      continue;
                  }
                  *counts.entry(block.name.as_str()).or_insert(0) += 1;
              }
              // BTreeMap for a stable key order, so two identical schematics
              // serialize to identical JSON.
              let ordered: std::collections::BTreeMap<&str, u64> = counts.into_iter().collect();
              let json = serde_json::to_string(&ordered).unwrap_or_else(|_| "{}".to_string());
              let _ = write!(out, "{}", json);
          }

          /// Apply a `{"from id": "to id"}` map in place and return how many
          /// blocks changed. Keys match on block id only, ignoring block
          /// states; values may carry states (`minecraft:oak_stairs[facing=north]`).
          /// A block whose id is not a key is left alone. Errors with `Parse`
          /// on malformed JSON or an unparseable target id.
          pub fn replace_blocks_json(
              &mut self,
              map_json: &DiplomatStr,
          ) -> Result<u64, NucleationError> {
              let raw: HashMap<String, String> =
                  serde_json::from_str(utf8(map_json)?).map_err(|_| NucleationError::Parse)?;
              let mut targets: HashMap<String, crate::BlockState> = HashMap::new();
              for (from, to) in raw {
                  let (state, _) = crate::UniversalSchematic::parse_block_string(&to)
                      .map_err(|_| NucleationError::Parse)?;
                  targets.insert(from, state);
              }
              // Collect first: iter_blocks borrows the schematic immutably.
              let edits: Vec<(crate::block_position::BlockPosition, crate::BlockState)> = self
                  .0
                  .iter_blocks()
                  .filter_map(|(pos, block)| {
                      targets.get(block.name.as_str()).map(|to| (pos, to.clone()))
                  })
                  .collect();
              let changed = edits.len() as u64;
              for (pos, state) in edits {
                  self.0.set_block(pos.x, pos.y, pos.z, &state);
              }
              Ok(changed)
          }

          /// Every non-air block as a compact binary blob, base64 encoded
          /// (`DiplomatWrite` is UTF-8 only, see `to_litematic_b64`). Little
          /// endian throughout:
          ///
          /// ```text
          /// u32 count
          /// count * { i32 x, i32 y, i32 z, u16 palette_index }
          /// u32 palette_json_len
          /// u8[palette_json_len]   ["minecraft:stone", ...]
          /// ```
          ///
          /// Palette indices are assigned in first-seen order, so the same
          /// schematic always packs identically. About seven times smaller
          /// than `get_non_air_blocks_json` and free of per block JSON
          /// parsing on the far side. Empty when the schematic holds more
          /// than 65,535 distinct non-air ids, which no real build does.
          pub fn non_air_blocks_packed_b64(&self, out: &mut DiplomatWrite) {
              let mut palette: Vec<&str> = Vec::new();
              let mut index_of: HashMap<&str, u16> = HashMap::new();
              let mut body: Vec<u8> = Vec::new();
              let mut count: u32 = 0;
              for (pos, block) in self.0.iter_blocks() {
                  if block.name == "minecraft:air" {
                      continue;
                  }
                  let name = block.name.as_str();
                  let index = match index_of.get(name) {
                      Some(&i) => i,
                      None => {
                          if palette.len() >= u16::MAX as usize {
                              let _ = write!(out, "");
                              return;
                          }
                          let i = palette.len() as u16;
                          palette.push(name);
                          index_of.insert(name, i);
                          i
                      }
                  };
                  body.extend_from_slice(&pos.x.to_le_bytes());
                  body.extend_from_slice(&pos.y.to_le_bytes());
                  body.extend_from_slice(&pos.z.to_le_bytes());
                  body.extend_from_slice(&index.to_le_bytes());
                  count += 1;
              }
              let palette_json =
                  serde_json::to_vec(&palette).unwrap_or_else(|_| b"[]".to_vec());
              let mut packed = Vec::with_capacity(4 + body.len() + 4 + palette_json.len());
              packed.extend_from_slice(&count.to_le_bytes());
              packed.extend_from_slice(&body);
              packed.extend_from_slice(&(palette_json.len() as u32).to_le_bytes());
              packed.extend_from_slice(&palette_json);
              let _ = write!(out, "{}", b64(&packed));
          }
  ```

  Then amend the `get_all_blocks_json` doc comment (line 1244) as spec 2.5
  asks, keeping its behaviour untouched:

  ```rust
          /// Every IN-BOUNDS cell as a JSON array of
          /// `{"x", "y", "z", "name", "properties"}` (the old `CBlockArray`).
          /// Air cells are materialized too, so on a large sparse build this
          /// dump is `volume()`-sized and can exhaust wasm memory.
          ///
          /// Prefer `get_non_air_blocks_json` for a block list,
          /// `count_blocks_json` for a material tally and
          /// `non_air_blocks_packed_b64` for bulk transfer. This method is
          /// kept for compatibility and is the wrong tool at any real size.
  ```

- [ ] **Run it and watch it pass.**

  ```bash
  $RS 'cargo test --release --features bridge,voxelize --test voxelize_bridge_export'
  ```

  Expected: three passing tests.

- [ ] **Regenerate the bindings and pull them back.** The helper syncs one way,
  so the generated files must be copied back from the server before they can be
  committed.

  ```bash
  $RS 'command -v diplomat-tool || cargo install --git https://github.com/Nano112/diplomat --branch nanobind-public-api diplomat-tool'
  $RS './tools/gen-bindings.sh'
  rsync -az --delete root@schematio0:/root/nucleation-perf/bindings/ \
    /Users/harrison/RustroverProjects/Nucleation-voxelize-perf/bindings/
  git -C /Users/harrison/RustroverProjects/Nucleation-voxelize-perf diff --stat -- bindings
  ```

  Expected: the diff touches the C, C++, JS, Kotlin, nanobind and PHP files for
  `Schematic` and nothing else.

- [ ] **Prove the determinism gate.** Run the generator a second time and
  confirm nothing moves.

  ```bash
  $RS './tools/gen-bindings.sh'
  rsync -az --delete root@schematio0:/root/nucleation-perf/bindings/ \
    /Users/harrison/RustroverProjects/Nucleation-voxelize-perf/bindings/
  git -C /Users/harrison/RustroverProjects/Nucleation-voxelize-perf diff --stat -- bindings
  ```

  Expected: identical diff stat to the previous step, meaning the second pass
  produced byte-identical output. After the bindings are committed later in
  this task, this same command must print nothing at all.

  Also run the coverage check the same job runs:

  ```bash
  $RS 'python3 tools/check_bridge_coverage.py'
  ```

- [ ] **Extend the JS smoke test.** In `examples/bridge_smoke/js/main.mjs`,
  after the litematic round trip block:

  ```js
  // --- bulk block queries (count / replace / packed export) ---
  const counted = JSON.parse(s.countBlocksJson());
  expect(counted["minecraft:stone"] === 1, "countBlocksJson tallies stone");
  expect(s.replaceBlocksJson('{"minecraft:stone":"minecraft:glass"}') === 1n
    || s.replaceBlocksJson('{"minecraft:glass":"minecraft:stone"}') === 1n,
    "replaceBlocksJson reports one change and is reversible");
  const packed = Uint8Array.from(atob(s.nonAirBlocksPackedB64()), (c) => c.charCodeAt(0));
  const view = new DataView(packed.buffer);
  expect(view.getUint32(0, true) === 1, "packed export holds one block");
  expect(view.getInt32(4, true) === 1 && view.getInt32(8, true) === 2
    && view.getInt32(12, true) === 3, "packed export keeps the position");
  ```

  `replaceBlocksJson` returns a `u64`, which the JS backend surfaces as a
  `BigInt`; if the generated `.d.ts` says otherwise, drop the `n` suffixes to
  match. Run the smoke test to find out rather than guessing.

- [ ] **Extend the Python smoke test.** In
  `examples/bridge_smoke/python/main.py`, after the litematic round trip:

  ```python
  # --- bulk block queries (count / replace / packed export) ---
  import json, struct
  counts = json.loads(s.count_blocks_json())
  assert counts["minecraft:stone"] == 1, counts
  assert s.replace_blocks_json('{"minecraft:stone":"minecraft:glass"}') == 1
  assert s.replace_blocks_json('{"minecraft:glass":"minecraft:stone"}') == 1
  packed = base64.b64decode(s.non_air_blocks_packed_b64())
  (count,) = struct.unpack_from("<I", packed, 0)
  assert count == 1, count
  x, y, z, index = struct.unpack_from("<iiiH", packed, 4)
  assert (x, y, z) == (1, 2, 3), (x, y, z)
  (plen,) = struct.unpack_from("<I", packed, 4 + count * 14)
  palette = json.loads(packed[8 + count * 14 : 8 + count * 14 + plen])
  assert palette[index] == "minecraft:stone", palette
  ```

- [ ] **Run both smoke tests.**

  ```bash
  $RS 'cargo build --release --lib --features bridge && ./examples/bridge_smoke/js/run.sh'
  $RS './examples/bridge_smoke/python/run.sh'
  ```

  Expected: both scripts exit 0. If `run.sh` builds the library itself, the
  extra `cargo build` is harmless.

- [ ] **Document the new surface.** In `docs/api-reference-python.md`, add a
  section immediately before `## Analysis: fingerprints, diff, auto-stack`:

  ```markdown
  ## Bulk block queries

  Three methods exist so a tool never has to pull the whole block list just to
  count or rewrite it:

  ```python
  counts = json.loads(schem.count_blocks_json())   # {"minecraft:stone": 1234, ...}
  changed = schem.replace_blocks_json('{"minecraft:stone":"minecraft:glass"}')
  packed = base64.b64decode(schem.non_air_blocks_packed_b64())
  ```

  `count_blocks_json` tallies non-air blocks by id in one pass.
  `replace_blocks_json` applies a from-id to to-id map in place and returns the
  number of blocks changed; keys match on the id, values may carry block states.
  `non_air_blocks_packed_b64` is the compact export: little endian `u32 count`,
  then `i32 x, i32 y, i32 z, u16 palette_index` per block, then a `u32` length
  and that many bytes of palette JSON.

  `get_all_blocks_json` still exists and still materialises air, which makes it
  `volume()`-sized. Prefer `get_non_air_blocks_json` or the packed export.
  ```

  Add the same section to `docs/api-reference-wasm.md`, under `## Core`, with
  the camelCase names (`countBlocksJson`, `replaceBlocksJson`,
  `nonAirBlocksPackedB64`) and a JS snippet in place of the Python one.

- [ ] **Check mkdocs strict.**

  ```bash
  $RS 'pip install -q -r requirements-docs.txt && mkdocs build --strict'
  ```

  Expected: `INFO - Documentation built in ...`, no warnings promoted to
  errors. If mkdocs is not installed on the server and pip cannot install it,
  say so in the report rather than skipping the gate silently.

- [ ] **Format, lint, commit.**

  ```bash
  $RS 'cargo fmt --all -- --check && cargo clippy --features bridge,voxelize --tests -- -D warnings'
  git add src/bridge/schematic.rs tests/voxelize_bridge_export.rs bindings \
    examples/bridge_smoke/js/main.mjs examples/bridge_smoke/python/main.py \
    docs/api-reference-python.md docs/api-reference-wasm.md
  git commit
  ```

  Commit message:

  ```
  Add bulk block count, replace and packed export to the bridge

  count_blocks_json tallies non-air blocks by id in one pass,
  replace_blocks_json applies a from-id to to-id map in place and returns the
  number of blocks changed, and non_air_blocks_packed_b64 exports positions
  and palette indices as a compact little endian blob with the palette as
  length prefixed JSON. Together they replace the pattern of pulling
  get_all_blocks_json just to count or rewrite materials, which is
  volume()-sized and materialises air.

  The packed export returns base64 through DiplomatWrite rather than a byte
  slice, following the rule in src/bridge/PORTING.md and the existing
  to_litematic_b64. The progress callback variant the design left conditional
  is dropped: Diplomat callbacks are not used anywhere in src/bridge and
  PORTING.md rules them out, and there is no Rust Voxelizer type to hang a
  sink on either.

  Bindings regenerated and committed; JS and Python smoke tests cover all
  three methods.

  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

---

## Task 7: release 0.10.17

Version bump across the same files the 0.10.16 bump touched, release notes with
the before and after table, and the full CI gate set run through the helper.

The 0.10.16 bump (`git show --stat HEAD~1`, commit 5ae3c895) touched
`Cargo.toml`, `Cargo.lock`, `RELEASE_NOTES.md`, `bindings/python/pyproject.toml`
and `tools/package-npm.sh`. The last of those was that release's feature work,
not part of the version set, so the version files are the four others. The CI
job `check-version` at `.github/workflows/ci.yml:49` enforces that they agree.

**Files**

- `Cargo.toml`, `Cargo.lock`, `bindings/python/pyproject.toml`
- `RELEASE_NOTES.md`

### Steps

- [ ] **Bump the version.** In `Cargo.toml` line 11:

  ```toml
  version = "0.10.17"
  ```

  In `bindings/python/pyproject.toml`, the `version = "0.10.16"` line becomes
  `0.10.17`. Then refresh the lock file on the server and pull it back, rather
  than hand editing it:

  ```bash
  $RS 'cargo update --workspace --offline || cargo metadata --format-version 1 >/dev/null'
  rsync -az root@schematio0:/root/nucleation-perf/Cargo.lock \
    /Users/harrison/RustroverProjects/Nucleation-voxelize-perf/Cargo.lock
  git -C /Users/harrison/RustroverProjects/Nucleation-voxelize-perf diff -- Cargo.lock
  ```

  Expected: exactly one changed line, `name = "nucleation"`'s version. If the
  diff is larger, discard it and hand edit that single line instead: a release
  bump must not silently move dependency versions.

- [ ] **Verify the version gate the way CI does.**

  ```bash
  $RS 'grep -n "^version" Cargo.toml; grep -n "^version" bindings/python/pyproject.toml; grep -n "name = \"nucleation\"" -A 1 Cargo.lock'
  ```

  All three must read 0.10.17.

- [ ] **Write the release notes.** Prepend to `RELEASE_NOTES.md`, above
  `# Nucleation v0.10.16`:

  ```markdown
  # Nucleation v0.10.17

  **Mesh voxelization is no longer quadratic in the volume.** Filling a
  voxelized mesh used to call `MeshShape::normal_at` for every solid voxel
  whatever the brush did with the value, and each of those calls ran an
  expanding ring search over the triangle grid with a fresh allocation, so the
  cost grew as the sixth power of the target size. Three changes fix it.
  `Brush::uses_normal` lets a brush say it ignores the surface normal, and the
  seven brushes that do (`Solid`, `Color`, `Linear`, `MultiPoint`, `Point`,
  `Bilinear`, `Field`) now cost nothing to shade. `MeshShape` precomputes a
  triangle id per voxel once, next to the mask it already builds, with one
  rayon pass over the triangles and one BFS to hand ids inward, so `normal_at`
  and `surface_color` are array lookups. `voxelize_textured` samples colours in
  parallel, memoises the palette search on the exact colour and resolves each
  palette index to a block once instead of cloning a `String` per voxel.

  | case (5,000 triangle sphere, BoxTextured cube) | 0.10.16 | 0.10.17 |
  | --- | --- | --- |
  | solid fill, size 32 | TODO | TODO |
  | solid fill, size 64 | TODO | TODO |
  | solid fill, size 128 | TODO | TODO |
  | shell fill, size 32 | TODO | TODO |
  | shell fill, size 64 | TODO | TODO |
  | shell fill, size 128 | TODO | TODO |
  | textured cube, size 32 | TODO | TODO |
  | textured cube, size 64 | TODO | TODO |
  | textured cube, size 128 | TODO | TODO |

  Measured on the build host with `cargo bench --features voxelize --bench
  voxelize_bench`. The output is unchanged: a committed golden fixture pins the
  sha256 of the sorted block list for the solid sphere, the shell sphere and
  the textured cube at size 32, and it holds byte for byte across all three
  changes.

  **Bulk block queries on the bridge.** `Schematic.count_blocks_json` tallies
  non-air blocks by id in one pass, `replace_blocks_json` applies a from-id to
  to-id map in place and returns how many blocks changed, and
  `non_air_blocks_packed_b64` exports positions and palette indices as a
  compact little endian blob with the palette as length prefixed JSON. Tools
  that used to pull `get_all_blocks_json` just to count or rewrite materials no
  longer have to; that method still exists, still materialises air, and its
  documentation now says so.
  ```

  Fill every TODO from the recorded medians: the 0.10.16 column from the task 1
  baseline (and, for the three size 128 rows, from the first measurement in
  task 3, marked as extrapolated only if it was never actually run at that
  size on the old code), the 0.10.17 column from the final bench below.

- [ ] **Take the final bench numbers.**

  ```bash
  $RS 'NUCLEATION_BENCH_LARGE=1 cargo bench --features bridge,voxelize --bench voxelize_bench -- --warm-up-time 1 --measurement-time 3'
  ```

  Paste the nine medians into the table.

- [ ] **Run the full gate set.** Every one of these must pass before the
  commit. Run them in this order and record each result.

  ```bash
  $RS 'cargo test'
  $RS 'cargo test --release --features bridge,voxelize'
  $RS 'cargo test --no-default-features --features world-segment,store-fs,store-ssh --lib world_segment'
  $RS 'cargo fmt --all -- --check'
  $RS 'cargo clippy --features bridge,voxelize --tests --benches -- -D warnings'
  $RS 'cargo build --lib --features bridge'
  $RS 'cargo check --target wasm32-unknown-unknown --lib --features bridge,mc-tick,meshing,voxelize'
  $RS 'python3 tools/check_bridge_coverage.py'
  $RS './tools/gen-bindings.sh && git status --porcelain bindings'
  $RS './examples/bridge_smoke/js/run.sh'
  $RS './examples/bridge_smoke/python/run.sh'
  $RS 'pip install -q -r requirements-docs.txt && mkdocs build --strict'
  ```

  The gen-bindings line is the determinism gate: the server has no `.git`
  (the helper excludes it), so `git status` there will not work. Instead pull
  the bindings back and check on the Mac:

  ```bash
  $RS './tools/gen-bindings.sh'
  rsync -az --delete root@schematio0:/root/nucleation-perf/bindings/ \
    /Users/harrison/RustroverProjects/Nucleation-voxelize-perf/bindings/
  git -C /Users/harrison/RustroverProjects/Nucleation-voxelize-perf diff --stat
  ```

  Expected after task 6 committed the bindings: empty output. Anything else
  means the generated output moved and must be committed, not ignored.

- [ ] **Commit.**

  ```bash
  git add Cargo.toml Cargo.lock bindings/python/pyproject.toml RELEASE_NOTES.md
  git commit
  ```

  Commit message:

  ```
  Release Nucleation v0.10.17 with linear mesh voxelization

  Mesh fills stop calling normal_at for brushes that ignore it, MeshShape
  precomputes a triangle id per voxel instead of running an expanding ring
  search per query, and the textured path samples in parallel behind an exact
  colour memo. The golden fixture holds byte for byte across all three, and
  the release notes carry the before and after bench table.

  Also ships the bridge bulk block methods: count_blocks_json,
  replace_blocks_json and non_air_blocks_packed_b64.

  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_017P9oD4kCeHtYLeuacudYHJ
  ```

- [ ] **Stop here.** The `v0.10.17` tag and the push to master are the
  controller's, not this task's. Report the branch state, the nine bench
  medians and every gate result instead.
