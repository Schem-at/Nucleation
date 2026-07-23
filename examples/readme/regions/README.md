# Regions, transforms, and stamping README media

This generator produces review assets for the root README's Regions section from the shipped animation and schematic APIs.

```bash
cargo run --release --example readme_regions --features rendering -- \
  /path/to/resource-pack.zip \
  render_work/readme-regions

python3 -m pip install Pillow
python3 examples/readme/regions/compose.py \
  render_work/readme-regions \
  /path/to/resource-pack.zip
```

The first command renders lossless PNG frames, operation receipts, and downloadable schematics. The compositor derives phase timing from `receipts.json`, renders labels with the pack's Minecraft bitmap font, and encodes looping 20 FPS GIFs.

It renders:

- `hero.gif` — independently assembled gatehouse storage regions, a named `west_wing` rotation, a full-schematic rotation, and a translated authoritative merged clone.
- `stamping.gif` — one `stall` storage region stamped three times, including a rotated/flipped variant and an excluded gold anchor that preserves the lapis destination.
- `axes.gif` — separate `rotate_x`, `rotate_y`, and `rotate_z` storage regions with valid directional blocks for every resulting state.
- `overlap.gif` — labeled `zeta`, `alpha`, `Main`, and authoritative merged panels. The `Main > alpha > zeta` name priority includes explicit air masking lower-priority content.

## Terminology used in the labels

- **Named storage region**: an independently stored subregion inside a `UniversalSchematic`, such as `west_wing` or `stall`.
- **Full schematic**: the default region plus every named storage region.
- **Source/destination bounds**: receipt-derived bounding-box gizmos. They are spatial boxes, not additional stored regions.
- **Merged model**: the authoritative visible composite after deterministic region-name priority and explicit-air masking.

Storage regions are allocated explicitly with `add_region(Region::new(...))`. Allocated bounds define rigid-transform pivots, while tight content bounds define stamp coverage.
