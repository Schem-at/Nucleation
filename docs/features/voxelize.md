# Voxelizing 3D models

## Voxelize 3D models


Real 3D models become schematics: GLB (node transforms, embedded textures) and
OBJ load into a `MeshModel`, and a voxelized mesh is, like everything else here,
just a `Shape`. Inside/outside comes from triangle-parity ray casting, and
normals from the nearest triangle, so lighting brushes work on it directly. The
Utah teapot under one spotlight, through the grayscale ladder:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/teapot-spotlight.png" width="640" alt="Voxelized Utah teapot lit by a single spotlight through a grayscale block palette">
</div>

```python
teapot = Voxelizer.shape_from_obj(teapot_obj, 56.0, 0.75)   # shell closes its thin ceramic walls
spot = Brush.spotlight(-38, 55, -52,  0.48, -0.54, 0.66,  46.0,  245, 242, 235)
spot.set_palette(gray_ramp)
BuildingTool.fill(s, teapot, spot)
```

The same teapot, printed layer by layer: the unbuilt volume stays on as a glass
ghost so the frame holds still while solid, height-colored layers sweep up.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/printer.gif" width="480" alt="The voxelized teapot materializing one layer at a time, a glass ghost filling in with solid height-colored blocks from the base up">
</div>

And textures project onto the voxels: each block takes the palette-closest color
of its nearest surface point (barycentric UVs, bilinear sampling). The classic
COLLADA duck, beak and eye catchlights intact:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/textured-duck.png" width="460" alt="The Khronos duck voxelized with its texture projected onto blocks">
</div>

```python
duck = Voxelizer.schematic_from_glb_textured(duck_glb, 44.0, 0.7, Palette.solid(), "duck")
# 25,641 blocks: yellow_wool body, orange beak, black eyes with snow-block catchlights
```

And it scales: a full Mario Kart 64 Rainbow Road, voxelized to a road eight
blocks wide, 51,000 blocks solved in 1.5 seconds by the scanline voxelizer:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/mariokart-track.png?v=4" width="760" alt="Rainbow Road N64 voxelized: the whole course as a glowing rainbow ribbon">

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/mariokart-closeup.png?v=4" width="620" alt="Closeup of the voxelized road: eight blocks wide, rainbow-striped surface curving into a banked loop">
</div>

A ribbon in the void is the easy case; Koopa Troopa Beach is the hard one: an
open island of sand, dirt track, cliffs, palms, and a central lagoon. The same
voxelizer call handles it, with a color-matched beach palette:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/mk64-koopa-beach.png?v=2" width="760" alt="Mario Kart 64 Koopa Troopa Beach voxelized: sand island, cyan shallows and central lagoon in an endless sea">
