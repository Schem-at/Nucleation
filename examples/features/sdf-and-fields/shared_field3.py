#!/usr/bin/env python3
"""Use one reusable Field for both surface displacement and material color."""

from __future__ import annotations

import sys
from pathlib import Path

from nucleation import (
    Brush,
    BuildingTool,
    Field,
    InterpolationSpace,
    Palette,
    Schematic,
)


def build() -> Schematic:
    # This scalar field is shared by two independent consumers below.
    field = Field.value_fbm3(frequency=0.11, seed=2026, octaves=5)

    # Consumer 1: perturb the sphere's surface by up to three blocks.
    surface = Field.sphere(14.0).displace(field, amplitude=3.0)

    # Consumer 2: map the same scalar values through a multi-stop color gradient.
    brush = Brush.field(
        field,
        stops=[0.0, 0.32, 0.68, 1.0],
        colors=[
            25, 20, 85,    # deep indigo
            20, 185, 210,  # cyan
            245, 185, 45,  # gold
            220, 45, 75,   # coral
        ],
        lo=-1.0,
        hi=1.0,
        space=InterpolationSpace.Oklab,
    )
    brush.set_palette(Palette.concrete().dithered())

    schematic = Schematic.create("shared-field")
    BuildingTool.fill(schematic, surface.to_shape(), brush)
    return schematic


def main() -> None:
    output = Path(sys.argv[1] if len(sys.argv) > 1 else "shared-field.litematic")
    output.parent.mkdir(parents=True, exist_ok=True)

    schematic = build()
    schematic.save(str(output))
    size = schematic.tight_dimensions()
    print(
        f"wrote {output}: {schematic.block_count()} blocks, "
        f"{size.x}x{size.y}x{size.z}"
    )


if __name__ == "__main__":
    main()
