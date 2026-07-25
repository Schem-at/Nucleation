from pathlib import Path
from nucleation import BuildAnimation, RenderConfig

animation = BuildAnimation.create("axis_rotations")

regions = [
    (
        "rotate_x",
        -8,
        "minecraft:copper_block",
        "minecraft:oak_stairs[facing=south,half=bottom,shape=straight]",
    ),
    (
        "rotate_y",
        0,
        "minecraft:gold_block",
        "minecraft:oak_stairs[facing=east,half=bottom,shape=straight]",
    ),
    (
        "rotate_z",
        8,
        "minecraft:diamond_block",
        "minecraft:oak_stairs[facing=east,half=bottom,shape=straight]",
    ),
]

for name, x, block, stair in regions:
    animation.create_region(name, x, 0, 0, x + 2, 1, 0)

    animation.begin_group()
    animation.set_block_in_region(name, x,     0, 0, block)
    animation.set_block_in_region(name, x + 1, 0, 0, block)
    animation.set_block_in_region(name, x + 2, 0, 0, stair)
    animation.set_block_in_region(
        name, x, 1, 0, "minecraft:sea_lantern"
    )
    animation.end_group()

animation.rotate_region_x("rotate_x", 90, 1450)
animation.rotate_region_y("rotate_y", 90, 1450)
animation.rotate_region_z("rotate_z", 90, 1450)

view = RenderConfig.create(760, 330)
view.set_isometric()
view.set_yaw(35)
view.set_pitch(24)
view.set_zoom(1.42)
view.set_sphere_fit(True)
view.set_background(9, 17, 29, 255)
view.set_grid(19, 7, -0.505, True, 0.35, 0.48, 0.68, 0.28)

pack = Path("pack.zip").read_bytes()
animation.render_gif(pack, view, "axes.gif", 20, 900)