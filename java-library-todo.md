# Nucleation JVM bindings — status

**Status: shipped (v1).** All checkboxes below are done. This document is
kept as the design-of-record and a guide to the moving parts.

A self-contained JNI library that turns Nucleation into a first-class
JVM citizen alongside `nucleation-py` and the WASM bindings.

- Group / artifact: `com.github.schemat:nucleation`
- Minimum JDK: 21
- License: MIT (inherited)
- Distribution: GitHub Releases (fat JAR, all platforms)

## Decisions locked in (all delivered)

| Decision | Choice | Status |
|---|---|---|
| Native binding | **JNI** | ✅ |
| Consumer | Standalone JVM library, mod-friendly | ✅ |
| Surface scope | Python-parity (Schematic/BlockState/Shape/Brush/BuildingTool/Builder/MchprsWorld/Meshing) | ✅ |
| API style | Idiomatic Java, `AutoCloseable` everywhere | ✅ |
| Minimum JDK | Java 21 | ✅ |
| Repo layout | Standalone `nucleation-jvm/` crate (no root Cargo.toml edit) | ✅ |
| Distribution | Fat JAR, all platforms bundled | ✅ |
| Publish target (v1) | GitHub Releases | ✅ |
| Package | `com.github.schemat.nucleation` | ✅ |
| Parity check | `tools/check_jvm_parity.rs` standalone tool | ✅ |
| Pre-push integration | `tools/prepush.py` JVM lane | ✅ |
| CI | `.github/workflows/jvm.yml` | ✅ |

The only files touched outside `nucleation-jvm/` are additive:
- `tools/check_jvm_parity.rs` (new)
- `tools/jvm_parity_exclusions.txt` (new)
- `tools/prepush.py` (additive: new `build_jvm_lane` + `_run_jvm_lane`)
- `.github/workflows/jvm.yml` (new; the existing `ci.yml` is untouched)
- `examples/java/` (new)

The root `Cargo.toml` is **not** modified — `nucleation-jvm/` is a standalone
crate that depends on `nucleation = { path = ".." }`. The MCHPRS
`[patch.crates-io]` is replicated locally in `nucleation-jvm/Cargo.toml`.

---

## Where everything lives

```
nucleation-jvm/
├── Cargo.toml                         # standalone crate, cdylib
├── README.md                          # consumer-facing docs
├── src/
│   ├── lib.rs                         # JNI_OnLoad + RegisterNatives
│   ├── handles.rs                     # jlong ↔ Box<T>, panic-safe
│   ├── errors.rs                      # panic capture, throw exception
│   ├── conv.rs                        # JString/byte[]/Map conversions
│   └── exports/
│       ├── mod.rs
│       ├── schematic.rs               # ~30 native methods
│       ├── blockstate.rs              # 6 native methods
│       ├── shape.rs                   # 13 shapes + brushes
│       ├── buildingtool.rs            # fill, rstack
│       ├── builder.rs                 # 13 builder fluent ops
│       ├── nucleation.rs              # free fns (debug, version)
│       └── simulation.rs              # gated #[cfg(feature="simulation")]
└── jvm/
    ├── build.gradle.kts               # JDK 21, JUnit 5, fat JAR assembly
    ├── settings.gradle.kts
    ├── gradle.properties
    ├── gradle/wrapper/                # gradle-wrapper.jar + properties
    ├── gradlew, gradlew.bat
    └── src/
        ├── main/java/com/github/schemat/nucleation/
        │   ├── NativeLoader.java        # platform-aware cdylib extraction
        │   ├── NucleationNative.java    # package-private native decls
        │   ├── Nucleation.java          # entry point (version, hasSimulation)
        │   ├── Schematic.java           # AutoCloseable, Iterable<Block>
        │   ├── BlockState.java          # AutoCloseable, immutable semantics
        │   ├── Block.java               # record
        │   ├── Dimensions.java          # record
        │   ├── Shape.java               # AutoCloseable; 12 factories
        │   ├── Brush.java               # AutoCloseable; solid, color
        │   ├── BuildingTool.java        # static fill/rstack
        │   ├── SchematicBuilder.java    # fluent ASCII builder
        │   ├── MchprsWorld.java         # AutoCloseable; tick, getSchematic
        │   └── exceptions/
        │       ├── NucleationException.java
        │       ├── SchematicParseException.java
        │       ├── InvalidBlockStateException.java
        │       └── UnsupportedFeatureException.java
        └── test/java/com/github/schemat/nucleation/
            ├── NucleationTest.java
            ├── SchematicTest.java        # 16 tests covering load/save/iter
            ├── BlockStateTest.java       # 5 tests
            ├── ShapeTest.java            # 4 tests
            └── SchematicBuilderTest.java # 2 tests

tools/
├── check_jvm_parity.rs                # standalone parity tool
└── jvm_parity_exclusions.txt          # documented gaps (e.g. meshing)

.github/workflows/jvm.yml              # 5-platform build + assemble + release

examples/java/                         # consumer reference project
```

---

## Test results (local, macOS arm64)

All 29 JUnit tests pass:

- `BlockStateTest`: 5 tests (create/read, immutability, multi-properties, toString, close)
- `NucleationTest`: 2 tests (version, hasSimulation)
- `SchematicBuilderTest`: 2 tests (ascii build, validate)
- `SchematicTest`: 16 tests (create/close, set/get block in 3 variants, dimensions, litematic/schem/snapshot round-trips, iteration, stream, copy independence, countBlockTypes, fillCuboid/Sphere, supported formats, post-close rejects, invalid bytes)
- `ShapeTest`: 4 tests (bounds, contains, BuildingTool fill, union composite)
- `MeshingTest`: 5 tests (feature compiled-in, default config, fluent setters, biome nullable, close idempotent)

```bash
./gradlew test
# BUILD SUCCESSFUL — 34 tests passed, 0 failed
```

Parity check is clean:

```
=== nucleation-jvm ↔ nucleation-py parity ===
Python methods checked: 360
  matched on JVM      : 69
  excluded            : 291
  missing on JVM      : 0
✅ All Python methods have JVM counterparts.
```

The 322 exclusions are documented in `tools/jvm_parity_exclusions.txt` —
mostly Python dunders (`__repr__` etc.), feature-gated meshing/rendering
APIs, and internal palette helpers.

---

## CI flow (`.github/workflows/jvm.yml`)

1. **`build-cdylib`** (matrix, 5 jobs): builds `libnucleation_jvm.{so,dylib,dll}`
   per platform.
   - `ubuntu-latest` × `x86_64-unknown-linux-gnu` → `linux-x64`
   - `ubuntu-latest` × `aarch64-unknown-linux-gnu` (cross) → `linux-arm64`
   - `macos-13` × `x86_64-apple-darwin` → `macos-x64`
   - `macos-14` × `aarch64-apple-darwin` → `macos-arm64`
   - `windows-latest` × `x86_64-pc-windows-msvc` → `windows-x64`
2. **`test-host`** (`ubuntu-latest`): builds host cdylib + runs `./gradlew test`.
3. **`parity-check`** (`ubuntu-latest`): runs `check_jvm_parity`.
4. **`assemble-jar`**: downloads all 5 cdylibs into the Gradle resources
   tree, runs `./gradlew jar javadocJar sourcesJar`, uploads the fat JAR.
5. **`release`** (only on `v*` tag push): attaches the JAR to the GitHub Release.

---

## Pre-push (`tools/prepush.py`)

A new `JVM` lane runs in parallel with the existing `Native`, `WASM`, and
`Quick` lanes:

1. `cargo build --release --manifest-path nucleation-jvm/Cargo.toml`
2. `nucleation-jvm/jvm/gradlew test`
3. JVM ↔ Python parity (`tools/check_jvm_parity.rs`)

The lane skips itself cleanly if `nucleation-jvm/` is absent.

---

## What's intentionally deferred (v2+)

These are listed as exclusions in `tools/jvm_parity_exclusions.txt`. They
don't block parity; they're known gaps to fill in future releases.

- **World I/O**: `from_world_directory`, `to_world`, `save_world`,
  `to_world_zip` (multi-region anvil import/export).
- **Meshing**: core GLB export pipeline IS shipped (`ResourcePack`,
  `MeshConfig`, `MeshResult.glbData()`, `MultiMeshResult`,
  `Schematic.mesh()` / `meshByRegion()`). The meshing feature is enabled
  by default in the cdylib so `Nucleation.hasMeshing()` returns `true` in
  standard builds. Still deferred: `ChunkMeshResult`, `RawMeshExport`,
  `TextureAtlas` introspection, `ItemModelConfig`/`Result`, `RenderConfig`,
  NUCM / USDZ alternate-format exports, chunked mesh iterators
  (`meshByChunk`, `meshChunks`, `meshChunksWithAtlas`, `toUsdz`,
  `toRawMesh`, `buildGlobalAtlas`).
- **Typed simulation / circuit-builder**: `PyCircuitBuilder`,
  `PyTypedCircuitExecutor`, `PyIoLayoutBuilder`, `PyIoLayout`, `PyValue`,
  `PyExecutionMode`, `PyIoType`, `PyLayoutFunction`, `PyOutputCondition`,
  `PySortStrategy`. Only the basic `MchprsWorld` is in v1.
- **DefinitionRegion**: every method (used by typed I/O above).
- **Brush variants** beyond `solid` and `color`: `linear_gradient`,
  `bilinear_gradient`, `point_gradient`, `curve_gradient`, `shaded`.
- **Internal helpers**: palette indexing, cache info, prepared blocks.
- **Bedrock-only convenience**: `set_blocks_flat`, multi-region copy.

These can be added incrementally without breaking the v1 surface — each is
purely additive in both Rust JNI exports and the Java class hierarchy.

---

## Re-licensing note

The library inherits **MIT** from upstream. The repository owner
(Schem-at) is the rights-holder. Mods that bundle the JAR may do so under
the permissive MIT terms (retain the copyright/permission notice).
