# Nucleation v0.1.173

Headlined by the new Minecraft item-model export. Also includes two entity
fixes and a multi-pack resource-pack loader that cuts across every binding.

## Minecraft Item Model Export

Convert any schematic into a Minecraft resource pack that renders as a custom 3D item model in-game.

### Overview

Nucleation can now generate Minecraft-compatible item models from schematics using a hybrid rendering approach. Full-cube blocks are composited into efficient plane-based slices (max 288 elements for a 48x48x48 build), while non-full-cube blocks (levers, torches, redstone, stairs, etc.) are rendered as individual model elements with their actual 3D geometry. The result is a ready-to-use resource pack that can be loaded directly into Minecraft 1.21.4+.

### How it works

1. The schematic is sliced into 2D planes for each of the 6 directions (north, south, east, west, up, down)
2. For each plane, visible block face textures are resolved through the full blockstate -> model -> texture chain using an existing resource pack
3. Face textures are composited into a single PNG per plane, with correct UV rotation for rotated blocks (levers, torches, etc.)
4. Non-full-cube blocks are extracted as individual model elements preserving their original shape and rotation
5. A Minecraft item model JSON is generated referencing all plane and element textures
6. Everything is packaged into a resource pack ZIP with proper `pack.mcmeta`, item definitions, model JSON, and texture PNGs

### Usage

### Python

```python
import nucleation

schematic = nucleation.Schematic.from_file("my_build.schem")
pack = nucleation.ResourcePack.from_file("1.21.4.zip")

config = nucleation.ItemModelConfig("my_build")
config.item = "paper"                # Minecraft item to bind to
config.custom_model_data = "1"       # Selector value

result = schematic.to_item_model(pack, config)
result.save_resource_pack("my_pack.zip")
```

To merge multiple schematics into a single resource pack:

```python
configs = [
    ("build_a", "1"),
    ("build_b", "2"),
    ("build_c", "3"),
]

results = []
for name, cmd in configs:
    config = nucleation.ItemModelConfig(name)
    config.custom_model_data = cmd
    results.append(schematic.to_item_model(pack, config))

nucleation.build_resource_pack(results, "merged_pack.zip")
```

Then in Minecraft:
```
/give @s minecraft:paper[custom_model_data={strings:["1"]}]
```

### JavaScript (WASM)

```js
import { Schematic, ResourcePack, ItemModelConfig, buildResourcePack } from "nucleation";

const schematic = Schematic.fromBytes(data, "schem");
const pack = new ResourcePack(packBytes);

const config = new ItemModelConfig("my_build");
config.item = "paper";
config.customModelData = "1";

const result = schematic.toItemModel(pack, config);
const zip = result.toResourcePackZip();

// Or merge multiple results
const mergedZip = buildResourcePack([result1, result2, result3]);
```

### C/C++ (FFI)

```c
NucItemModelConfig* cfg = itemmodel_config_new("my_build");
itemmodel_config_set_item(cfg, "paper");
itemmodel_config_set_custom_model_data(cfg, "1");

NucItemModelResult* result = schematic_to_item_model(schematic, pack, cfg);

// Single result -> ZIP
uint32_t zip_len;
uint8_t* zip = itemmodel_result_to_resource_pack_zip(result, &zip_len);

// Or merge multiple results
NucItemModelResult* results[] = {r1, r2, r3};
uint8_t* merged = itemmodel_build_resource_pack(results, 3, &zip_len);
```

### Configuration

| Field | Default | Description |
|-------|---------|-------------|
| `model_name` | `"schematic"` | Name used in file paths and texture references |
| `namespace` | `"nucleation"` | Resource pack namespace |
| `center` | `true` | Center the schematic within model coordinate bounds |
| `texture_resolution` | `16` | Pixels per block face |
| `item` | `"paper"` | Minecraft item to bind the model to |
| `custom_model_data` | `"1"` | Value for selecting the model via `custom_model_data` component |

### Result stats

`ItemModelResult` includes statistics about the generated model:

- `plane_count` - number of plane elements (efficient composited slices)
- `element_count` - total elements including individual non-full-cube blocks
- `texture_count` - number of generated texture PNGs
- `dimensions` - schematic dimensions as `[x, y, z]`

### Constraints

- Maximum schematic size: 48x48x48 blocks (Minecraft model coordinate range: -16 to 32)
- Requires a Minecraft resource pack (1.21.4+) for texture resolution
- Generated resource packs target Minecraft 1.21.4+ item model format (`pack_format: 46`)

## Entity Fixes

Two bugs that caused entities to be silently dropped at export time have been
fixed.

### Schematic bounding box now tracks entity positions

Previously, `add_entity` / `add_block_entity` updated only the entity list and
not the schematic's tight bounds. `to_compact()` (used by the `.litematic` and
`.schem` exporters) then filtered out any entity whose position fell outside
that bbox — so entities placed outside the block envelope were quietly lost in
the exported file, and Litematica would also filter them on load.

The new `Region::get_content_bounds()` (plus matching
`UniversalSchematic::get_content_bounds()`) returns the union of non-air
blocks, entity positions, and block-entity positions. `to_compact()` now sizes
the compact region from content bounds and keeps every entity that was added.
`litematic::to_litematic` uses content bounds for `EnclosingSize` so Litematica
loads all entities correctly.

### `.schem` entity NBT now matches the Sponge v3 spec

`convert_entities` previously wrote the vanilla Minecraft chunk-format entity
NBT directly (lowercase top-level `id`) and an internal validator silently
dropped any entity whose compound lacked a capital `Id`, which meant every
entity was discarded at export time. The reader had a matching bug.

Export now emits v3-compliant wrappers — `{ Id, Pos, Data: { vanilla MC entity
NBT } }` — with `Motion`/`Rotation` defaults populated in `Data` so
spec-compliant loaders (WorldEdit, FastAsyncWorldEdit, etc.) accept the file.
v2 export emits the flat layout defined by the v2 spec. `parse_entities`
transparently handles both shapes, so existing `.schem` files produced by
older Nucleation versions still load.

## Multi-pack Resource Pack Loading

`ResourcePackSource` now accepts one *or* multiple resource packs, matching
Minecraft's own pack-priority model (lowest priority first, each subsequent
pack overlays the previous on per-key collision). This is additive — the
existing `from_file` / `from_bytes` entry points are unchanged.

### Rust

```rust
use nucleation::meshing::ResourcePackSource;

// Single pack (existing API — unchanged)
let pack = ResourcePackSource::from_file("vanilla.zip")?;

// Multiple packs — priority order, lowest first
let pack = ResourcePackSource::from_files([
    "vanilla.zip",
    "base_mod.zip",
    "texture_pack_override.zip",
])?;

// Or from byte buffers (WASM-friendly)
let pack = ResourcePackSource::from_bytes_list(vec![vanilla_bytes, override_bytes])?;
```

### Python

```python
pack = nucleation.ResourcePack.from_files(["vanilla.zip", "override.zip"])
# or
pack = nucleation.ResourcePack.from_bytes_list([vanilla_bytes, override_bytes])
```

### JavaScript (WASM)

```js
const pack = ResourcePackWrapper.fromBytesList([vanillaBytes, overrideBytes]);
```

### C/C++ (FFI)

```c
const uint8_t* ptrs[] = { vanilla_bytes, override_bytes };
size_t        lens[] = { vanilla_len,   override_len };
NucResourcePack* pack = resourcepack_from_bytes_list(ptrs, lens, 2);
```

The merged pack is a drop-in for every mesher entry point — `to_mesh`,
`to_usdz`, `to_item_model`, `build_global_atlas`, etc. — so you can load once
and reuse across outputs.

Built on top of the corresponding upstream addition in `schematic-mesher`
(`ResourcePack::overlay`, `load_resource_packs`, `load_resource_packs_from_bytes`),
available on the `feature/mesh-output-api` branch.
