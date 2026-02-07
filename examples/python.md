## 1. Importing

```python
import nucleation                             # compiled PyO3 module

Schematic = nucleation.Schematic             # class names exported by #[pyclass(name = …)]
BlockState = nucleation.BlockState
```

---

## 2. `BlockState`   (`nucleation.BlockState`)

| Method / property | Signature (Python)                                 | What it does                                                 | Mini-example                                                      |
| ----------------- | -------------------------------------------------- | ------------------------------------------------------------ | ----------------------------------------------------------------- |
| `BlockState()`    | `BlockState(name: str)`                            | Create a new Minecraft block state with no extra properties. | `stone = BlockState("minecraft:stone")`                           |
| `with_property`   | `with_property(key: str, value: str) → BlockState` | Returns **a copy** with an extra property.                   | `oak_log = BlockState("minecraft:log").with_property("axis","y")` |
| `name`            | `str` (read-only)                                  | Block identifier.                                            | `print(stone.name)  # "minecraft:stone"`                          |
| `properties`      | `dict[str,str]` (read-only)                        | All properties in a plain dict.                              | `oak_log.properties → {"axis": "y"}`                              |
| `str()`           | `str(block_state)`                                 | Mojang-style string (`block[foo=bar]`).                      |                                                                   |
| `repr()`          | `repr(block_state)`                                | Debug-style, e.g. `<BlockState 'minecraft:stone'>`.          |                                                                   |

---

## 3. `Schematic`   (`nucleation.Schematic`)

### 3-second constructor

```python
sch = Schematic("My build")  # empty schematic with that name
```

### File ↔ bytes I/O

| Call                                                      | What it accepts / returns                                       |
| --------------------------------------------------------- | --------------------------------------------------------------- |
| `from_data(data: bytes)`                                  | Auto-detects Litematic **or** WorldEdit `.schematic` in memory. |
| `from_litematic(data: bytes)`<br>`to_litematic() → bytes` | Explicit Litematic import / export.                             |
| `from_schematic(data: bytes)`<br>`to_schematic() → bytes` | Explicit WorldEdit import / export.                             |

### Basic block editing

| Call                        | Signature                                                                      | Notes                                                                                                                                    |
| --------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `set_block`                 | `set_block(x,y,z, block_name: str)`                                            | Quickly place a block without properties/NBT.                                                                                            |
| `set_block_with_properties` | `set_block_with_properties(x,y,z, block_name: str, properties: dict[str,str])` | Pass a plain dict of properties.                                                                                                         |
| `set_block_from_string`     | `set_block_from_string(x,y,z, block_string: str)`                              | Accepts a **full** string like `minecraft:barrel[facing=up]{signal=13}`; also auto-creates a matching block entity when NBT is supplied. |

### Copy / paste & chunk helpers

| Call                                                                                                                                    | What it does                                                                                                                                              |                                                                                                 |
| --------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| \`copy\_region(from\_schematic, min\_x,min\_y,min\_z, max\_x,max\_y,max\_z, target\_x,target\_y,target\_z, excluded\_blocks: list\[str] | None)\`                                                                                                                                                   | Copies a cuboid region (optionally skipping specific block types) and pastes it with an offset. |
| `get_chunks(chunk_w, chunk_h, chunk_l, strategy=None, camera_x=0.0, camera_y=0.0, camera_z=0.0)`                                        | Splits the schematic into chunks and **orders** them with one of:<br>`"distance_to_camera"`, `"top_down"`, `"bottom_up"`, `"center_outward"`, `"random"`. |                                                                                                 |

### Queries

| Property / method          | Description                                        |
| -------------------------- | -------------------------------------------------- |
| `get_block(x,y,z)`         | Returns a `BlockState` **or** `None`.              |
| `get_block_entity(x,y,z)`  | `dict` with `id`, `position`, `nbt` **or** `None`. |
| `get_all_block_entities()` | List of the above dicts.                           |
| `get_all_blocks()`         | List of `{x,y,z,name,properties}` dicts.           |
| `dimensions`               | `(width, height, length)` bounding box size.       |
| `block_count`              | Total number of non-air blocks.                    |
| `volume`                   | Total voxels in bounding box.                      |
| `region_names`             | Names of all stored regions.                       |
| `debug_info()`             | Quick human string with name + region count.       |
| `str(schematic)`           | Pretty ASCII printout (blocks only).               |
| `repr(schematic)`          | `<Schematic 'Name', N blocks>`                     |

---

## 4. Standalone helpers

| Function                                     | Signature | Use-case                                                                         |
| -------------------------------------------- | --------- | -------------------------------------------------------------------------------- |
| `nucleation.debug_schematic(schematic)`      | `→ str`   | Same output as `schematic.debug_info()` + pretty ASCII map; handy for `print()`. |
| `nucleation.debug_json_schematic(schematic)` | `→ str`   | Human-readable JSON dump of the entire structure.                                |

---

## 5. Quick “hello world”

```python
import nucleation as nuc

sch = nuc.Schematic("Demo")
sch.set_block(0, 0, 0, "minecraft:stone")
sch.set_block_from_string(1, 0, 0,
    'minecraft:barrel[facing=up]{signal=7}'     # auto-fills redstone items!
)

print(nuc.debug_schematic(sch))

with open("demo.litematic", "wb") as f:
    f.write(sch.to_litematic())
```

---

---

## 6. 3D Mesh Generation

Generate 3D meshes from schematics using Minecraft resource packs.

```python
from nucleation import Schematic, ResourcePack, MeshConfig

# Load schematic
with open("build.litematic", "rb") as f:
    data = f.read()
sch = Schematic("build")
sch.load_from_bytes(data)

# Load resource pack
pack = ResourcePack.from_file("resourcepack.zip")
print(f"Loaded {pack.blockstate_count} blockstates, {pack.model_count} models, {pack.texture_count} textures")

# Configure meshing
config = MeshConfig(
    cull_hidden_faces=True,
    ambient_occlusion=True,
    ao_intensity=0.4,
    cull_occluded_blocks=True,
    greedy_meshing=True,
)

# Generate GLB
result = sch.to_mesh(pack, config)
result.save("output.glb")
print(f"GLB: {result.vertex_count} vertices, {result.triangle_count} triangles")

# Generate USDZ (for Apple AR Quick Look)
usdz = sch.to_usdz(pack, config)
usdz.save("output.usdz")

# Generate raw mesh data
raw = sch.to_raw_mesh(pack, config)
positions = raw.positions_flat()   # list[float], 3 per vertex
normals = raw.normals_flat()       # list[float], 3 per vertex
uvs = raw.uvs_flat()               # list[float], 2 per vertex
indices = raw.indices()            # list[int]
print(f"Raw: {raw.vertex_count} vertices, {raw.triangle_count} triangles")
```

### Per-region and per-chunk meshing

```python
# One mesh per region
multi = sch.mesh_by_region(pack, config)
multi.save_all("output_dir/")

# One mesh per 16x16x16 chunk
chunks = sch.mesh_by_chunk(pack, config)
chunks.save_all("chunks_dir/")

# Custom chunk size
chunks32 = sch.mesh_by_chunk_size(pack, config, 32)
```

### Resource pack querying

```python
blockstates = pack.list_blockstates()   # list[str]
models = pack.list_models()
textures = pack.list_textures()

# Query specific entries
json_str = pack.get_blockstate_json("minecraft:stone")
info = pack.get_texture_info("minecraft:block/stone")
# info = {"width": 16, "height": 16, "is_animated": False, "frame_count": 1}
pixels = pack.get_texture_pixels("minecraft:block/stone")  # bytes

# Add custom entries
pack.add_blockstate_json("mymod:custom", blockstate_json_str)
pack.add_model_json("mymod:block/custom", model_json_str)
pack.add_texture("mymod:block/custom", 16, 16, rgba_bytes)
```

---

### Gotchas & tips

* **Everything is immutable-copy except `set_block*`** – methods that start with `set_` mutate the schematic; others usually return a fresh object or `dict`.
* `set_block_from_string` understands signal strengths for barrels (`{signal=0–15}`) and automatically fills the barrel with enough redstone blocks to match the comparator level.
* Chunk ordering strategies are deterministic when `"random"` is chosen – they hash the schematic name for seeding.

Happy building & scripting!
