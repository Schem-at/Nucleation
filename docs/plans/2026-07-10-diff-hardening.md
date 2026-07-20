# Nucleation Diff/FFI Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans (TDD per task). Steps use checkbox syntax.

**Goal:** Close the six review findings (N1–N6 of schemati `docs/superpowers/specs/2026-07-10-vcs-hardening-design.md`) in the diff/fingerprint engine and FFI, bump version, rebuild the cdylib.

**Architecture:** Fixes are local to `src/diff/`, `src/fingerprint/`, `src/ffi.rs`. No public Rust API breaks; FFI signatures unchanged. TDD: each task writes the failing test first (`cargo test`), then the fix.

**Tech stack:** Rust (workspace at `~/RustroverProjects/Nucleation`), `cargo test` must be fully green after every task. Do NOT commit — leave the tree for review.

## Global constraints
- FFI signatures and existing JSON schema tags stay backward compatible (`nucleation.diff/1` may gain fields, never lose/rename).
- Bindings (python/wasm/jvm) recompile untouched unless a task says otherwise.

### Task 1 (N2): fingerprint + diff see block entities
- Un-ignore the NBT diff test at `src/diff/mod.rs:1023-1058`; make it pass by including block-entity data in the diff token comparison (`cells()` token derivation around `src/diff/mod.rs:146-160`).
- Fingerprint (`src/fingerprint/mod.rs:236-285` block iteration + token building at `:53-71`): hash block-entity NBT (stable serialization — sort compound keys) into each occupied cell's token.
- New tests: two schematics differing only in sign text → different fingerprint AND diff reports the cell as changed; identical schematics with identical block entities → equal fingerprints.

### Task 2 (N3): bound summary_json regions
- In `src/diff/mod.rs:532-555` (+ region building `src/diff/regions.rs:52-84`): cap `regions` at 100, keeping the largest by changed-cell count; add `regions_truncated: bool` and `region_total: usize` fields. Test with a synthetic 200-scattered-cells diff.

### Task 3 (N5): refine_offset pass cap
- `src/diff/mod.rs:276-304` / `src/diff/align.rs:224`: cap the brute-force search at ≤343 compare passes; when `(2*stride+1)^3` exceeds it, increase step so the pass count stays under the cap (coarser grid). Test: construct spans that would yield stride 12 and assert the search completes with bounded compare() invocations (instrument via a counter or refactor the pass loop into a testable fn).

### Task 4 (N6): no silent empty CStrings on diff paths
- `src/ffi.rs:9770,10001,10012`: replace `CString::new(s).unwrap_or_default()` with a helper that replaces interior NULs with U+FFFD. Unit test the helper.

### Task 5 (N4): diff_free_glb allocation fix
- `src/ffi.rs:10106-10130`: make the GLB export return a `Box<[u8]>`-backed buffer (len == capacity) or store capacity alongside; `diff_free_glb` frees with the matching layout. Test via a rust-side round-trip (allocate through the FFI fn, free, run under `cargo test` — miri optional, do not add CI).

### Task 6 (N1): catch_unwind on the whole diff/fingerprint FFI surface
- Wrap the bodies of `schematic_diff`, `schematic_diff_opts`, `diff_free`, `diff_distance`, `diff_support`, `diff_to_json`, `diff_from_json`, `diff_summary_json`, `diff_regions_json`, `diff_added/removed/changed/swapped/markers`, `diff_to_overlay_glb`, `diff_free_glb`, and all fingerprint externs (grep `pub extern "C" fn` in the diff/fingerprint sections of `src/ffi.rs`) in `std::panic::catch_unwind(AssertUnwindSafe(..))`; on Err set the existing last-error mechanism (grep how other externs record errors) and return null/0/-1 per return type. Test: a deliberately panicking internal hook (cfg(test) fn or a diff on a crafted invalid input known to panic) returns null instead of aborting — if no such input exists, test the wrapper helper directly.

### Task 7 (N7): version bump + build
- Bump `Cargo.toml` version (0.2.17 → 0.2.18), `cargo build --release` the cdylib, run the FULL `cargo test` suite. Report the built dylib path (`target/release/libnucleation.dylib`) — schemati re-pointing happens outside this plan.
