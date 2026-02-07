# Nucleation FFI Reference

Complete reference for the Nucleation C FFI. All symbols are exported from the shared library built with `--features ffi`.

---

## Table of Contents

1. [Building with FFI](#1--building-with-ffi)
2. [C-Compatible Data Structures](#2--c-compatible-data-structures)
3. [Memory Management](#3--memory-management)
4. [Schematic Lifecycle](#4--schematic-lifecycle)
5. [Format Loading and Saving](#5--format-loading-and-saving)
6. [Format Management](#6--format-management)
7. [Metadata](#7--metadata)
8. [Block Operations](#8--block-operations)
9. [Block Accessors](#9--block-accessors)
10. [BlockState Wrapper](#10--blockstate-wrapper)
11. [Schematic Info and Queries](#11--schematic-info-and-queries)
12. [Palette and Bounding Box](#12--palette-and-bounding-box)
13. [Block Entities](#13--block-entities)
14. [Chunk Operations](#14--chunk-operations)
15. [Copy and Region Operations](#15--copy-and-region-operations)
16. [Transformations](#16--transformations)
17. [Definition Regions (Schematic-Attached)](#17--definition-regions-schematic-attached)
18. [Definition Regions (Standalone)](#18--definition-regions-standalone)
19. [Building Tools (Shapes, Brushes, Fill)](#19--building-tools-shapes-brushes-fill)
20. [SchematicBuilder](#20--schematicbuilder)
21. [Sign Extraction and InSign](#21--sign-extraction-and-insign)
22. [Debug and Print Utilities](#22--debug-and-print-utilities)
23. [Meshing (feature = "meshing")](#23--meshing-feature--meshing)
24. [C Code Examples](#24--c-code-examples)

---

## 1 -- Building with FFI

```bash
# Build a cdylib that exports all FFI symbols
cargo build --release --features ffi
# -> target/release/libnucleation.{so|dll|dylib}
```

The crate must be compiled with **`--features ffi`** or these symbols will not be exported.

For meshing support, add both features:

```bash
cargo build --release --features ffi,meshing
```

---

## 2 -- C-Compatible Data Structures

All structures use `#[repr(C)]` and can be directly used from C/C++.

### ByteArray

```c
typedef struct {
    unsigned char* data;
    size_t         len;
} ByteArray;
```

Returned by serialization functions (`schematic_to_litematic`, `schematic_to_schematic`, etc.). Free with `free_byte_array`.

### StringArray

```c
typedef struct {
    char** data;   // array of null-terminated C strings
    size_t len;
} StringArray;
```

Returned by query functions (`schematic_get_region_names`, `schematic_get_palette`, etc.). Free with `free_string_array`.

### IntArray

```c
typedef struct {
    int*   data;
    size_t len;
} IntArray;
```

Returned by dimension/position queries. Free with `free_int_array`.

### CFloatArray

```c
typedef struct {
    float* data;
    size_t len;
} CFloatArray;
```

Returned by meshing and float-valued queries. Free with `free_float_array`.

### CProperty

```c
typedef struct {
    char* key;
    char* value;
} CProperty;
```

Used as input for `schematic_set_block_with_properties` and returned by `blockstate_get_properties`.

### CPropertyArray

```c
typedef struct {
    CProperty* data;
    size_t     len;
} CPropertyArray;
```

Free with `free_property_array`.

### CBlock

```c
typedef struct {
    int   x;
    int   y;
    int   z;
    char* name;             // block name, e.g., "minecraft:stone"
    char* properties_json;  // JSON object string, e.g., "{\"facing\":\"north\"}"
} CBlock;
```

### CBlockArray

```c
typedef struct {
    CBlock* data;
    size_t  len;
} CBlockArray;
```

Free with `free_block_array`.

### CBlockEntity

```c
typedef struct {
    char* id;        // entity ID, e.g., "minecraft:chest"
    int   x;
    int   y;
    int   z;
    char* nbt_json;  // full NBT data as JSON string
} CBlockEntity;
```

### CBlockEntityArray

```c
typedef struct {
    CBlockEntity* data;
    size_t        len;
} CBlockEntityArray;
```

Free with `free_block_entity_array`.

### CChunk

```c
typedef struct {
    int         chunk_x;
    int         chunk_y;
    int         chunk_z;
    CBlockArray blocks;
} CChunk;
```

### CChunkArray

```c
typedef struct {
    CChunk* data;
    size_t  len;
} CChunkArray;
```

Free with `free_chunk_array`.

### CBoundingBox

```c
typedef struct {
    int min_x, min_y, min_z;
    int max_x, max_y, max_z;
} CBoundingBox;
```

Returned by value (stack-allocated). No free required.

### Opaque Pointer Types

These are opaque Rust types. Always use the corresponding `*_free` function when done.

| Type                       | Description                     |
|----------------------------|---------------------------------|
| `SchematicWrapper*`        | Wraps a `UniversalSchematic`    |
| `BlockStateWrapper*`       | Wraps a `BlockState`            |
| `DefinitionRegionWrapper*` | Wraps a `DefinitionRegion`      |
| `ShapeWrapper*`            | Wraps a building shape          |
| `BrushWrapper*`            | Wraps a building brush          |
| `SchematicBuilderWrapper*` | Wraps a `SchematicBuilder`      |

---

## 3 -- Memory Management

**Rule**: Every heap-allocated value returned by the library must be freed using the matching `free_*` function. Do **not** call C `free()` on these -- the memory was allocated by Rust.

### free_string

```c
void free_string(char* string);
```

Frees a C string (`char*`) returned by any library function. Safe to call with NULL.

### free_byte_array

```c
void free_byte_array(ByteArray array);
```

Frees a `ByteArray` returned by serialization or meshing functions.

### free_string_array

```c
void free_string_array(StringArray array);
```

Frees a `StringArray` and every string it contains.

### free_int_array

```c
void free_int_array(IntArray array);
```

Frees an `IntArray` returned by dimension/position queries.

### free_float_array

```c
void free_float_array(CFloatArray array);
```

Frees a `CFloatArray` returned by meshing or float-valued queries.

### free_property_array

```c
void free_property_array(CPropertyArray array);
```

Frees a `CPropertyArray` and the key/value strings in each `CProperty`.

### free_block_array

```c
void free_block_array(CBlockArray array);
```

Frees a `CBlockArray` and the `name`/`properties_json` strings in each `CBlock`.

### free_block_entity_array

```c
void free_block_entity_array(CBlockEntityArray array);
```

Frees a `CBlockEntityArray` and the `id`/`nbt_json` strings in each `CBlockEntity`.

### free_chunk_array

```c
void free_chunk_array(CChunkArray array);
```

Frees a `CChunkArray` and recursively frees each chunk's `CBlockArray`.

---

## 4 -- Schematic Lifecycle

### schematic_new

```c
SchematicWrapper* schematic_new(void);
```

Creates a new, empty schematic named "Default". Returns an opaque pointer. **Caller must free with `schematic_free`.**

### schematic_free

```c
void schematic_free(SchematicWrapper* schematic);
```

Frees a schematic and all its internal data. Safe to call with NULL.

---

## 5 -- Format Loading and Saving

### schematic_from_data

```c
int schematic_from_data(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);
```

Auto-detects the format (litematic or sponge schematic) and populates the schematic from raw bytes.

| Return | Meaning                          |
|--------|----------------------------------|
| `0`    | Success                          |
| `-1`   | NULL pointer argument            |
| `-2`   | Parse/decode error               |
| `-3`   | Unknown/unsupported format       |

### schematic_from_litematic

```c
int schematic_from_litematic(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);
```

Loads from Litematic (`.litematic`) format. Returns `0` on success, `-1` for NULL args, `-2` on parse error.

### schematic_from_schematic

```c
int schematic_from_schematic(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);
```

Loads from Sponge Schematic (`.schematic` / `.schem`) format. Returns `0` on success, `-1` for NULL args, `-2` on parse error.

### schematic_from_mcstructure

```c
int schematic_from_mcstructure(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);
```

Loads from Bedrock McStructure (`.mcstructure`) format. Returns `0` on success, `-1` for NULL args, `-2` on parse error.

### schematic_to_litematic

```c
ByteArray schematic_to_litematic(const SchematicWrapper* schematic);
```

Serializes the schematic to Litematic format. Returns a `ByteArray` with the encoded bytes. On error or NULL input, returns `{NULL, 0}`. **Caller must free with `free_byte_array`.**

### schematic_to_schematic

```c
ByteArray schematic_to_schematic(const SchematicWrapper* schematic);
```

Serializes the schematic to Sponge Schematic format. **Caller must free with `free_byte_array`.**

### schematic_to_mcstructure

```c
ByteArray schematic_to_mcstructure(const SchematicWrapper* schematic);
```

Serializes the schematic to Bedrock McStructure format. **Caller must free with `free_byte_array`.**

### schematic_to_schematic_version

```c
ByteArray schematic_to_schematic_version(const SchematicWrapper* schematic, const char* version);
```

Serializes to Sponge Schematic format at a specific version (e.g., `"2"`, `"3"`). Uses the format manager internally. **Caller must free with `free_byte_array`.**

### schematic_save_as

```c
ByteArray schematic_save_as(const SchematicWrapper* schematic, const char* format, const char* version);
```

Generic serialization via the format manager. `format` is a format name (e.g., `"litematic"`, `"sponge"`, `"mcstructure"`). `version` may be NULL for the default version. Returns encoded bytes. **Caller must free with `free_byte_array`.**

---

## 6 -- Format Management

### schematic_get_supported_import_formats

```c
StringArray schematic_get_supported_import_formats(void);
```

Returns an array of all supported import format names (e.g., `"litematic"`, `"sponge"`, ...). **Free with `free_string_array`.**

### schematic_get_supported_export_formats

```c
StringArray schematic_get_supported_export_formats(void);
```

Returns an array of all supported export format names. **Free with `free_string_array`.**

### schematic_get_format_versions

```c
StringArray schematic_get_format_versions(const char* format);
```

Returns available version strings for a given export format. **Free with `free_string_array`.**

### schematic_get_default_format_version

```c
char* schematic_get_default_format_version(const char* format);
```

Returns the default version string for a given export format, or NULL if the format is unknown. **Free with `free_string`.**

### schematic_get_available_schematic_versions

```c
StringArray schematic_get_available_schematic_versions(void);
```

Convenience function: returns available Sponge Schematic exporter versions. **Free with `free_string_array`.**

---

## 7 -- Metadata

### schematic_get_name / schematic_set_name

```c
char* schematic_get_name(const SchematicWrapper* schematic);
void  schematic_set_name(SchematicWrapper* schematic, const char* name);
```

Get returns NULL if not set. **Free returned string with `free_string`.** Set is a no-op on NULL args.

### schematic_get_author / schematic_set_author

```c
char* schematic_get_author(const SchematicWrapper* schematic);
void  schematic_set_author(SchematicWrapper* schematic, const char* author);
```

Get returns NULL if not set. **Free returned string with `free_string`.**

### schematic_get_description / schematic_set_description

```c
char* schematic_get_description(const SchematicWrapper* schematic);
void  schematic_set_description(SchematicWrapper* schematic, const char* description);
```

Get returns NULL if not set. **Free returned string with `free_string`.**

### schematic_get_created / schematic_set_created

```c
int64_t schematic_get_created(const SchematicWrapper* schematic);
void    schematic_set_created(SchematicWrapper* schematic, uint64_t created);
```

Timestamps are milliseconds since Unix epoch. `get` returns `-1` if not set.

### schematic_get_modified / schematic_set_modified

```c
int64_t schematic_get_modified(const SchematicWrapper* schematic);
void    schematic_set_modified(SchematicWrapper* schematic, uint64_t modified);
```

Same semantics as `created`. Returns `-1` if not set.

### schematic_get_lm_version / schematic_set_lm_version

```c
int  schematic_get_lm_version(const SchematicWrapper* schematic);
void schematic_set_lm_version(SchematicWrapper* schematic, int version);
```

Litematic format version number. Returns `-1` if not set.

### schematic_get_mc_version / schematic_set_mc_version

```c
int  schematic_get_mc_version(const SchematicWrapper* schematic);
void schematic_set_mc_version(SchematicWrapper* schematic, int version);
```

Minecraft data version number. Returns `-1` if not set.

### schematic_get_we_version / schematic_set_we_version

```c
int  schematic_get_we_version(const SchematicWrapper* schematic);
void schematic_set_we_version(SchematicWrapper* schematic, int version);
```

WorldEdit version number. Returns `-1` if not set.

---

## 8 -- Block Operations

### schematic_set_block

```c
int schematic_set_block(SchematicWrapper* schematic, int x, int y, int z, const char* block_name);
```

Sets a block at `(x, y, z)` with the given name (e.g., `"minecraft:stone"`). No properties. Returns `0` on success, `-1` on NULL args.

### schematic_set_block_with_properties

```c
int schematic_set_block_with_properties(
    SchematicWrapper* schematic,
    int x, int y, int z,
    const char*      block_name,
    const CProperty* properties,
    size_t           properties_len
);
```

Sets a block with explicit properties. `properties` is a caller-owned array of `CProperty` structs. Returns `0` on success, `-1` on NULL args.

### schematic_set_block_from_string

```c
int schematic_set_block_from_string(SchematicWrapper* schematic, int x, int y, int z, const char* block_string);
```

Parses a full block string (e.g., `"minecraft:chest[facing=north]{Items:[...]}"`) and places the block with properties and optionally a block entity. Returns `0` on success, `-1` on NULL args, `-2` on parse error.

### schematic_set_block_with_nbt

```c
int schematic_set_block_with_nbt(
    SchematicWrapper* schematic,
    int x, int y, int z,
    const char* block_name,
    const char* nbt_json
);
```

Sets a block and attaches NBT data. `nbt_json` is a JSON object string parsed as `{"key": "value", ...}`. Returns `0` on success, `-1` on NULL args, `-2` on error.

### schematic_set_block_in_region

```c
int schematic_set_block_in_region(
    SchematicWrapper* schematic,
    const char* region_name,
    int x, int y, int z,
    const char* block_name
);
```

Sets a block in a specific named region. Returns `0` on success, `-1` on NULL args, `-2` if the region does not exist or the operation fails.

### schematic_set_blocks (batch)

```c
int schematic_set_blocks(
    SchematicWrapper* schematic,
    const int*  positions,
    size_t      positions_len,
    const char* block_name
);
```

Batch-sets blocks at multiple positions to the same block name. `positions` is a flat array of `[x0,y0,z0, x1,y1,z1, ...]` with `positions_len` elements (must be a multiple of 3).

For simple block names, uses an optimized path that pre-expands the region and batch-inserts with a single palette lookup. For complex block strings containing `[` or ending with `}`, falls back to per-position parsing.

| Return   | Meaning                            |
|----------|------------------------------------|
| `>= 0`  | Number of blocks successfully set  |
| `-1`    | NULL pointer argument              |
| `-2`    | `positions_len` not a multiple of 3|

### schematic_get_blocks (batch)

```c
StringArray schematic_get_blocks(
    const SchematicWrapper* schematic,
    const int* positions,
    size_t     positions_len
);
```

Batch-gets block names at multiple positions. Returns a `StringArray` with one entry per position (NULL entries for empty/out-of-bounds). **Free with `free_string_array`.**

---

## 9 -- Block Accessors

### schematic_get_block

```c
char* schematic_get_block(const SchematicWrapper* schematic, int x, int y, int z);
```

Returns the block name at `(x, y, z)`, or NULL if no block exists. **Free with `free_string`.**

### schematic_get_block_string

```c
char* schematic_get_block_string(const SchematicWrapper* schematic, int x, int y, int z);
```

Returns the full block string representation including properties (e.g., `"minecraft:oak_stairs[facing=north,half=bottom]"`), or NULL if no block exists. **Free with `free_string`.**

### schematic_get_block_with_properties

```c
BlockStateWrapper* schematic_get_block_with_properties(
    const SchematicWrapper* schematic, int x, int y, int z
);
```

Returns a `BlockStateWrapper*` with the full block state (name + properties), or NULL. **Free with `blockstate_free`.**

### schematic_get_all_blocks

```c
CBlockArray schematic_get_all_blocks(const SchematicWrapper* schematic);
```

Returns all non-air blocks in the schematic as a `CBlockArray`. Each `CBlock` includes position, name, and a `properties_json` string. **Free with `free_block_array`.**

### schematic_get_block_count

```c
int schematic_get_block_count(const SchematicWrapper* schematic);
```

Returns the total number of non-air blocks. Returns `0` on NULL.

---

## 10 -- BlockState Wrapper

### blockstate_new

```c
BlockStateWrapper* blockstate_new(const char* name);
```

Creates a new block state with the given name and no properties. **Free with `blockstate_free`.**

### blockstate_free

```c
void blockstate_free(BlockStateWrapper* bs);
```

Frees a `BlockStateWrapper`. Safe to call with NULL.

### blockstate_with_property

```c
BlockStateWrapper* blockstate_with_property(
    BlockStateWrapper* block_state,
    const char* key,
    const char* value
);
```

Returns a **new** `BlockStateWrapper` with the property added. The original is **not modified**. Both the original and the returned value must be freed with `blockstate_free`.

### blockstate_get_name

```c
char* blockstate_get_name(const BlockStateWrapper* block_state);
```

Returns the block name. **Free with `free_string`.**

### blockstate_get_properties

```c
CPropertyArray blockstate_get_properties(const BlockStateWrapper* block_state);
```

Returns all properties as key-value pairs. **Free with `free_property_array`.**

---

## 11 -- Schematic Info and Queries

### schematic_get_dimensions

```c
IntArray schematic_get_dimensions(const SchematicWrapper* schematic);
```

Returns `[width, height, length]` as an `IntArray` of length 3. These are the allocated (bounding) dimensions. **Free with `free_int_array`.**

### schematic_get_allocated_dimensions

```c
IntArray schematic_get_allocated_dimensions(const SchematicWrapper* schematic);
```

Alias for `schematic_get_dimensions`. **Free with `free_int_array`.**

### schematic_get_tight_dimensions

```c
IntArray schematic_get_tight_dimensions(const SchematicWrapper* schematic);
```

Returns the tight-fit dimensions `[width, height, length]` based on only the blocks that actually exist. **Free with `free_int_array`.**

### schematic_get_tight_bounds_min

```c
IntArray schematic_get_tight_bounds_min(const SchematicWrapper* schematic);
```

Returns `[min_x, min_y, min_z]` of the tight bounding box, or `{NULL, 0}` if the schematic is empty. **Free with `free_int_array`.**

### schematic_get_tight_bounds_max

```c
IntArray schematic_get_tight_bounds_max(const SchematicWrapper* schematic);
```

Returns `[max_x, max_y, max_z]` of the tight bounding box, or `{NULL, 0}` if the schematic is empty. **Free with `free_int_array`.**

### schematic_get_volume

```c
int schematic_get_volume(const SchematicWrapper* schematic);
```

Returns the total volume of the schematic's bounding box (width * height * length). Returns `0` on NULL.

### schematic_get_region_names

```c
StringArray schematic_get_region_names(const SchematicWrapper* schematic);
```

Returns all region names. **Free with `free_string_array`.**

---

## 12 -- Palette and Bounding Box

### schematic_get_palette

```c
StringArray schematic_get_palette(const SchematicWrapper* schematic);
```

Returns the merged palette (block names from all regions). **Free with `free_string_array`.**

### schematic_get_default_region_palette

```c
StringArray schematic_get_default_region_palette(const SchematicWrapper* schematic);
```

Returns the palette of the default region only. **Free with `free_string_array`.**

### schematic_get_palette_from_region

```c
StringArray schematic_get_palette_from_region(
    const SchematicWrapper* schematic,
    const char* region_name
);
```

Returns the palette of a specific region by name. Returns `{NULL, 0}` if the region does not exist. **Free with `free_string_array`.**

### schematic_get_all_palettes

```c
char* schematic_get_all_palettes(const SchematicWrapper* schematic);
```

Returns a JSON string mapping region names to their palettes, e.g., `{"default":["minecraft:stone","minecraft:air"], ...}`. **Free with `free_string`.**

### schematic_get_bounding_box

```c
CBoundingBox schematic_get_bounding_box(const SchematicWrapper* schematic);
```

Returns the overall bounding box. Stack-allocated; no free needed.

### schematic_get_region_bounding_box

```c
CBoundingBox schematic_get_region_bounding_box(
    const SchematicWrapper* schematic,
    const char* region_name
);
```

Returns the bounding box of a specific region. Returns zeroed struct if the region is not found.

---

## 13 -- Block Entities

### schematic_get_block_entity

```c
CBlockEntity* schematic_get_block_entity(
    const SchematicWrapper* schematic, int x, int y, int z
);
```

Returns a heap-allocated `CBlockEntity` at the given position, or NULL if none exists. The `nbt_json` field contains the full NBT as JSON. **Free the returned pointer with `free()` after freeing its inner strings with `free_string`, or wrap it in a `CBlockEntityArray` of length 1 and use `free_block_entity_array`.**

### schematic_get_all_block_entities

```c
CBlockEntityArray schematic_get_all_block_entities(const SchematicWrapper* schematic);
```

Returns all block entities in the schematic. **Free with `free_block_entity_array`.**

---

## 14 -- Chunk Operations

### schematic_get_chunk_blocks

```c
CBlockArray schematic_get_chunk_blocks(
    const SchematicWrapper* schematic,
    int offset_x, int offset_y, int offset_z,
    int width, int height, int length
);
```

Returns all blocks within a sub-region (chunk) defined by an offset and size. **Free with `free_block_array`.**

### schematic_get_chunks

```c
CChunkArray schematic_get_chunks(
    const SchematicWrapper* schematic,
    int chunk_width, int chunk_height, int chunk_length
);
```

Splits the schematic into chunks of the given size using the default bottom-up loading strategy. **Free with `free_chunk_array`.**

### schematic_get_chunks_with_strategy

```c
CChunkArray schematic_get_chunks_with_strategy(
    const SchematicWrapper* schematic,
    int chunk_width, int chunk_height, int chunk_length,
    const char* strategy,
    float camera_x, float camera_y, float camera_z
);
```

Splits the schematic into chunks with a specified loading strategy.

| Strategy string         | Behavior                                   |
|------------------------|--------------------------------------------|
| `"bottom_up"`         | Default. Load from bottom layers first.    |
| `"top_down"`          | Load from top layers first.                |
| `"center_outward"`    | Load from center outward.                  |
| `"distance_to_camera"`| Sort by distance to `(camera_x/y/z)`.     |
| `"random"`            | Random order.                              |
| `NULL` or other       | Defaults to `"bottom_up"`.                 |

**Free with `free_chunk_array`.**

---

## 15 -- Copy and Region Operations

### schematic_copy_region

```c
int schematic_copy_region(
    SchematicWrapper*       target,
    const SchematicWrapper* source,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z,
    int target_x, int target_y, int target_z,
    const char** excluded_blocks,
    size_t       excluded_blocks_len
);
```

Copies a rectangular region from `source` into `target` at the given target offset. `excluded_blocks` is an optional array of block strings to skip during the copy (can be NULL with `excluded_blocks_len = 0`).

| Return | Meaning                                  |
|--------|------------------------------------------|
| `0`    | Success                                  |
| `-1`   | NULL pointer argument                    |
| `-2`   | Copy operation failed                    |
| `-3`   | Failed to parse an excluded block string |

---

## 16 -- Transformations

All transformation functions return `0` on success, `-1` on NULL input, `-2` on error (region variants only).

### Whole-Schematic Transforms

```c
int schematic_flip_x(SchematicWrapper* schematic);
int schematic_flip_y(SchematicWrapper* schematic);
int schematic_flip_z(SchematicWrapper* schematic);
int schematic_rotate_x(SchematicWrapper* schematic, int degrees);
int schematic_rotate_y(SchematicWrapper* schematic, int degrees);
int schematic_rotate_z(SchematicWrapper* schematic, int degrees);
```

Flips or rotates the entire schematic along the given axis. `degrees` should be a multiple of 90.

### Per-Region Transforms

```c
int schematic_flip_region_x(SchematicWrapper* schematic, const char* region_name);
int schematic_flip_region_y(SchematicWrapper* schematic, const char* region_name);
int schematic_flip_region_z(SchematicWrapper* schematic, const char* region_name);
int schematic_rotate_region_x(SchematicWrapper* schematic, const char* region_name, int degrees);
int schematic_rotate_region_y(SchematicWrapper* schematic, const char* region_name, int degrees);
int schematic_rotate_region_z(SchematicWrapper* schematic, const char* region_name, int degrees);
```

Same operations but applied only to a named region. Returns `-2` if the region name is not found.

---

## 17 -- Definition Regions (Schematic-Attached)

Definition regions are named spatial regions stored on a schematic, used for marking areas, filtering, and metadata.

### schematic_create_definition_region

```c
int schematic_create_definition_region(SchematicWrapper* schematic, const char* name);
```

Creates an empty definition region with the given name. Returns `0` on success.

### schematic_create_definition_region_from_point

```c
int schematic_create_definition_region_from_point(
    SchematicWrapper* schematic, const char* name,
    int x, int y, int z
);
```

Creates a definition region containing a single point.

### schematic_create_definition_region_from_bounds

```c
int schematic_create_definition_region_from_bounds(
    SchematicWrapper* schematic, const char* name,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);
```

Creates a definition region initialized with a bounding box.

### schematic_create_region

```c
DefinitionRegionWrapper* schematic_create_region(
    SchematicWrapper* schematic, const char* name,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);
```

Creates a definition region from bounds, inserts it into the schematic, and also returns a pointer to a `DefinitionRegionWrapper` for further manipulation. **Free the wrapper with `definitionregion_free`.**

### schematic_add_definition_region

```c
int schematic_add_definition_region(
    SchematicWrapper* schematic,
    const char* name,
    const DefinitionRegionWrapper* region
);
```

Adds (or replaces) a definition region by name from a standalone wrapper.

### schematic_get_definition_region

```c
DefinitionRegionWrapper* schematic_get_definition_region(
    const SchematicWrapper* schematic, const char* name
);
```

Returns a cloned copy of a definition region by name, or NULL. **Free with `definitionregion_free`.**

### schematic_remove_definition_region

```c
int schematic_remove_definition_region(SchematicWrapper* schematic, const char* name);
```

Removes a definition region. Returns `0` on success, `-2` if the name was not found.

### schematic_get_definition_region_names

```c
StringArray schematic_get_definition_region_names(const SchematicWrapper* schematic);
```

Returns all definition region names. **Free with `free_string_array`.**

### schematic_update_region

```c
int schematic_update_region(
    SchematicWrapper* schematic, const char* name,
    const DefinitionRegionWrapper* region
);
```

Replaces (or inserts) a definition region by name from a standalone wrapper. Same as `schematic_add_definition_region`.

### schematic_definition_region_add_bounds

```c
int schematic_definition_region_add_bounds(
    SchematicWrapper* schematic, const char* name,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);
```

Adds a bounding box to an existing named definition region. Returns `-2` if region not found.

### schematic_definition_region_add_point

```c
int schematic_definition_region_add_point(
    SchematicWrapper* schematic, const char* name,
    int x, int y, int z
);
```

Adds a single point to an existing named definition region. Returns `-2` if region not found.

### schematic_definition_region_set_metadata

```c
int schematic_definition_region_set_metadata(
    SchematicWrapper* schematic, const char* name,
    const char* key, const char* value
);
```

Sets a metadata key-value pair on a named definition region. Returns `-2` if region not found.

### schematic_definition_region_shift

```c
int schematic_definition_region_shift(
    SchematicWrapper* schematic, const char* name,
    int dx, int dy, int dz
);
```

Shifts all positions in a named definition region by `(dx, dy, dz)`. Returns `-2` if region not found.

---

## 18 -- Definition Regions (Standalone)

Standalone `DefinitionRegionWrapper*` objects can be created, manipulated, and then attached to schematics.

### Lifecycle

```c
DefinitionRegionWrapper* definitionregion_new(void);
void                     definitionregion_free(DefinitionRegionWrapper* ptr);
```

### Construction

```c
DefinitionRegionWrapper* definitionregion_from_bounds(
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);

DefinitionRegionWrapper* definitionregion_from_positions(
    const int* positions, size_t positions_len
);
// positions is flat [x0,y0,z0, x1,y1,z1, ...], positions_len must be multiple of 3

DefinitionRegionWrapper* definitionregion_from_bounding_boxes(
    const int* boxes, size_t boxes_len
);
// boxes is flat [min_x0,min_y0,min_z0,max_x0,max_y0,max_z0, ...], boxes_len must be multiple of 6
```

### Mutation (in-place)

```c
int definitionregion_add_bounds(DefinitionRegionWrapper* ptr,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
int definitionregion_add_point(DefinitionRegionWrapper* ptr, int x, int y, int z);
int definitionregion_shift(DefinitionRegionWrapper* ptr, int dx, int dy, int dz);
int definitionregion_expand(DefinitionRegionWrapper* ptr, int x, int y, int z);
int definitionregion_contract(DefinitionRegionWrapper* ptr, int amount);
int definitionregion_simplify(DefinitionRegionWrapper* ptr);
int definitionregion_merge(DefinitionRegionWrapper* ptr, const DefinitionRegionWrapper* other);
int definitionregion_union_into(DefinitionRegionWrapper* ptr, const DefinitionRegionWrapper* other);
int definitionregion_set_color(DefinitionRegionWrapper* ptr, uint32_t color);
int definitionregion_exclude_block(DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic, const char* block_name);
int definitionregion_add_filter(DefinitionRegionWrapper* ptr, const char* filter);
```

All return `0` on success, `-1` on NULL input.

### Immutable Operations (return new region)

Each returns a **new** `DefinitionRegionWrapper*`. **Free with `definitionregion_free`.**

```c
DefinitionRegionWrapper* definitionregion_intersect(
    const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_intersected(
    const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_union(
    const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_subtract(
    const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_subtracted(
    const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_shifted(
    const DefinitionRegionWrapper* ptr, int dx, int dy, int dz);
DefinitionRegionWrapper* definitionregion_expanded(
    const DefinitionRegionWrapper* ptr, int x, int y, int z);
DefinitionRegionWrapper* definitionregion_contracted(
    const DefinitionRegionWrapper* ptr, int amount);
DefinitionRegionWrapper* definitionregion_copy(
    const DefinitionRegionWrapper* ptr);
DefinitionRegionWrapper* definitionregion_clone_region(
    const DefinitionRegionWrapper* ptr);  // alias for copy
DefinitionRegionWrapper* definitionregion_filter_by_block(
    const DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic, const char* block_name);
DefinitionRegionWrapper* definitionregion_filter_by_properties(
    const DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic, const char* properties_json);
// properties_json is a JSON object e.g. "{\"facing\":\"north\"}"
```

### Queries

```c
int definitionregion_is_empty(const DefinitionRegionWrapper* ptr);
// Returns 1 if empty, 0 if not, -1 on NULL

int definitionregion_volume(const DefinitionRegionWrapper* ptr);
// Returns total volume (number of positions), -1 on NULL

int definitionregion_contains(const DefinitionRegionWrapper* ptr, int x, int y, int z);
// Returns 1 if contained, 0 if not, -1 on NULL

int definitionregion_is_contiguous(const DefinitionRegionWrapper* ptr);
// Returns 1 if all boxes are contiguous, 0 if not, -1 on NULL

int definitionregion_connected_components(const DefinitionRegionWrapper* ptr);
// Returns count of connected components, -1 on NULL

int definitionregion_box_count(const DefinitionRegionWrapper* ptr);
// Returns number of bounding boxes in the region, -1 on NULL

int definitionregion_intersects_bounds(const DefinitionRegionWrapper* ptr,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
// Returns 1 if intersects, 0 if not, -1 on NULL
```

### Spatial Data Extraction

```c
CBoundingBox definitionregion_get_bounds(const DefinitionRegionWrapper* ptr);
// Returns overall bounding box (zeroed if empty)

CBoundingBox definitionregion_get_box(const DefinitionRegionWrapper* ptr, size_t index);
// Returns the bounding box at the given index (zeroed if out of range)

IntArray definitionregion_get_boxes(const DefinitionRegionWrapper* ptr);
// Flat array [min_x0,min_y0,min_z0,max_x0,max_y0,max_z0, ...]. Free with free_int_array.

IntArray definitionregion_dimensions(const DefinitionRegionWrapper* ptr);
// Returns [width, height, length]. Free with free_int_array.

IntArray definitionregion_center(const DefinitionRegionWrapper* ptr);
// Returns [x, y, z] center (integer). Free with free_int_array.

CFloatArray definitionregion_center_f32(const DefinitionRegionWrapper* ptr);
// Returns [x, y, z] center (float). Free with free_float_array.

IntArray definitionregion_positions(const DefinitionRegionWrapper* ptr);
// All positions as flat [x0,y0,z0, x1,y1,z1, ...]. Free with free_int_array.

IntArray definitionregion_positions_sorted(const DefinitionRegionWrapper* ptr);
// Same as positions but sorted. Free with free_int_array.

CBlockArray definitionregion_blocks(
    const DefinitionRegionWrapper* ptr, const SchematicWrapper* schematic);
// Returns all blocks within the region. Free with free_block_array.
```

### Metadata

```c
int   definitionregion_set_metadata(DefinitionRegionWrapper* ptr,
    const char* key, const char* value);
int   definitionregion_set_metadata_mut(DefinitionRegionWrapper* ptr,
    const char* key, const char* value);  // alias
char* definitionregion_get_metadata(const DefinitionRegionWrapper* ptr, const char* key);
// Returns NULL if key not found. Free with free_string.

char*       definitionregion_get_all_metadata(const DefinitionRegionWrapper* ptr);
// Returns JSON object string. Free with free_string.

StringArray definitionregion_metadata_keys(const DefinitionRegionWrapper* ptr);
// Free with free_string_array.
```

### Sync Back to Schematic

```c
int definitionregion_sync(
    const DefinitionRegionWrapper* ptr,
    SchematicWrapper* schematic,
    const char* name
);
```

Inserts (or replaces) the definition region back into the schematic under the given name. Returns `0` on success.

---

## 19 -- Building Tools (Shapes, Brushes, Fill)

### Shapes

```c
ShapeWrapper* shape_sphere(int cx, int cy, int cz, float radius);
ShapeWrapper* shape_cuboid(int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
void          shape_free(ShapeWrapper* ptr);
```

### Brushes

```c
BrushWrapper* brush_solid(const char* block_name);
// A brush that fills every position with the same block.

BrushWrapper* brush_color(unsigned char r, unsigned char g, unsigned char b);
// A brush that picks the closest concrete/wool block to the given RGB color.

BrushWrapper* brush_linear_gradient(
    int x1, int y1, int z1, unsigned char r1, unsigned char g1, unsigned char b1,
    int x2, int y2, int z2, unsigned char r2, unsigned char g2, unsigned char b2,
    int space
);
// Linear color gradient between two points.
// space: 0 = RGB interpolation, 1 = Oklab interpolation.

BrushWrapper* brush_shaded(
    unsigned char r, unsigned char g, unsigned char b,
    float light_x, float light_y, float light_z
);
// Single color with directional shading based on surface normals.

BrushWrapper* brush_bilinear_gradient(
    int ox, int oy, int oz,   // origin
    int ux, int uy, int uz,   // U-axis endpoint
    int vx, int vy, int vz,   // V-axis endpoint
    unsigned char r00, unsigned char g00, unsigned char b00,  // color at origin
    unsigned char r10, unsigned char g10, unsigned char b10,  // color at U
    unsigned char r01, unsigned char g01, unsigned char b01,  // color at V
    unsigned char r11, unsigned char g11, unsigned char b11,  // color at U+V
    int space
);
// Bilinear interpolation across a quad defined by origin, U, and V directions.
// space: 0 = RGB, 1 = Oklab.

BrushWrapper* brush_point_gradient(
    const int*           positions,  // flat [x0,y0,z0, x1,y1,z1, ...]
    const unsigned char* colors,     // flat [r0,g0,b0, r1,g1,b1, ...]
    size_t count,
    float  falloff,
    int    space
);
// Multi-point gradient. Each point has a position and color.
// falloff controls how quickly influence drops off with distance.
// space: 0 = RGB, 1 = Oklab.

void brush_free(BrushWrapper* ptr);
```

### Fill

```c
int buildingtool_fill(
    SchematicWrapper*    schematic,
    const ShapeWrapper*  shape,
    const BrushWrapper*  brush
);
```

Fills the given shape using the given brush. Returns `0` on success, `-1` on NULL args.

### Convenience Fill Functions

```c
int schematic_fill_cuboid(
    SchematicWrapper* schematic,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z,
    const char* block_name
);

int schematic_fill_sphere(
    SchematicWrapper* schematic,
    int cx, int cy, int cz,
    float radius,
    const char* block_name
);
```

Both return `0` on success, `-1` on NULL args. These create a solid brush internally.

---

## 20 -- SchematicBuilder

A builder pattern for creating schematics from layer-based templates.

### schematicbuilder_new

```c
SchematicBuilderWrapper* schematicbuilder_new(void);
```

Creates a new empty builder. **Free with `schematicbuilder_free`.**

### schematicbuilder_free

```c
void schematicbuilder_free(SchematicBuilderWrapper* ptr);
```

### schematicbuilder_name

```c
int schematicbuilder_name(SchematicBuilderWrapper* ptr, const char* name);
```

Sets the schematic name. Returns `0` on success.

### schematicbuilder_map

```c
int schematicbuilder_map(SchematicBuilderWrapper* ptr, char ch, const char* block);
```

Maps a character to a block name for use in layer definitions. Returns `0` on success.

### schematicbuilder_layers

```c
int schematicbuilder_layers(SchematicBuilderWrapper* ptr, const char* layers_json);
```

Sets the layers from a JSON array of arrays of strings, e.g., `[["##","##"],["..",".."],["##","##"]]`. Each inner array is one row, each string is one layer. Returns `0` on success, `-2` on JSON parse error.

### schematicbuilder_build

```c
SchematicWrapper* schematicbuilder_build(SchematicBuilderWrapper* ptr);
```

Consumes the builder and returns a new schematic. Returns NULL on error. **The builder pointer is invalidated after this call -- do not free it.** **Free the schematic with `schematic_free`.**

### schematicbuilder_from_template

```c
SchematicBuilderWrapper* schematicbuilder_from_template(const char* template);
```

Creates a builder from a template string. Returns NULL on error. **Free with `schematicbuilder_free`.**

---

## 21 -- Sign Extraction and InSign

### schematic_extract_signs

```c
char* schematic_extract_signs(const SchematicWrapper* schematic);
```

Extracts all sign block entities and returns a JSON array, e.g., `[{"pos":[1,2,3],"text":["Line1","Line2",...]}, ...]`. **Free with `free_string`.**

### schematic_compile_insign

```c
char* schematic_compile_insign(const SchematicWrapper* schematic);
```

Compiles InSign commands found in sign text. Returns a JSON result string, or NULL on error. **Free with `free_string`.**

---

## 22 -- Debug and Print Utilities

### schematic_debug_info

```c
char* schematic_debug_info(const SchematicWrapper* schematic);
```

Returns a short status string like `"Schematic name: MyBuild, Regions: 3"`. Returns NULL on NULL input. **Free with `free_string`.**

### schematic_print

```c
char* schematic_print(const SchematicWrapper* schematic);
```

Returns a formatted visual layout of the schematic. **Free with `free_string`.**

### schematic_print_schematic

```c
char* schematic_print_schematic(const SchematicWrapper* schematic);
```

Same as `schematic_print`. **Free with `free_string`.**

### debug_schematic

```c
char* debug_schematic(const SchematicWrapper* schematic);
```

Returns debug info plus a visual layout. **Free with `free_string`.**

### debug_json_schematic

```c
char* debug_json_schematic(const SchematicWrapper* schematic);
```

Returns debug info plus a JSON-formatted layout. **Free with `free_string`.**

---

## 23 -- Meshing (feature = "meshing")

All meshing functions require building with `--features ffi,meshing`. Wrapper types are opaque pointers.

### Resource Pack

```c
// Lifecycle
FFIResourcePack* resourcepack_from_bytes(const unsigned char* data, size_t data_len);
void             resourcepack_free(FFIResourcePack* ptr);

// Stats
size_t      resourcepack_blockstate_count(const FFIResourcePack* ptr);
size_t      resourcepack_model_count(const FFIResourcePack* ptr);
size_t      resourcepack_texture_count(const FFIResourcePack* ptr);
StringArray resourcepack_namespaces(const FFIResourcePack* ptr);
// Free StringArray with free_string_array.

// Listing
StringArray resourcepack_list_blockstates(const FFIResourcePack* ptr);
StringArray resourcepack_list_models(const FFIResourcePack* ptr);
StringArray resourcepack_list_textures(const FFIResourcePack* ptr);
// Free each StringArray with free_string_array.

// Query
char*     resourcepack_get_blockstate_json(const FFIResourcePack* ptr, const char* name);
char*     resourcepack_get_model_json(const FFIResourcePack* ptr, const char* name);
IntArray  resourcepack_get_texture_info(const FFIResourcePack* ptr, const char* name);
// Returns [width, height, animated (0/1), frame_count]. Free with free_int_array.
ByteArray resourcepack_get_texture_pixels(const FFIResourcePack* ptr, const char* name);
// Returns raw RGBA pixel data. Free with free_byte_array.
// Free char* returns with free_string.

// Mutation
int resourcepack_add_blockstate_json(FFIResourcePack* ptr,
    const char* name, const char* json);
int resourcepack_add_model_json(FFIResourcePack* ptr,
    const char* name, const char* json);
int resourcepack_add_texture(FFIResourcePack* ptr,
    const char* name, uint32_t width, uint32_t height,
    const unsigned char* pixels, size_t pixels_len);
// All return 0 on success, -1 on NULL, -2 on error.
```

### Mesh Config

```c
FFIMeshConfig* meshconfig_new(void);
void           meshconfig_free(FFIMeshConfig* ptr);

// Setters
void meshconfig_set_cull_hidden_faces(FFIMeshConfig* ptr, int val);    // 0/1
void meshconfig_set_cull_occluded_blocks(FFIMeshConfig* ptr, int val); // 0/1
void meshconfig_set_ambient_occlusion(FFIMeshConfig* ptr, int val);    // 0/1
void meshconfig_set_ao_intensity(FFIMeshConfig* ptr, float val);
void meshconfig_set_greedy_meshing(FFIMeshConfig* ptr, int val);       // 0/1
void meshconfig_set_biome(FFIMeshConfig* ptr, const char* biome);      // NULL to clear
void meshconfig_set_atlas_max_size(FFIMeshConfig* ptr, uint32_t size);

// Getters
int     meshconfig_get_cull_hidden_faces(const FFIMeshConfig* ptr);
int     meshconfig_get_cull_occluded_blocks(const FFIMeshConfig* ptr);
int     meshconfig_get_ambient_occlusion(const FFIMeshConfig* ptr);
float   meshconfig_get_ao_intensity(const FFIMeshConfig* ptr);
int     meshconfig_get_greedy_meshing(const FFIMeshConfig* ptr);
char*   meshconfig_get_biome(const FFIMeshConfig* ptr);
// Free with free_string. Returns NULL if no biome set.
uint32_t meshconfig_get_atlas_max_size(const FFIMeshConfig* ptr);
```

### Mesh Generation

```c
FFIMeshResult*      schematic_to_mesh(
    const SchematicWrapper*, const FFIResourcePack*, const FFIMeshConfig*);
FFIMultiMeshResult* schematic_mesh_by_region(
    const SchematicWrapper*, const FFIResourcePack*, const FFIMeshConfig*);
FFIChunkMeshResult* schematic_mesh_by_chunk(
    const SchematicWrapper*, const FFIResourcePack*, const FFIMeshConfig*);
FFIChunkMeshResult* schematic_mesh_by_chunk_size(
    const SchematicWrapper*, const FFIResourcePack*, const FFIMeshConfig*, int chunk_size);
FFIMeshResult*      schematic_to_usdz(
    const SchematicWrapper*, const FFIResourcePack*, const FFIMeshConfig*);
FFIRawMeshExport*   schematic_to_raw_mesh(
    const SchematicWrapper*, const FFIResourcePack*, const FFIMeshConfig*);
```

All return NULL on error. Free with the corresponding `*_free` function.

### MeshResult

```c
void        meshresult_free(FFIMeshResult* ptr);
ByteArray   meshresult_glb_data(const FFIMeshResult* ptr);   // Free with free_byte_array
size_t      meshresult_vertex_count(const FFIMeshResult* ptr);
size_t      meshresult_triangle_count(const FFIMeshResult* ptr);
int         meshresult_has_transparency(const FFIMeshResult* ptr);  // 0/1
CFloatArray meshresult_bounds(const FFIMeshResult* ptr);     // Free with free_float_array
```

### MultiMeshResult

```c
void           multimeshresult_free(FFIMultiMeshResult* ptr);
StringArray    multimeshresult_region_names(const FFIMultiMeshResult* ptr);
// Free with free_string_array
FFIMeshResult* multimeshresult_get_mesh(const FFIMultiMeshResult* ptr, const char* region_name);
// Free returned mesh with meshresult_free
size_t         multimeshresult_total_vertex_count(const FFIMultiMeshResult* ptr);
size_t         multimeshresult_total_triangle_count(const FFIMultiMeshResult* ptr);
size_t         multimeshresult_mesh_count(const FFIMultiMeshResult* ptr);
```

### ChunkMeshResult

```c
void           chunkmeshresult_free(FFIChunkMeshResult* ptr);
IntArray       chunkmeshresult_chunk_coordinates(const FFIChunkMeshResult* ptr);
// Flat [cx0,cy0,cz0, cx1,cy1,cz1, ...]. Free with free_int_array.
FFIMeshResult* chunkmeshresult_get_mesh(const FFIChunkMeshResult* ptr, int cx, int cy, int cz);
// Free returned mesh with meshresult_free
size_t         chunkmeshresult_total_vertex_count(const FFIChunkMeshResult* ptr);
size_t         chunkmeshresult_total_triangle_count(const FFIChunkMeshResult* ptr);
size_t         chunkmeshresult_chunk_count(const FFIChunkMeshResult* ptr);
```

### RawMeshExport

```c
void        rawmeshexport_free(FFIRawMeshExport* ptr);
size_t      rawmeshexport_vertex_count(const FFIRawMeshExport* ptr);
size_t      rawmeshexport_triangle_count(const FFIRawMeshExport* ptr);
CFloatArray rawmeshexport_positions(const FFIRawMeshExport* ptr);   // [x,y,z,...] free_float_array
CFloatArray rawmeshexport_normals(const FFIRawMeshExport* ptr);     // [nx,ny,nz,...] free_float_array
CFloatArray rawmeshexport_uvs(const FFIRawMeshExport* ptr);         // [u,v,...] free_float_array
CFloatArray rawmeshexport_colors(const FFIRawMeshExport* ptr);      // [r,g,b,a,...] free_float_array
IntArray    rawmeshexport_indices(const FFIRawMeshExport* ptr);      // free_int_array
ByteArray   rawmeshexport_texture_rgba(const FFIRawMeshExport* ptr); // free_byte_array
uint32_t    rawmeshexport_texture_width(const FFIRawMeshExport* ptr);
uint32_t    rawmeshexport_texture_height(const FFIRawMeshExport* ptr);
```

---

## 24 -- C Code Examples

### Example 1: Create a schematic, place blocks, and save

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Forward declarations (from nucleation FFI)
typedef void SchematicWrapper;
typedef void BlockStateWrapper;
typedef struct { unsigned char* data; size_t len; } ByteArray;
typedef struct { char** data; size_t len; } StringArray;
typedef struct { int* data; size_t len; } IntArray;
typedef struct { char* key; char* value; } CProperty;

extern SchematicWrapper* schematic_new(void);
extern void schematic_free(SchematicWrapper* ptr);
extern int  schematic_set_block(SchematicWrapper* ptr, int x, int y, int z, const char* name);
extern int  schematic_set_block_with_properties(SchematicWrapper* ptr,
    int x, int y, int z, const char* name, const CProperty* props, size_t props_len);
extern int  schematic_set_block_from_string(SchematicWrapper* ptr,
    int x, int y, int z, const char* block_string);
extern int  schematic_set_blocks(SchematicWrapper* ptr,
    const int* positions, size_t positions_len, const char* block_name);
extern void schematic_set_name(SchematicWrapper* ptr, const char* name);
extern void schematic_set_author(SchematicWrapper* ptr, const char* author);
extern ByteArray schematic_to_litematic(const SchematicWrapper* ptr);
extern ByteArray schematic_to_schematic(const SchematicWrapper* ptr);
extern ByteArray schematic_save_as(const SchematicWrapper* ptr,
    const char* format, const char* version);
extern void free_byte_array(ByteArray arr);
extern void free_string(char* ptr);
extern char* schematic_debug_info(const SchematicWrapper* ptr);
extern int schematic_get_block_count(const SchematicWrapper* ptr);
extern IntArray schematic_get_dimensions(const SchematicWrapper* ptr);
extern void free_int_array(IntArray arr);

int main(void) {
    // Create a new schematic
    SchematicWrapper* sch = schematic_new();
    schematic_set_name(sch, "My First Build");
    schematic_set_author(sch, "Builder");

    // Place individual blocks
    schematic_set_block(sch, 0, 0, 0, "minecraft:stone");
    schematic_set_block(sch, 1, 0, 0, "minecraft:stone");
    schematic_set_block(sch, 2, 0, 0, "minecraft:stone");

    // Place a block with properties
    CProperty stair_props[] = {
        { "facing", "north" },
        { "half", "bottom" },
        { "shape", "straight" }
    };
    schematic_set_block_with_properties(sch, 0, 1, 0,
        "minecraft:oak_stairs", stair_props, 3);

    // Place a block from a full block string
    schematic_set_block_from_string(sch, 1, 1, 0,
        "minecraft:oak_stairs[facing=east,half=bottom,shape=straight]");

    // Batch-set a floor of 100 blocks
    int floor_positions[300]; // 100 * 3
    for (int i = 0; i < 10; i++) {
        for (int j = 0; j < 10; j++) {
            int idx = (i * 10 + j) * 3;
            floor_positions[idx]     = i;
            floor_positions[idx + 1] = -1;
            floor_positions[idx + 2] = j;
        }
    }
    int placed = schematic_set_blocks(sch, floor_positions, 300, "minecraft:oak_planks");
    printf("Placed %d floor blocks\n", placed);

    // Query info
    char* info = schematic_debug_info(sch);
    printf("Debug: %s\n", info);
    free_string(info);

    printf("Block count: %d\n", schematic_get_block_count(sch));

    IntArray dims = schematic_get_dimensions(sch);
    if (dims.data) {
        printf("Dimensions: %d x %d x %d\n", dims.data[0], dims.data[1], dims.data[2]);
        free_int_array(dims);
    }

    // Save as Litematic
    ByteArray litematic_data = schematic_to_litematic(sch);
    if (litematic_data.data) {
        FILE* f = fopen("output.litematic", "wb");
        fwrite(litematic_data.data, 1, litematic_data.len, f);
        fclose(f);
        printf("Saved litematic (%zu bytes)\n", litematic_data.len);
        free_byte_array(litematic_data);
    }

    // Save as Sponge Schematic
    ByteArray schem_data = schematic_to_schematic(sch);
    if (schem_data.data) {
        FILE* f = fopen("output.schem", "wb");
        fwrite(schem_data.data, 1, schem_data.len, f);
        fclose(f);
        printf("Saved schematic (%zu bytes)\n", schem_data.len);
        free_byte_array(schem_data);
    }

    schematic_free(sch);
    return 0;
}
```

### Example 2: Load a file, query blocks, convert formats

```c
#include <stdio.h>
#include <stdlib.h>

typedef void SchematicWrapper;
typedef struct { unsigned char* data; size_t len; } ByteArray;
typedef struct { char** data; size_t len; } StringArray;
typedef struct { int* data; size_t len; } IntArray;
typedef struct {
    int x, y, z;
    char* name;
    char* properties_json;
} CBlock;
typedef struct { CBlock* data; size_t len; } CBlockArray;

extern SchematicWrapper* schematic_new(void);
extern void schematic_free(SchematicWrapper* ptr);
extern int  schematic_from_data(SchematicWrapper* ptr,
    const unsigned char* data, size_t data_len);
extern char* schematic_get_block(const SchematicWrapper* ptr, int x, int y, int z);
extern char* schematic_get_block_string(const SchematicWrapper* ptr, int x, int y, int z);
extern CBlockArray schematic_get_all_blocks(const SchematicWrapper* ptr);
extern StringArray schematic_get_palette(const SchematicWrapper* ptr);
extern StringArray schematic_get_region_names(const SchematicWrapper* ptr);
extern IntArray schematic_get_dimensions(const SchematicWrapper* ptr);
extern IntArray schematic_get_tight_dimensions(const SchematicWrapper* ptr);
extern int schematic_get_block_count(const SchematicWrapper* ptr);
extern int schematic_get_volume(const SchematicWrapper* ptr);
extern ByteArray schematic_save_as(const SchematicWrapper* ptr,
    const char* format, const char* version);
extern void free_string(char* ptr);
extern void free_string_array(StringArray arr);
extern void free_int_array(IntArray arr);
extern void free_block_array(CBlockArray arr);
extern void free_byte_array(ByteArray arr);

int main(int argc, char* argv[]) {
    if (argc < 2) {
        printf("Usage: %s <schematic_file>\n", argv[0]);
        return 1;
    }

    // Read file into memory
    FILE* f = fopen(argv[1], "rb");
    if (!f) { perror("fopen"); return 1; }
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char* file_data = malloc(fsize);
    fread(file_data, 1, fsize, f);
    fclose(f);

    // Load with auto-detection
    SchematicWrapper* sch = schematic_new();
    int rc = schematic_from_data(sch, file_data, fsize);
    free(file_data);

    if (rc != 0) {
        printf("Failed to load schematic (error: %d)\n", rc);
        schematic_free(sch);
        return 1;
    }

    // Print dimensions
    IntArray dims = schematic_get_dimensions(sch);
    if (dims.data) {
        printf("Allocated dimensions: %d x %d x %d\n",
            dims.data[0], dims.data[1], dims.data[2]);
        free_int_array(dims);
    }

    IntArray tight = schematic_get_tight_dimensions(sch);
    if (tight.data) {
        printf("Tight dimensions: %d x %d x %d\n",
            tight.data[0], tight.data[1], tight.data[2]);
        free_int_array(tight);
    }

    printf("Block count: %d\n", schematic_get_block_count(sch));
    printf("Volume: %d\n", schematic_get_volume(sch));

    // List regions
    StringArray regions = schematic_get_region_names(sch);
    printf("Regions (%zu):\n", regions.len);
    for (size_t i = 0; i < regions.len; i++) {
        printf("  - %s\n", regions.data[i]);
    }
    free_string_array(regions);

    // List palette
    StringArray palette = schematic_get_palette(sch);
    printf("Palette (%zu entries):\n", palette.len);
    for (size_t i = 0; i < palette.len; i++) {
        printf("  [%zu] %s\n", i, palette.data[i]);
    }
    free_string_array(palette);

    // Query a specific block
    char* block = schematic_get_block(sch, 0, 0, 0);
    if (block) {
        printf("Block at (0,0,0): %s\n", block);
        free_string(block);
    }

    char* block_full = schematic_get_block_string(sch, 0, 0, 0);
    if (block_full) {
        printf("Full block string at (0,0,0): %s\n", block_full);
        free_string(block_full);
    }

    // Convert to a different format
    ByteArray converted = schematic_save_as(sch, "litematic", NULL);
    if (converted.data) {
        FILE* out = fopen("converted.litematic", "wb");
        fwrite(converted.data, 1, converted.len, out);
        fclose(out);
        printf("Converted to litematic (%zu bytes)\n", converted.len);
        free_byte_array(converted);
    }

    schematic_free(sch);
    return 0;
}
```

### Example 3: Using building tools

```c
#include <stdio.h>
#include <stdlib.h>

typedef void SchematicWrapper;
typedef void ShapeWrapper;
typedef void BrushWrapper;
typedef struct { unsigned char* data; size_t len; } ByteArray;

extern SchematicWrapper* schematic_new(void);
extern void schematic_free(SchematicWrapper* ptr);
extern void schematic_set_name(SchematicWrapper* ptr, const char* name);

// Convenience fills
extern int schematic_fill_cuboid(SchematicWrapper* ptr,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z,
    const char* block_name);
extern int schematic_fill_sphere(SchematicWrapper* ptr,
    int cx, int cy, int cz, float radius, const char* block_name);

// Shape/brush/fill API
extern ShapeWrapper* shape_sphere(int cx, int cy, int cz, float radius);
extern ShapeWrapper* shape_cuboid(int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z);
extern void shape_free(ShapeWrapper* ptr);

extern BrushWrapper* brush_solid(const char* block_name);
extern BrushWrapper* brush_color(unsigned char r, unsigned char g, unsigned char b);
extern BrushWrapper* brush_linear_gradient(
    int x1, int y1, int z1, unsigned char r1, unsigned char g1, unsigned char b1,
    int x2, int y2, int z2, unsigned char r2, unsigned char g2, unsigned char b2,
    int space);
extern void brush_free(BrushWrapper* ptr);

extern int buildingtool_fill(SchematicWrapper* ptr,
    const ShapeWrapper* shape, const BrushWrapper* brush);

extern ByteArray schematic_to_litematic(const SchematicWrapper* ptr);
extern void free_byte_array(ByteArray arr);

int main(void) {
    SchematicWrapper* sch = schematic_new();
    schematic_set_name(sch, "Building Tools Demo");

    // Quick fill: stone platform
    schematic_fill_cuboid(sch, -10, -1, -10, 10, -1, 10, "minecraft:stone_bricks");

    // Quick fill: glass dome
    schematic_fill_sphere(sch, 0, 5, 0, 8.0f, "minecraft:glass");

    // Advanced: gradient sphere using shape + brush API
    ShapeWrapper* sphere = shape_sphere(0, 15, 0, 6.0f);

    // Linear gradient from red to blue (Oklab interpolation)
    BrushWrapper* gradient = brush_linear_gradient(
        0, 9, 0,   255, 0, 0,    // bottom: red
        0, 21, 0,  0, 0, 255,    // top: blue
        1                         // Oklab space
    );

    buildingtool_fill(sch, sphere, gradient);

    // Another sphere with a solid color brush
    ShapeWrapper* sphere2 = shape_sphere(20, 5, 0, 4.0f);
    BrushWrapper* solid = brush_solid("minecraft:gold_block");
    buildingtool_fill(sch, sphere2, solid);

    // Color brush: picks the closest concrete color
    ShapeWrapper* cube = shape_cuboid(-5, 0, 15, 5, 10, 25);
    BrushWrapper* color = brush_color(128, 0, 200); // purple-ish
    buildingtool_fill(sch, cube, color);

    // Cleanup brushes and shapes
    shape_free(sphere);
    shape_free(sphere2);
    shape_free(cube);
    brush_free(gradient);
    brush_free(solid);
    brush_free(color);

    // Save
    ByteArray data = schematic_to_litematic(sch);
    if (data.data) {
        FILE* f = fopen("tools_demo.litematic", "wb");
        fwrite(data.data, 1, data.len, f);
        fclose(f);
        free_byte_array(data);
    }

    schematic_free(sch);
    return 0;
}
```

### Example 4: Using definition regions

```c
#include <stdio.h>
#include <stdlib.h>

typedef void SchematicWrapper;
typedef void DefinitionRegionWrapper;
typedef struct { char** data; size_t len; } StringArray;
typedef struct { int* data; size_t len; } IntArray;
typedef struct {
    int min_x, min_y, min_z;
    int max_x, max_y, max_z;
} CBoundingBox;
typedef struct {
    int x, y, z;
    char* name;
    char* properties_json;
} CBlock;
typedef struct { CBlock* data; size_t len; } CBlockArray;

extern SchematicWrapper* schematic_new(void);
extern void schematic_free(SchematicWrapper* ptr);
extern int  schematic_set_block(SchematicWrapper* ptr, int x, int y, int z, const char* name);
extern int  schematic_fill_cuboid(SchematicWrapper* ptr,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z,
    const char* block_name);

// Definition region (standalone)
extern DefinitionRegionWrapper* definitionregion_new(void);
extern DefinitionRegionWrapper* definitionregion_from_bounds(
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
extern void definitionregion_free(DefinitionRegionWrapper* ptr);
extern int  definitionregion_add_bounds(DefinitionRegionWrapper* ptr,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
extern int  definitionregion_set_metadata(DefinitionRegionWrapper* ptr,
    const char* key, const char* value);
extern int  definitionregion_volume(const DefinitionRegionWrapper* ptr);
extern int  definitionregion_contains(const DefinitionRegionWrapper* ptr, int x, int y, int z);
extern CBoundingBox definitionregion_get_bounds(const DefinitionRegionWrapper* ptr);
extern IntArray definitionregion_dimensions(const DefinitionRegionWrapper* ptr);
extern CBlockArray definitionregion_blocks(const DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic);

// Filter / set operations
extern DefinitionRegionWrapper* definitionregion_filter_by_block(
    const DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic, const char* block_name);
extern DefinitionRegionWrapper* definitionregion_intersect(
    const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
extern DefinitionRegionWrapper* definitionregion_subtract(
    const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);

// Schematic-attached
extern int  schematic_create_definition_region_from_bounds(SchematicWrapper* ptr,
    const char* name, int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
extern int  schematic_definition_region_set_metadata(SchematicWrapper* ptr,
    const char* name, const char* key, const char* value);
extern StringArray schematic_get_definition_region_names(const SchematicWrapper* ptr);
extern int  definitionregion_sync(const DefinitionRegionWrapper* ptr,
    SchematicWrapper* schematic, const char* name);

extern void free_string_array(StringArray arr);
extern void free_int_array(IntArray arr);
extern void free_block_array(CBlockArray arr);

int main(void) {
    SchematicWrapper* sch = schematic_new();

    // Build a structure
    schematic_fill_cuboid(sch, 0, 0, 0, 20, 10, 20, "minecraft:stone");
    schematic_fill_cuboid(sch, 5, 0, 5, 15, 10, 15, "minecraft:oak_planks");
    schematic_fill_cuboid(sch, 8, 0, 8, 12, 10, 12, "minecraft:glass");

    // Create a definition region covering the outer shell
    DefinitionRegionWrapper* outer = definitionregion_from_bounds(0, 0, 0, 20, 10, 20);
    DefinitionRegionWrapper* inner = definitionregion_from_bounds(1, 1, 1, 19, 9, 19);
    DefinitionRegionWrapper* shell = definitionregion_subtract(outer, inner);

    printf("Shell volume: %d\n", definitionregion_volume(shell));
    printf("Contains (0,0,0): %d\n", definitionregion_contains(shell, 0, 0, 0));
    printf("Contains (10,5,10): %d\n", definitionregion_contains(shell, 10, 5, 10));

    // Filter: find only stone blocks in the shell
    DefinitionRegionWrapper* stone_shell = definitionregion_filter_by_block(
        shell, sch, "minecraft:stone");
    printf("Stone blocks in shell: %d\n", definitionregion_volume(stone_shell));

    // Get the blocks within the filtered region
    CBlockArray blocks = definitionregion_blocks(stone_shell, sch);
    printf("First 3 blocks:\n");
    for (size_t i = 0; i < 3 && i < blocks.len; i++) {
        printf("  (%d,%d,%d) = %s\n",
            blocks.data[i].x, blocks.data[i].y, blocks.data[i].z,
            blocks.data[i].name);
    }
    free_block_array(blocks);

    // Attach a region to the schematic with metadata
    definitionregion_set_metadata(shell, "purpose", "outer_shell");
    definitionregion_set_metadata(shell, "material", "mixed");
    definitionregion_sync(shell, sch, "shell");

    // Also create directly on the schematic
    schematic_create_definition_region_from_bounds(sch, "interior",
        5, 0, 5, 15, 10, 15);
    schematic_definition_region_set_metadata(sch, "interior", "purpose", "room");

    // List all definition regions
    StringArray names = schematic_get_definition_region_names(sch);
    printf("Definition regions (%zu):\n", names.len);
    for (size_t i = 0; i < names.len; i++) {
        printf("  - %s\n", names.data[i]);
    }
    free_string_array(names);

    // Cleanup
    definitionregion_free(outer);
    definitionregion_free(inner);
    definitionregion_free(shell);
    definitionregion_free(stone_shell);
    schematic_free(sch);
    return 0;
}
```

---

## Important Notes and Gotchas

* **Thread safety** -- The library is generally not thread-safe. Do not mutate the same `SchematicWrapper` from multiple threads without external synchronization.
* **NULL-checking** -- All functions guard against NULL pointers and return safe defaults (`0`, `-1`, `NULL`, or empty arrays).
* **Error codes** -- By convention, `0` means success. Negative values indicate errors: `-1` for NULL arguments, `-2` for operation failures, `-3` for unknown formats.
* **Memory ownership** -- Every heap-allocated return value has a corresponding `free_*` function. Do not use C `free()` on Rust-allocated memory. The only exception is `CBoundingBox`, which is returned by value on the stack.
* **Opaque pointers** -- Never dereference `SchematicWrapper*`, `DefinitionRegionWrapper*`, etc. directly. They are Rust heap objects.
* **Meshing FFI** -- Meshing functions are only available when built with `--features ffi,meshing`. All meshing wrapper pointers must be freed with their corresponding `*_free()` functions.
* **SchematicBuilder** -- The `schematicbuilder_build` function **consumes** the builder. Do not call `schematicbuilder_free` on a builder that has already been built.
