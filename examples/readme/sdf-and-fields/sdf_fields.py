"""Executable Python source for docs/features/sdf-and-fields.md."""

import os
from pathlib import Path

from nucleation import (
    Brush,
    BuildingTool,
    Field3,
    InterpolationSpace,
    Palette,
    Schematic,
    Sdf,
)


# --8<-- [start:graph]
field = Field3.value_noise_fbm(frequency=0.13, seed=73, octaves=3)

body = Sdf.ellipsoid(11, 7, 11).offset_by_field(field, amplitude=1.7)
shaft = Sdf.capped_cylinder(3.2, 12)
equator = Sdf.torus(9.2, 1.45)
form = body.subtract(shaft).smooth_union(equator, 0.7)
# --8<-- [end:graph]


# --8<-- [start:build]
brush = Brush.field3(
    field,
    [0.0, 0.5, 1.0],
    [25, 38, 105, 42, 185, 165, 245, 185, 48],
    -1.0,
    1.0,
    InterpolationSpace.Oklab,
)
brush.set_palette(Palette.concrete().dithered())

observatory = Schematic.create("field_observatory")
BuildingTool.fill(observatory, form.to_shape(), brush)
# --8<-- [end:build]


# --8<-- [start:inspect]
size = observatory.tight_dimensions()
value_range = field.output_range()
restored = Sdf.from_json_string(form.to_json())
assert observatory.block_count() == 3_175
assert (size.x, size.y, size.z) == (22, 14, 24)
assert (value_range.min, value_range.max) == (-1.0, 1.0)
assert form.eval_at(0, 0, 0) > 0  # the subtracted shaft is empty
assert abs(restored.eval_at(5, 2, 1) - form.eval_at(5, 2, 1)) < 1e-6
# --8<-- [end:inspect]

output = Path(os.environ.get("SDF_FIELDS_OUT", "field-observatory.schem"))
output.parent.mkdir(parents=True, exist_ok=True)
observatory.save_to_file(str(output))
print(f"SDFs and fields Python example: OK ({output})")
