# Nucleation Python API Reference

Nucleation provides native Python bindings via PyO3. Install with `pip install nucleation` and import as `import nucleation`. All types follow Python naming conventions (snake_case methods, PascalCase types).

---

## Table of Contents

- [Core](#core)
  - [Schematic](#schematic)
  - [BlockState](#blockstate)
  - [SchematicBuilder](#schematicbuilder)
  - [DefinitionRegion](#definitionregion)
- [Building](#building)
  - [Shape](#shape)
  - [Brush](#brush)
  - [BuildingTool](#buildingtool)
- [Simulation](#simulation-feature-gated) *(feature: `simulation`)*
  - [MchprsWorld](#mchprsworld)
  - [CircuitBuilder](#circuitbuilder)
  - [TypedCircuitExecutor](#typedcircuitexecutor)
  - [IoLayoutBuilder](#iolayoutbuilder)
  - [IoLayout](#iolayout)
  - [Value](#value)
  - [IoType](#iotype)
  - [LayoutFunction](#layoutfunction)
  - [ExecutionMode](#executionmode)
  - [OutputCondition](#outputcondition)
  - [SortStrategy](#sortstrategy)
- [Meshing](#meshing-feature-gated) *(feature: `meshing`)*
  - [ResourcePack](#resourcepack)
  - [MeshConfig](#meshconfig)
  - [MeshResult](#meshresult)
  - [MultiMeshResult](#multimeshresult)
  - [ChunkMeshResult](#chunkmeshresult)
  - [TextureAtlas](#textureatlas)
  - [RawMeshExport](#rawmeshexport)
- [Module Functions](#module-functions)

---

## Core

### Schematic

The primary class for creating, loading, editing, and exporting Minecraft schematics.

```python
from nucleation import Schematic

schematic = Schematic()
schematic = Schematic(name="My Build")
```

#### Constructor

```python
Schematic(name: Optional[str] = None) -> Schematic
```

Creates a new empty schematic with an optional name.

#### Format I/O — Import

All import methods mutate the existing schematic instance.

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_data` | `from_data(data: bytes) -> None` | Auto-detect format and load. Supports Litematic, Sponge Schematic, McStructure. |
| `from_litematic` | `from_litematic(data: bytes) -> None` | Load from `.litematic` format (Java Edition, Litematica mod). |
| `from_schematic` | `from_schematic(data: bytes) -> None` | Load from `.schematic` / `.schem` format (Sponge/WorldEdit). |
| `from_mcstructure` | `from_mcstructure(data: bytes) -> None` | Load from `.mcstructure` format (Bedrock Edition). |
| `from_mca` | `from_mca(data: bytes) -> None` | Load from a single MCA region file. |
| `from_mca_bounded` | `from_mca_bounded(data: bytes, min_x: int, min_y: int, min_z: int, max_x: int, max_y: int, max_z: int) -> None` | Load MCA with coordinate bounds. |
| `from_world_zip` | `from_world_zip(data: bytes) -> None` | Load from a zipped Minecraft world. |
| `from_world_zip_bounded` | `from_world_zip_bounded(data: bytes, min_x: int, min_y: int, min_z: int, max_x: int, max_y: int, max_z: int) -> None` | Load zipped world with coordinate bounds. |
| `from_world_directory` | `from_world_directory(path: str) -> None` | Load from an unzipped world folder on disk. |
| `from_world_directory_bounded` | `from_world_directory_bounded(path: str, min_x: int, min_y: int, min_z: int, max_x: int, max_y: int, max_z: int) -> None` | Load world folder with coordinate bounds. |
| `from_snapshot` | `from_snapshot(data: bytes) -> None` | Load from snapshot format (fast binary, `.nusn`). |

#### Format I/O — Export

| Method | Signature | Description |
|--------|-----------|-------------|
| `to_litematic` | `to_litematic() -> bytes` | Export to Litematic format. |
| `to_schematic` | `to_schematic() -> bytes` | Export to Sponge Schematic (default version). |
| `to_schematic_version` | `to_schematic_version(version: str) -> bytes` | Export to a specific Sponge Schematic version. |
| `to_mcstructure` | `to_mcstructure() -> bytes` | Export to Bedrock McStructure format. |
| `to_world` | `to_world(options_json: Optional[str] = None) -> dict[str, bytes]` | Export as world files. Returns `{path: bytes}`. |
| `to_world_zip` | `to_world_zip(options_json: Optional[str] = None) -> bytes` | Export as a zipped Minecraft world. |
| `save_world` | `save_world(directory: str, options_json: Optional[str] = None) -> None` | Write world files to a directory on disk. |
| `to_snapshot` | `to_snapshot() -> bytes` | Export to snapshot format (fast binary, `.nusn`). |
| `save` | `save(path: str, format: str = "auto", version: Optional[str] = None, settings: Optional[str] = None) -> None` | Save to file. Auto-detects format from extension when `format="auto"`. |
| `save_as` | `save_as(format: str, version: Optional[str] = None, settings: Optional[str] = None) -> bytes` | Generic export to any registered format. |

**Static format discovery methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `get_supported_import_formats` | `@staticmethod get_supported_import_formats() -> list[str]` | List all importable format names. |
| `get_supported_export_formats` | `@staticmethod get_supported_export_formats() -> list[str]` | List all exportable format names. |
| `get_format_versions` | `@staticmethod get_format_versions(format: str) -> list[str]` | List available versions for a format. |
| `get_default_format_version` | `@staticmethod get_default_format_version(format: str) -> Optional[str]` | Get the default version for a format. |
| `get_export_settings_schema` | `@staticmethod get_export_settings_schema(format: str) -> Optional[str]` | JSON schema for export settings. |
| `get_import_settings_schema` | `@staticmethod get_import_settings_schema(format: str) -> Optional[str]` | JSON schema for import settings. |
| `get_available_schematic_versions` | `@staticmethod get_available_schematic_versions() -> list[str]` | Available Sponge Schematic versions. |

#### Block Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `set_block` | `set_block(x: int, y: int, z: int, block_name: str) -> bool` | Set a block by name. Returns success. |
| `set_block_in_region` | `set_block_in_region(region_name: str, x: int, y: int, z: int, block_name: str) -> bool` | Set a block in a named region. |
| `set_block_from_string` | `set_block_from_string(x: int, y: int, z: int, block_string: str) -> None` | Parse a full block string with properties and NBT, e.g. `"minecraft:chest[facing=north]{Items:[...]}"`. |
| `set_block_with_properties` | `set_block_with_properties(x: int, y: int, z: int, block_name: str, properties: dict[str, str]) -> None` | Set a block with a property dict. |
| `set_block_with_nbt` | `set_block_with_nbt(x: int, y: int, z: int, block_name: str, nbt_data: dict) -> None` | Set a block with block entity NBT data. |
| `get_block` | `get_block(x: int, y: int, z: int) -> Optional[BlockState]` | Get the BlockState at a position. |
| `get_block_string` | `get_block_string(x: int, y: int, z: int) -> Optional[str]` | Get the full block string with properties. |
| `get_block_with_properties` | `get_block_with_properties(x: int, y: int, z: int) -> Optional[BlockState]` | Get the full BlockState. |

#### Batch Block Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `set_blocks` | `set_blocks(positions: list[tuple[int, int, int]], block_name: str) -> int` | Set the same block at multiple positions. Returns count. |
| `get_blocks` | `get_blocks(positions: list[tuple[int, int, int]]) -> list[Optional[str]]` | Get block names at multiple positions. |
| `fill_cuboid` | `fill_cuboid(min_x: int, min_y: int, min_z: int, max_x: int, max_y: int, max_z: int, block_state: str) -> None` | Fill a cuboid with a single block. |
| `fill_sphere` | `fill_sphere(cx: int, cy: int, cz: int, radius: float, block_state: str) -> None` | Fill a sphere with a single block. |

#### Region Copying

| Method | Signature | Description |
|--------|-----------|-------------|
| `copy_region` | `copy_region(from_schematic: Schematic, min_x: int, min_y: int, min_z: int, max_x: int, max_y: int, max_z: int, target_x: int, target_y: int, target_z: int, excluded_blocks: Optional[list[str]] = None) -> None` | Copy a region from another schematic, optionally excluding blocks. |

#### Block Entities

| Method | Signature | Description |
|--------|-----------|-------------|
| `get_block_entity` | `get_block_entity(x: int, y: int, z: int) -> Optional[dict]` | Get block entity at position. Returns `{"id": str, "position": (int, int, int), "nbt": dict}`. |
| `get_all_block_entities` | `get_all_block_entities() -> list[dict]` | Get all block entities. |

#### Mobile Entities

| Method | Signature | Description |
|--------|-----------|-------------|
| `entity_count` | `entity_count() -> int` | Get mobile entity count. |
| `get_entities` | `get_entities() -> list[dict]` | Get all entities as `[{"id": str, "position": (float, float, float), "nbt": dict}, ...]`. |
| `add_entity` | `add_entity(id: str, x: float, y: float, z: float, nbt_json: Optional[str] = None) -> None` | Add a mobile entity. |
| `remove_entity` | `remove_entity(index: int) -> bool` | Remove an entity by index. |

#### Transformations

Rotation values must be 90, 180, or 270 degrees.

| Method | Signature | Description |
|--------|-----------|-------------|
| `flip_x` / `flip_y` / `flip_z` | `flip_x() -> None` | Mirror along the given axis. |
| `rotate_x` / `rotate_y` / `rotate_z` | `rotate_y(degrees: int) -> None` | Rotate around the given axis. |
| `flip_region_x` / `flip_region_y` / `flip_region_z` | `flip_region_x(region_name: str) -> None` | Mirror a named region. |
| `rotate_region_x` / `rotate_region_y` / `rotate_region_z` | `rotate_region_y(region_name: str, degrees: int) -> None` | Rotate a named region. |

#### Metadata Properties

All metadata fields are available as Python properties with getters and setters.

| Property | Type | Description |
|----------|------|-------------|
| `name` | `Optional[str]` | Schematic name. Use `set_name(name)`. |
| `author` | `Optional[str]` | Author. Use `set_author(author)`. |
| `description` | `Optional[str]` | Description. Use `set_description(desc)`. |
| `created` | `Optional[int]` | Creation timestamp (ms). Use `set_created(ts)`. |
| `modified` | `Optional[int]` | Modification timestamp (ms). Use `set_modified(ts)`. |
| `lm_version` | `Optional[int]` | Litematic format version. Use `set_lm_version(v)`. |
| `mc_version` | `Optional[int]` | Minecraft data version. Use `set_mc_version(v)`. |
| `we_version` | `Optional[int]` | WorldEdit version. Use `set_we_version(v)`. |

#### Dimensions & Bounds

| Property / Method | Type | Description |
|-------------------|------|-------------|
| `dimensions` | `(int, int, int)` | Property: tight content dimensions (width, height, length). |
| `allocated_dimensions` | `(int, int, int)` | Property: full allocated buffer dimensions. |
| `block_count` | `int` | Property: total non-air blocks. |
| `volume` | `int` | Property: bounding box volume. |
| `region_names` | `list[str]` | Property: all region names. |
| `get_bounding_box` | `((int,int,int), (int,int,int))` | Returns `(min, max)` tuples. |
| `get_region_bounding_box` | `(region_name: str) -> ((int,int,int), (int,int,int))` | Bounding box of a named region. |
| `get_tight_dimensions` | `(int, int, int)` | Content-only dimensions. |
| `get_tight_bounds_min` | `Optional[(int, int, int)]` | Minimum corner of content bounds. |
| `get_tight_bounds_max` | `Optional[(int, int, int)]` | Maximum corner of content bounds. |

#### Palette Access

| Method | Signature | Description |
|--------|-----------|-------------|
| `get_palette` | `get_palette() -> list[BlockState]` | Merged palette. |
| `get_default_region_palette` | `get_default_region_palette() -> list[BlockState]` | Default region palette. |
| `get_palette_from_region` | `get_palette_from_region(region_name: str) -> list[BlockState]` | Palette for a specific region. |
| `get_all_palettes` | `get_all_palettes() -> dict` | All palettes with `"default"` and `"regions"` keys. |

#### Block Iteration

| Method | Signature | Description |
|--------|-----------|-------------|
| `get_all_blocks` | `get_all_blocks() -> list[dict]` | All non-air blocks as `[{"x", "y", "z", "name", "properties"}, ...]`. |
| `get_chunks` | `get_chunks(chunk_width: int, chunk_height: int, chunk_length: int, strategy: Optional[str] = None, camera_x: float = 0, camera_y: float = 0, camera_z: float = 0) -> list[dict]` | Chunk-based iteration with optional loading strategy. |

**Loading strategies:** `"distance_to_camera"`, `"top_down"`, `"bottom_up"`, `"center_outward"`, `"random"`

#### Cache Management

| Method | Signature | Description |
|--------|-----------|-------------|
| `clear_cache` | `clear_cache() -> None` | Clear internal block lookup caches. |
| `cache_info` | `cache_info() -> (int, int)` | Returns `(hits, misses)`. |

#### Sign Text & Insign

| Method | Signature | Description |
|--------|-----------|-------------|
| `extract_signs` | `extract_signs() -> list[dict]` | Extract all sign text as `[{"pos": [x,y,z], "text": str}, ...]`. |
| `compile_insign` | `compile_insign() -> dict` | Compile Insign annotations from signs into structured metadata. |

#### Definition Regions

| Method | Signature | Description |
|--------|-----------|-------------|
| `add_definition_region` | `add_definition_region(name: str, region: DefinitionRegion) -> None` | Add a pre-built region. |
| `get_definition_region` | `get_definition_region(name: str) -> DefinitionRegion` | Get a region by name. |
| `remove_definition_region` | `remove_definition_region(name: str) -> bool` | Remove a region. |
| `get_definition_region_names` | `get_definition_region_names() -> list[str]` | List all region names. |
| `create_definition_region` | `create_definition_region(name: str) -> None` | Create empty. |
| `create_definition_region_from_point` | `create_definition_region_from_point(name: str, x: int, y: int, z: int) -> None` | Create from a point. |
| `create_definition_region_from_bounds` | `create_definition_region_from_bounds(name: str, min: (int,int,int), max: (int,int,int)) -> None` | Create from bounds. |
| `create_region` | `create_region(name: str, min: (int,int,int), max: (int,int,int)) -> DefinitionRegion` | Create and return. |
| `update_region` | `update_region(name: str, region: DefinitionRegion) -> None` | Update existing. |
| `definition_region_add_bounds` | `definition_region_add_bounds(name: str, min: (int,int,int), max: (int,int,int)) -> None` | Add bounds to existing. |
| `definition_region_add_point` | `definition_region_add_point(name: str, x: int, y: int, z: int) -> None` | Add a point to existing. |
| `definition_region_set_metadata` | `definition_region_set_metadata(name: str, key: str, value: str) -> None` | Set region metadata. |
| `definition_region_shift` | `definition_region_shift(name: str, x: int, y: int, z: int) -> None` | Shift a region. |

#### Debug

| Method | Signature | Description |
|--------|-----------|-------------|
| `debug_info` | `debug_info() -> str` | Structured debug information. |
| `__str__` | `str(schematic)` | String representation. |
| `__repr__` | `repr(schematic)` | Detailed representation. |

---

### BlockState

Represents a Minecraft block with its name and properties.

```python
from nucleation import BlockState

bs = BlockState("minecraft:oak_stairs")
bs = bs.with_property("facing", "north")
print(bs.name)        # "minecraft:oak_stairs"
print(bs.properties)  # {"facing": "north"}
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `__init__` | `BlockState(name: str)` | Create with no properties. |
| `with_property` | `with_property(key: str, value: str) -> BlockState` | Return new BlockState with added property. |
| `name` | `@property name -> str` | Block name. |
| `properties` | `@property properties -> dict[str, str]` | Property dict. |
| `__str__` / `__repr__` | | String representations. |

---

### SchematicBuilder

A fluent builder for constructing schematics using ASCII art character-to-block mappings.

```python
from nucleation import SchematicBuilder

schematic = (SchematicBuilder()
    .name("Wall")
    .map('#', "minecraft:stone_bricks")
    .map('.', "minecraft:air")
    .layers([
        ["###", "#.#", "###"],
        ["###", "#.#", "###"],
    ])
    .build())
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `__init__` | `SchematicBuilder()` | Create a new builder. |
| `name` | `name(name: str) -> SchematicBuilder` | Set schematic name (chainable). |
| `map` | `map(ch: str, block: str) -> SchematicBuilder` | Map a character to a block (chainable). |
| `layers` | `layers(layers: list[list[str]]) -> SchematicBuilder` | Set 3D layers (chainable). |
| `build` | `build() -> Schematic` | Build the schematic. |
| `from_template` | `@staticmethod from_template(template: str) -> SchematicBuilder` | Create from a named template. |

---

### DefinitionRegion

A logical volume composed of bounding boxes. Used for circuit I/O definitions, spatial queries, and metadata.

```python
from nucleation import DefinitionRegion

region = DefinitionRegion.from_bounds((0, 0, 0), (10, 5, 10))
region = region.add_point(15, 3, 8)
print(region.volume())  # Total voxel count
```

#### Constructors

| Method | Signature | Description |
|--------|-----------|-------------|
| `__init__` | `DefinitionRegion()` | Create empty. |
| `from_bounds` | `@staticmethod from_bounds(min: (int,int,int), max: (int,int,int)) -> DefinitionRegion` | Create from bounding box. |
| `from_bounding_boxes` | `@staticmethod from_bounding_boxes(boxes: list[((int,int,int), (int,int,int))]) -> DefinitionRegion` | Create from multiple boxes. |
| `from_positions` | `@staticmethod from_positions(positions: list[(int,int,int)]) -> DefinitionRegion` | Create from individual points (auto-merged). |

#### Mutating Operations (chainable, return `self`)

| Method | Signature | Description |
|--------|-----------|-------------|
| `add_bounds` | `add_bounds(min: (int,int,int), max: (int,int,int)) -> DefinitionRegion` | Add a bounding box. |
| `add_point` | `add_point(x: int, y: int, z: int) -> DefinitionRegion` | Add a single point. |
| `add_filter` | `add_filter(filter: str) -> DefinitionRegion` | Add a block filter. |
| `exclude_block` | `exclude_block(block_name: str) -> DefinitionRegion` | Exclude positions with a specific block. |
| `set_metadata` | `set_metadata(key: str, value: str) -> DefinitionRegion` | Set metadata. |
| `set_color` | `set_color(color: int) -> DefinitionRegion` | Set visualization color (ARGB). |
| `merge` | `merge(other: DefinitionRegion) -> DefinitionRegion` | Merge another region. |

#### Mutating Void Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `subtract` | `subtract(other: DefinitionRegion) -> None` | Remove points in `other`. |
| `intersect` | `intersect(other: DefinitionRegion) -> None` | Keep only shared points. |
| `union_into` | `union_into(other: DefinitionRegion) -> None` | Union in place. |
| `shift` | `shift(x: int, y: int, z: int) -> None` | Translate in place. |
| `expand` | `expand(x: int, y: int, z: int) -> None` | Expand bounds. |
| `contract` | `contract(amount: int) -> None` | Contract bounds inward. |
| `simplify` | `simplify() -> None` | Merge overlapping boxes. |
| `sync` | `sync() -> None` | Internal sync operation. |

#### Immutable Operations (return new instances)

| Method | Signature | Description |
|--------|-----------|-------------|
| `union` | `union(other: DefinitionRegion) -> DefinitionRegion` | Return the union. |
| `subtracted` | `subtracted(other: DefinitionRegion) -> DefinitionRegion` | Return the difference. |
| `intersected` | `intersected(other: DefinitionRegion) -> DefinitionRegion` | Return the intersection. |
| `shifted` | `shifted(x: int, y: int, z: int) -> DefinitionRegion` | Return shifted copy. |
| `expanded` | `expanded(x: int, y: int, z: int) -> DefinitionRegion` | Return expanded copy. |
| `contracted` | `contracted(amount: int) -> DefinitionRegion` | Return contracted copy. |
| `copy` | `copy() -> DefinitionRegion` | Deep copy. |

#### Querying

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_empty` | `is_empty() -> bool` | Whether region is empty. |
| `contains` | `contains(x: int, y: int, z: int) -> bool` | Test point membership. |
| `volume` | `volume() -> int` | Total voxel count. |
| `is_contiguous` | `is_contiguous() -> bool` | Single connected component. |
| `connected_components` | `connected_components() -> int` | Number of components. |
| `box_count` | `box_count() -> int` | Number of bounding boxes. |
| `intersects_bounds` | `intersects_bounds(min: (int,int,int), max: (int,int,int)) -> bool` | Bounding box intersection test. |

#### Filtering

| Method | Signature | Description |
|--------|-----------|-------------|
| `filter_by_block` | `filter_by_block(schematic: Schematic, block_name: str) -> DefinitionRegion` | Keep positions with a specific block. |
| `filter_by_properties` | `filter_by_properties(schematic: Schematic, properties: dict[str, str]) -> DefinitionRegion` | Keep positions matching properties. |

#### Data Accessors

| Method | Signature | Description |
|--------|-----------|-------------|
| `positions` | `positions() -> list[(int, int, int)]` | All positions. |
| `positions_sorted` | `positions_sorted() -> list[(int, int, int)]` | Sorted by Y, X, Z (for deterministic bit assignment). |
| `get_bounds` | `get_bounds() -> Optional[dict]` | `{"min": (int,int,int), "max": (int,int,int)}` or None. |
| `get_box` | `get_box(index: int) -> Optional[((int,int,int), (int,int,int))]` | Single box by index. |
| `get_boxes` | `get_boxes() -> list[((int,int,int), (int,int,int))]` | All boxes. |
| `dimensions` | `dimensions() -> (int, int, int)` | Width, height, length. |
| `center` | `center() -> Optional[(int, int, int)]` | Integer center. |
| `center_f32` | `center_f32() -> Optional[(float, float, float)]` | Float center. |
| `get_metadata` | `get_metadata(key: str) -> Optional[str]` | Get metadata value. |
| `get_all_metadata` | `get_all_metadata() -> dict[str, str]` | All metadata. |
| `metadata_keys` | `metadata_keys() -> list[str]` | All keys. |

#### Special Methods

| Method | Description |
|--------|-------------|
| `__len__` | Number of positions (`len(region)`). |
| `__bool__` | True if non-empty (`if region:`). |
| `__copy__` / `__deepcopy__` | Support for `copy.copy()` and `copy.deepcopy()`. |
| `__repr__` | String representation. |

---

## Building

### Shape

Geometric shape primitives for use with `BuildingTool`.

| Method | Signature | Description |
|--------|-----------|-------------|
| `sphere` | `@staticmethod sphere(cx: int, cy: int, cz: int, radius: float) -> Shape` | Sphere shape. |
| `cuboid` | `@staticmethod cuboid(min_x: int, min_y: int, min_z: int, max_x: int, max_y: int, max_z: int) -> Shape` | Axis-aligned cuboid. |

---

### Brush

Block-filling patterns. RGB brushes auto-map to the closest Minecraft block color.

| Method | Signature | Description |
|--------|-----------|-------------|
| `solid` | `@staticmethod solid(block_state: str) -> Brush` | Single solid block. |
| `color` | `@staticmethod color(r: int, g: int, b: int, palette_filter: Optional[list[str]] = None) -> Brush` | Match RGB to closest block. |
| `linear_gradient` | `@staticmethod linear_gradient(x1, y1, z1, r1, g1, b1, x2, y2, z2, r2, g2, b2, space: Optional[int] = None, palette_filter: Optional[list[str]] = None) -> Brush` | Gradient between two points. `space`: 0=RGB, 1=Oklab. |
| `shaded` | `@staticmethod shaded(r: int, g: int, b: int, lx: float, ly: float, lz: float, palette_filter: Optional[list[str]] = None) -> Brush` | Lambertian shading with light direction. |
| `bilinear_gradient` | `@staticmethod bilinear_gradient(ox, oy, oz, ux, uy, uz, vx, vy, vz, r00, g00, b00, r10, g10, b10, r01, g01, b01, r11, g11, b11, space: Optional[int] = None, palette_filter: Optional[list[str]] = None) -> Brush` | 4-corner quad gradient (origin + U axis + V axis). |
| `point_gradient` | `@staticmethod point_gradient(points: list[((int,int,int), (int,int,int))], falloff: Optional[float] = None, space: Optional[int] = None, palette_filter: Optional[list[str]] = None) -> Brush` | IDW point gradient. Points: `[((x,y,z), (r,g,b)), ...]`. Default falloff: 2.0. |

---

### BuildingTool

Applies a brush to a shape.

```python
from nucleation import Schematic, Shape, Brush, BuildingTool

s = Schematic()
shape = Shape.sphere(0, 10, 0, 5.0)
brush = Brush.color(255, 0, 0)
BuildingTool.fill(s, shape, brush)
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `fill` | `@staticmethod fill(schematic: Schematic, shape: Shape, brush: Brush) -> None` | Fill shape with brush in schematic. |

---

## Simulation (feature-gated)

> Requires the `simulation` feature flag at compile time. Provides full MCHPRS-based redstone simulation.

### MchprsWorld

A live redstone simulation world created from a schematic.

```python
world = schematic.create_simulation_world()
world.on_use_block(5, 3, 10)  # Toggle lever
world.tick(4)                   # Advance 4 ticks
world.flush()
print(world.is_lit(5, 5, 10))  # Check lamp
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `on_use_block` | `on_use_block(x: int, y: int, z: int) -> None` | Right-click a block (toggle lever, press button). |
| `tick` | `tick(ticks: int) -> None` | Advance simulation by N ticks. |
| `flush` | `flush() -> None` | Propagate pending redstone changes. |
| `is_lit` | `is_lit(x: int, y: int, z: int) -> bool` | Check if a redstone lamp is powered. |
| `get_lever_power` | `get_lever_power(x: int, y: int, z: int) -> bool` | Check lever state. |
| `get_redstone_power` | `get_redstone_power(x: int, y: int, z: int) -> int` | Get power level (0-15). |
| `set_signal_strength` | `set_signal_strength(x: int, y: int, z: int, strength: int) -> None` | Set custom IO signal (0-15). |
| `get_signal_strength` | `get_signal_strength(x: int, y: int, z: int) -> int` | Get custom IO signal. |
| `check_custom_io_changes` | `check_custom_io_changes() -> None` | Detect IO changes. |
| `poll_custom_io_changes` | `poll_custom_io_changes() -> list[dict]` | Get and clear changes: `[{"x", "y", "z", "old_power", "new_power"}, ...]`. |
| `peek_custom_io_changes` | `peek_custom_io_changes() -> list[dict]` | Get changes without clearing. |
| `clear_custom_io_changes` | `clear_custom_io_changes() -> None` | Clear change queue. |
| `sync_to_schematic` | `sync_to_schematic() -> None` | Write world state back to schematic. |
| `get_schematic` | `get_schematic() -> Schematic` | Get copy of current schematic. |
| `into_schematic` | `into_schematic() -> Schematic` | Consume world and return schematic. |

---

### CircuitBuilder

Fluent builder for creating typed circuit executors.

```python
from nucleation import CircuitBuilder, IoType, LayoutFunction, DefinitionRegion

cb = CircuitBuilder(schematic)
cb.with_input_auto("A", IoType.unsigned_int(4), input_region)
cb.with_output_auto("sum", IoType.unsigned_int(5), output_region)
executor = cb.build()
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `__init__` | `CircuitBuilder(schematic: Schematic)` | Create from schematic. |
| `from_insign` | `@staticmethod from_insign(schematic: Schematic) -> CircuitBuilder` | Create from Insign annotations. |
| `with_input` | `with_input(name: str, io_type: IoType, layout: LayoutFunction, region: DefinitionRegion) -> None` | Add typed input. |
| `with_input_sorted` | `with_input_sorted(name: str, io_type: IoType, layout: LayoutFunction, region: DefinitionRegion, sort: SortStrategy) -> None` | Add input with custom sort. |
| `with_input_auto` | `with_input_auto(name: str, io_type: IoType, region: DefinitionRegion) -> None` | Add input with auto layout. |
| `with_input_auto_sorted` | `with_input_auto_sorted(name: str, io_type: IoType, region: DefinitionRegion, sort: SortStrategy) -> None` | Auto layout + custom sort. |
| `with_output` / `with_output_sorted` / `with_output_auto` / `with_output_auto_sorted` | *(same patterns)* | Add outputs. |
| `with_state_mode` | `with_state_mode(mode: str) -> None` | Set `"stateless"`, `"stateful"`, or `"manual"`. |
| `validate` | `validate() -> None` | Validate configuration (raises on error). |
| `build` | `build() -> TypedCircuitExecutor` | Build the executor. |
| `build_validated` | `build_validated() -> TypedCircuitExecutor` | Validate and build. |
| `input_count` / `output_count` | `input_count() -> int` | Number of inputs/outputs. |
| `input_names` / `output_names` | `input_names() -> list[str]` | Names of inputs/outputs. |

---

### TypedCircuitExecutor

Executes a redstone circuit with typed inputs and outputs.

```python
from nucleation import ExecutionMode, Value

result = executor.execute(
    {"A": 7, "B": 3},
    ExecutionMode.until_stable(4, 100)
)
print(result["outputs"]["sum"])  # Read typed output
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_layout` | `@staticmethod from_layout(world: MchprsWorld, layout: IoLayout) -> TypedCircuitExecutor` | Create from world and layout. |
| `from_insign` | `@staticmethod from_insign(schematic: Schematic) -> TypedCircuitExecutor` | Create from Insign annotations. |
| `set_state_mode` | `set_state_mode(mode: str) -> None` | Set `"stateless"`, `"stateful"`, `"manual"`. |
| `reset` | `reset() -> None` | Reset circuit state. |
| `execute` | `execute(inputs: dict[str, Any], mode: ExecutionMode) -> dict` | Execute with typed inputs. Returns `{"outputs": dict, "ticks_elapsed": int, "condition_met": bool}`. |
| `tick` | `tick(ticks: int) -> None` | Manual: advance ticks. |
| `flush` | `flush() -> None` | Manual: propagate changes. |
| `set_input` | `set_input(name: str, value: Value) -> None` | Manual: set input. |
| `read_output` | `read_output(name: str) -> Any` | Manual: read output. |
| `input_names` / `output_names` | `input_names() -> list[str]` | List names. |
| `get_layout_info` | `get_layout_info() -> dict` | Detailed layout with bit positions. |

---

### IoLayoutBuilder

Builder for IO layouts mapping typed data to physical positions.

| Method | Signature | Description |
|--------|-----------|-------------|
| `__init__` | `IoLayoutBuilder()` | Create new builder. |
| `add_input` | `add_input(name: str, io_type: IoType, layout: LayoutFunction, positions: list[(int,int,int)]) -> IoLayoutBuilder` | Add input with positions. |
| `add_input_auto` | `add_input_auto(name: str, io_type: IoType, positions: list[(int,int,int)]) -> IoLayoutBuilder` | Auto layout. |
| `add_input_from_region` | `add_input_from_region(name: str, io_type: IoType, layout: LayoutFunction, region: DefinitionRegion) -> IoLayoutBuilder` | Input from region. |
| `add_input_from_region_auto` | `add_input_from_region_auto(name: str, io_type: IoType, region: DefinitionRegion) -> IoLayoutBuilder` | Auto layout from region. |
| `add_output` / `add_output_auto` / `add_output_from_region` / `add_output_from_region_auto` | *(same patterns)* | Add outputs. |
| `build` | `build() -> IoLayout` | Finalize. |

---

### IoLayout

A finalized IO layout.

| Method | Signature | Description |
|--------|-----------|-------------|
| `input_names` | `input_names() -> list[str]` | Input names. |
| `output_names` | `output_names() -> list[str]` | Output names. |

---

### Value

A typed value for circuit I/O.

| Method | Signature | Description |
|--------|-----------|-------------|
| `u32` | `@staticmethod u32(value: int) -> Value` | Unsigned 32-bit integer. |
| `i32` | `@staticmethod i32(value: int) -> Value` | Signed 32-bit integer. |
| `f32` | `@staticmethod f32(value: float) -> Value` | 32-bit float. |
| `bool` | `@staticmethod bool(value: bool) -> Value` | Boolean. |
| `string` | `@staticmethod string(value: str) -> Value` | String. |
| `to_py` | `to_py() -> Any` | Convert to native Python value. |
| `type_name` | `type_name() -> str` | Type name string. |

---

### IoType

Defines the data type for a circuit I/O port.

| Method | Signature | Description |
|--------|-----------|-------------|
| `unsigned_int` | `@staticmethod unsigned_int(bits: int) -> IoType` | N-bit unsigned integer. |
| `signed_int` | `@staticmethod signed_int(bits: int) -> IoType` | N-bit signed integer. |
| `float32` | `@staticmethod float32() -> IoType` | 32-bit float. |
| `boolean` | `@staticmethod boolean() -> IoType` | Boolean. |
| `ascii` | `@staticmethod ascii(chars: int) -> IoType` | ASCII string (7 bits per char). |

---

### LayoutFunction

Defines how bits map to physical redstone positions.

| Method | Signature | Description |
|--------|-----------|-------------|
| `one_to_one` | `@staticmethod one_to_one() -> LayoutFunction` | 1 bit per position (default). |
| `packed4` | `@staticmethod packed4() -> LayoutFunction` | 4 bits per position (signal strength 0-15). |
| `custom` | `@staticmethod custom(mapping: list[int]) -> LayoutFunction` | Custom bit-to-position mapping. |
| `row_major` | `@staticmethod row_major(rows: int, cols: int, bits_per_element: int) -> LayoutFunction` | Row-major 2D layout. |
| `column_major` | `@staticmethod column_major(rows: int, cols: int, bits_per_element: int) -> LayoutFunction` | Column-major 2D layout. |
| `scanline` | `@staticmethod scanline(width: int, height: int, bits_per_pixel: int) -> LayoutFunction` | Image scanline layout. |

---

### ExecutionMode

Controls circuit execution behavior.

| Method | Signature | Description |
|--------|-----------|-------------|
| `fixed_ticks` | `@staticmethod fixed_ticks(ticks: int) -> ExecutionMode` | Run for exactly N ticks. |
| `until_condition` | `@staticmethod until_condition(output_name: str, condition: OutputCondition, max_ticks: int, check_interval: int) -> ExecutionMode` | Run until output meets condition. |
| `until_change` | `@staticmethod until_change(max_ticks: int, check_interval: int) -> ExecutionMode` | Run until any output changes. |
| `until_stable` | `@staticmethod until_stable(stable_ticks: int, max_ticks: int) -> ExecutionMode` | Run until outputs are stable. |

---

### OutputCondition

Predicate for `ExecutionMode.until_condition()`.

| Method | Signature | Description |
|--------|-----------|-------------|
| `equals` | `@staticmethod equals(value: Value) -> OutputCondition` | Output equals value. |
| `not_equals` | `@staticmethod not_equals(value: Value) -> OutputCondition` | Not equal. |
| `greater_than` | `@staticmethod greater_than(value: Value) -> OutputCondition` | Greater than. |
| `less_than` | `@staticmethod less_than(value: Value) -> OutputCondition` | Less than. |
| `bitwise_and` | `@staticmethod bitwise_and(mask: int) -> OutputCondition` | AND mask is non-zero. |

---

### SortStrategy

Controls bit assignment order for IO positions.

| Method | Signature | Description |
|--------|-----------|-------------|
| `yxz` | `@staticmethod yxz() -> SortStrategy` | Y, X, Z ascending (default). |
| `xyz` | `@staticmethod xyz() -> SortStrategy` | X, Y, Z ascending. |
| `zyx` | `@staticmethod zyx() -> SortStrategy` | Z, Y, X ascending. |
| `y_desc_xz` | `@staticmethod y_desc_xz() -> SortStrategy` | Y descending, X, Z ascending. |
| `x_desc_yz` | `@staticmethod x_desc_yz() -> SortStrategy` | X descending, Y, Z ascending. |
| `z_desc_yx` | `@staticmethod z_desc_yx() -> SortStrategy` | Z descending, Y, X ascending. |
| `descending` | `@staticmethod descending() -> SortStrategy` | All descending. |
| `distance_from` | `@staticmethod distance_from(x: int, y: int, z: int) -> SortStrategy` | Distance ascending. |
| `distance_from_desc` | `@staticmethod distance_from_desc(x: int, y: int, z: int) -> SortStrategy` | Distance descending. |
| `preserve` | `@staticmethod preserve() -> SortStrategy` | Keep original order. |
| `reverse` | `@staticmethod reverse() -> SortStrategy` | Reverse order. |
| `from_string` | `@staticmethod from_string(s: str) -> SortStrategy` | Parse from string. |
| `name` | `@property name -> str` | Strategy name. |

---

## Meshing (feature-gated)

> Requires the `meshing` feature flag. Generates 3D meshes from schematics using Minecraft resource packs.

### ResourcePack

Loads Minecraft resource pack assets for mesh generation.

```python
from nucleation import ResourcePack

pack = ResourcePack.from_file("path/to/resourcepack.zip")
# or
with open("resourcepack.zip", "rb") as f:
    pack = ResourcePack.from_bytes(f.read())
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_file` | `@staticmethod from_file(path: str) -> ResourcePack` | Load from a ZIP file on disk. |
| `from_bytes` | `@staticmethod from_bytes(data: bytes) -> ResourcePack` | Load from ZIP bytes in memory. |
| `blockstate_count` | `@property -> int` | Number of blockstate definitions. |
| `model_count` | `@property -> int` | Number of models. |
| `texture_count` | `@property -> int` | Number of textures. |
| `namespaces` | `@property -> list[str]` | Resource pack namespaces. |
| `list_blockstates` | `list_blockstates() -> list[str]` | All blockstate names. |
| `list_models` | `list_models() -> list[str]` | All model names. |
| `list_textures` | `list_textures() -> list[str]` | All texture names. |
| `get_blockstate_json` | `get_blockstate_json(name: str) -> Optional[str]` | Raw blockstate JSON. |
| `get_model_json` | `get_model_json(name: str) -> Optional[str]` | Raw model JSON. |
| `get_texture_info` | `get_texture_info(name: str) -> Optional[dict]` | `{"width", "height", "is_animated", "frame_count"}`. |
| `get_texture_pixels` | `get_texture_pixels(name: str) -> Optional[bytes]` | RGBA8 pixel data. |
| `add_blockstate_json` | `add_blockstate_json(name: str, json: str) -> None` | Add/override blockstate. |
| `add_model_json` | `add_model_json(name: str, json: str) -> None` | Add/override model. |
| `add_texture` | `add_texture(name: str, width: int, height: int, pixels: bytes) -> None` | Add texture (RGBA8). |
| `stats` | `stats() -> dict` | Summary statistics. |

---

### MeshConfig

Configuration for mesh generation.

```python
from nucleation import MeshConfig

config = MeshConfig(
    cull_hidden_faces=True,
    ambient_occlusion=True,
    ao_intensity=0.4,
    biome="plains",
    atlas_max_size=4096,
    greedy_meshing=False
)
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cull_hidden_faces` | `bool` | `True` | Remove faces between adjacent solid blocks. |
| `ambient_occlusion` | `bool` | `True` | Enable AO darkening in corners. |
| `ao_intensity` | `float` | `0.4` | AO strength (0.0-1.0). |
| `biome` | `Optional[str]` | `None` | Biome for tinting grass/water/foliage. |
| `atlas_max_size` | `int` | `4096` | Max texture atlas dimension. |
| `cull_occluded_blocks` | `bool` | `True` | Skip fully enclosed blocks. |
| `greedy_meshing` | `bool` | `False` | Merge coplanar faces. |

All parameters are also available as read/write properties after construction.

---

### MeshResult

Result of mesh generation. Contains vertex data in three transparency layers plus a texture atlas.

```python
result = schematic.to_mesh(pack, config)
result.save("output.glb")

# Access raw vertex data
positions = result.opaque_positions()  # list[float]
indices = result.opaque_indices()      # list[int]
```

#### Export

| Method | Signature | Description |
|--------|-----------|-------------|
| `save` | `save(path: str) -> None` | Save as GLB file. |
| `save_nucm` | `save_nucm(path: str) -> None` | Save as NUCM cache file. |
| `nucm_data` | `nucm_data() -> bytes` | NUCM binary data. |
| `usdz_data` | `usdz_data() -> bytes` | USDZ binary data. |
| `atlas_rgba` | `atlas_rgba() -> bytes` | Atlas RGBA pixel data. |

#### Per-Layer Vertex Data

| Method | Returns | Description |
|--------|---------|-------------|
| `opaque_positions` | `list[float]` | Vertex positions `[x,y,z,...]` for solid blocks. |
| `opaque_normals` | `list[float]` | Vertex normals. |
| `opaque_uvs` | `list[float]` | Texture coordinates. |
| `opaque_colors` | `list[float]` | Vertex colors (biome tint, AO). |
| `opaque_indices` | `list[int]` | Triangle indices. |
| `cutout_positions` / `cutout_indices` | *(same)* | Alpha-tested blocks (leaves, flowers). |
| `transparent_positions` / `transparent_indices` | *(same)* | Translucent blocks (glass, water). |

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `glb_data` | `bytes` | GLB binary data. |
| `vertex_count` / `total_vertices` | `int` | Total vertices. |
| `triangle_count` / `total_triangles` | `int` | Total triangles. |
| `has_transparency` | `bool` | Has transparent geometry. |
| `is_empty` | `bool` | No geometry. |
| `lod_level` | `int` | Level of detail. |
| `chunk_coord` | `Optional[list[int]]` | `[cx, cy, cz]` if from chunk meshing. |
| `bounds` | `list[float]` | `[minX, minY, minZ, maxX, maxY, maxZ]`. |
| `atlas_width` / `atlas_height` | `int` | Atlas dimensions. |

---

### MultiMeshResult

Per-region mesh results from `mesh_by_region()`.

| Method | Signature | Description |
|--------|-----------|-------------|
| `get_mesh` | `get_mesh(region_name: str) -> Optional[MeshResult]` | Get mesh for a region. |
| `get_all_meshes` | `get_all_meshes() -> dict[str, MeshResult]` | All meshes by region name. |
| `save_all` | `save_all(prefix: str) -> list[str]` | Save as `{prefix}_{region}.glb`. |
| `nucm_data` | `nucm_data() -> bytes` | NUCM binary data. |
| `save_nucm` | `save_nucm(path: str) -> None` | Save NUCM file. |
| `region_names` | `@property -> list[str]` | Region names. |
| `total_vertex_count` | `@property -> int` | Combined vertices. |
| `total_triangle_count` | `@property -> int` | Combined triangles. |
| `mesh_count` | `@property -> int` | Number of meshes. |

---

### ChunkMeshResult

Chunk-based mesh results from `mesh_by_chunk()`.

| Method | Signature | Description |
|--------|-----------|-------------|
| `get_mesh` | `get_mesh(cx: int, cy: int, cz: int) -> Optional[MeshResult]` | Get mesh for a chunk. |
| `get_all_meshes` | `get_all_meshes() -> dict[(int,int,int), MeshResult]` | All meshes by coordinate. |
| `save_all` | `save_all(prefix: str) -> list[str]` | Save as `{prefix}_{x}_{y}_{z}.glb`. |
| `nucm_data` | `nucm_data() -> bytes` | NUCM binary data. |
| `save_nucm` | `save_nucm(path: str) -> None` | Save NUCM file. |
| `chunk_coordinates` | `@property -> list[(int,int,int)]` | All chunk coordinates. |
| `total_vertex_count` | `@property -> int` | Combined vertices. |
| `total_triangle_count` | `@property -> int` | Combined triangles. |
| `chunk_count` | `@property -> int` | Number of chunks. |

---

### TextureAtlas

Pre-built shared texture atlas for consistent chunk meshing.

| Method | Signature | Description |
|--------|-----------|-------------|
| `rgba_data` | `rgba_data() -> bytes` | RGBA8 pixel data. |
| `width` | `@property -> int` | Width in pixels. |
| `height` | `@property -> int` | Height in pixels. |
| `region_count` | `@property -> int` | Number of texture regions. |

---

### RawMeshExport

Raw mesh data for custom rendering pipelines.

| Method | Signature | Description |
|--------|-----------|-------------|
| `positions_flat` | `positions_flat() -> list[float]` | `[x,y,z,...]` |
| `normals_flat` | `normals_flat() -> list[float]` | Normals. |
| `uvs_flat` | `uvs_flat() -> list[float]` | UVs. |
| `colors_flat` | `colors_flat() -> list[float]` | Colors. |
| `indices` | `indices() -> list[int]` | Triangle indices. |
| `texture_rgba` | `texture_rgba() -> bytes` | Texture data. |
| `texture_width` / `texture_height` | `@property -> int` | Texture dimensions. |
| `vertex_count` / `triangle_count` | `@property -> int` | Counts. |

#### Meshing Methods on Schematic

These methods are available on `Schematic` when the `meshing` feature is enabled.

| Method | Signature | Description |
|--------|-----------|-------------|
| `to_mesh` | `to_mesh(pack: ResourcePack, config: Optional[MeshConfig] = None) -> MeshResult` | Full schematic mesh. |
| `mesh_by_region` | `mesh_by_region(pack: ResourcePack, config: Optional[MeshConfig] = None) -> MultiMeshResult` | Per-region meshes. |
| `mesh_by_chunk` | `mesh_by_chunk(pack: ResourcePack, config: Optional[MeshConfig] = None) -> ChunkMeshResult` | 16x16x16 chunk meshes. |
| `mesh_by_chunk_size` | `mesh_by_chunk_size(pack: ResourcePack, chunk_size: int, config: Optional[MeshConfig] = None) -> ChunkMeshResult` | Custom chunk size. |
| `mesh_chunks` | `mesh_chunks(pack: ResourcePack, config: Optional[MeshConfig] = None, chunk_size: int = 16) -> list[MeshResult]` | Flat list of chunk meshes. |
| `build_global_atlas` | `build_global_atlas(pack: ResourcePack, config: Optional[MeshConfig] = None) -> TextureAtlas` | Pre-build shared atlas. |
| `mesh_chunks_with_atlas` | `mesh_chunks_with_atlas(pack: ResourcePack, atlas: TextureAtlas, config: Optional[MeshConfig] = None, chunk_size: int = 16) -> list[MeshResult]` | Chunk meshes with shared atlas. |
| `to_usdz` | `to_usdz(pack: ResourcePack, config: Optional[MeshConfig] = None) -> MeshResult` | USDZ mesh. |
| `to_raw_mesh` | `to_raw_mesh(pack: ResourcePack, config: Optional[MeshConfig] = None) -> RawMeshExport` | Raw mesh data. |
| `register_mesh_exporter` | `register_mesh_exporter(pack: ResourcePack) -> None` | Register mesh as `save_as` format. |

---

## Module Functions

Top-level functions available directly from the `nucleation` module.

| Function | Signature | Description |
|----------|-----------|-------------|
| `load_schematic` | `load_schematic(path: str) -> Schematic` | Load a schematic file from disk (auto-detects format). |
| `save_schematic` | `save_schematic(schematic: Schematic, path: str, format: str = "auto", version: Optional[str] = None) -> None` | Save a schematic to disk. |
| `debug_schematic` | `debug_schematic(schematic: Schematic) -> str` | Formatted debug output. |
| `debug_json_schematic` | `debug_json_schematic(schematic: Schematic) -> str` | JSON-formatted debug output. |
