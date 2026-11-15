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
npm --prefix apps/sim-lab-wasm run build -- --base=/sim-lab/
```

These commands compile TypeScript and bundle the browser app. **Rust/WASM
compilation belongs on `harrison-work`**, per the user's environment choice.
Do not run Sim Lab's `npm run engine` or `tools/package-npm.sh` on the Mac;
they invoke the Rust build. Copy the reviewed laptop-produced engine assets
into place before building Sim Lab, and keep their build metadata with them.

The Atlas opens `/sim-lab/?archive=<sha256>` only after **Open live simulation**
is clicked. The server advertises whether a given file can launch and explains
format or size limits. The whole World Record Plot exceeds the current live
world limit; its annotated map and candidate bounds remain available. There is
no automatic crop or simulation of that plot.

## Interaction and evidence

- Use **/** to focus search. Search covers names, mechanism tags, source names,
  paths and recorded native data versions.
- Cards render 60 at a time. Switch to the table for a dense comparison.
- Stars save content hashes in this browser's local storage. No account or
  external service is involved.
- Details have keyboard-accessible tabs, an original-file download, source
  attribution and native saved-state properties. **Escape** closes details.
- Thumbnails are the server's saved-block top projections or explicit metadata
  previews. The fallback mechanism symbols are decorative, not screenshots.
- **Captured run** replays a retained capture with `replay.js`; it does not run
  the simulation engine. **Live simulation** displays actual iframe-reported
  tick, throughput, entity and engine state. Messages must come from the exact
  iframe and the same origin.
- Unknown evidence is **Unvalidated**. Functional checks, vanilla parity,
  unsupported imports and source-labeled negatives are distinct statuses.
  Opening a file is not a new functionality or parity result.

The original user-provided plot stays in the local archive. This app and its
tracked metadata do not publish that binary.

## Files and API

`index.html`, `app.js` and `style.css` implement the interface. `replay.js`
provides `mountReplay(container, capture)` and returns a cleanup callback.
`tools/serve_tick_archive.py` owns the API and routes:

| Route | Purpose |
|---|---|
| `/api/library` | Summary and compact archive rows |
| `/api/entry/<sha256>` | Full provenance rows, inspection, captures and capabilities |
| `/api/file/<sha256>` | Original archived file |
| `/api/thumbnail/<sha256>` | Bounded saved-file projection or metadata SVG |
| `/api/capture/<id>` | Retained capture data |
| `/api/plot-map` | Annotated World Record Plot survey |
| `/api/plot` | Plot curation metadata |
| `/sim-lab/` | Built live simulation application |

Selections use `#entry=<sha256>` so a detail view can be bookmarked. The static
frontend never invokes a Rust compiler or executes downloaded source files.
