from nucleation import Schematic, ResourcePack, RenderConfig

# Textures come from a Minecraft resource pack; the schematic is any
# supported format (.schem, .litematic, .nbt, .mcstructure, ...).
pack = ResourcePack.from_file("pack.zip")
schem = Schematic.open("build.schem")

# Isometric (orthographic at yaw 45 / pitch 35.26) with a transparent
# background. Set an opaque colour instead for a solid backdrop, e.g.
# config.set_background(0.05, 0.05, 0.08, 1.0).
config = RenderConfig.isometric(width=1024, height=1024)
config.set_background(0.0, 0.0, 0.0, 0.0)

schem.render("build.png", config=config, pack=pack)
