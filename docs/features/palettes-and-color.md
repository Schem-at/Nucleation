# Palettes and color

## Paintings, in blocks


Everything above composes, pointed at art: flat-texture palettes built by
color-logic filters, chroma-boosted matching (so muted pigments land on
saturated blocks, not gray clays), and per-voxel ordered dithering. Van Gogh's
Starry Night, 128 blocks wide:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/painting-starry-night.png" width="760" alt="Van Gogh's Starry Night as block pixel art, 128 blocks wide">

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/painting-gallery.png" width="760" alt="Sunflowers, The Great Wave off Kanagawa, and Girl with a Pearl Earring as block pixel art">

```python
palette = flat_art_palette().dithered()          # PaletteBuilder + map-art excludes
r, g, b = boost(*pixel, sat=1.35)                # chroma exaggeration pre-match
s.set_block(x, 0, y, palette.closest_block_dithered(r, g, b, x, 0, y))
```

The full recipe, including the flat-palette filter chain, is `scene_paintings`
in [`tools/readme-media/generate.py`](../../tools/readme-media/generate.py).

And a flat image is also a heightmap: read each pixel's brightness as a column
height and paint it with its own color, and the same Starry Night lifts off the
canvas into rolling, luminous terrain, the moon and stars its highest peaks.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/heightmap.png" width="760" alt="Van Gogh's Starry Night as 3D relief terrain, pixel brightness lifted into hills with the moon and stars as peaks">
</div>

---

## Reference

A `Palette` is a set of colored blocks used to translate colors into block
choices. Everything here works identically in every binding (snake_case in
Python/Rust, camelCase in JS/Kotlin/PHP).

## Presets and custom palettes

```python
Palette.wool()          # the 16 wools
Palette.concrete()      # the 16 concretes
Palette.terracotta()    # unglazed terracottas (17)
Palette.wood()          # the planks family (13)
Palette.grayscale()     # opaque full cubes with near-neutral measured color (81)
Palette.decorative()    # the broad decorative set (~950 blocks)
Palette.structural()    # bricks, stones, and building blocks (~316)
Palette.solid()         # solid blocks (no transparency/gravity/tile entities)
Palette.all()           # every colored block except technical ones
Palette.from_block_ids('["minecraft:stone", "minecraft:oak_planks"]')

# List what any palette holds, or count it:
Palette.wood().block_ids_json()   # ["minecraft:birch_planks", "minecraft:spruce_planks", ...]
Palette.concrete().len()          # 16
```

`PaletteBuilder` composes filters over the block database — vanilla tags,
definition kinds, and classification flags:

```python
b = PaletteBuilder.create()
b.tag("wool")                  # require a vanilla tag (AND across calls)
b.kind("stair")                # require a definition kind (OR across calls)
b.exclude_tag("mineable/axe")
b.full_blocks_only()           # model-derived full-cube geometry
b.exclude_transparent()
b.survival_only()
b.exclude_keyword("infested")
palette = b.build()
```

## Two ways to get a ramp

**`ramp_ids(start, end, steps)`** picks exactly `steps` **distinct** blocks
forming the smoothest ramp the palette can make between two colors. Targets
are spaced evenly along the Oklab line; blocks are assigned by a minimum-cost
monotonic matching over their projections onto that line, so off-hue blocks
are penalized and the result never repeats a block. It errors when the
palette has fewer than `steps` blocks or the endpoints are equal.

```python
Palette.grayscale().ramp_ids_json(255, 255, 255, 0, 0, 0, 24)
# 24 distinct blocks, white_wool ... iron_block ... deepslate_tiles ... black_concrete
```

Known limitation: matching runs on each block's *average* texture color, so a
block whose average is neutral but whose texture is patterned (birch wood's
bark eyes, ores' flecks) can be selected even though it reads as noise —
exclude such blocks by keyword if they bother you.

**`gradient_ids(start, end, steps)`** samples the color line per step and
snaps each sample to the *closest* block — repeats allowed. This is the right
tool for value→block lookups where you index the list:

```python
ramp = json.loads(Palette.wool().gradient_ids_json(255, 80, 40, 60, 40, 180, 8))
s.set_block(px, 0, pz, ramp[escape_iterations(px, pz)])
```

Index a ramp by any value — escape time, height, temperature:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/mandelbrot.png" width="420" alt="128x128 block mandelbrot from a concrete ramp">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/build-timelapse.gif" width="420" alt="The mandelbrot materializing in escape-time order">
</div>

`sorted_by_lightness()` reorders any palette dark→light (Oklab L) for direct
indexing, and `closest_block(r, g, b)` answers single lookups.

## Palettes inside other systems

- Every color/gradient **brush** takes `set_palette(palette)` — snapping
  happens per placed block.
- **SDF material rules** accept palettes in `gradient` fill rules
  ([SDF guide](sdf-and-fields.md)).
- The **scripting engines** expose `palette_gradient_ids`,
  `palette_block_ids`, and `palette_closest_block`
  ([scripting guide](scripting.md)).

Verified runnable examples: [`docs/readme-snippets/`](../readme-snippets/).
