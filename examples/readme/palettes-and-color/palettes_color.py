"""Executable Python source for docs/features/palettes-and-color.md."""

import json
import os
from pathlib import Path

from nucleation import Palette, PaletteBuilder, Schematic


# --8<-- [start:choose]
builder = PaletteBuilder.create()
builder.full_blocks_only()
builder.exclude_transparent()
builder.exclude_falling()
builder.survival_only()
builder.color_near(42, 132, 92, 0.20)
safe_green = builder.build()

concrete = Palette.concrete()
gray = Palette.from_block_ids(
    '["minecraft:black_concrete","minecraft:gray_concrete",'
    '"minecraft:light_gray_concrete","minecraft:white_concrete"]'
)
assert safe_green.len() > 0
assert concrete.len() == 16
assert gray.len() == 4
# --8<-- [end:choose]


# --8<-- [start:build]
atlas = Schematic.create("color_atlas")

# A distinct 12-block ramp. No block id may repeat.
ramp = json.loads(concrete.ramp_ids_json(20, 50, 150, 250, 200, 30, 12))
for x in range(32):
    atlas.set_block(x, 15, 0, ramp[x * len(ramp) // 32])

# A 32-sample lookup table. Repeated ids are expected on a 16-color palette.
gradient = json.loads(concrete.gradient_ids_json(20, 50, 150, 250, 200, 30, 32))
for x, block in enumerate(gradient):
    atlas.set_block(x, 13, 0, block)

# Ordered dithering extends a four-block grayscale palette across 32 values.
for y in range(12):
    for x in range(32):
        value = x * 255 // 31
        block = gray.closest_block_dithered(value, value, value, x, y, 0)
        atlas.set_block(x, y, 0, block)
# --8<-- [end:build]


# --8<-- [start:inspect]
size = atlas.tight_dimensions()
assert atlas.block_count() == 448
assert (size.x, size.y, size.z) == (32, 16, 1)
assert len(ramp) == len(set(ramp)) == 12
assert len(gradient) == 32 and len(set(gradient)) < len(gradient)
# --8<-- [end:inspect]

output = Path(os.environ.get("PALETTES_COLOR_OUT", "color-atlas.schem"))
output.parent.mkdir(parents=True, exist_ok=True)
atlas.save_to_file(str(output))
print(f"Palettes and color Python example: OK ({output})")
