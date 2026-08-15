"""Executable Python source for docs/features/shapes-and-brushes.md."""

import os
from pathlib import Path

from nucleation import (
    Brush,
    BuildingTool,
    InterpolationSpace,
    Palette,
    Schematic,
    Shape,
)


# --8<-- [start:build]
from nucleation import Brush, BuildingTool, InterpolationSpace, Palette, Schematic, Shape

garden = Schematic.create("orbital_garden")

# Shape chooses cells; Brush chooses the state written to those cells.
plinth = Shape.cuboid(-20, 0, -16, 20, 2, 16)
BuildingTool.fill(garden, plinth, Brush.solid("minecraft:stone_bricks"))

# A mask limits the write to stone bricks already inside the sphere.
weathering = Shape.sphere(-10, 2, 0, 8)
BuildingTool.fill_replacing(
    garden,
    weathering,
    Brush.solid("minecraft:mossy_stone_bricks"),
    '["minecraft:stone_bricks"]',
)

# A parametric torus supplies t in [0, 1] to a closed color gradient.
stops = [0.0, 0.25, 0.5, 0.75, 1.0]
colors = [255, 48, 48,  255, 190, 32,  64, 190, 255,  174, 72, 255,  255, 48, 48]
orbit = Shape.torus(0, 14, 0, 12, 3, 0, 1, 0)
rainbow = Brush.curve_gradient(stops, colors, InterpolationSpace.Oklab)
rainbow.set_palette(Palette.wool())
BuildingTool.fill(garden, orbit, rainbow)

# Boolean composition produces one hollow shell from two overlapping spheres.
shell = Shape.sphere(-4, 14, 0, 6).union_with(Shape.sphere(4, 14, 0, 6)).hollow(1)
clay = Brush.shaded(224, 130, 84, -1.0, 0.7, -0.3)
clay.set_palette(Palette.terracotta())
BuildingTool.fill(garden, shell, clay)
# --8<-- [end:build]


# --8<-- [start:inspect]
size = garden.tight_dimensions()
print(garden.block_count())
print((size.x, size.y, size.z))
print(garden.get_block_string(-20, 0, -16))
# --8<-- [end:inspect]

assert garden.block_count() == 6_627
assert (size.x, size.y, size.z) == (41, 21, 33)
assert garden.get_block_string(-20, 0, -16) == "minecraft:stone_bricks"

output = Path(os.environ.get("SHAPES_BRUSHES_OUT", "orbital-garden.schem"))
output.parent.mkdir(parents=True, exist_ok=True)
garden.save_to_file(str(output))
print(f"Shapes and brushes Python example: OK ({output})")
