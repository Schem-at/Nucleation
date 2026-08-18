"""Generate the data-driven guide's render, animation, and download."""

import os
from pathlib import Path

from PIL import Image
from nucleation import (
    AnimationEffect,
    BuildAnimation,
    Diff,
    RenderConfig,
    Renderer,
    Schematic,
)


def barrel_position(x, y, channel):
    alternate = x & 1
    z = 5 + alternate if (channel & 1) == (y & 1) else 5 * alternate
    return (
        -((channel + y) & 1),
        -2 - channel - 3 * y,
        6 * (x // 2) + z - 2,
    )


def pixels(path):
    with Image.open(path) as source:
        image = source.convert("RGB")
        return image.width, image.height, list(image.getdata())


def make_rom(width, image_pixels):
    rom = Schematic.create("image_rom")
    for index, (red, green, blue) in enumerate(image_pixels):
        x, y = index % width, index // width
        for channel, signal in enumerate((blue >> 4, green >> 4, red >> 4)):
            rom.set_block(
                *barrel_position(x, y, channel),
                f"minecraft:barrel{{signal={signal}}}",
            )
    return rom


def make_animation(width, image_pixels):
    animation = BuildAnimation.create("image_rom")
    animation.set_step_ms(210)
    animation.set_stagger_total_ms(180)

    def settle():
        effect = AnimationEffect.create(360)
        effect.add_keyframe("opacity", 0.0, 0.0, "outCubic")
        effect.add_keyframe("opacity", 0.34, 1.0, "outCubic")
        effect.add_keyframe("scale", 0.0, 0.92, "outCubic")
        effect.add_keyframe("scale", 1.0, 1.0, "outCubic")
        return effect

    # The source image is scanned one row at a time. Each row arrives as a
    # short, restrained sweep; barrels never rotate independently.
    height = len(image_pixels) // width
    for y in range(height):
        animation.with_effect(settle()).begin_group()
        for x in range(width):
            red, green, blue = image_pixels[y * width + x]
            for channel, signal in enumerate((blue >> 4, green >> 4, red >> 4)):
                animation.set_block(
                    *barrel_position(x, y, channel),
                    f"minecraft:barrel{{signal={signal}}}",
                )
        animation.end_group()

    camera = AnimationEffect.create(3_000)
    camera.add_keyframe("rotateY", 0.0, -4.0, "inOutSine")
    camera.add_keyframe("rotateY", 0.5, 4.0, "inOutSine")
    camera.add_keyframe("rotateY", 1.0, -4.0, "inOutSine")
    animation.animate_camera(camera, 0)
    return animation


root = Path(__file__).resolve().parents[3]
input_path = Path(os.environ.get(
    "DATA_DRIVEN_INPUT",
    Path(__file__).with_name("rom-input.png"),
))
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
still_out = Path(os.environ.get(
    "NUCLEATION_STILL_OUT",
    root / "docs/media/readme/data-driven-generation/image-rom.png",
))
gif_out = Path(os.environ.get(
    "NUCLEATION_OUT",
    root / "docs/media/readme/data-driven-generation/image-rom.gif",
))
schem_out = Path(os.environ.get(
    "NUCLEATION_SCHEM_OUT",
    root / "docs/downloads/readme/data-driven-generation/image-rom.schem",
))
for path in (still_out, gif_out, schem_out):
    path.parent.mkdir(parents=True, exist_ok=True)

width, height, image_pixels = pixels(input_path)
rom = make_rom(width, image_pixels)
animation = make_animation(width, image_pixels)
rom.save_to_file(str(schem_out))

animation_schem = schem_out.with_name("image-rom-animation.schem")
animation.save_to_file(str(animation_schem))
saved = Schematic.load_from_file(str(schem_out))
animated = Schematic.load_from_file(str(animation_schem))
assert Diff.compute(saved, animated, "exact").distance() == 0
animation_schem.unlink()

still = RenderConfig.create(720, 520)
still.set_isometric()
still.set_sphere_fit(True)
still.set_zoom(0.94)
still.set_background(0, 0, 0, 0)
Renderer.render_to_file(rom, pack, still, str(still_out))

motion = RenderConfig.create(520, 430)
motion.set_isometric()
motion.set_sphere_fit(True)
motion.set_zoom(0.9)
motion.set_background(0, 0, 0, 0)
frames = animation.render_gif(pack, motion, str(gif_out), 15, 700)

print(f"source {width} by {height}; saved {rom.block_count()} barrels to {schem_out}")
print(f"rendered still to {still_out}")
print(f"rendered {frames} frames to {gif_out}")
