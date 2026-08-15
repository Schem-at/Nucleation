"""Generate the Palettes and color guide media and download."""

import json
import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, Palette, RenderConfig, Renderer, Schematic


def palette_data():
    concrete = Palette.concrete()
    gray = Palette.from_block_ids(
        '["minecraft:black_concrete","minecraft:gray_concrete",'
        '"minecraft:light_gray_concrete","minecraft:white_concrete"]'
    )
    ramp = json.loads(concrete.ramp_ids_json(20, 50, 150, 250, 200, 30, 12))
    gradient = json.loads(concrete.gradient_ids_json(20, 50, 150, 250, 200, 30, 32))
    return gray, ramp, gradient


def make_atlas():
    gray, ramp, gradient = palette_data()
    atlas = Schematic.create("color_atlas")
    for x in range(32):
        atlas.set_block(x, 15, 0, ramp[x * len(ramp) // 32])
    for x, block in enumerate(gradient):
        atlas.set_block(x, 13, 0, block)
    for y in range(12):
        for x in range(32):
            value = x * 255 // 31
            atlas.set_block(
                x,
                y,
                0,
                gray.closest_block_dithered(value, value, value, x, y, 0),
            )
    return atlas


root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
still_out = Path(os.environ.get(
    "NUCLEATION_STILL_OUT",
    root / "docs/media/readme/palettes-and-color/color-atlas.png",
))
gif_out = Path(os.environ.get(
    "NUCLEATION_OUT",
    root / "docs/media/readme/palettes-and-color/color-atlas-build.gif",
))
schem_out = Path(os.environ.get(
    "NUCLEATION_SCHEM_OUT",
    root / "docs/downloads/readme/palettes-and-color/color-atlas.schem",
))
for path in (still_out, gif_out, schem_out):
    path.parent.mkdir(parents=True, exist_ok=True)

atlas = make_atlas()
atlas.save_to_file(str(schem_out))

still = RenderConfig.create(720, 480)
still.set_isometric()
still.set_yaw(0)
still.set_pitch(0)
still.set_zoom(1.15)
still.set_background(0, 0, 0, 0)
Renderer.render_to_file(atlas, pack, still, str(still_out))

gray, ramp, gradient = palette_data()
animation = BuildAnimation.create("color_atlas")
animation.set_default_effect(AnimationEffect.drop_and_pop(360, 1.8))
animation.set_stagger_total_ms(2_100)
for y in range(12):
    animation.begin_group()
    for x in range(32):
        value = x * 255 // 31
        animation.set_block(
            x,
            y,
            0,
            gray.closest_block_dithered(value, value, value, x, y, 0),
        )
    animation.end_group()
animation.begin_group()
for x, block in enumerate(gradient):
    animation.set_block(x, 13, 0, block)
animation.end_group()
animation.begin_group()
for x in range(32):
    animation.set_block(x, 15, 0, ramp[x * len(ramp) // 32])
animation.end_group()

camera = AnimationEffect.create(3_200)
camera.add_keyframe("rotateY", 0.0, -5, "inOutSine")
camera.add_keyframe("rotateY", 0.5, 5, "inOutSine")
camera.add_keyframe("rotateY", 1.0, -5, "inOutSine")
animation.animate_camera(camera, 0)

motion = RenderConfig.create(500, 360)
motion.set_isometric()
motion.set_yaw(0)
motion.set_pitch(0)
motion.set_zoom(1.08)
motion.set_background(0, 0, 0, 0)
frames = animation.render_gif(pack, motion, str(gif_out), 15, 600)

print(f"saved {schem_out}")
print(f"rendered still to {still_out}")
print(f"rendered {frames} frames to {gif_out}")
