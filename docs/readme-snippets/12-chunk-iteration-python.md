# Chunk iteration & world streaming (Python)

```python
import json, tempfile
from nucleation import Schematic, Shape, Brush, BuildingTool, WorldStream, WorldSink

s = Schematic.create("build")
BuildingTool.fill(s, Shape.sphere(0, 0, 0, 20), Brush.solid("minecraft:stone"))

# Split into 16-cubed chunks, walked center-outward (chunks[0] is nearest the
# given point). Strategies: bottom_up · top_down · center_outward ·
# distance_to_camera · random.
chunks = json.loads(s.get_chunks_with_strategy_json(16, 16, 16, "center_outward", 0, 0, 0))
print("chunk boxes:", len(chunks), "| first (nearest center):",
      (chunks[0]["chunk_x"], chunks[0]["chunk_y"], chunks[0]["chunk_z"]))

# Stream a playable world folder chunk-by-chunk into a transformed copy — only
# one chunk resident at a time, so memory stays flat on an arbitrarily large world.
src = tempfile.mkdtemp(suffix="-src"); s.save_world(src, "")
out = tempfile.mkdtemp(suffix="-out")
stream, sink = WorldStream.open_dir(src), WorldSink.create(out, "")
n = 0
while True:
    try:
        chunk = stream.next()          # a WorldChunkView; edit it before writing
    except Exception:
        break                          # end of stream is signalled by raising
    n += 1
    sink.write_chunk(chunk)
sink.finish()
print("streamed + rewrote chunks:", n)

back = Schematic.from_world_directory_bounded(out, -20, 0, -20, 20, 40, 20)
print("reloaded blocks:", back.block_count())
```

Output:

```text
chunk boxes: 32 | first (nearest center): (1, 0, 0)
streamed + rewrote chunks: 12
reloaded blocks: 17329
```

_Environment: CPython 3.14.6 + nucleation 0.3.10 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- `get_chunks_with_strategy_json`'s last three args are the camera point (only used by `distance_to_camera`). WorldStream also has `.from_zip(bytes)` and `open_dir_bounded` / `from_zip_bounded` to clip to a box; WorldSink has `open_existing(dir)` + `put_chunk` to patch chunks of an existing world in place. See src/bridge/world_stream.rs. -->
