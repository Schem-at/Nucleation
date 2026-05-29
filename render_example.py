from nucleation import Schematic, ResourcePack, RenderConfig, Projection

pack = ResourcePack.from_file("pack.zip")
schem = Schematic.open("f-117-nighthawk.litematic")

# Isometric framing with a fully transparent background.
cfg = RenderConfig.isometric(width=1024, height=768)
cfg.zoom = 0.85                          # tighten framing (smaller zoom => bigger object)
cfg.set_background(0.0, 0.0, 0.0, 0.0)   # transparent PNG (alpha 0)

# render_to_file(pack, path, config). For a perspective view with a solid
# background instead:
#   cfg = RenderConfig(width=1024, height=768, fov=28.0,
#                      background=(0.05, 0.05, 0.08, 1.0),
#                      projection=Projection.Perspective)
schem.render_to_file(pack, "night.png", cfg)
