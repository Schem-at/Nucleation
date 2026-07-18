# World round-trip (Python)

```python
import tempfile, pathlib
from nucleation import Schematic

# A few chunks worth of plaza: 40x5x40 stone base with a checkerboard top.
plaza = Schematic.create("plaza")
plaza.fill_cuboid(0, 0, 0, 39, 3, 39, "minecraft:stone")
for tx in range(5):
    for tz in range(5):
        block = ("minecraft:white_concrete", "minecraft:black_concrete")[(tx + tz) % 2]
        plaza.fill_cuboid(tx * 8, 4, tz * 8, tx * 8 + 7, 4, tz * 8 + 7, block)

world_dir = tempfile.mkdtemp(suffix="-world")
plaza.save_world(world_dir, "")  # a playable world: region/*.mca + level.dat
print("world files:", sorted(p.name for p in pathlib.Path(world_dir).rglob("*") if p.is_file()))

back = Schematic.from_world_directory_bounded(world_dir, 0, 0, 0, 39, 4, 39)
d = back.tight_dimensions()
print("re-imported:", (d.x, d.y, d.z), "blocks:", back.block_count())
print("probe (8,4,8):", back.get_block_name(8, 4, 8))
print("probe (16,4,8):", back.get_block_name(16, 4, 8))
```

Output:

```text
world files: ['level.dat', 'r.0.0.mca', 'session.lock']
re-imported: (40, 5, 40) blocks: 8000
probe (8,4,8): minecraft:white_concrete
probe (16,4,8): minecraft:black_concrete
```

_Environment: CPython 3.14.6 + nucleation 0.3.6 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- `save_world(dir, options_json)` takes a second options-JSON arg ("" = defaults). The bounds of `from_world_directory_bounded` are inclusive block coordinates. For chunk-at-a-time work the streaming surface is `WorldStream` / `WorldChunkView` / `WorldSink` (see src/bridge/world_stream.rs); the whole-schematic round-trip shown here is the one-shot form. -->
