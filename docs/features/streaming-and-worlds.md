# Chunk iteration, streaming, and worlds

## Read, iterate, and stream


Everything above *writes* blocks. This is how you read them back and process
builds too big to hold in memory. Any schematic splits into fixed chunks in a
traversal order you choose: `bottom_up`, `top_down`, `center_outward`,
`distance_to_camera`, or `random`. Freeze a center-outward walk 60% of the way
through and the iterator's wavefront reads straight off the terrain:
plasma-tinted columns have been visited, green ones haven't yet.

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/streaming-chunks.png" width="760" alt="A rolling terrain iterated 16x16 column by column, tinted by center-outward chunk order with the unvisited rim still natural green">

```python
import json
# Walk a build in 16×16×16 chunks, center-outward from a point:
for chunk in json.loads(s.get_chunks_with_strategy_json(16, 16, 16, "center_outward", 0, 0, 0)):
    handle(chunk["chunk_x"], chunk["chunk_z"], chunk["blocks"])
```

The same idea scales past memory: stream a real world folder chunk-by-chunk and
write a transformed copy, with only one chunk resident at a time. RAM stays flat
whether the world is 10 MB or 10 GB.

```python
from nucleation import WorldStream, WorldSink

stream = WorldStream.open_dir("world/")     # or .from_zip(bytes), or *_bounded(...)
sink   = WorldSink.create("world-out/", "")
while True:
    try:
        chunk = stream.next()               # a WorldChunkView
    except Exception:
        break                               # end of stream is signalled by raising
    # inspect or edit here: chunk.set_block(...), chunk.to_schematic(), ...
    sink.write_chunk(chunk)
sink.finish()
```

And the chunk is a **two-way bridge to the building tools**. `to_schematic()`
reads a chunk out; `WorldChunkView.from_schematic(schematic, cx, cz)` writes one
back. So *anything that fills a schematic becomes a custom world generator*, one
chunk at a time. Fill an SDF (or OSM footprints, a heightmap, noise) clipped to
each chunk and stream it straight to a playable world. Intersecting with the
chunk means the field is only evaluated inside the chunk being written, so it
never materializes:

```python
from nucleation import Schematic, Shape, Brush, BuildingTool, WorldChunkView, WorldSink

sdf  = Shape.sdf(island_json)
sink = WorldSink.create("world/", "")
for cx in range(-8, 8):
    for cz in range(-8, 8):
        chunk = Schematic.create("c")
        box = Shape.cuboid(cx*16, -16, cz*16, cx*16 + 15, 48, cz*16 + 15)
        BuildingTool.fill(chunk, sdf.intersection_with(box), Brush.solid("minecraft:stone"))
        sink.write_chunk(WorldChunkView.from_schematic(chunk, cx, cz))
sink.finish()
```

Here's the OSM Financial District doing exactly that: 179 buildings streamed out
one 16×16 chunk column at a time, a diagonal wavefront assembling the whole
2.4M-block skyline:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/worldgen-osm.gif" width="760" alt="The voxel Financial District generating chunk column by chunk column in a diagonal sweep until the full skyline stands">

The source is whatever you fill with. Swap the OSM footprints for an SDF and the
*same* generator streams a terrain instead, each chunk the SDF evaluated only
inside that chunk, materializing center-outward:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/worldgen-sdf.gif" width="560" alt="An SDF island terrain generating chunk by chunk, growing outward from its center until the whole island stands">
</div>

Run the same bridge the other way and it's a *processing pipeline*: `WorldStream`
→ `to_schematic` → transform with any tool → `from_schematic` → `WorldSink`. The
OSM city, an SDF, a heightmap, a filter: the same three moves
([generator + filter snippet](../readme-snippets/17-world-generator-python.md)).

## Worlds


Schematics round-trip through *playable worlds*: export a real world folder
(`level.dat` + region files), import any world back, bounded to a box or
[streamed chunk-by-chunk](#read-iterate-and-stream) in constant memory:

```python
plaza.save_world(world_dir, "")
back = Schematic.from_world_directory_bounded(world_dir, 0, 0, 0, 39, 4, 39)
```
