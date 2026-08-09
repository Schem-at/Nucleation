# Developing Nucleation — the iteration recipe

This document exists because iterations were taking **1h15–1h45**. The causes were
measured, not guessed, and they were mostly not "the compiler is slow".

## TL;DR

```bash
brew install sccache            # prerequisite: .cargo/config.toml wires it as rustc-wrapper
tools/dev.sh fast               # the loop — seconds
tools/dev.sh fast mc-tick       # ...plus tests for just the crate you touched
tools/dev.sh pre-land           # before handing work over — minutes
tools/dev.sh full               # the merge gate — everything, exhaustive
tools/doctor.sh                 # "why is my loop slow?" in one screen
```

## The one rule: iterate on the canonical feature set

```
bridge-full,routing,hdl,meshing
```

The crate can be built as `default` / `simulation` / `bridge` / `bridge-full` /
`+routing` / `+hdl` / `+meshing`, for three wasm32 targets, plus release wheels.
**Every distinct feature set is a distinct fingerprint for all 159
dependencies**, so each one you invoke builds and stores its own copy of
everything. Seven feature sets is seven target directories wearing one name.

So: iterate on one set. `cargo ck` and `cargo ct` (aliases in
`.cargo/config.toml`) are that set; `tools/dev.sh fast` runs them. The other
feature permutations are still checked — by `tools/dev.sh full`, which is the
merge gate. Nothing is given up, it is just moved off the inner loop.

## Tier table

| Tier | Command | Measured | What it covers |
|---|---|---|---|
| **fast** | `tools/dev.sh fast [crate]` | **3.6–4.9s** | `cargo check` on the canonical set, `--all-targets` (1s); unit tests — the named crate's `--lib`, else every workspace crate except the root (1s); three **sampled** sim cases (~1-2s each) |
| **pre-land** | `tools/dev.sh pre-land` | minutes | full `cargo test` on the canonical set (includes the root crate's ~3 min `--lib` suite) + `cargo test --workspace` (all integration suites); wasm32 checks ×3 (pnr-core, nucleation-routing, browser engine); eda-studio headless `npm run verify`; Python bridge smoke |
| **full** | `tools/dev.sh full` | the merge gate | everything above **unsampled** + all 8 feature permutations + `tools/check_routing.sh` + exhaustive sim suites (`EDA_EXHAUSTIVE=1`: adder, rca, alu, ppa, genlib ×4, hdl ×3) + npm/wasm + Python wheel + `tools/prepush.py` (bindings freshness/determinism, bridge coverage, all five language smokes) |

Two things are deliberately kept out of `fast`, both for **runtime**, not build
time — and both are still run by `pre-land` and `full`:

- **mc-tick's slow integration suites.** `machine_graph` is 89.3s and `timeline`
  14.3s; every other suite is under a second and the whole `--lib` set is 0.48s.
  So `fast <crate>` runs `--lib` only.
- **The root crate's own `--lib` suite**: 1,276 tests, ~3 minutes of pure
  runtime (a filtered single-test run costs the same, so it is not a build
  artefact). `fast` with no crate argument runs
  `cargo test --workspace --exclude nucleation --lib` instead.

`fast` and `pre-land` **sample** the exhaustive sim suites. Those are 256-case
and 3040-check loops — a merge gate, not an iteration cost. `full` sets
`EDA_EXHAUSTIVE=1` and passes no `--cases`, so it runs them whole. **Do not
weaken `full`.** If you need a sampled run bigger or smaller, set
`EDA_SAMPLE_CASES=N`.

## Why the loop was slow (measured 2026-08-09)

Baseline, then after the fixes below, same machine and identical commands:

| Measurement | Before | After |
|---|---|---|
| Touch one file, `cargo check` (canonical) | **180.3s** | **11.3s** (16×) |
| `cargo test -p mc-tick` | **52.8s** | **12.3s** (4.3×) |
| No-op `cargo check`, steady state | 0.4s | 0.4s (unchanged) |
| No-op `cargo check`, first after a big build | 31.8s | 34.5s (unchanged) |
| `target/` on disk | **102 GB** | **19 GB** (−83 GB) |
| Files in `target/debug/deps` | **774,187** | **1,843** |
| Free disk | 58 GiB (93% full) | 141 GiB (84% full) |
| `tools/dev.sh fast` | n/a (no such tier) | **3.6–4.9s** |

**One hypothesis was wrong and is worth recording.** The 31.8s no-op check was
assumed to be fingerprint scanning over 774,187 files. It was not: after the
file count fell to 1,843 the same no-op check still cost 34.5s *the first time*,
then **0.4s on every subsequent run** — and it was 0.4s in steady state before
the cleanup too. That ~30s is a cold page cache (and build-dir lock contention),
paid once after a large build evicts cargo's metadata from RAM; it is not
proportional to `target/` size. The genuine wins were in *rebuilds* and *disk*,
not in the no-op path.

Three real causes, in order of size:

### 1. Debug info on 159 dependencies (the disk bomb, and the rebuild cost)

macOS `dev` defaults are `-C debuginfo=2 -C split-debuginfo=unpacked`, which
leaves a fan of `.o` object files per crate **per feature combination** — 85 GB
and 774,187 files in `target/debug`. Emitting and writing all that is also why
touching one file cost 180s.

`[profile.dev.package."*"] debug = 0` in `Cargo.toml` removes debug info from
dependencies; our own crates keep `debug = "line-tables-only"`, so panics and
`RUST_BACKTRACE` still name file and line. Only in-debugger variable inspection
is lost, and only for dependencies.

Need to single-step our code? `RUSTFLAGS="-C debuginfo=2" cargo test ...` for
that run — don't edit the profile.

### 2. Concurrent cargo processes fighting over one build-dir lock

During the baseline there were **three** cargo processes running at once: the
dev-loop check, a wasm32 build, and RustRover's flycheck
(`cargo check --workspace --all-targets`, i.e. *default* features). Cargo takes
an exclusive lock per target directory, so these serialise — the "no-op check
timed out at 30s" was partly `Blocking waiting for file lock on build
directory`. Worse, the IDE's *different* feature set maintains its own full set
of artifacts.

**Fix your IDE:** point RustRover / rust-analyzer at the canonical feature set
so it shares artifacts with your loop instead of doubling them.

- RustRover: *Settings → Languages & Frameworks → Rust → Cargo* → set the
  feature set to `bridge-full,routing,hdl,meshing` (not "all features", not
  default).
- VS Code / rust-analyzer, in `.vscode/settings.json`:
  ```json
  { "rust-analyzer.cargo.features": ["bridge-full", "routing", "hdl", "meshing"] }
  ```

`tools/doctor.sh` warns when more than one cargo process is running.

### 3. No compiler cache across feature sets

`sccache` (wired as `build.rustc-wrapper` in `.cargo/config.toml`) caches
dependency compilations keyed by the exact rustc invocation. Switching feature
sets, or recovering from a `cargo clean`, now pulls the 159 dependencies out of
the cache instead of rebuilding them. Workspace crates compile *incrementally*
and bypass the cache by design — the win is entirely on dependencies.

`brew install sccache` is therefore a **prerequisite**: cargo hard-fails with
"could not execute process sccache" if the wrapper binary is missing. Inspect it
with `sccache -s` (verified working here: 39 hits / 444 misses filling a cold
cache during the first rebuild).

Because the wrapper is committed, any environment *without* sccache must opt
out with an empty override — `RUSTC_WRAPPER=""`. `.github/workflows/ci.yml` (the
release pipeline) does exactly that at workflow level, since it cross-compiles
on three OSes and builds wheels inside manylinux containers where sccache is not
installed. `dev-tiers.yml` installs sccache instead.

### Not a cause: the linker

No alternative linker is configured, deliberately. Xcode's default here is
`ld-prime` (ld-1167.4.1), already Apple's fast parallel linker on arm64;
Homebrew's `lld` pulls the full ~2 GB `llvm`. And the two dominant measurements
are `cargo check`, which never links. Revisit only if link time actually shows
up in `cargo build --timings`.

## Two known failure modes that are not your change

`pre-land` was measured at **644s** on master. Both of its failures were
environmental, and both are worth recognising before you go bug-hunting:

- **`simulation::typed_executor::compiled::tests::bench_compiled_vs_schematic_start`**
  asserts `compiled_us <= full_us * 2` — a wall-clock ratio, inside a unit test.
  It fails under parallel load and passes 3/3 in isolation. This is the same
  effect as the known-unreliable bench gate: if this is the only red test, check
  whether something else was building at the time before believing it. A timing
  assertion like this belongs in `benches/`, not in the `--lib` suite.
- **`studio: headless verify` dying with `page.goto ... ERR_CONNECTION_REFUSED`**
  usually means a stray `vite preview` from an earlier session still holds the
  preview port, so `npm run verify` could not start its own server (the real
  clue, `Port 8461 is already in use`, is buried ~20 lines earlier). `pre-land`
  now pre-flights the port and names the process holding it — often the pid in
  `apps/preview.pid`.

## Disk hygiene

`target/` grows without bound because each feature combination, each wasm32
target, and each release wheel build adds its own artifacts. It reached
**102 GB** on a disk that was 93% full.

Run `tools/doctor.sh`. It reports target size, `debug/deps` file count, free
disk, whether sccache and a fast linker are active, and concurrent cargo
processes — and warns at the thresholds where this repo started hurting
(target > 40 GB, deps > 200k files, incremental > 5 GB, free < 25 GiB).

When it warns:

```bash
rm -rf target/debug/incremental   # cheapest, always safe (was 8.9 GB)
rm -rf target/debug               # full debug prune; one warm rebuild to recover
cargo clean                       # everything, including release + wasm32
```

Prefer `rm -rf target/debug` over a full `cargo clean`: it keeps
`target/release` and `target/wasm32-unknown-unknown` (~16 GB combined), which
the wheel and npm builds depend on and which the `dev` profile change does not
invalidate. With sccache warm, recovering from a debug prune is much cheaper
than it used to be.

Never commit build output. `/target` and friends are in `.gitignore`; if you add
a new build directory, add it there in the same commit.

## Wheel and wasm rebuilds

**The Python wheel and the npm/wasm package only need rebuilding when the bridge
surface changes** — i.e. when you touch `src/bridge/**` (or anything it
re-exports), the manifests, or the generated `bindings/`. Editing an internal
crate, a test, or a doc does not require either.

Both are guarded, so a no-op rebuild is cheap rather than minutes:

- `tools/package-npm.sh` stamps its inputs (Rust sources, manifests, generated
  JS glue, npm veneers, and the feature set) into `$OUT/.build-stamp` and exits
  early when they are unchanged. Force with `NUCLEATION_FORCE_REBUILD=1`.
- Regenerate bindings with `tools/gen-bindings.sh` only after a bridge change;
  `tools/prepush.py` and CI verify that the committed `bindings/` regenerate
  byte-identically, so a stale regeneration cannot slip through.

## CI

`.github/workflows/dev-tiers.yml` runs the tiers on push/PR with sccache and
target caching: `fast` and `pre-land` on every push, `full` on pull requests
into `master` and on tags. `.github/workflows/ci.yml` remains the
release/publish pipeline.
