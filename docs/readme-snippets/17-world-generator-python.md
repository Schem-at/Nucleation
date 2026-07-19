# Nucleation as a world generator / processor (Python)

`WorldChunkView.from_schematic` is the write-side twin of `to_schematic`: fill
a schematic with *any* nucleation tool (shapes, SDF, brushes, footprints), clip
it to a chunk, and stream it out — or read a chunk, transform it, and write it
back. Either way the whole world never sits in memory at once.

```python
import tempfile, json
from nucleation import (Schematic, Shape, Brush, BuildingTool, WorldChunkView,
                        WorldSink, WorldStream)

# GENERATE: an SDF island streamed to a world one chunk at a time. Intersecting
# the SDF with each chunk's cuboid means it's only evaluated inside that chunk,
# so a world of any size streams in flat memory.
sdf = Shape.sdf('{"type": "displace", "amplitude": 5, "frequency": 0.08, "seed": 3,'
                ' "child": {"type": "ellipsoid", "radii": [40, 10, 40]}}')
stone = Brush.solid("minecraft:stone")
gen_dir = tempfile.mkdtemp(suffix="-world")
sink = WorldSink.create(gen_dir, "")
made = 0
for cx in range(-4, 4):
    for cz in range(-4, 4):
        chunk = Schematic.create("c")
        box = Shape.cuboid(cx*16, -16, cz*16, cx*16+15, 32, cz*16+15)
        BuildingTool.fill(chunk, sdf.intersection_with(box), stone)   # clipped to the chunk
        if chunk.block_count() == 0:
            continue
        sink.write_chunk(WorldChunkView.from_schematic(chunk, cx, cz))
        made += 1
sink.finish()
print("generated chunks:", made)

# FILTER: read that world back chunk-by-chunk, swap stone -> gold, write a copy.
out_dir = tempfile.mkdtemp(suffix="-gold")
stream, sink2 = WorldStream.open_dir(gen_dir), WorldSink.create(out_dir, "")
seen = 0
while True:
    try:
        view = stream.next()
    except Exception:
        break
    s = view.to_schematic()
    mn, mx = s.tight_bounds_min(), s.tight_bounds_max()
    BuildingTool.fill_replacing(s, Shape.cuboid(mn.x, mn.y, mn.z, mx.x, mx.y, mx.z),
                                Brush.solid("minecraft:gold_block"), '["minecraft:stone"]')
    sink2.write_chunk(WorldChunkView.from_schematic(s, view.cx(), view.cz()))
    seen += 1
sink2.finish()
gold = Schematic.from_world_directory_bounded(out_dir, -48,-16,-48, 48,32,48)
blks = json.loads(gold.get_chunk_blocks_json(-48,-16,-48, 96,48,96))
print("filtered chunks:", seen,
      "| gold:", sum(1 for b in blks if b["name"]=="minecraft:gold_block"),
      "| stone left:", sum(1 for b in blks if b["name"]=="minecraft:stone"))
```

Output:

```text
generated chunks: 32
filtered chunks: 32 | gold: 66318 | stone left: 0
```

_Environment: CPython 3.14.6 + nucleation 0.3.16 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- The generator source is arbitrary: swap the SDF for Geo.extrude_footprints (OSM), a heightmap, a mandelbrot, layered noise — anything that fills a schematic. Clip with `shape.intersection_with(chunk_box)` so only the current chunk is evaluated. WorldChunkView.from_schematic ignores blocks outside the (cx, cz) column, so slightly-oversized fills are fine. -->
