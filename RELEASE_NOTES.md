# Nucleation v0.2.17

JVM: packed bulk block export. Adds a palette + stride-4 int array
encoding for pulling large regions out of the JVM binding in one call,
instead of one JNI round-trip per block.

# Nucleation v0.2.16

SDF (signed distance field) shape and terrain generation, available
across every binding: build a schematic by sampling an SDF JSON tree
against material rules (`from_sdf` / `from_sdf_bounded`, with a
standalone `sdf_eval` for point queries). JVM also picked up
`setBlockEntity` with SNBT write support, and `from_insign` now strips
sign blocks after compiling their annotations instead of leaving them
in the output.

# Nucleation v0.2.15

Small follow-up to v0.2.14: `schematic-mesher` resolves from its
GitHub source again (a crates.io publish had briefly broken that),
plus full binding parity for the datafixer and entity/block-entity
SNBT API introduced in v0.2.14, and a `MeshOutput::from` fix so the
local (non-service) mesher path constructs by value correctly.

# Nucleation v0.2.14

The big one in this range. Relicensed the project from AGPL-3.0-only
to MIT across every file. Landed the streaming world API — constant-
memory parsing, generation, and diffing of world saves without holding
the whole world in memory, plus `.mca`/world-folder docs to match.
Added redstone graph export with integration tests, meshing
performance work (palette-indexed block sources), and merged in a
contributor's fork carrying dataconverter and litematic/entity
improvements.

Also where the JVM binding caught up hard on this window: full
`MchprsWorld` simulation parity with Python, the item-model generation
API, the redstone graph + typed circuit executor API, and a fix for
released fat JARs that had been shipping without `mchprs` compiled in
(simulation now on by default).

# Nucleation v0.2.13

Exposes `footprint()` — a translation-invariant shape fingerprint used
by the fingerprint/classification engine — as a vector across all
bindings, rather than only being reachable through the Rust API.

# Nucleation v0.2.12

New fingerprint & signature engine: canonical `Fingerprint`/`Signature`
types, symmetry-group-aware rigid transforms, an FFT-based
translation-invariant `Footprint`, a rule-based classifier with
shipped rulesets (structural, redstone computational/survival) loaded
from RON, and synthetic-fixture benchmarks. Exposed to WASM as `Diff`
and `Fingerprint` bindings. Also added synchronous Redis and S3 `Store`
backends alongside the existing filesystem one.

# Nucleation v0.2.11

Render background color and orthographic/isometric projection support
for `RenderConfig`, implemented in core and exposed across
Python/WASM/FFI (Python via a `Projection` enum, WASM/FFI via
`orthographic`/`setOrthographic`-style booleans — a documented,
intentional naming divergence, see `api_parity_exclusions.txt`). Also
fixes an `i64` overflow in `Region::coords_to_index` for large
regions.

# Nucleation v0.2.10

Build script fix. v0.2.9's `assemble-jvm-jar` job failed at the
`processResources` step under Gradle 9:

    Entry native/linux-arm64/libnucleation_jvm.so is a duplicate but no
    duplicate handling strategy has been set.

Two compounding sources of the duplicate:

1. `collectNatives` was copying `src/main/resources/native/**/*.{so,
   dylib,dll}` into `build/native-staging/`. Those files were already
   on the default resources classpath, so they got bundled twice.
2. `processResources` had no `duplicatesStrategy` set, which under
   Gradle 9 (strict by default) fails the build instead of warning.

Fixed in `nucleation-jvm/jvm/build.gradle.kts`:
- Dropped the redundant `preStaged` from() in `collectNatives` — pre-
  staged cdylibs reach the JAR through the default resources path
  alone, no need to re-copy them.
- Added `duplicatesStrategy = DuplicatesStrategy.EXCLUDE` to
  `processResources` as a safety net in case the host cargo target and
  a pre-staged cdylib happen to overlap on the same platform.

No source / API changes since v0.2.7.

v0.2.8 retired (deprecated macos-13 runner).
v0.2.9 retired (Gradle 9 duplicate-resources failure).

See v0.2.7 release notes for the feature work.
