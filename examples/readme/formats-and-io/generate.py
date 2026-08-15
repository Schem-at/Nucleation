"""Generate the Formats and I/O guide media and downloads."""

from base64 import b64decode
import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig, Renderer, Schematic


def make_fixture():
    fixture = Schematic.create("round_trip")
    fixture.fill_cuboid(0, 0, 0, 3, 0, 3, "minecraft:stone_bricks")
    fixture.set_block(1, 1, 1, "minecraft:oak_stairs[facing=east,half=bottom]")
    fixture.set_block(2, 1, 1, "minecraft:lever[face=floor,facing=east,powered=false]")
    fixture.set_block_with_nbt(
        0, 1, 0,
        "minecraft:chest[facing=south]",
        '{"CustomName":"Treasure"}',
    )
    return fixture


root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(os.environ.get(
    "NUCLEATION_OUT",
    root / "docs/media/readme/formats-and-io/round-trip-build.gif",
))
still_out = Path(os.environ.get(
    "NUCLEATION_STILL_OUT",
    root / "docs/media/readme/formats-and-io/format-fixture.png",
))
downloads = Path(os.environ.get(
    "NUCLEATION_DOWNLOAD_DIR",
    root / "docs/downloads/readme/formats-and-io",
))
gif_out.parent.mkdir(parents=True, exist_ok=True)
still_out.parent.mkdir(parents=True, exist_ok=True)
downloads.mkdir(parents=True, exist_ok=True)

fixture = make_fixture()
formats = {
    "litematic": ("", ".litematic"),
    "schematic": ("v3", ".schem"),
    "structure_snbt": ("", ".snbt"),
    "snapshot": ("", ".nusn"),
    "mcstructure": ("", ".mcstructure"),
}
for format_name, (version, extension) in formats.items():
    data = b64decode(fixture.save_as_b64(format_name, version, ""))
    (downloads / f"round-trip{extension}").write_bytes(data)

still = RenderConfig.create(600, 380)
still.set_isometric()
still.set_sphere_fit(True)
still.set_background(0, 0, 0, 0)
still.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.22)
Renderer.render_to_file(fixture, pack, still, str(still_out))

animation = BuildAnimation.create("format_fixture")
animation.set_default_effect(AnimationEffect.drop_and_pop(450, 2.5))
animation.set_stagger_total_ms(1_700)
animation.begin_group()
for x in range(4):
    for z in range(4):
        animation.set_block(x, 0, z, "minecraft:stone_bricks")
animation.end_group()
animation.set_block(0, 1, 0, "minecraft:chest[facing=south]")
animation.set_block(1, 1, 1, "minecraft:oak_stairs[facing=east,half=bottom]")
animation.set_block(2, 1, 1, "minecraft:lever[face=floor,facing=east,powered=false]")

camera = AnimationEffect.create(2_700)
camera.add_tween("rotateY", -9, 9, "inOutSine")
animation.animate_camera(camera, 0)

motion = RenderConfig.create(460, 340)
motion.set_isometric()
motion.set_sphere_fit(True)
motion.set_background(0, 0, 0, 0)
motion.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.22)
frames = animation.render_gif(pack, motion, str(gif_out), 15, 700)

print(f"wrote five formats to {downloads}")
print(f"rendered still to {still_out}")
print(f"rendered {frames} frames to {gif_out}")
