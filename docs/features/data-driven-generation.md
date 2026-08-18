# Data-driven generation

A generator can start from anything the host language can decode. Nucleation
takes over at `set_block`, where a coordinate and a block descriptor become a
schematic. This page follows one PNG through that boundary.

`rom-input.png` is 16 by 10. Each pixel becomes three barrels, blue, green,
red, on three Y levels; the high nibble of each channel is stored as comparator
strength 0 to 15. The 480 barrels form a memory bank two blocks thick, thirty
tall, and forty-nine deep.

<div class="bb-kineglyph bb-kineglyph--rom" data-kineglyph="data-driven-generation" data-theme="nucleation" data-autoplay="false" data-controls="false" data-readout="false" aria-busy="true">
  <img class="kg-fallback--dark" src="../../media/kineglyph/data-driven-generation.svg" alt="The 16 by 10 creeper image with pixel (6, 1) outlined, its three channel bytes reduced to signal strengths 3, 11, and 4, and a sample of the barrel layout">
  <img class="kg-fallback--light" src="../../media/kineglyph/data-driven-generation.light.svg" alt="">
</div>

The barrel sample is the top-left 8 by 4 pixels, built by Nucleation's WASM
build in the browser. Drag to orbit. The four sliders are the constants in
`barrel_position`; the elevation in the corner recolours by channel and
outlines the three barrels of pixel (6, 1). Open **barrel_position** to copy
the function with the current constants.

## The generator

=== "Python"

    ```console
    python -m pip install nucleation Pillow
    ```

    ```python
    --8<-- "examples/readme/data-driven-generation/data_driven.py:example"
    ```

=== "JavaScript"

    ```console
    npm install nucleation pngjs
    ```

    ```javascript
    --8<-- "examples/readme/data-driven-generation/data_driven.mjs:example"
    ```

=== "Rust"

    ```console
    cargo add nucleation image
    ```

    ```rust
    --8<-- "examples/readme/data-driven-generation/rust/src/main.rs:example"
    ```

The three programs read the same PNG and save cell-exact equivalent Sponge
schematics. The JavaScript binding returns bytes rather than writing a file,
because it also runs in browsers; Node writes them, a page would hand the
`Uint8Array` to a download or upload.

<figure class="bb-rom-result">
  <img src="../../media/readme/data-driven-generation/image-rom.gif" alt="The 480-barrel ROM assembling one image row at a time, three levels per row, under a slowly turning isometric camera">
  <figcaption>The saved schematic, assembling one image row per step. <a href="../../media/readme/data-driven-generation/rom-input.png">rom-input.png</a> · <a href="../../downloads/readme/data-driven-generation/image-rom.schem">image-rom.schem</a></figcaption>
</figure>

## `barrel_position`

Nothing about the arrangement is a Nucleation convention. A flat wall, a
serpentine bank, or coordinates read from another file would go through the
same `set_block` call. This one is a staggered, paired layout, and each term
of the function owns one part of it.

| Axis | Term | What it does |
| --- | --- | --- |
| Y | `-2 - channel - 3 * y` | Each image row takes three levels; blue sits above green above red. |
| X | `-((channel + y) & 1)` | Successive levels alternate between the two X columns, so the wall is a stagger, not a stack. |
| Z | `6 * (x // 2) + z - 2` | Two image columns share a six-block pitch. |
| Z | `z = 5 + (x & 1)` or `5 * (x & 1)` | Inside the pitch the pair sits at 5 and 6 when channel and row parity agree, and at 0 and 5 when they differ, so the pairing shifts by one image column from one level to the next. |

## Barrels as memory

`minecraft:barrel{signal=12}` is Nucleation descriptor shorthand for a barrel
holding enough items to give a comparator output of 12. Add `item=` only when
the filler matters.

The strength is the channel's high nibble:

```python
signal = red >> 4  # 0..15
```

Write the shift directly. In Python `red & 0xF0 >> 4` shifts first and keeps
the low nibble instead.

Every barrel here can carry a different signal, so each needs its own
descriptor and `set_block` is the right call. `set_blocks` is for many
coordinates sharing one descriptor; `prepare_block` and `place` skip NBT
parsing. For a large image, group positions by signal and call `set_blocks`
once per strength: sixteen calls instead of one per barrel, same output.

## Verified output

```bash
./tools/verify-data-driven-docs.sh
```

The verifier runs the three programs above in separate directories, diffs the
three schematics with the exact preset, reads back all 480 barrels' signal
NBT, meshes a barrel through the browser build's resource pack, and
regenerates the render and the 57-frame animation.

See [Fast schematic generation](fast-generation.md) when the input is large
enough to batch, and [Palettes and color](palettes-and-color.md) when RGB
should choose visible materials rather than stored strengths.
