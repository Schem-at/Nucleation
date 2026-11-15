# Textured archive previews

Circuit Atlas uses native 768×576 orthographic PNG renders of the archived saved
state. The images come from Nucleation's resource-pack mesher and renderer on
**harrison-work**, using real Minecraft block models and textures. No simulation
ticks, placement updates or inventory repairs are run to obtain an image.
Imports use the production format converters; original files remain unchanged.

The completed cache contains **2,297 verified PNGs**: all **848 original Java
schematics**, **883 first-pass plot components**, and **566 floor-aware exports**.
The ten remaining first-pass components contain only invisible barrier blocks;
the UI reports no visible geometry. Fourteen Bedrock files were excluded from
the Java batch. All **803 geometric door candidates** have actual rendered images.
See the [run summary](validation/atlas-thumbnails.json) and compressed
[per-image and batch records](validation/atlas-render-records.json.gz).

The local cache is `ARCHIVE/rendered-thumbnails/`. Its manifest records original
BLAKE3/SHA-256 identity, PNG hashes, dimensions, mesh counts, resource-pack hashes,
timings and renderer warnings. Reconciliation verifies every input hash and PNG
hash, decodes every PNG, and checks its dimensions before publishing the cache.
The API returns an actual PNG or an explicit missing/error response; the former
abstract symbols and top-down projections are no longer used as thumbnails.
Saved-cell projections remain separately available at `/api/projection/<sha256>`.

The visual asset subsets were extracted from the user's existing PrismLauncher
Minecraft client JARs. DataVersions through 4671 use 1.21.11 assets; newer saves
use the available 26.3 assets. This selection concerns appearance, not a claim
about either version's simulation behavior. Client JARs, asset ZIPs, schematic
binaries, compiled engines and PNG caches are retained locally outside Git.

## Reproduce

Build Rust only on harrison-work:

```sh
CARGO_BUILD_JOBS=4 cargo build --locked --release \
  --features rendering --example render_archive_thumbnails
```

Prepare jobs on the archive machine (the two root arguments name paths on the
render host, where original files are copied with their archive-relative paths):

```sh
python3 tools/render_tick_archive.py --archive "$ARCHIVE" jobs \
  --include-plot --input-root /path/to/render-host/originals \
  --output-root /path/to/render-host/pngs --jobs-dir /path/to/jobs
```

This writes one JSONL and one rsync `--files-from` list per resource-pack version.
`--source world-record-parts` selects the extracted plot collection; `--resume`
omits verified cached successes. Use `--extension .schematic` to regenerate legacy
imports after a conversion fix. The native renderer appends progress records:

```sh
target/release/examples/render_archive_thumbnails \
  minecraft-1.21.11-models.zip jobs-1.21.11.jsonl results.jsonl
```

Copy the PNGs to `ARCHIVE/rendered-thumbnails/pngs/` and retain the result JSONL.
With `blake3` and `Pillow` installed in the archive's tooling venv:

```sh
python tools/render_tick_archive.py --archive "$ARCHIVE" reconcile results.jsonl
python tools/render_tick_archive.py --archive "$ARCHIVE" report \
  --output-dir docs/performance/schematic-candidates/validation
python3 tools/serve_tick_archive.py --archive "$ARCHIVE" --port 8767
```

`tools/render_tick_archive.py pack` can rebuild an asset subset from a supplied
client JAR. It checks the JAR's ZIP CRC and records both SHA-256 hashes. A new
destination is required to avoid overwriting retained pack provenance.

## Limits and evidence

- A rendered mechanism is not a working-door or vanilla-parity result. The
  plot's isolated pieces have their own [extraction report](world-record-parts.md).
- Entity rendering uses the native mesher's supported models at floored block
  positions. Chunked meshing can skip entities that collide with block/entity
  cells. Every affected result records these limits.
- Large inputs use chunked meshing. Import/meshing limits are 32 million cells
  and 2 million non-air blocks; failures produce no fabricated thumbnail.
- Bedrock files and unconverted text layouts remain explicitly separate from
  the Java render batch. Source versions and existing validation labels remain.
- This work uncovered valid Java modified UTF-8 sign text rejected by the bounded
  NBT reader, and a legacy file using Int dimensions instead of Short. The fixes
  retain parsing limits and have independent regression tests. Legacy piston
  heads and the base's extended state now use exact Java-derived mappings.
- Render timings include import, meshing and graphics work. They are not circuit
  simulation benchmarks and must not be used to infer TPS or optimization gains.
