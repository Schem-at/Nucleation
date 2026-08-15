"""Generate the SDFs and fields guide media and download."""

import os
from pathlib import Path

from nucleation import (
    AnimationEffect,
    Brush,
    BuildAnimation,
    BuildingTool,
    Field3,
    InterpolationSpace,
    Palette,
    RenderConfig,
    Renderer,
    Schematic,
    Sdf,
)


def make_observatory():
    field = Field3.value_noise_fbm(0.13, 73, 3)
    body = Sdf.ellipsoid(11, 7, 11).offset_by_field(field, 1.7)
    form = body.subtract(Sdf.capped_cylinder(3.2, 12)).smooth_union(
        Sdf.torus(9.2, 1.45),
        0.7,
    )
    brush = Brush.field3(
        field,
        [0.0, 0.5, 1.0],
        [25, 38, 105, 42, 185, 165, 245, 185, 48],
        -1.0,
        1.0,
        InterpolationSpace.Oklab,
    )
    brush.set_palette(Palette.concrete().dithered())
    result = Schematic.create("field_observatory")
    BuildingTool.fill(result, form.to_shape(), brush)
    return result


root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
still_out = Path(os.environ.get(
    "NUCLEATION_STILL_OUT",
    root / "docs/media/readme/sdf-and-fields/field-observatory.png",
))
gif_out = Path(os.environ.get(
    "NUCLEATION_OUT",
    root / "docs/media/readme/sdf-and-fields/field-observatory-build.gif",
))
schem_out = Path(os.environ.get(
    "NUCLEATION_SCHEM_OUT",
    root / "docs/downloads/readme/sdf-and-fields/field-observatory.schem",
))
for path in (still_out, gif_out, schem_out):
    path.parent.mkdir(parents=True, exist_ok=True)

observatory = make_observatory()
observatory.save_to_file(str(schem_out))

still = RenderConfig.create(720, 520)
still.set_isometric()
still.set_sphere_fit(True)
still.set_zoom(1.08)
still.set_background(0, 0, 0, 0)
still.set_fitted_grid(2, 1, -7.502, False, 0.42, 0.52, 0.60, 0.20)
Renderer.render_to_file(observatory, pack, still, str(still_out))

lo = observatory.tight_bounds_min()
hi = observatory.tight_bounds_max()
animation = BuildAnimation.create("field_observatory")
animation.set_default_effect(AnimationEffect.drop_and_pop(420, 2.0))
animation.set_stagger_total_ms(2_100)
for y in range(lo.y, hi.y + 1):
    row = []
    for x in range(lo.x, hi.x + 1):
        for z in range(lo.z, hi.z + 1):
            block = observatory.get_block_string(x, y, z)
            if block != "minecraft:air":
                row.append((x, z, block))
    if row:
        animation.begin_group()
        for x, z, block in row:
            animation.set_block(x, y, z, block)
        animation.end_group()

animation.animate_camera(AnimationEffect.turntable(3_400), 0)
motion = RenderConfig.create(500, 420)
motion.set_isometric()
motion.set_sphere_fit(True)
motion.set_zoom(1.06)
motion.set_background(0, 0, 0, 0)
motion.set_fitted_grid(2, 1, -7.502, False, 0.42, 0.52, 0.60, 0.18)
frames = animation.render_gif(pack, motion, str(gif_out), 15, 600)

print(f"saved {schem_out}")
print(f"rendered still to {still_out}")
print(f"rendered {frames} frames to {gif_out}")
