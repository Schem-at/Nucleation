# Nucleation v0.3.1

**Fixes broken v0.3.0 native release artifacts.** The v0.3.0 per-platform
libraries were built with the core `bridge` feature only, so every
meshing/simulation/rendering export was missing — PHP's eager `FFI::cdef`
could not even bind the release zip's own bindings. All native artifacts
(platform zips, JVM jar natives) now ship the full `bridge-full` surface,
matching the wheels, and CI now installs and exercises every wheel and the
assembled jar (including a simulation-symbol check) before anything ships.

Also in this release:

- **First-class palettes** in every language: `Palette` (solid / structural /
  decorative / concrete / wool / terracotta / grayscale presets, custom
  palettes from a JSON block-id list, closest-block lookup),
  `PaletteBuilder` (blockpedia filter flags + keyword include/exclude), and
  `Brush.setPalette(...)` on all color/gradient brushes — bindings are no
  longer locked to the built-in all-blocks palette. Default palettes now
  exclude technical blocks (portals, fluids, fire, piston internals).
- **JVM jar is multi-platform**: natives for linux x64/arm64, macOS
  x64/arm64, and Windows x64 are bundled in JNA layout (previously linux
  x64 only).
- **crates.io publishing works again**: the published crate ships without
  the git-only features (`simulation` — MCHPRS; `meshing`/`rendering` —
  schematic-mesher); use the git dependency for those.

---

# Nucleation v0.3.0

**Breaking: every language binding is now generated from a single source of truth.**

The four hand-written binding layers (C FFI via `#[no_mangle]` externs, WASM via
wasm-bindgen, Python via pyo3, JVM via hand-written JNI) and the experimental
ext-php-rs extension are gone, replaced by Diplomat-generated bindings for
C, C++, JS/WASM, Kotlin (JNA), Python (nanobind), and PHP (ext-ffi) — all generated
from `src/bridge/` by `tools/gen-bindings.sh` into `bindings/`, and regenerated +
diffed in CI so they can never go stale. The regex parity linters are deleted;
coverage vs the old 544-function C surface is enforced by
`tools/check_bridge_coverage.py` against a frozen baseline.

API changes to be aware of:
- One unified error model: every fallible call returns/raises `NucleationError`
  (12 variants). The thread-local `schematic_last_error`, per-function int/null
  sentinels, and error-string returns are gone.
- Constructors are `create`/`from_*`; accessors drop `get_`/`set_` prefixes
  (per-language casing applies, e.g. `getBlockName` in JS/PHP).
- Domain methods moved off the `Schematic` god-object onto their own types
  (`Diff`, `Fingerprint`, `Autostack`, `StoreIo`, `Renderer`, meshing types,
  `SchematicRegions`).
- Binary payloads (litematic/schem/GLB/PNG/…) cross the boundary base64-encoded
  (`*_b64` methods); arrays/lists cross as JSON strings.
- The mesh progress callback is replaced by a polling `MeshJob`
  (start → `poll_progress` → `take_result`).
- Memory management is generated: no more `free_*` functions anywhere.

See `src/bridge/PORTING.md` for the binding rules and
`tools/bridge_coverage/exclusions.txt` for the audited old→new name map.

**Complete API documentation across all bindings.** Every public function on
the bridge surface (509 total) now carries a doc comment, propagated by the
generator into all seven languages; the 140 previously undocumented functions
(meshing config, simulation value/layout/ordering types, transforms,
definition regions, …) were documented from their implementations, including
defaults, units, and coordinate/rotation conventions.

**Editing-operation performance.**
- `set_block_from_string` now caches parsed block strings (properties + NBT)
  per schematic, and placed block entities Arc-share the cached NBT
  (copy-on-write). Repeatedly placing the same NBT-bearing block (e.g. filled
  chests) is ~41× faster (0.30 → 12.4 M blocks/s); property-bearing blocks
  (e.g. repeaters) are ~3.6× faster (5.7 → 20.6 M blocks/s).
- `copy_region` from a single-region source (the common case) now translates
  palette indices through a precomputed source→target map instead of hashing
  a `BlockState` per block: ~3.8× faster (64 → 242 M blocks/s), same
  resulting content (covered by a fast-vs-slow-path equivalence test).

---

# Nucleation v0.2.18

Maintenance release, no user-facing API changes. The FFI layer
(`src/ffi.rs`, 10k+ lines) is now split into per-domain modules under
`src/ffi/`, matching the existing WASM/Python binding structure —
verified byte-identical exported C symbols across every feature
combination before and after. Format parsing (`src/formats/`,
`src/dataconverter/`) converged onto a proper `thiserror`-based error
type instead of ad-hoc `Box<dyn Error>`/`String` errors; the public
`UniversalSchematic::to_schematic`/`from_schematic` signatures are
unchanged. Also merged in a diff palette-swap-dominance feature that
had been sitting on an unmerged branch, cleared out several stale
branches, ran a full `clippy --fix` pass, and fixed a comparator
custom-IO test that had the wrong block orientation baked in (it now
actually exercises redpiler's IN→wire→OUT signal path instead of
silently testing nothing while ignored).

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
