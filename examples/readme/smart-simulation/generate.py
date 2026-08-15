"""Generate the animation, state comparison, and download for the simulation guide."""

import os
from pathlib import Path

from nucleation import (
    AnimationEffect,
    BuildAnimation,
    MchprsWorld,
    RenderConfig,
    Renderer,
)

from smart_simulation import scene


root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(os.environ.get(
    "NUCLEATION_OUT",
    root / "docs/media/readme/smart-simulation/smart-circuit.gif",
))
idle_out = Path(os.environ.get(
    "NUCLEATION_IDLE_OUT",
    root / "docs/media/readme/smart-simulation/circuit-idle.png",
))
powered_out = Path(os.environ.get(
    "NUCLEATION_POWERED_OUT",
    root / "docs/media/readme/smart-simulation/circuit-powered.png",
))
schem_out = Path(os.environ.get(
    "NUCLEATION_SCHEM_OUT",
    root / "docs/downloads/readme/smart-simulation/smart-circuit.schem",
))
for path in (gif_out, idle_out, powered_out, schem_out):
    path.parent.mkdir(parents=True, exist_ok=True)

scene.save_to_file(str(schem_out))

still = RenderConfig.create(560, 300)
still.set_isometric()
still.set_sphere_fit(True)
still.set_background(0, 0, 0, 0)
still.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.22)
Renderer.render_to_file(scene, pack, still, str(idle_out))

powered_world = MchprsWorld.create(scene)
powered_world.on_use_block(0, 1, 0)
powered_world.tick(2)
powered_world.flush()
powered_world.sync_to_schematic()
powered = powered_world.get_schematic()
Renderer.render_to_file(powered, pack, still, str(powered_out))

animation = BuildAnimation.create("smart_circuit")
animation.set_default_effect(AnimationEffect.drop_and_pop(420, 2.2))
animation.set_stagger_total_ms(1_800)
animation.begin_group()
for x in range(9):
    for z in range(3):
        animation.set_block(x, 0, z, "minecraft:smooth_stone")
animation.end_group()
animation.set_block(0, 1, 0, "minecraft:lever[face=floor,facing=east,powered=false]")
for x in range(1, 7):
    animation.set_block(
        x, 1, 0,
        "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]",
    )
animation.set_block(7, 1, 0, "minecraft:redstone_lamp[lit=false]")
animation.set_block(0, 1, 2, "minecraft:barrel[facing=west]")

camera = AnimationEffect.create(2_900)
camera.add_tween("rotateY", -8, 8, "inOutSine")
animation.animate_camera(camera, 0)

motion = RenderConfig.create(460, 300)
motion.set_isometric()
motion.set_sphere_fit(True)
motion.set_background(0, 0, 0, 0)
motion.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.22)
frames = animation.render_gif(pack, motion, str(gif_out), 15, 700)

print(f"saved {schem_out}")
print(f"rendered idle and powered states")
print(f"rendered {frames} frames to {gif_out}")
