# Fields and patterns: a fractured Voronoi planet (Python)

Three `cells` modes and one depth gradient, all evaluated per voxel and snapped
through dithered palettes. The `f1` field shades each cell (light center, dark
rim); the `f2MinusF1` crack field cuts recessed buffer grooves between cells;
and the depth into the sphere, `R - length(p)` (the signed distance an SDF
returns, i.e. the gradient normal to the surface), ramps the glow from
orange-yellow glass at the crust through shroomlight into glowstone at the core.

```python
import json, math
from nucleation import Schematic, Palette, Sdf

R, freq, seed = 30, 0.09, 4
crust, inset, crack_w, f1_rim = 7.0, 1.0, 1.2, 4.0
cells = {"frequency": freq, "seed": seed, "jitter": 1.0}
f1_field    = json.dumps({"type": "cells", "mode": "f1",        **cells})
crack_field = json.dumps({"type": "cells", "mode": "f2MinusF1", **cells})

cell_pal = Palette.from_block_ids(json.dumps([
    "minecraft:black_concrete", "minecraft:blackstone", "minecraft:deepslate",
    "minecraft:polished_deepslate", "minecraft:gray_concrete"])).dithered()
glow_pal = Palette.from_block_ids(json.dumps([
    "minecraft:yellow_stained_glass", "minecraft:orange_stained_glass",
    "minecraft:shroomlight", "minecraft:glowstone"])).dithered()

def stops(t, s):                       # color at t along (pos, rgb) stops
    t = max(0.0, min(1.0, t))
    for (a, ca), (b, cb) in zip(s, s[1:]):
        if t <= b:
            f = (t - a) / (b - a) if b > a else 0.0
            return tuple(round(ca[i] + (cb[i] - ca[i]) * f) for i in range(3))
    return s[-1][1]

CELL = [(0.0, (118, 120, 128)), (1.0, (10, 11, 16))]                       # center -> rim
GLOW = [(0.0, (233, 233, 70)), (0.20, (220, 130, 52)),
        (0.45, (241, 147, 71)), (1.0, (172, 131, 84))]                     # glass -> glowstone

def glow(depth, x, y, z):
    # normalize over ~2x the crust so a groove shows the full transition
    return glow_pal.closest_block_dithered(*stops(depth / (crust * 2.0), GLOW), x, y, z)

s = Schematic.create("planet")
for x in range(-R, R + 1):
    for y in range(-R, R + 1):
        for z in range(-R, R + 1):
            d = math.sqrt(x * x + y * y + z * z)
            if d > R:
                continue
            depth = R - d                                  # normal distance into the sphere
            fx, fy, fz = x + 0.5, y + 0.5, z + 0.5
            if depth > crust:                              # glowing core
                block = glow(depth, x, y, z)
            elif Sdf.eval(crack_field, fx, fy, fz) < crack_w:   # recessed buffer groove
                if depth < inset:
                    continue                               # air gap at the surface
                block = glow(depth, x, y, z)
            else:                                          # cell crust
                f1 = Sdf.eval(f1_field, fx, fy, fz)
                block = cell_pal.closest_block_dithered(
                    *stops(min(f1 / f1_rim, 1.0), CELL), x, y, z)
            s.set_block(x, y, z, block)

dim = s.tight_dimensions()
print("planet:", (dim.x, dim.y, dim.z), "blocks:", s.block_count())
```

Output:

```text
planet: (60, 60, 60) blocks: 109027
```

The three fields are independent knobs: swap `f1` for `f2` to shade toward the
second-nearest seed, widen `crack_w` for chunkier grooves, or replace
`R - length(p)` with any SDF's value to run the same glow gradient normal to a
torus, a box, or a warped blob instead of a sphere.

_Environment: CPython 3.14.6 + nucleation 0.3.17 wheel (bridge-full, cp312-abi3), macOS arm64._
