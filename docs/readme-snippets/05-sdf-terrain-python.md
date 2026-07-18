# SDF terrain from JSON (Python)

```python
from nucleation import Sdf

island = '''{"type": "displace", "amplitude": 3, "frequency": 0.1, "seed": 7,
             "child": {"type": "ellipsoid", "radii": [14, 8, 14]}}'''
rules = '''{"fill": [
  {"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},
  {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
  {"block": "minecraft:stone"}]}'''

terrain = Sdf.schematic_from_sdf(island, rules, False, 0, 0, 0, 0, 0, 0)
d = terrain.tight_dimensions()
print("terrain:", (d.x, d.y, d.z), "blocks:", terrain.block_count())
```

Output:

```text
terrain: (29, 18, 29) blocks: 6927
```

_Environment: CPython 3.14.6 + nucleation 0.3.3 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- Awkward: `schematic_from_sdf` has no bounds-free overload — with has_bounds=False the six trailing min/max args are ignored but must still be passed. Also `dimensions()` reports the allocated (region-padded) box for SDF output; `tight_dimensions()` is the real content size. -->
