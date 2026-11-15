# Circuit Atlas

A local interface to the retained schematic archive. The Atlas frontend uses
plain HTML, CSS and JavaScript modules: no frontend build and no external CDN.
It groups byte-identical files while preserving their source paths, supports
search and saved favorites, and shows static properties, recorded captures and
an explicitly launched live Sim Lab session as separate kinds of evidence.

From the repository root:

```sh
python3 tools/serve_tick_archive.py \
  --archive /Users/harrison/RustroverProjects/Nucleation-schematic-archive \
  --port 8767
```

Open **http://localhost:8767/**. The server binds to loopback. Keep the archive
directory available; the browser loads its library, originals and evidence
through the local API. The server reads the library when it starts, so restart
it after rebuilding the archive index.

## Live simulation assets

Build the engine on harrison-work with `cargo build --locked --release
--target wasm32-unknown-unknown --lib --features bridge,mc-tick,meshing`.
Copy that binary to the Mac, then assemble the generated JavaScript glue and a
retained mesher resource pack without compiling Rust locally:

```sh
python3 tools/prepare_tick_atlas.py --wasm /path/to/nucleation.wasm \
  --pack /path/to/mesher-pack.zip --engine-commit ENGINE_COMMIT --compiler 1.98.0
```

Static browsing works without a live engine. Live simulation also needs a built
Sim Lab app, the compiled engine in `apps/sim-lab-wasm/public/engine/`, and the
retained rendering assets used by Sim Lab. Once those assets have been prepared,
build only the JavaScript app with:

```sh
npm --prefix apps/sim-lab-wasm ci --ignore-scripts
npm --prefix apps/sim-lab-wasm run build
```

These commands compile TypeScript and bundle the browser app. **Rust/WASM
compilation belongs on `harrison-work`**, per the user's environment choice.
Do not run Sim Lab's `npm run engine` or `tools/package-npm.sh` on the Mac;
they invoke the Rust build. Copy the reviewed laptop-produced engine assets
into place before building Sim Lab, and keep their build metadata with them.

The build uses relative asset URLs, so it works under `/sim-lab/` and at a
standalone preview root. The Atlas opens `/sim-lab/?archive=<sha256>&embed=atlas`
when **Live simulation** or **Start live simulation** is clicked. The server advertises whether a given file can launch and explains
format or size limits. The whole World Record Plot exceeds the current live
world limit. The **Plot doors** collection contains extracted physical door
candidates; **Plot fragments** retains the other isolated components. Their
rendered geometry, saved inventories, controls, source-coordinate bounds and
selection-edge contacts can be inspected independently. Physical isolation is
not a passing functionality or vanilla-parity check. The original plot and its
annotated map remain available as context.

## Interaction and evidence

- Use **/** to focus search. Search covers names, mechanism tags, source names,
  paths and recorded native data versions.
- Cards render 60 at a time. Switch to the table for a dense comparison.
- Stars save content hashes in this browser's local storage. No account or
  external service is involved.
- Details have keyboard-accessible tabs, an original-file download, source
  attribution and native saved-state properties. **Escape** closes details.
- Live mode expands the canvas. **Show details** restores its sidebar;
  **Open in new tab** opens the full workspace. The embedded toolbar offers
  run, step, reset and camera framing, with recording and mesh controls under
  **more**. Switching away from live mode pauses the simulation.
- Thumbnails are native 768×576 textured isometric PNG renders of the saved
  schematic using Minecraft models and textures. No simulation ticks run to
  generate a thumbnail. Missing or failed renders show an explicit text state;
  no placeholder geometry is substituted. Entity rendering has the native
  mesher's model and positioning limitations, recorded with the render result.
- **Plot doors** selects `world-record-parts` and `world-record-floor-parts` entries classified as
  `door_candidate`, excluding components flagged `infrastructure_review`.
  **Plot fragments** keeps the remaining extracted content, including shared
  floors that may connect several mechanisms.
  Review tags expose missing controls, inventory omissions and possible cutoffs;
  appearance alone does not establish a working mechanism.
- **Captured run** replays a retained capture with `replay.js`; it does not run
  the simulation engine. **Live simulation** displays actual iframe-reported
  tick, throughput, entity and engine state. Messages must come from the exact
  iframe and the same origin.
- Unknown evidence is **Unvalidated**. Functional checks, vanilla parity,
  unsupported imports and source-labeled negatives are distinct statuses.
  **Simulation checked** means a bounded idle/control/replay assessment, with
  functionality still unproven. It is deliberately separate from passing checks.
  Opening a file is not a new functionality or parity result.
- Admitted builds can carry an **Executable build adapter**: typed inputs and
  outputs, tick budgets, source-pinned test vectors, native runner code and a
  recorded result. Contracts and suites use the same real-circuit operations
  for tests and benchmarks; the web server only serves their files.

The original user-provided plot stays in the local archive. This app and its
tracked metadata do not publish that binary.

## Local originals and door collections

The Atlas code is version controlled; the downloaded archive, extracted
schematics, thumbnails and downloadable ZIPs stay in a separate directory
outside Git. Existing small regression fixtures in the repository are retained
for CI. Back up the local archive separately from the code repository.

To add a supplied file without altering the original:

```sh
python3 tools/import_tick_local_schematic.py /path/to/build.litematic \
  --archive /path/to/Nucleation-schematic-archive \
  --title 'Build title' --authors 'Author names'
python3 tools/index_tick_archive.py --archive /path/to/Nucleation-schematic-archive
```

Restart the Atlas server afterwards. The import appears under **Your schematics**
and is searchable by title and filename. Importing alone does not establish
functionality or generate a thumbnail: render and validation results remain
separate, source-hash-pinned records. Native rendering and simulation checks run
on harrison-work.

To collect the existing plot extractions using their retained native and vanilla
evidence:

```sh
python3 tools/export_tick_plot_doors.py --archive /path/to/Nucleation-schematic-archive
```

The exporter requires complete retained assessments under
`validation/plot-native-v3` and the validation overlay. It checks source hashes,
retained run configuration and replay results before publishing a versioned
directory under `collections/plot-doors/`. Saved coordinates, inventories,
entities and source data versions remain byte-identical to the extractions.
No missing items, controls or cropped mechanisms are repaired.

- **Verified plot doors** requires an independently checked opening/closing
  cycle and matching activated and idle vanilla tick-end captures. Its download
  includes the typed adapter, test vectors and recorded evidence.
- **Interactive candidates** contains piston-responsive builds with successful
  controls and deterministic replay. `repeatable` entries also alternate between
  two exact observed endpoint states; `responsive` entries do not meet that extra
  condition. Neither tier establishes a complete door aperture. Known validation
  failures and source-labelled broken builds are excluded; selection-edge risks
  remain visible.

Each collection has a ZIP download, and each included entry has a `.run.json`
describing its tested control, coordinate origin and timed inputs. The bundle's
`run_door.py` replays those inputs through the installed Python package:

```sh
python -m pip install nucleation==0.12.3
python run_door.py verified/part0556.run.json
```

This runner checks exact source bytes, origin/version, tick advancement and
recorded control transitions. Its elapsed time includes Python checks and is
not a benchmark. Completion does not independently prove an aperture or vanilla
parity; those claims come from the attached verified adapter/oracle evidence.

## Files and API

`index.html`, `app.js` and `style.css` implement the interface. `replay.js`
provides `mountReplay(container, capture)` and returns a cleanup callback.
`tools/serve_tick_archive.py` owns the API and routes:

| Route | Purpose |
|---|---|
| `/api/library` | Summary and compact archive rows |
| `/api/entry/<sha256>` | Full provenance rows, inspection, captures and capabilities |
| `/api/file/<sha256>` | Original archived file |
| `/api/thumbnail/<sha256>` | Native textured PNG, or 404 when unavailable |
| `/api/capture/<id>` | Retained capture data |
| `/api/adapter-file/<id>/<file>` | Allowlisted adapter contract, suite, code, instructions or result |
| `/api/door-bundle/verified-doors` | Local verified extractions, adapters and protocols ZIP |
| `/api/door-bundle/interactive-door-candidates` | Local motion candidates and protocols ZIP |
| `/api/door-protocol/<sha256>` | Tested control protocol for an exported plot entry |
| `/api/plot-map` | Annotated World Record Plot survey |
| `/api/plot` | Plot curation metadata |
| `/sim-lab/` | Built live simulation application |

Selections use `#entry=<sha256>` so a detail view can be bookmarked. The static
frontend never invokes a Rust compiler or executes downloaded source files.
Collections can also be linked directly, for example `#collection=plot-doors`.
Floor-separated entries expose their parent, original platform cells, overlap
and movement-review notes alongside the first-pass extraction properties.

`verify-ui.mjs` checks typed ports, evidence labels, thumbnails and mobile sizing
in a separate headless browser. It uses Sim Lab's Playwright development
dependency; set `CHROME_PATH` to an installed Chromium/Chrome executable if the
Playwright browser bundle is not installed. It does not operate personal tabs.

`verify-live.mjs` checks the served JS/CSS URLs and MIME types, responsive live
workspace, reset/step/pause controls and parent telemetry using the real WASM
engine. `../sim-lab-wasm/verify-coordinates.mjs` checks saved-origin alignment,
mesh vertices, picking, piston updates and differential simulation on the
reported plot entries and generated translated multi-region builds.
