# Nucleation v0.2.7

Fix the Bedrock `.mcstructure` exporter so redstone blocks survive the
round-trip, add a first-class JVM binding (`com.github.schemat:nucleation`)
with Java / Kotlin / mod consumers in mind, and expand the pre-push gate
to actually exercise every binding.

## Fixed — Bedrock `.mcstructure` accuracy

Three concrete bugs that produced the community report on 2026-05-12
("hoppers all face down, sticky pistons and repeaters from hologram only
didn't build, random block replacements"):

- **Block properties were stripped before translation.** `to_mcstructure`
  passed only the bare block name (e.g. `minecraft:hopper`) to blockpedia,
  losing the `facing`, `delay`, `extended`, etc. properties. Every block
  came out with Bedrock defaults — hence the headline hopper-down symptom.
  Fixed by rebuilding the full `name[k=v,...]` form before parse.
- **Block-entity NBT wasn't translated Java → Bedrock.** The exporter
  wrote Java-shaped tile-entity payloads (`id: "minecraft:hopper"`, Java
  item layout) straight into Bedrock structures, which silently rejected
  them — leaving hoppers without items, signs without text, etc. Fixed by
  wiring blockpedia's new `BlockEntityTranslator::translate_java_to_bedrock`
  into the export pipeline.
- **`block_position` was a private module** despite being referenced in
  the public signatures of `set_block_entity` / `get_block_entity`.
  Now `pub mod block_position` so external test code can construct
  `BlockPosition` directly.

New regression coverage:
- `tests/mcstructure_redstone_tests.rs` — 16 round-trip tests covering
  hopper (5 facings), repeater (powered + unpowered, full state),
  sticky piston, observer, comparator, lever, wall_torch, chest, stairs.
- `tests/mcstructure_be_tests.rs` — 5 block-entity preservation tests:
  hopper items, chest with 3 items, dispenser, furnace burn state,
  unknown-BE pass-through.
- `tests/mcstructure_fixture_fuzz_tests.rs` — 2 stability tests against
  every `.mcstructure` fixture: block-count invariance and per-cell
  block-name preservation modulo air-trimming.

Requires **blockpedia 0.1.9** which lands the symmetric
`translate_java_to_bedrock` and 9 new redstone state-translation tests.

## Added — `nucleation-jvm`: a Java / Kotlin / mod-ready JAR

A new top-level crate `nucleation-jvm` plus a Gradle project that produces
`nucleation.jar` for consumption from mods or any JVM application.
Purely additive — existing FFI / WASM / Python binding code is untouched.

Highlights:
- Standalone cdylib crate (no Cargo-workspace edit needed)
- Idiomatic Java surface: `Schematic` (`AutoCloseable` + `Iterable<Block>`),
  `BlockState` (immutable, `withProperty` returns new), `Block` /
  `Dimensions` records, `Shape` (12 primitives + 4 composite ops),
  `Brush`, `BuildingTool`, `SchematicBuilder`, `MchprsWorld`
  (simulation-feature-gated), and the full meshing pipeline:
  `ResourcePack`, `MeshConfig`, `MeshResult.glbData()`, `MultiMeshResult`.
- All wrappers `AutoCloseable` + `Cleaner`-backstopped — no leaks even
  without try-with-resources.
- `NativeLoader` extracts the platform cdylib from the JAR at runtime
  with descriptive errors that list bundled platforms on mismatch.
- 34 JUnit 5 tests, all green locally.
- CI builds cdylibs for 5 platforms (Linux x64/arm64, macOS x64/arm64,
  Windows x64), assembles a fat JAR, and attaches it to the release.
- `build-cross.sh` + Gradle `crossJar` task for local cross-builds via
  `cross` / Docker.

## Added — pre-push gate covers every binding

Pre-push now runs **24 checks across 5 lanes** (was 13 across 3):

| Lane | Checks |
|---|---|
| Native | Rust `cargo check` × 6 feature combos, `cargo test` × 2, insign IO test, `maturin build` |
| WASM | `cargo check`, `build-wasm.sh`, 4 Node test files |
| Python | 3 Python test files (against `.venv/bin/python`) |
| JVM | `cargo build nucleation-jvm`, Gradle test, JVM ↔ Python parity |
| Quick | version consistency, WASM/Python/FFI parity |

Bug-revealing tests the new lanes uncovered (now fixed):
- `node_simple_circuit_test.js`: was loading WASM synchronously and
  using camelCase method names that no longer exist
- `python_test.py`: was using `.with_metadata()` chaining that's now
  `.set_metadata()`
- `python_new_api_test.py`, `python_simple_circuit_test.py`: were
  testing against a stale system-Python `nucleation 0.1.184` install

## Tooling

- New `tools/check_jvm_parity.rs` standalone parity tool ensures the
  JVM surface tracks `nucleation-py`: 69 methods matched, 291 documented
  exclusions, 0 unaccounted gaps.
- Gradle wrapper bumped to 9.0.0 for JDK 25 support.
- `mcstructure-accuracy-todo.md` — design-of-record for the redstone fix.
- `java-library-todo.md` — design-of-record for the JVM binding.
- `.gitignore`: cover JVM target/build dirs and pytest temp directories
  (drops from ~3.4k unversioned files to a clean tree).

## Dependency bump

`blockpedia` from 0.1.8 → 0.1.9 (Java ↔ Bedrock block-entity translator).
