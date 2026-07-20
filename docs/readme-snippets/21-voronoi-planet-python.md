# Fields and patterns: a fractured Voronoi planet (Python)

Three `cells` modes and one depth gradient, all evaluated per voxel and snapped
through dithered palettes. The `f1` field shades each cell (light center, dark
rim); the `f2MinusF1` crack field cuts recessed buffer grooves between cells;
and the depth into the sphere, `R - length(p)` (the signed distance an SDF
returns, i.e. the gradient normal to the surface), fills the grooves with a
transparent glass shell over light-emitting blocks that brighten with depth.

```python
import json, math
from nucleation import Schematic, Palette, Sdf

R, freq, seed = 60, 0.06, 4
crust, inset, glass, crack_w, f1_rim = 14.0, 3.0, 2.0, 2.0, 8.0  # inset = river depth
cells = {"frequency": freq, "seed": seed, "jitter": 1.0}
f1_field    = json.dumps({"type": "cells", "mode": "f1",        **cells})
crack_field = json.dumps({"type": "cells", "mode": "f2MinusF1", **cells})

cell_pal  = Palette.from_block_ids(json.dumps([
    "minecraft:black_concrete", "minecraft:blackstone", "minecraft:deepslate",
    "minecraft:polished_deepslate", "minecraft:gray_concrete"])).dithered()
glass_pal = Palette.from_block_ids(json.dumps([
    "minecraft:yellow_stained_glass", "minecraft:orange_stained_glass"])).dithered()
emit_pal  = Palette.from_block_ids(json.dumps([
    "minecraft:shroomlight", "minecraft:glowstone"])).dithered()

def stops(t, s):                       # color at t along (pos, rgb) stops
    t = max(0.0, min(1.0, t))
    for (a, ca), (b, cb) in zip(s, s[1:]):
        if t <= b:
            f = (t - a) / (b - a) if b > a else 0.0
            return tuple(round(ca[i] + (cb[i] - ca[i]) * f) for i in range(3))
    return s[-1][1]

CELL  = [(0.0, (118, 120, 128)), (1.0, (10, 11, 16))]     # center -> rim
GLASS = [(0.0, (242, 172, 55)),  (1.0, (224, 118, 46))]   # amber -> orange glass
EMIT  = [(0.0, (246, 150, 72)),  (1.0, (180, 133, 82))]   # shroomlight -> glowstone

def glow(depth, x, y, z):
    below = depth - inset                                 # depth beneath the crack mouth
    if below < glass:                                     # a couple layers of glass shell
        return glass_pal.closest_block_dithered(*stops(below / glass, GLASS), x, y, z)
    t = min((below - glass) / crust, 1.0)                 # emitters, brighter with depth
    return emit_pal.closest_block_dithered(*stops(t, EMIT), x, y, z)

s = Schematic.create("planet")
for x in range(-R, R + 1):
    for y in range(-R, R + 1):
        for z in range(-R, R + 1):
            d = math.sqrt(x * x + y * y + z * z)
            if d > R:
                continue
            depth = R - d                                 # normal distance into the sphere
            fx, fy, fz = x + 0.5, y + 0.5, z + 0.5
            if depth > crust:                             # glowing core
                block = glow(depth, x, y, z)
            elif Sdf.eval(crack_field, fx, fy, fz) < crack_w:   # recessed buffer groove
                if depth < inset:
                    continue                              # air gap at the surface
                block = glow(depth, x, y, z)
            else:                                         # cell crust
                f1 = Sdf.eval(f1_field, fx, fy, fz)
                block = cell_pal.closest_block_dithered(
                    *stops(min(f1 / f1_rim, 1.0), CELL), x, y, z)
            s.set_block(x, y, z, block)

dim = s.tight_dimensions()
print("planet:", (dim.x, dim.y, dim.z), "blocks:", s.block_count())
```

Output:

```text
planet: (119, 120, 121) blocks: 855272
```

The pieces are independent knobs: swap `f1` for `f2` to shade toward the
second-nearest seed, widen `crack_w` for chunkier grooves, raise `glass` for a
thicker shell over the emitters, or replace `R - length(p)` with any SDF's value
to run the same normal-depth glow into a torus, a box, or a warped blob.

_Environment: CPython 3.14.6 + nucleation 0.3.17 wheel (bridge-full, cp312-abi3), macOS arm64._
