# Local Atlas imports and plot-door collection

Checked 15 November 2026. Source schematics, PNGs, full captures and ZIPs remain
in the separate local `Nucleation-schematic-archive` directory, outside Git.
The repository contains the tools, UI and this report. Existing CI fixtures
remain tracked; the 803 plot-door source blobs do not occur in `master` at
`f2fb59f7708a4715878f349d0612bf8ba2b069ec`.

## Supplied hipster original

`Fastest_3x3_Hipster_-_SpaceWalker_and_SoulBanished (2).litematic` was absent
from the Atlas. It is now indexed under `user-local-schematics`, with attribution,
a native textured thumbnail and a working live-browser launch.

- Exact source SHA-256: `a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0`.
- Saved data version 2730, 990 non-air blocks, 48 block entities, 13 × 8 × 18.
- Fresh check on **harrison-work**, using the published Python package
  `nucleation==0.12.3`: initial block states matched the existing benchmark
  fixture; all **640 tick-end frames** matched the retained Minecraft 1.21.11
  golden, with **2,874,240 block-state assertions** over 4,491 watched cells.
- This is a block-state comparison against retained reference data, not a new
  vanilla capture or a full comparison of every inventory/entity state.
- Exact-source checker and result: local `validation/hipster-import/`.

The importer preserves the original bytes. Reimports retain title/author
metadata and different contents with the same filename receive distinct paths.
It rejects archive destinations inside Git, including nonexistent subfolders.

## Individual plot files

The collection selects from the already extracted 803 door candidates; it does
not repartition the plot, repair missing inventory or add controls. Classification
uses the retained native-v3 assessments and vanilla/functional overlay:

| Export tier | Files | Evidence |
|---|---:|---|
| Verified | 1 | Complete aperture cycle plus matching activated and idle vanilla captures |
| Repeatable candidate | 138 | A/B/A/B/A exact observed endpoint states, without moving blocks at those endpoints |
| Responsive candidate | 376 | Piston response, actual control use and deterministic restore/replay |

The two candidate tiers are **not verified complete doors**. The verified file
is `part0556`, the lamp 2×2 door, with its typed adapter and test suite. Known
validation failures (120), source-labelled broken builds (3), and builds lacking
a qualifying piston/control response (165) are excluded. Cutoff and inventory
risks remain attached to included files.

All **515** exported files are byte-identical to their isolated sources. Each
has a `.run.json` containing the saved/local control coordinates, original data
version, origin, setup and recorded inputs. The two ZIP downloads separate the
verified door from the 514 motion candidates.

## Fresh execution and interface checks

Every exported protocol ran through the installed native Python package 0.12.3
on **harrison-work**: **515/515 passed**, four control uses each, 400 ticks each
(80 setup + 320 trial), **206,000 ticks total**. This check verifies source
identity, origin/version, tick advancement and exact recorded control state
transitions. It does not repeat the native-v3 trace fingerprint comparison or
establish a new aperture/vanilla result. Python-check elapsed times are not
performance benchmarks. No Rust compilation was needed.

Local evidence:

- Export: `collections/plot-doors/9eafeb8f4a07425c/`.
- Manifest SHA-256: `1561356f0c836b00c3be9c126a82349f69d14bf85a432ecbcb6f4ed9c52dfcc9`.
- `validation/door-export-check/`: all 515 fresh results, the exact export
  manifest, runner snapshots and hashes. Results pin package version 0.12.3.
- `validation/atlas-door-collection/`: 22/22 API checks (121 requests), browser
  UI checks, real WASM load/step checks, and desktop/mobile screenshots.

Eight evidence-classification tests cover replay failures, missing control use,
stationary versus alternating endpoints, source-labelled negatives and the
strict verified threshold. API checks verify ZIP CRCs, every exported source
hash, protocol associations, original downloads and excluded-entry labels.
The browser checks search for the exact supplied hipster, display its actual
thumbnail and evidence, exercise collection downloads, and advance its live
simulation. They do not claim browser vanilla parity.

See [Atlas instructions](../../../apps/schematic-atlas/README.md) for importing,
reindexing, exporting and running an individual protocol. Recreating native and
vanilla evidence requires the separately retained archive; a Git clone alone
does not contain the user's source files or reference captures.
