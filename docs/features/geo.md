# Geodata: elevation and OSM

## The real world, in blocks


Texture mapping and the color math, animated: a voxel Earth spinning under a
fixed sun. Every frame, every surface block is re-picked by its luminosity
through the dithered palette, so continents sweep through a true day/night
terminator:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/globe-day-night.gif?v=2" width="440" alt="A voxel Earth rotating through a day/night cycle, every block re-picked by luminosity">
</div>

And real geodata voxelizes straight from public sources. The geo entry points
take data, not URLs: you fetch and project, they build the blocks. The
Matterhorn is an AWS elevation grid through `Geo.heightmap_terrain` (300×300
columns, ~53 m/block, then snow/scree/meadow bands by elevation and slope):

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/geo-mountains.png" width="760" alt="The Matterhorn and surrounding range voxelized from elevation tiles">

…and Wall Street is OpenStreetMap footprints through `Geo.extrude_footprints`:
179 buildings, each a `Shape.polygon_prism` extruded to its tagged height at
1 block = 2 m, stacked tallest-wins and banded by height:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/geo-city.png" width="760" alt="Manhattan's Financial District voxelized from OSM building data">

```python
# Each footprint is [x, z] block coords + a height; Geo rasterizes and extrudes.
buildings = [{"polygon": [[0,0],[12,0],[12,8],[0,8]], "height": 40, "block": "minecraft:white_concrete"}, ...]
city = Geo.extrude_footprints(json.dumps(buildings), "minecraft:gray_concrete", "fidi")
```

That whole 2.4-million-block district is one schematic, and it streams straight
out to a *playable Minecraft world*, region files and all, chunk by chunk in
constant memory (see [Read, iterate, and stream](streaming-and-worlds.md)):

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/geo-city-skyline.png" width="820" alt="A low-angle skyline of the voxelized Financial District, ready to load as a Minecraft world">

```python
city.save_world("fidi-world/", "")     # or stream chunk-by-chunk with WorldSink
```

All four are reproducible recipes in
[`tools/readme-media/generate.py`](../../tools/readme-media/generate.py)
(`globe`, `mountains`, `city`), and the geo API has a
[runnable snippet](../readme-snippets/16-geo-osm-python.md).
