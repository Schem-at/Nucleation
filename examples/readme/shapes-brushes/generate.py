"""Generate media and download for the Shapes and brushes guide."""

import os
from pathlib import Path

from nucleation import (
    AnimationEffect,
    Brush,
    BuildAnimation,
    BuildingTool,
    InterpolationSpace,
    Palette,
    RenderConfig,
    Renderer,
    Schematic,
    Shape,
)


def make_orbit():
    stops = [0.0, 0.25, 0.5, 0.75, 1.0]
    colors = [255, 48, 48, 255, 190, 32, 64, 190, 255, 174, 72, 255, 255, 48, 48]
    orbit = Shape.torus(0, 14, 0, 12, 3, 0, 1, 0)
    rainbow = Brush.curve_gradient(stops, colors, InterpolationSpace.Oklab)
    rainbow.set_palette(Palette.wool())
    return orbit, rainbow


def make_garden():
    garden = Schematic.create("orbital_garden")
    BuildingTool.fill(
        garden,
        Shape.cuboid(-20, 0, -16, 20, 2, 16),
        Brush.solid("minecraft:stone_bricks"),
    )
    BuildingTool.fill_replacing(
        garden,
        Shape.sphere(-10, 2, 0, 8),
        Brush.solid("minecraft:mossy_stone_bricks"),
        '["minecraft:stone_bricks"]',
    )
    orbit, rainbow = make_orbit()
    BuildingTool.fill(garden, orbit, rainbow)
    shell = Shape.sphere(-4, 14, 0, 6).union_with(Shape.sphere(4, 14, 0, 6)).hollow(1)
    clay = Brush.shaded(224, 130, 84, -1.0, 0.7, -0.3)
    clay.set_palette(Palette.terracotta())
    BuildingTool.fill(garden, shell, clay)
    return garden


root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(
    os.environ.get(
        "NUCLEATION_OUT",
        root / "docs/media/readme/shapes-brushes/torus-sweep.gif",
    )
)
still_out = Path(
    os.environ.get(
        "NUCLEATION_STILL_OUT",
        root / "docs/media/readme/shapes-brushes/orbital-garden.png",
    )
)
schem_out = Path(
    os.environ.get(
        "NUCLEATION_SCHEM_OUT",
        root / "docs/downloads/readme/shapes-brushes/orbital-garden.schem",
    )
)
for path in (gif_out, still_out, schem_out):
    path.parent.mkdir(parents=True, exist_ok=True)

garden = make_garden()
garden.save_to_file(str(schem_out))

still = RenderConfig.create(700, 500)
still.set_isometric()
still.set_sphere_fit(True)
still.set_background(0, 0, 0, 0)
still.set_fitted_grid(2, 1, -0.502, False, 0.42, 0.52, 0.60, 0.22)
Renderer.render_to_file(garden, pack, still, str(still_out))

orbit, rainbow = make_orbit()
animation = BuildAnimation.create("parametric_torus")
animation.set_default_effect(AnimationEffect.drop_and_pop(460, 3.5))
animation.set_stagger_total_ms(1_800)
groups = animation.fill_along_parameter(orbit, rainbow, 24)
assert groups == 24

camera = AnimationEffect.create(3_000)
camera.add_keyframe("rotateY", 0.0, -12, "inOutSine")
camera.add_keyframe("rotateY", 0.5, 12, "inOutSine")
camera.add_keyframe("rotateY", 1.0, -12, "inOutSine")
animation.animate_camera(camera, 0)

motion = RenderConfig.create(460, 380)
motion.set_isometric()
motion.set_sphere_fit(True)
motion.set_background(0, 0, 0, 0)
motion.set_fitted_grid(2, 1, 10.5, False, 0.42, 0.52, 0.60, 0.18)
frames = animation.render_gif(pack, motion, str(gif_out), 15, 650)

print(f"saved {schem_out}")
print(f"rendered still to {still_out}")
print(f"rendered {frames} frames to {gif_out}")
