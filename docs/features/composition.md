# Composition: stacking the primitives

## Everything composes


Nothing on this page is a special case. Shapes, SDF booleans, deformation,
texture projection, and the palette engine are pieces you stack. This ring is
five of them in one build: a `torus` SDF with a lattice of spheres subtracted
for holes, a noise `warp` to deform it, then Van Gogh's Starry Night wrapped
around the ring and tube and matched through the dithered flat-art palette.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/compose-torus.gif" width="600" alt="A warped torus with lattice holes, wearing Van Gogh's Starry Night wrapped around it, rotating on a turntable">
</div>

```python
# One SDF: a torus, minus a repeating lattice of spheres, warped by noise.
torus = Shape.sdf('''{"type": "warp", "amplitude": 3, "frequency": 0.045, "seed": 11, "child":
  {"type": "smoothSubtract", "k": 1.5,
   "a": {"type": "torus", "majorRadius": 26, "minorRadius": 9},
   "b": {"type": "repeat", "spacing": [11,11,11], "child": {"type": "sphere", "radius": 3.5}}}}''')
BuildingTool.fill(s, torus, Brush.solid("minecraft:stone"))

# Wrap the painting on: UV from the torus geometry, color through the palette.
pal = flat_art_palette().dithered()
for x, y, z in solid_voxels(s):
    u, v = torus_uv(x, y, z)               # angle around the ring, angle around the tube
    s.set_block(x, y, z, pal.closest_block_dithered(*starry_night(u, v), x, y, z))
```

Swap the torus for a mesh, the painting for a heightmap, the palette for
grayscale, and it is a different build with the same five moves. There's a
[runnable version](../readme-snippets/18-compose-torus-python.md) you can paste
and adapt; the full recipe is `scene_compose` in
[`tools/readme-media/generate.py`](../../tools/readme-media/generate.py).
