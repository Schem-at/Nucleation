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

## Generate random-access worlds

`WorldStream` reads persisted chunks. `WorldGenerator` is deliberately separate:
it is an immutable source graph that generates chunk `(cx, cz)` directly from
absolute world coordinates. A request never allocates a scratch `Schematic`, and
portable bindings never call host code per voxel.

```python
from nucleation import (
    Brush, GeneratedChunkCoverage, Sdf, WorldGenerator, WorldSink,
)

terrain = Sdf.plane(0.0, 1.0, 0.0, -12.0).displace(3.0, 0.04, 42, 4)
source = WorldGenerator.sdf(
    terrain,
    Brush.solid("minecraft:stone"),
    -32,
    64,
    "example:terrain",  # stable source identity
    "recipe-v1",        # data/recipe version for cache identity
)

# Deterministic random access; requests may arrive in any order.
generated = source.generate(-3, 7)
assert generated.cx() == -3 and generated.cz() == 7
assert generated.coverage() == GeneratedChunkCoverage.Complete
chunk = generated.take_view()  # existing WorldChunkView / WorldSink contract

# Export remains explicitly finite and lazy. The iterator holds one result at a
# time and uses canonical region-major order to avoid revisiting flushed regions.
sink = WorldSink.create("world/", "")
stream = source.stream(-8, -8, 7, 7)  # inclusive chunk bounds
while stream.remaining() > 0:
    sink.write_chunk(stream.next().take_view())
sink.finish()
```

A generated result carries three distinct concepts:

- `Complete`: the source defines the entire requested chunk, including air.
- `Partial`: the source contributes sparse data inside the chunk.
- `Outside`: the request is outside available coverage; this is not generated air.

It also carries caller-supplied source identity and version. Change the version
when a dataset, seed, recipe, projection, or other cache-relevant input changes.
Generation failures are errors, not missing coverage.

### SDF, projected OSM data, and composition

`WorldGenerator.sdf` evaluates a validated native SDF at voxel centers and builds
Anvil sections and palettes directly. Geometry and brushes execute entirely in
Rust. Because the current SDF evaluator is `f32`, requests beyond exact
half-integer voxel-center precision fail instead of silently aliasing neighboring
blocks.

`WorldGenerator.cellular_sdf` turns a **bounded** SDF motif into a non-periodic
hashed-cell source. Signed cell coordinates deterministically select position
jitter, yaw, horizontal scale, elevation, and optional feature presence. Reusing
one `CellularSdfConfig` across material layers keeps a cliff, river, waterfall,
forest, path, and cabin spatially coordinated without modulo-wrapping the world:

```python
from nucleation import Brush, CellularSdfConfig, Sdf, WorldGenerator

cells = CellularSdfConfig.create(
    192, 160, 0x6A09E667,  # cell dimensions and world seed
    14.0, 10.0, 13.0,     # maximum X/Z jitter and yaw
    0.88, 1.14,            # horizontal scale range
    -2, 3,                 # vertical offset range
    1, 1, 0,               # presence ratio and feature salt
)
landmark = WorldGenerator.cellular_sdf(
    Sdf.sphere(24.0), Brush.solid("minecraft:stone"),
    -32, 64, cells, "example:landmarks", "recipe-v1",
)
```

The motif must expose conservative finite, non-empty bounds. Chunk generation uses
checked 64-bit anchor/candidate arithmetic, expands to the necessary neighboring-cell
halo, enforces a fixed candidate budget, culls transforms whose bounds miss the
request, and clips all writes to the requested chunk and configured Y interval.
Cellular evaluation subtracts each integer cell anchor before converting the small
local delta to `f32`, preserving voxel-center precision at legal extreme world
coordinates. Continuous hash lanes are exact `[0, 1)` values; discrete presence and
Y-offset choices use unbiased multiply-high selection. `feature_salt` changes only
presence selection, so optional layers can share transforms while varying
independently. Caller provenance should be bumped whenever the recipe or generator
contract changes.

`WorldGenerator.projected_footprints` is the geospatial adapter. It accepts the
same projected-footprint JSON used by `Geo.extrude_footprints`, spatially clips
work to the requested chunk, and reports sparse coverage. Network fetching, OSM
PBF parsing, source revision selection, and latitude/longitude projection remain
explicit caller responsibilities—Nucleation never guesses a CRS or silently
substitutes missing tiles.

```python
import json
from nucleation import GeneratedChunkOverlayMode, WorldGenerator

buildings = WorldGenerator.projected_footprints(
    json.dumps([
        {
            "polygon": [[15, 0], [18, 0], [18, 3], [15, 3]],
            "min_y": 1,
            "height": 12,
            "block": "minecraft:bricks",
        }
    ]),
    "",                    # optional base slab block
    "osm:city-buildings",
    "planet-2026-07-25",
)

world = WorldGenerator.composite("example:world", "v1")
world.add_layer(source, GeneratedChunkOverlayMode.Replace)
world.add_layer(buildings, GeneratedChunkOverlayMode.Replace)
```

Composition is ordered and explicit. `Replace` lets later non-air voxels win;
`KeepExisting` only fills air. A composite is complete if any layer is complete,
partial if at least one layer contributes and none is complete, and outside if
all layers are outside. Source graphs are capped at 64 direct layers.

Native Rust integrations can implement the public `ChunkSource: Send + Sync`
trait for raster/DEM, database, object-store, network, or custom vector sources.
Those adapters return the same bounded `ChunkResult`; traversal, scheduling,
caching, cancellation, and persistence remain separate concerns.

The complete deterministic SDF + projected-feature example is
[`infinite_riverfall_world.py`](../../examples/features/streaming-and-worlds/infinite_riverfall_world.py).
The OSM Financial District below uses the same projected-vector boundary: 179
buildings streamed one 16×16 chunk column at a time.

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/worldgen-osm.gif" width="760" alt="The voxel Financial District generating chunk column by chunk column in a diagonal sweep until the full skyline stands">

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/worldgen-sdf.gif" width="560" alt="An SDF island terrain generating chunk by chunk, growing outward from its center until the whole island stands">
</div>

## Worlds


Schematics round-trip through *playable worlds*: export a real world folder
(`level.dat` + region files), import any world back, bounded to a box or
[streamed chunk-by-chunk](#read-iterate-and-stream) in constant memory:

```python
plaza.save_world(world_dir, "")
back = Schematic.from_world_directory_bounded(world_dir, 0, 0, 0, 39, 4, 39)
```
