# OSM footprints → a playable world (Python)

```python
import json, tempfile, pathlib
from nucleation import Geo, Schematic

# Footprints as you'd project them from OSM ways — [x, z] block coordinates
# plus a height in blocks. Geo rasterizes, extrudes, and stacks tallest-wins.
buildings = [
    {"polygon": [[0, 0], [12, 0], [12, 8], [0, 8]], "height": 40, "block": "minecraft:white_concrete"},
    {"polygon": [[16, 2], [24, 2], [24, 10], [16, 10]], "height": 22, "block": "minecraft:sandstone"},
    {"polygon": [[4, 12], [20, 12], [20, 20], [4, 20]], "height": 12, "block": "minecraft:bricks"},
]
city = Geo.extrude_footprints(json.dumps(buildings), "minecraft:gray_concrete", "block")
d = city.tight_dimensions()
print("city:", (d.x, d.y, d.z), "blocks:", city.block_count())

# Straight out to a playable Minecraft world (region files + level.dat):
world_dir = tempfile.mkdtemp(suffix="-city")
city.save_world(world_dir, "")
print("world:", sorted(p.name for p in pathlib.Path(world_dir).rglob("*.mca")), "+ level.dat")

back = Schematic.from_world_directory_bounded(world_dir, 0, 0, 0, d.x - 1, d.y - 1, d.z - 1)
print("reloaded blocks:", back.block_count(), "| tip block:", back.get_block_name(6, 40, 4))
```

Output:

```text
city: (25, 41, 21) blocks: 7309
world: ['r.0.0.mca'] + level.dat
reloaded blocks: 7309 | tip block: minecraft:white_concrete
```

_Environment: CPython 3.14.6 + nucleation 0.3.14 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- Geo is pure/network-free: fetch tiles or query Overpass yourself, project lat/lon to block coords, then hand Geo the footprints. `Shape.polygon_prism(polygon_json, y_min, y_max)` is the single-footprint primitive underneath. For builds too big to hold at once, write the world chunk-by-chunk with WorldSink instead of save_world (see 12-chunk-iteration-python.md). Real recipe: scene_city in tools/readme-media/generate.py. -->
