# Diff Engine Hardening — Work Order (TDD)

**Status:** ready to implement
**Branch:** `claude/ecstatic-brahmagupta-qg4oai`
**Scope:** `src/diff/` (core + bindings), `src/fingerprint/` (only where tests need fixtures)
**Method:** Test-Driven. For every item below: **write the test first, run it, confirm it fails (or confirm the bug), then implement, then confirm green.** Do not batch implementations ahead of their tests.

---

## 0. Why this matters (read before touching code)

The diff engine is the foundation for turning Nucleation into a **full version-control system for Minecraft builds**. The target topology:

- **File-backed object store** — content-addressed: each hash → one file (Git-like).
- **Three runtimes, one core:** a **client mod** (fancy UI front-end), a **server plugin** (command-driven; the mod's UI takes over when the client mod is present), and a **remote website (schemat.io, PHP)**. All three call into Nucleation.

This context drives the priorities:

1. **Determinism is non-negotiable.** A content-addressed VCS hashes diff/serialization output. If the same two builds can produce two different JSON byte streams across runs/machines/language bindings, dedup breaks, history forks spuriously, and the PHP site, the plugin, and the mod will disagree on the same object's hash. Every nondeterministic tie-break is a latent corruption bug.
2. **Honest metrics.** `support` and `distance` will be surfaced in UI and used for merge/conflict heuristics. Misleading values become wrong UX and wrong merges.
3. **Coverage of intent.** A VCS that says "no changes" when a chest's contents changed is broken for the use case. Block entities/entities must eventually be in scope.
4. **Scale.** Worlds and large builds are first-class inputs (the README sells world import). Alignment must not silently give up at scale.

Keep these in mind: when a design choice is ambiguous, **favor the option that is deterministic, reproducible across bindings, and honest in its reported numbers.**

---

## Ground rules for the implementer

- Tests live inline as `#[cfg(test)] mod tests` in the relevant file, matching existing style in `src/diff/mod.rs`.
- Fixtures come from `crate::fingerprint::testgen` (`filled_box`, `edited`, `repalette`, `translated`, `rotated_y`, …). Add new generators there if needed.
- Run a single test with: `cargo test --lib diff::` (add feature flags as needed, e.g. `--features meshing` for overlay/region work that touches meshing).
- After each item: run `cargo test`, `cargo clippy --all-targets`, `cargo fmt`. The repo has `./pre-push.sh` — run it before the final push.
- Commit per item with a clear message (e.g. `fix(diff): deterministic tie-breaking in alignment and swap collapse`). Push to the branch above. **Do not open a PR.**
- When an item changes serialized output (`to_json`), add/refresh a round-trip + stability assertion.

---

## P1 — Determinism (do this first)

### Item 1.1 — Deterministic tie-breaking in `hough_translate`

**Bug:** `src/diff/align.rs:44-51` collects votes from a `HashMap` into a `Vec` and sorts by count descending. Equal-vote translations are ordered by HashMap iteration order, which Rust's `RandomState` randomizes per process. The chosen `transform.translate` can differ run-to-run for symmetric/featureless/ambiguous builds.

**Test first** (`align.rs` or `mod.rs` tests):
- Construct a build with a deliberate translation ambiguity (e.g. a small motif that appears at two candidate offsets so the top votes tie). Run `diff` (or `hough_translate` directly) **many times in a loop (≥50)** and assert the chosen `transform.translate` is identical every iteration. Because the bug is probabilistic, a loop is what exposes it; a single call may pass by luck.
- Confirm it fails (flakily today — run it a few times if needed; the loop should catch it).

**Implement:** make the sort total and content-defined. After sorting by vote count descending, break ties by a deterministic key — e.g. `(Reverse(count), translate.0, translate.1, translate.2)`. Use `sort_by` / `sort_by_key` with that composite key so the result is independent of insertion order.
**Green:** the loop test passes deterministically.

### Item 1.2 — Deterministic swap collapse

**Bug:** `src/diff/collapse_swaps` (`mod.rs:197-237`) iterates `confusion: HashMap<Token, HashMap<Token, usize>>` to build the `swaps: Vec` and uses `max_by_key` for the dominant target token. Both the output ordering of `palette_swaps` and the `max_by_key` tie-break depend on HashMap iteration order. `palette_swaps` ordering leaks into `to_json`.

**Test first:**
- Build A→B with **two distinct palette swaps** (e.g. stone→cobble and dirt→gravel over equal-sized regions) plus a token whose targets tie 50/50. Run `diff` ≥50× and assert `d.palette_swaps` is byte-for-byte identical each run (compare the `Vec<(Token, Token)>` directly), and that `to_json()` is identical each run.
- Confirm failure.

**Implement:**
- Collect `confusion` into a `Vec`, sort the outer iteration by source token (and the inner `max_by_key` tie broken by target token name) before deciding swaps.
- Sort the final `swaps` Vec by `(source_token, target_token)`.
- For the dominant-target selection, when counts tie pick the lexicographically smallest target token (deterministic).

**Green:** swap ordering and JSON stable across runs.

### Item 1.3 — `to_json` stability guard (regression net)

**Test first:** add a test that runs `diff` on a fixed fixture **twice in the same process** and asserts `d1.to_json() == d2.to_json()` exactly. Also assert the per-cell arrays (`added`/`removed`/`changed`/`swapped`) are emitted in a deterministic order — if they currently come straight from `compare`'s HashMap iteration (`mod.rs:159-179`), they are **also nondeterministic** and must be sorted before serialization.

> **Note for implementer:** check `compare` — `added`/`removed`/`changed` are pushed while iterating `amap`/`bmap` HashMaps, so their order is nondeterministic too. The cleanest fix is to **sort the cell vectors by position** at the end of `diff_at` (or in `to_json`). Sorting in `diff_at` makes the whole `Diff` value canonical, which is what the VCS layer will want. Prefer that. Positions are unique per vector, so the sort is total.

**Implement:** sort `added`, `removed`, `changed`, `swapped` by `IVec3` (lexicographic) at the end of `diff_at`. Keep `palette_swaps` sorted per 1.2.

**Green:** identical JSON across runs; existing round-trip test still passes.

---

## P2 — Honest metrics

### Item 2.1 — Fix `support` semantics

**Problem:** `support = matched / max(|A|,|B|)` where `matched` counts only exact token matches (`mod.rs:166-167, 284`). A perfectly aligned but fully re-paletted build reports `support ≈ 0`, contradicting the "alignment confidence" docstring.

**Decision required (default if unspecified): make `support` an alignment metric** = `(matched + changed + swapped) / max(|A|,|B|)` — i.e. every cell that found a counterpart at the chosen transform counts as aligned, regardless of equivalence. `added`/`removed` (cells with no counterpart) remain the only "unaligned" cells.

**Test first:**
- A vs `repalette(A, stone, cobble)` under `exact`: assert `support == 1.0` (every cell aligned; it's a pure swap) while `distance == 1`.
- A vs `edited(A, n)` with only `added`/`removed`: assert `support < 1.0` and equals `1 - (added+removed)/max`.
- Confirm the first assertion fails today.

**Implement:** thread `changed` and `swapped` counts into the support computation in `diff_at`. Update the docstring on `Diff::support` and the README's "support" note to match the new definition.

**Green:** both assertions pass; update any existing test that asserted the old `support` value.

### Item 2.2 — `u64` distance, overflow-safe

**Problem:** `distance` is `u32` and computed as `cost * len as u32` (`mod.rs:269-272`); world-scale diffs with non-unit costs can overflow.

**Test first:** a test using a `CostModel` with large weights and a fixture big enough that `u32` would overflow but `u64` won't (or assert via `saturating`/checked arithmetic that no wrap occurs). Keep the fixture small by using a large cost weight rather than millions of blocks.

**Implement:** change `Diff::distance` to `u64`. Use `u64` math (`as u64`) in `diff_at`. Update `to_json`/`from_json` (`as_u64` already used on read; widen the field). Update bindings (`src/wasm/diff.rs`, `src/python/diff.rs`) to surface `u64`/`number`/`int` consistently. Update tests asserting `distance` to the new type.

**Green:** no overflow; bindings compile; round-trip intact.

---

## P3 — Magic numbers → configuration

### Item 3.1 — Hoist the 80% swap threshold into config

**Problem:** the `cnt * 100 >= total * 80` swap threshold (`mod.rs:223`) is hardcoded.

**Test first:** a test that sets the threshold low vs high on the same fixture and asserts the swap-vs-change classification flips accordingly.

**Implement:** add `swap_dominance_pct: u32` (default 80) to `CostModel` **or** a new field on `DiffSpec`/`AlignOptions` — put it wherever fits the existing override plumbing (`SpecOverrides` already overrides costs; consider adding `swap_dominance_pct` there too). Thread it through `collapse_swaps`. Default behavior unchanged.

**Green:** threshold is configurable; default-path tests unchanged.

---

## P4 — Regions consistency

### Item 4.1 — Regions must include swapped cells

**Problem:** `RegionKind` has no `Swapped` and `regions()` ingests only added/removed/changed (`regions.rs:7-33`), so `summary_json`'s region map silently drops palette-swapped areas — inconsistent with the lossless per-cell JSON.

**Test first:** a fixture producing palette swaps; call `regions(&d)` and assert the swapped cells appear in some region (and that a region's `kind` reflects `Swapped`, with mixed clusters → `Mixed`).

**Implement:** add `RegionKind::Swapped`; ingest `diff.swapped` positions in `regions()`. Make sure `Mixed` still triggers when a connected cluster spans multiple kinds.

**Green:** swapped cells are clustered and surfaced in `summary_json`.

---

## P5 — Coverage of intent (larger; land behind tests, may iterate)

### Item 5.1 — Air sensitivity under `exact` (test + fix if needed)

**Problem/uncertainty:** `tokenize` returns a token whenever a rule matches (`classifier.rs:85-99`). If the `exact` ruleset tokenizes air, then explicit-air in A vs implicit-air in B can surface as spurious `removed`/`changed`.

**Test first:** build A with an explicit `minecraft:air` block set where B simply has nothing. Diff under `exact`. Assert the result is **zero distance** (air is absence, not a block). Confirm whether it fails today.

**Implement (only if it fails):** ensure air is never tokenized as a present cell in the diff path — either filter `minecraft:air` (and `cave_air`/`void_air`) in `cells()` before tokenizing, or ensure the rulesets map air to `None`. Prefer filtering in `cells()` so it's preset-independent and consistent across bindings. Document the decision in the spec.

**Green:** explicit vs implicit air diffs to zero across presets.

### Item 5.2 — Block-entity / entity awareness (design note + first test)

This is the biggest gap for VCS-of-builds: identical blocks but different chest contents / sign text / entities currently diff to zero (`cells()` uses `iter_blocks` only, `mod.rs:133`).

**For this work order, do NOT fully implement** — instead:
- Write a **currently-failing (or `#[ignore]`d) test** documenting the desired behavior: two builds identical in blocks but differing in a chest's NBT should produce a non-zero diff with the changed cell reported.
- Write a short design stub at the bottom of this file (section "Appendix: block-entity diffing") proposing: extend `Cell`/the change records with an optional NBT digest, compare block-entity NBT at matched positions, and add a `changed`-with-nbt category or fold into `changed`. Flag the binding/JSON schema implications (schema version bump `nucleation.diff/2`).

Leave it ignored so the suite stays green; we'll design the full thing together next.

---

## P6 — Scale & performance (land after P1–P4)

### Item 6.1 — Don't silently give up on large featureless alignment

**Problem:** Hough ignores tokens with count > `anchor_max_count` (64, `align.rs:27`); FFT bails if any axis > `limit` (96, `mod.rs:248`). A large translated low-diversity build gets no anchors AND fails FFT → falls back to `(0,0,0)` and reports a huge spurious diff.

**Test first:** a translated featureless box larger than 96 on an axis (today's `diff_aligns_a_featureless_translated_box` uses a *small* box). Assert it still aligns to distance 0 — this will fail today.

**Implement (choose, document the choice):**
- Downsample/block-pool the occupancy grid before FFT so large builds fit the size budget (align coarsely, then refine), **or**
- Raise/auto-scale `limit` with a memory guard, **or**
- Add a coarse-stride Hough that votes on a subsampled anchor set even for high-count tokens.
Prefer the downsample-then-refine approach: coarse FFT to get an approximate offset, then a small local search (±few blocks) using `compare` to snap to the exact translation. Keep peak memory bounded.

**Green:** large translated build aligns; add an assertion on bounded grid size if you downsample.

### Item 6.2 — Stop recomputing `compare` for the chosen offset

**Problem:** on FFT fallback, `compare` runs up to 3× (`score(ft)`, `score(t)` at `mod.rs:250-256`, then again in `diff_at` at `mod.rs:266`).

**Test:** this is a perf/cleanup item — guard it with a correctness test (output unchanged on a representative fixture) rather than a timing test. Optionally add a `#[bench]` under `benches/` if cheap.

**Implement:** restructure so the winning offset's `RawDiff`/score is reused by `diff_at` instead of recomputed. Keep results identical.

**Green:** identical diff output; one fewer `compare` pass on the fallback path.

---

## Acceptance checklist (all must hold before final push)

- [ ] `cargo test` green (all features that compile in CI: default, `--features simulation`, `--features meshing`).
- [ ] `cargo clippy --all-targets` clean; `cargo fmt` applied.
- [ ] `./pre-push.sh` passes.
- [ ] Determinism: a diff run twice in-process yields byte-identical `to_json()` (Item 1.3 test).
- [ ] `support` reports `1.0` for a pure re-palette under `exact` (Item 2.1 test).
- [ ] `distance` is `u64` end-to-end including bindings.
- [ ] Swap threshold configurable; regions include swapped cells.
- [ ] Air test passes; block-entity test present (ignored) + appendix design stub written.
- [ ] Large featureless translated build aligns to distance 0.
- [ ] Commits are per-item with clear messages; pushed to `claude/ecstatic-brahmagupta-qg4oai`; **no PR opened.**

---

## Appendix: notes toward the VCS layer (for when we resume designing)

These are not part of this work order — captured so the diff fixes above stay aligned with where we're heading. The diff engine is the *delta* primitive; the VCS layer will sit on top.

- **Canonical serialization is the contract.** Once `to_json` is deterministic + position-sorted (P1), it can be the basis for content-addressing deltas. Consider a canonical *binary* encoding later (smaller, faster to hash) with the JSON as the debug/interchange form. The PHP site, plugin, and mod must all produce identical bytes for identical inputs — add a cross-binding determinism test (same fixture through Rust and Python, assert equal hashes).
- **Object model (file-backed, Git-like):** `blob` = a build snapshot (canonical fingerprint of the whole build → hash → file); `delta` = a `Diff` between two blobs; `commit` = parent(s) + tree/blob hash + metadata; `ref` = name → commit hash. The diff engine already gives us deltas and `fingerprint()` gives us blob identity — the alignment transform stored in the delta is what lets us reconstruct B from A.
- **Apply/replay already exists in spirit:** `world_stream`'s `patch_chunk` replay loop (README) is the apply-a-delta operation. Generalize `Diff::apply(&A) -> B` as a first-class method (added/removed/changed/swapped → block ops at the stored transform). Add a round-trip test: `apply(diff(A,B), A) == B` under the preset. This is the property the whole VCS rests on.
- **Three-way merge** = diff(base, ours) + diff(base, theirs), detect overlapping changed cells as conflicts. The region clustering (P4) is the natural unit for presenting conflicts in UI.
- **Preset choice is policy:** `exact` for lossless history, `structural`/`redstone` for semantic dedup. The VCS should record which preset produced a delta.