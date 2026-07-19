# Everything composes: a warped, holey, painted torus (Python)

Five ideas stack into one build. The [README turntable](../../README.md#everything-composes)
wraps Van Gogh's Starry Night around this torus; this network-free version paints
it by hue around the ring so it runs anywhere. Swap `hsv_to_rgb(...)` for an image
sample and you have the painting version.

```python
import json, math, colorsys
from nucleation import Schematic, Shape, Brush, BuildingTool, Palette

# One SDF: a torus, minus a repeating lattice of spheres (holes), warped by noise.
torus = Shape.sdf(json.dumps({
    "type": "warp", "amplitude": 3, "frequency": 0.045, "seed": 11, "child": {
        "type": "smoothSubtract", "k": 1.5,
        "a": {"type": "torus", "majorRadius": 26, "minorRadius": 9},
        "b": {"type": "repeat", "spacing": [11, 11, 11],
              "child": {"type": "sphere", "radius": 3.5}}}}))

s = Schematic.create("ring")
BuildingTool.fill(s, torus, Brush.solid("minecraft:stone"))

# Color every voxel by where it sits on the ring, matched through a dithered palette.
pal = Palette.wool().dithered()
used, painted = set(), 0
for b in json.loads(s.get_all_blocks_json()):
    if b["name"] != "minecraft:stone":
        continue
    x, y, z = b["x"], b["y"], b["z"]
    hue = (math.atan2(z, x) / (2 * math.pi)) % 1.0            # angle around the ring
    r, g, bl = (round(c * 255) for c in colorsys.hsv_to_rgb(hue, 0.85, 1.0))
    block = pal.closest_block_dithered(r, g, bl, x, y, z)
    s.set_block(x, y, z, block)
    used.add(block); painted += 1

d = s.tight_dimensions()
print("holey warped torus:", (d.x, d.y, d.z))
print("painted voxels:", painted, "| distinct wool colors used:", len(used))
```

Output:

```text
holey warped torus: (72, 21, 72)
painted voxels: 38161 | distinct wool colors used: 11
```

_Environment: CPython 3.14.6 + nucleation 0.3.16 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- The SDF vocabulary (torus, box, capsule, cappedCone, ...; union/intersect/subtract, smooth variants; round/shell; translate/rotate/scale/mirror/repeat; displace/warp) is documented in docs/guides/sdf-terrain.md. To wrap a real image instead of a hue, compute (u, v) from the torus geometry (angle around the ring, angle around the tube) and sample the image there — that's scene_compose in tools/readme-media/generate.py. -->
