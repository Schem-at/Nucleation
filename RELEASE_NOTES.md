# Nucleation v0.2.8

CI / packaging redo of v0.2.7. The two workflows are merged so the JVM
fat JAR builds in the same pipeline as every other artifact and is
attached to the GitHub Release alongside crates / wheels / npm package.

## What changed since v0.2.7

- **Single unified workflow.** `.github/workflows/jvm.yml` has been
  merged into `.github/workflows/ci.yml`. The 4 JVM jobs (matrix cdylib
  build, host Gradle test, JVM ↔ Python parity, fat-JAR assembly) now
  run in the same pipeline as the Rust / WASM / Python jobs, and the
  `publish` step waits on the JAR before creating the release.
  - Side effect: `actions/cache@v4` now caches `nucleation-jvm/target`
    keyed on `${runner.os}-${target}-cargo-jvm-${{ hash(Cargo.{toml,lock}) }}`
    for faster reruns.

- **CI bug fixes that landed too late for v0.2.7:**
  - `build-jvm-cdylib (windows-x64)` was running bash backslash
    continuations under PowerShell, which parses `--` as a unary
    operator. Fixed by collapsing the command to a single line and
    explicitly setting `shell: bash`.
  - `parity-jvm` ran `rustc -o target/check_jvm_parity` on a clean
    runner where `target/` didn't exist. `mkdir -p target` first.

No source code changes since v0.2.7 — the Rust / Python / JS surface is
identical. The only meaningful behaviour change is that a `git push
--tags` now produces a complete release in one go, with the
`nucleation-<version>.jar` attached.

## Carried forward from v0.2.7

- Bedrock `.mcstructure` export: properties preserved through
  Java→Bedrock translation; block-entity NBT translated symmetrically
  via blockpedia 0.1.9's new `translate_java_to_bedrock`.
- `nucleation-jvm`: standalone JVM crate + Gradle fat-JAR for Java /
  Kotlin / mod consumers.
- Pre-push gate runs 24 checks across 5 lanes covering every binding.

See the v0.2.7 release notes for the full feature description.
