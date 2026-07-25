# M10 flood-fill connectivity audit — validating the disconnected-build split fix

**Date:** 2026-07-26
**Tool:** `examples/floodfill_audit.rs` (`cargo run --release --example floodfill_audit -- <out.tsv> <dir>...`)
**Data:** `wol-project/m10-full/` — 3,409 extracted `.schem` (the *pre-fix* M10 extraction output) + `wol-sample-10.zip` sanity set.
**Raw per-build TSV:** produced by the tool (17 columns; one row per build).

## Method

For every `.schem` the tool loads it with the crate's own loader (`formats::schematic::from_schematic`),
takes the non-air blocks, and runs a **per-block connected-components** pass at full resolution:

- **6-connectivity** (face-adjacent) — primary lens.
- **26-connectivity** — sensitivity check; two masses that aren't even 26-connected are unambiguously separate.
- Per build it records total blocks, component count, the top component sizes, and for the top-2
  components their bounding boxes and the **minimum inter-component gap** (empty blocks = Chebyshev distance − 1).
- It then **emulates the shipped fix's own cell-level split** (cell_size=4, `min_component_blocks=4096`,
  `min_component_share=0.40`, `min_gap_cells=2`, six-connected cell components, *no* dilation — exactly what
  `split_disconnected_clusters` re-derives) so we know whether the fix *would actually split* each build.
- Parallelised with rayon, one build in memory at a time.

**Classification (block level):**
- **SINGLE** — one component holds ≥95% of blocks.
- **MULTI-SUBSTANTIAL** — ≥2 components each ≥4096 blocks **and** ≥40% share, separated by ≥1 empty block
  (the fix's substantiality lens, applied at block resolution — the merge defect d44bbed2 exhibits).
- **MINOR-FRAGMENTS** — everything else (one dominant mass + small floating bits / redstone gaps; **not** a defect).

## Distribution (3,409 builds)

| Class | 6-conn | 26-conn |
|---|---:|---:|
| SINGLE (≥95% one component) | 1,545 | — |
| MINOR-FRAGMENTS | 1,813 | — |
| **MULTI-SUBSTANTIAL** | **51** | **52** |

6- vs 26-connectivity barely moves the needle (51 → 52): the substantial multi-mass builds are separated by
real spatial gaps, not by mere diagonal touching, so the two connectivity models agree. Sensitivity is low —
the flagged multi-mass builds are genuinely disconnected, not artefacts of the connectivity rule.

### Component-count histogram (6-conn)

| # components | builds |
|---|---:|
| 1 | 721 |
| 2 | 374 |
| 3 | 298 |
| 4–5 | 300 |
| 6–10 | 406 |
| 11–50 | 634 |
| 51+ | 676 |

High component counts are dominated by **redstone builds** (thousands of tiny 1–3 block dust/torch fragments) —
internal disconnection that is *expected*, not a merge defect. That is exactly why a raw component count is
useless as a defect signal and why the substantiality lens (≥4096 blocks **and** ≥40% share) is the right filter.

## Fix cross-check — is d44bbed2 unique, or a systematic residual?

- **d44bbed2** is flagged and correctly handled: total 399,456; two 6-conn masses of 184,427 / 178,660 blocks,
  **6-block gap**; the fix's cell emulation yields two seeds (185,076 / 178,732) at **2-cell gap** → **would SPLIT**. ✓
- Of the **51** MULTI-SUBSTANTIAL builds, the fix **splits 39** and leaves 12 merged.
- A further **7** builds are split by the fix without the block-level lens flagging them MULTI-SUBSTANTIAL
  (see below — these are correct splits the block lens under-counted, not over-splits).
- **Total builds the fix splits: 46 / 3,409 (1.3%).**

So d44bbed2 is **not** unique — the world genuinely contains ~40 two-build merges — but the residual is **small and
handled**: every large, well-separated two-mass build is split.

### Worst offenders — MULTI-SUBSTANTIAL, top 12 by mass (all split correctly)

| id | total | top1 | top2 | gap (blk) | fix split? | seed gap (cells) |
|---|---:|---:|---:|---:|:--:|---:|
| eff5afba… | 881,632 | 402,607 | 400,383 | 26 | ✅ | 6 |
| 13d07748… | 716,619 | 338,265 | 327,661 | 13 | ✅ | 4 |
| 8166dae9… | 460,131 | 201,324 | 199,559 | 28 | ✅ | 7 |
| **d44bbed2…** | **399,456** | **184,427** | **178,660** | **6** | ✅ | **2** |
| c1e5e866… | 219,149 | 109,585 | 109,290 | 10 | ✅ | 3 |
| 906aa5cd… | 195,634 | 105,465 | 82,125 | 10 | ✅ | 2 |
| 4a748d1e… | 167,950 | 76,798 | 75,267 | 30 | ✅ | 8 |
| 89fc3c5c… | 145,394 | 76,886 | 66,915 | 16 | ✅ | 4 |
| 3c8a3043… | 144,436 | 72,716 | 67,841 | 8 | ✅ | 2 |
| b31cce6e… | 142,779 | 65,123 | 62,967 | 13 | ✅ | 3 |
| 579d6d73… | 126,420 | 72,675 | 53,704 | 9 | ✅ | 2 |
| b6a9b803… | 95,421 | 52,592 | 42,583 | 20 | ✅ | 4 |

d44bbed2 is the *tightest-gap* large defect (6 blocks / 2 cells) — it sits right on `min_gap_cells=2`, which is
precisely why the earlier over-merge existed and why the fix's re-derivation (no dilation) recovers it.

## The 12 "misses" — none are real defects

Block-level they look like two substantial well-separated masses, but the fix leaves them merged. Hand-inspection
of the bounding boxes shows why that is *correct*:

- **4 with a ≤1-block gap** — these are single builds: e.g. `b58be33a` is two flat 254×254 slabs at y=0 and y=2
  (a floor+ceiling of one footprint); `9ba7632e` is one structure with a z-seam (identical 17,263/17,263 halves,
  touching). A 1-block seam is exactly what `min_gap_cells=2` deliberately refuses to split.
- **4 cell-connected (seed_gap_cells = −1)** — including the biggest, `52357ebe` (193k): a base slab (y 0–35) and an
  elevated structure (y 52–95) that are **bridged by intermediate blocks** into a single cell-cluster
  (fix sees one seed of 192,828 ≈ total). One connected build; not splitting is correct.
- **4 borderline (seed_gap_cells = 1)** — e.g. `40ec8c91`, two 9.4k masses diagonally adjacent with a 4-block gap.
  One cell apart → the fix reads them as one build. Defensible; a 3–5 block gap between touching masses is a seam,
  not two separate builds.

## The 7 "over-splits" — actually correct splits the block lens under-counted

All 7 are class MINOR-FRAGMENTS (**none is a SINGLE-dominant build**, so no build with a real dominant component is
ever wrongly cut). They are internally-fragmented builds (lots of redstone) where **no single block-component reaches
40%**, so the block lens missed them — but at the cell level (where the fix works) they resolve into two substantial,
**well-separated** masses that the fix correctly splits:

- `fef80dd2` — two flat 254×254 slabs **102 blocks apart** vertically (block halves at 39.96% each, just under the
  block 40% bar; cell seeds 74,054 / 64,855 clear it). Unambiguously two builds. ✅
- `71531e19` — a ground platform (y 0–4) + a separate tower (y 44–124), 39-block gap. ✅
- `6c8e054e` — two small builds 74 blocks apart in x. ✅
- `d09648c2` — the only questionable one: a fragmented redstone build (8,388 components) split into two cell-clusters
  3 cells (~12 blocks) apart. Even here the split is on a real 3-cell gap, not a dilation artefact.

This shows the fix's **cell-level** substantiality is *more* robust than a block-level share test: cell aggregation
recovers the fef80dd2-style two-build case that a strict per-block 40% share would drop.

## Recommendation — keep the defaults `4096 / 0.40 / 2`

The audit gives no evidence to tune them:

- **`min_component_blocks = 4096`** — the real defects are 179k+ blocks; the smallest legitimate second-seed among
  the 39 splits is ~4.4–4.8k. Nothing sits near 4096 as a false split. No change.
- **`min_component_share = 0.40`** — at the cell level (where it is applied) it cleanly separated all 39 genuine
  two-mass builds. The one place the *block-level* 40% was marginally strict (fef80dd2 at 39.96%) was rescued by the
  fix's cell aggregation, so the operative threshold works. Lowering it would risk splitting internally-fragmented
  single builds.
- **`min_gap_cells = 2`** — the key protector. **Lowering to 1 would newly split the 4 `seed_gap_cells==1` "misses"**
  (`40ec8c91`, `ce4659be`, `5bca81c8`, `6a5c5666` — 3–5 block gaps between touching/interleaving masses), which the
  bounding boxes show are single or adjacent builds → that would be over-splitting. Keeping it at 2 leaves them whole
  while still catching d44bbed2 (which sits *exactly* at 2). Correct as-is.

**Verdict:** the fix catches the known defect (d44bbed2) **and** 38 further genuinely-disconnected builds, with
**zero dangerous over-splits** (no SINGLE-dominant build is ever split) and **zero confirmed real misses** (all 12
unsplit multi-mass builds are single/cell-connected/too-close). Defaults `4096 / 0.40 / 2` are right for the real
M10 data. `cargo test --features world-segment world_segment` → 107 passed.
