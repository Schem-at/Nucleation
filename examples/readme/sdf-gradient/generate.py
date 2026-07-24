#!/usr/bin/env python3
"""Generate a typed SDF island with a concrete color gradient."""

import sys

from nucleation import (
    Brush,
    BuildingTool,
    InterpolationSpace,
    Palette,
    Schematic,
    Sdf,
)

output = sys.argv[1] if len(sys.argv) > 1 else "sdf-gradient.litematic"

field = Sdf.ellipsoid(14, 8, 14).displace(
    amplitude=3,
    frequency=0.1,
    seed=7,
    octaves=3,
)
brush = Brush.linear_gradient(
    0, -8, 0, 45, 70, 170,
    0,  8, 0, 235, 190, 70,
    InterpolationSpace.Oklab,
)
brush.set_palette(Palette.concrete().dithered())

schematic = Schematic.create("sdf-gradient")
BuildingTool.fill(schematic, field.to_shape(), brush)
schematic.save(output)

size = schematic.tight_dimensions()
print(
    f"wrote {output}: {schematic.block_count()} blocks, "
    f"{size.x}x{size.y}x{size.z}"
)
