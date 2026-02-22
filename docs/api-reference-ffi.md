# Nucleation C FFI API Reference

Nucleation provides a comprehensive C Foreign Function Interface for embedding in any language that supports C interop (C, C++, Swift, C#, Go, etc.). All functions use `extern "C"` linkage and `#[repr(C)]` data structures.

---

## Table of Contents

- [Conventions](#conventions)
- [Data Structures](#data-structures)
- [Error Handling](#error-handling)
- [Memory Management](#memory-management)
- [Schematic Lifecycle](#schematic-lifecycle)
- [Format I/O](#format-io)
  - [Auto-Detection](#auto-detection)
  - [Litematic](#litematic)
  - [Sponge Schematic](#sponge-schematic)
  - [McStructure (Bedrock)](#mcstructure-bedrock)
  - [MCA Region Files](#mca-region-files)
  - [World ZIP](#world-zip)
  - [World Directory](#world-directory)
  - [Format Discovery](#format-discovery)
  - [Snapshot](#snapshot)
  - [File I/O](#file-io)
  - [Generic Export](#generic-export)
- [Block Operations](#block-operations)
  - [Single Block](#single-block)
  - [Batch Operations](#batch-operations)
  - [Block Querying](#block-querying)
- [BlockState](#blockstate)
- [Block Entities](#block-entities)
- [Mobile Entities](#mobile-entities)
- [Metadata](#metadata)
- [Dimensions & Bounds](#dimensions--bounds)
- [Palette Access](#palette-access)
- [Transformations](#transformations)
- [Region Copying](#region-copying)
- [Chunk Iteration](#chunk-iteration)
- [Definition Regions](#definition-regions)
  - [Creation & Lifecycle](#creation--lifecycle)
  - [Schematic Region Operations](#schematic-region-operations)
  - [Standalone Region Operations](#standalone-region-operations)
  - [Querying](#querying)
  - [Set Operations](#set-operations)
  - [Filtering](#filtering)
  - [Manipulation](#manipulation)
- [Building Tools](#building-tools)
  - [Shapes](#shapes)
  - [Brushes](#brushes)
  - [Fill](#fill)
- [Schematic Builder](#schematic-builder)
- [Sign Text & Insign](#sign-text--insign)
- [Debug & Utility](#debug--utility)
- [Simulation](#simulation-feature-gated) *(feature: `simulation`)*
  - [MchprsWorld](#mchprsworld)
  - [Value Types](#value-types)
  - [IoType](#iotype)
  - [LayoutFunction](#layoutfunction)
  - [OutputCondition](#outputcondition)
  - [ExecutionMode](#executionmode)
  - [SortStrategy](#sortstrategy)

---

## Conventions

### Return Codes

All functions returning `int` use a consistent error code scheme:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `-1` | Null pointer input |
| `-2` | Operation error (parse failure, region not found, etc.) |
| `-3` | Format-specific error (bad JSON, unknown format) |

For detailed error messages after a failure, call `schematic_last_error()`.

### Ownership Rules

- **Caller-owned**: All returned pointers, arrays, and strings must be freed by the caller using the appropriate `free_*` function.
- **Borrowed**: Pointer parameters are borrowed (not consumed) unless explicitly documented otherwise.
- **Opaque types**: `SchematicWrapper*`, `BlockStateWrapper*`, `DefinitionRegionWrapper*`, `ShapeWrapper*`, `BrushWrapper*`, `SchematicBuilderWrapper*`, and simulation types are opaque — access only through API functions.

### String Encoding

All strings are null-terminated UTF-8 encoded `char*`. Block names follow the Minecraft `namespace:block_name` format (e.g., `"minecraft:stone"`).

---

## Data Structures

```c
// Generic arrays
typedef struct {
    unsigned char* data;
    size_t len;
} ByteArray;

typedef struct {
    char** data;
    size_t len;
} StringArray;

typedef struct {
    int* data;
    size_t len;
} IntArray;

typedef struct {
    float* data;
    size_t len;
} CFloatArray;

// Block property pair
typedef struct {
    char* key;
    char* value;
} CProperty;

typedef struct {
    CProperty* data;
    size_t len;
} CPropertyArray;

// Block data (from queries)
typedef struct {
    int x, y, z;
    char* name;
    char* properties_json;  // JSON string of properties, or NULL
} CBlock;

typedef struct {
    CBlock* data;
    size_t len;
} CBlockArray;

// Block entity (tile entity)
typedef struct {
    char* id;
    int x, y, z;
    char* nbt_json;  // JSON string of NBT data
} CBlockEntity;

typedef struct {
    CBlockEntity* data;
    size_t len;
} CBlockEntityArray;

// Mobile entity
typedef struct {
    char* id;
    double x, y, z;
    char* nbt_json;
} CEntity;

typedef struct {
    CEntity* data;
    size_t len;
} CEntityArray;

// Chunk
typedef struct {
    int chunk_x, chunk_y, chunk_z;
    CBlockArray blocks;
} CChunk;

typedef struct {
    CChunk* data;
    size_t len;
} CChunkArray;

// World export file map
typedef struct {
    char* path;
    unsigned char* data;
    size_t data_len;
} CFileEntry;

typedef struct {
    CFileEntry* entries;
    size_t len;
} CFileMap;

// Bounding box
typedef struct {
    int min_x, min_y, min_z;
    int max_x, max_y, max_z;
} CBoundingBox;

// Opaque types (forward declarations)
typedef struct SchematicWrapper SchematicWrapper;
typedef struct BlockStateWrapper BlockStateWrapper;
typedef struct DefinitionRegionWrapper DefinitionRegionWrapper;
typedef struct ShapeWrapper ShapeWrapper;
typedef struct BrushWrapper BrushWrapper;
typedef struct SchematicBuilderWrapper SchematicBuilderWrapper;

// Simulation opaque types (feature-gated)
typedef struct MchprsWorldWrapper MchprsWorldWrapper;
typedef struct ValueWrapper ValueWrapper;
typedef struct IoTypeWrapper IoTypeWrapper;
typedef struct LayoutFunctionWrapper LayoutFunctionWrapper;
typedef struct OutputConditionWrapper OutputConditionWrapper;
typedef struct ExecutionModeWrapper ExecutionModeWrapper;
typedef struct SortStrategyWrapper SortStrategyWrapper;
```

---

## Error Handling

```c
// Get the last error message. Returns NULL if no error.
// Caller must free with free_string().
char* schematic_last_error(void);
```

Always check return codes. On failure, call `schematic_last_error()` for a human-readable message.

---

## Memory Management

Every returned array, string, or opaque pointer must be freed by the caller.

```c
void free_byte_array(ByteArray array);
void free_string_array(StringArray array);
void free_int_array(IntArray array);
void free_float_array(CFloatArray array);
void free_string(char* string);
void free_property_array(CPropertyArray array);
void free_block_array(CBlockArray array);
void free_block_entity_array(CBlockEntityArray array);
void free_chunk_array(CChunkArray array);
void free_entity_array(CEntityArray array);
void free_file_map(CFileMap map);
```

---

## Schematic Lifecycle

```c
// Create a new empty schematic.
SchematicWrapper* schematic_new(void);

// Free a schematic and all owned data.
void schematic_free(SchematicWrapper* schematic);
```

---

## Format I/O

### Auto-Detection

```c
// Load from binary data, auto-detecting the format.
// Supports Litematic, Sponge Schematic, McStructure.
// Returns: 0 success, -1 null, -2 parse error, -3 unknown format
int schematic_from_data(
    SchematicWrapper* schematic,
    const unsigned char* data,
    size_t data_len
);
```

### Litematic

```c
int schematic_from_litematic(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);
ByteArray schematic_to_litematic(const SchematicWrapper* schematic);
```

### Sponge Schematic

```c
int schematic_from_schematic(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);
ByteArray schematic_to_schematic(const SchematicWrapper* schematic);

// Export to a specific Sponge Schematic version (e.g., "2", "3")
ByteArray schematic_to_schematic_version(const SchematicWrapper* schematic, const char* version);

// List available versions
StringArray schematic_get_available_schematic_versions(void);
```

### McStructure (Bedrock)

```c
int schematic_from_mcstructure(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);
ByteArray schematic_to_mcstructure(const SchematicWrapper* schematic);
```

### MCA Region Files

```c
// Load a single MCA region file
int schematic_from_mca(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);

// Load MCA with coordinate bounds
int schematic_from_mca_bounded(
    SchematicWrapper* schematic,
    const unsigned char* data, size_t data_len,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);
```

### World ZIP

```c
int schematic_from_world_zip(SchematicWrapper* schematic, const unsigned char* data, size_t data_len);

int schematic_from_world_zip_bounded(
    SchematicWrapper* schematic,
    const unsigned char* data, size_t data_len,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);

// Export as zipped world. options_json may be NULL.
ByteArray schematic_to_world_zip(const SchematicWrapper* schematic, const char* options_json);
```

### World Directory

```c
// Load from a world directory on disk
int schematic_from_world_directory(SchematicWrapper* schematic, const char* path);

int schematic_from_world_directory_bounded(
    SchematicWrapper* schematic, const char* path,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);

// Export as world file map (path -> bytes)
CFileMap schematic_to_world(const SchematicWrapper* schematic, const char* options_json);

// Write world files to a directory. Returns 0 on success.
int schematic_save_world(const SchematicWrapper* schematic, const char* directory, const char* options_json);
```

### Format Discovery

```c
StringArray schematic_get_supported_import_formats(void);
StringArray schematic_get_supported_export_formats(void);
StringArray schematic_get_format_versions(const char* format);

// Returns NULL if format not found. Caller must free_string().
char* schematic_get_default_format_version(const char* format);
char* schematic_get_export_settings_schema(const char* format);
char* schematic_get_import_settings_schema(const char* format);
```

### Snapshot

```c
// Export to fast binary snapshot format (.nusn)
ByteArray schematic_to_snapshot(const SchematicWrapper* schematic);

// Load from snapshot format
int schematic_from_snapshot(
    SchematicWrapper* schematic,
    const unsigned char* data,
    size_t data_len
);
```

### File I/O

```c
// Save to file with auto-detection from extension.
// format: format name or NULL for auto-detect from path extension
// version: format version or NULL for default
// Returns: 0 success, -1 null args, -2 serialize error, -3 IO error
int schematic_save_to_file(
    const SchematicWrapper* schematic,
    const char* path,
    const char* format,
    const char* version
);
```

### Generic Export

```c
// Export to any registered format.
// format: format name (e.g., "litematic", "schematic")
// version: format version or NULL for default
// settings: JSON settings string or NULL
ByteArray schematic_save_as(
    const SchematicWrapper* schematic,
    const char* format,
    const char* version,
    const char* settings
);
```

---

## Block Operations

### Single Block

```c
// Set block by simple name (no properties)
int schematic_set_block(SchematicWrapper* schematic, int x, int y, int z, const char* block_name);

// Set block with explicit properties
int schematic_set_block_with_properties(
    SchematicWrapper* schematic,
    int x, int y, int z,
    const char* block_name,
    const CProperty* properties,
    size_t properties_len
);

// Parse full block string: "minecraft:chest[facing=north]{Items:[...]}"
int schematic_set_block_from_string(SchematicWrapper* schematic, int x, int y, int z, const char* block_string);

// Set block with NBT data (JSON string)
int schematic_set_block_with_nbt(
    SchematicWrapper* schematic,
    int x, int y, int z,
    const char* block_name,
    const char* nbt_json
);

// Set block in a named region
int schematic_set_block_in_region(
    SchematicWrapper* schematic,
    const char* region_name,
    int x, int y, int z,
    const char* block_name
);
```

### Batch Operations

Positions are flat arrays: `[x0, y0, z0, x1, y1, z1, ...]` with `positions_len = count * 3`.

```c
// Set same block at multiple positions. Returns count set, or negative on error.
int schematic_set_blocks(
    SchematicWrapper* schematic,
    const int* positions, size_t positions_len,
    const char* block_name
);

// Get block names at multiple positions. NULL entries for empty/air.
StringArray schematic_get_blocks(
    const SchematicWrapper* schematic,
    const int* positions, size_t positions_len
);
```

### Block Querying

```c
// Get block name at position. Caller must free_string(). Returns NULL if empty.
char* schematic_get_block(const SchematicWrapper* schematic, int x, int y, int z);

// Get full block string with properties. Caller must free_string().
char* schematic_get_block_string(const SchematicWrapper* schematic, int x, int y, int z);

// Get block with properties. Caller must blockstate_free(). Returns NULL if empty.
BlockStateWrapper* schematic_get_block_with_properties(const SchematicWrapper* schematic, int x, int y, int z);

// Get all non-air blocks. Caller must free_block_array().
CBlockArray schematic_get_all_blocks(const SchematicWrapper* schematic);

// Get blocks in a sub-region. Caller must free_block_array().
CBlockArray schematic_get_chunk_blocks(
    const SchematicWrapper* schematic,
    int offset_x, int offset_y, int offset_z,
    int width, int height, int length
);
```

---

## BlockState

```c
// Create a new BlockState. Caller must blockstate_free().
BlockStateWrapper* blockstate_new(const char* name);

// Free a BlockState.
void blockstate_free(BlockStateWrapper* bs);

// Add a property. Returns a NEW BlockStateWrapper (original unchanged).
// Caller must free BOTH the original and the returned wrapper.
BlockStateWrapper* blockstate_with_property(
    BlockStateWrapper* block_state,
    const char* key,
    const char* value
);

// Get block name. Caller must free_string().
char* blockstate_get_name(const BlockStateWrapper* block_state);

// Get all properties. Caller must free_property_array().
CPropertyArray blockstate_get_properties(const BlockStateWrapper* block_state);
```

---

## Block Entities

```c
// Get block entity at position. Returns NULL if none.
// Caller should treat as a CBlockEntityArray of length 1 for freeing.
CBlockEntity* schematic_get_block_entity(const SchematicWrapper* schematic, int x, int y, int z);

// Get all block entities. Caller must free_block_entity_array().
CBlockEntityArray schematic_get_all_block_entities(const SchematicWrapper* schematic);
```

---

## Mobile Entities

```c
size_t schematic_entity_count(const SchematicWrapper* schematic);

// Caller must free_entity_array().
CEntityArray schematic_get_entities(const SchematicWrapper* schematic);

// nbt_json may be NULL.
int schematic_add_entity(SchematicWrapper* schematic, const char* id, double x, double y, double z, const char* nbt_json);

int schematic_remove_entity(SchematicWrapper* schematic, size_t index);
```

---

## Metadata

```c
// All getters return NULL if not set. Caller must free_string().
char* schematic_get_name(const SchematicWrapper* schematic);
void  schematic_set_name(SchematicWrapper* schematic, const char* name);

char* schematic_get_author(const SchematicWrapper* schematic);
void  schematic_set_author(SchematicWrapper* schematic, const char* author);

char* schematic_get_description(const SchematicWrapper* schematic);
void  schematic_set_description(SchematicWrapper* schematic, const char* description);

// Returns -1 if not set.
int64_t schematic_get_created(const SchematicWrapper* schematic);
void    schematic_set_created(SchematicWrapper* schematic, uint64_t created);

int64_t schematic_get_modified(const SchematicWrapper* schematic);
void    schematic_set_modified(SchematicWrapper* schematic, uint64_t modified);

// Returns -1 if not set.
int schematic_get_lm_version(const SchematicWrapper* schematic);
void schematic_set_lm_version(SchematicWrapper* schematic, int version);

int schematic_get_mc_version(const SchematicWrapper* schematic);
void schematic_set_mc_version(SchematicWrapper* schematic, int version);

int schematic_get_we_version(const SchematicWrapper* schematic);
void schematic_set_we_version(SchematicWrapper* schematic, int version);
```

---

## Dimensions & Bounds

```c
// Returns IntArray [width, height, length]. Caller must free_int_array().
IntArray schematic_get_dimensions(const SchematicWrapper* schematic);
IntArray schematic_get_allocated_dimensions(const SchematicWrapper* schematic);
IntArray schematic_get_tight_dimensions(const SchematicWrapper* schematic);

// Returns IntArray [x, y, z]. Caller must free_int_array().
IntArray schematic_get_tight_bounds_min(const SchematicWrapper* schematic);
IntArray schematic_get_tight_bounds_max(const SchematicWrapper* schematic);

int schematic_get_block_count(const SchematicWrapper* schematic);
int schematic_get_volume(const SchematicWrapper* schematic);

// Returns IntArray [min_x, min_y, min_z, max_x, max_y, max_z]. Caller must free_int_array().
IntArray schematic_get_bounding_box(const SchematicWrapper* schematic);
IntArray schematic_get_region_bounding_box(const SchematicWrapper* schematic, const char* region_name);

// Caller must free_string_array().
StringArray schematic_get_region_names(const SchematicWrapper* schematic);
```

---

## Palette Access

```c
// Get merged palette. Caller must free_string_array().
StringArray schematic_get_palette(const SchematicWrapper* schematic);

// Get default region palette. Caller must free_string_array().
StringArray schematic_get_default_region_palette(const SchematicWrapper* schematic);

// Get palette for a named region. Caller must free_string_array().
StringArray schematic_get_palette_from_region(const SchematicWrapper* schematic, const char* region_name);

// Get all palettes as JSON. Caller must free_string().
char* schematic_get_all_palettes(const SchematicWrapper* schematic);
```

---

## Transformations

### Whole Schematic

```c
int schematic_flip_x(SchematicWrapper* schematic);
int schematic_flip_y(SchematicWrapper* schematic);
int schematic_flip_z(SchematicWrapper* schematic);

// degrees must be 90, 180, or 270
int schematic_rotate_x(SchematicWrapper* schematic, int degrees);
int schematic_rotate_y(SchematicWrapper* schematic, int degrees);
int schematic_rotate_z(SchematicWrapper* schematic, int degrees);
```

### Named Region

```c
int schematic_flip_region_x(SchematicWrapper* schematic, const char* region_name);
int schematic_flip_region_y(SchematicWrapper* schematic, const char* region_name);
int schematic_flip_region_z(SchematicWrapper* schematic, const char* region_name);

int schematic_rotate_region_x(SchematicWrapper* schematic, const char* region_name, int degrees);
int schematic_rotate_region_y(SchematicWrapper* schematic, const char* region_name, int degrees);
int schematic_rotate_region_z(SchematicWrapper* schematic, const char* region_name, int degrees);
```

---

## Region Copying

```c
// Copy a region from source to target schematic.
// excluded_blocks is an array of block name strings, or NULL.
int schematic_copy_region(
    SchematicWrapper* target,
    const SchematicWrapper* source,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z,
    int target_x, int target_y, int target_z,
    const char** excluded_blocks,
    size_t excluded_blocks_len
);
```

---

## Chunk Iteration

```c
// Get chunks with default bottom-up strategy.
CChunkArray schematic_get_chunks(
    const SchematicWrapper* schematic,
    int chunk_width, int chunk_height, int chunk_length
);

// Get chunks with a loading strategy.
// Strategies: "distance_to_camera", "top_down", "bottom_up", "center_outward", "random"
CChunkArray schematic_get_chunks_with_strategy(
    const SchematicWrapper* schematic,
    int chunk_width, int chunk_height, int chunk_length,
    const char* strategy,
    float camera_x, float camera_y, float camera_z
);
```

---

## Definition Regions

### Creation & Lifecycle

```c
DefinitionRegionWrapper* definitionregion_new(void);
void definitionregion_free(DefinitionRegionWrapper* ptr);

// Create from a single bounding box
DefinitionRegionWrapper* definitionregion_from_bounds(
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);

// Create from flat position array [x0,y0,z0,x1,y1,z1,...], len = count * 3
DefinitionRegionWrapper* definitionregion_from_positions(const int* positions, size_t positions_len);

// Create from flat box array [min_x,min_y,min_z,max_x,max_y,max_z,...], len = count * 6
DefinitionRegionWrapper* definitionregion_from_bounding_boxes(const int* boxes, size_t boxes_len);

// Deep copy
DefinitionRegionWrapper* definitionregion_copy(const DefinitionRegionWrapper* ptr);
DefinitionRegionWrapper* definitionregion_clone_region(const DefinitionRegionWrapper* ptr);
```

### Schematic Region Operations

Manage definition regions attached to a schematic.

```c
int schematic_add_definition_region(SchematicWrapper* schematic, const char* name, const DefinitionRegionWrapper* region);
DefinitionRegionWrapper* schematic_get_definition_region(const SchematicWrapper* schematic, const char* name);
int schematic_remove_definition_region(SchematicWrapper* schematic, const char* name);
StringArray schematic_get_definition_region_names(const SchematicWrapper* schematic);

// Create empty
int schematic_create_definition_region(SchematicWrapper* schematic, const char* name);

// Create from geometry
int schematic_create_definition_region_from_point(SchematicWrapper* schematic, const char* name, int x, int y, int z);
int schematic_create_definition_region_from_bounds(
    SchematicWrapper* schematic, const char* name,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z
);

// Create and return the region
DefinitionRegionWrapper* schematic_create_region(
    SchematicWrapper* schematic, const char* name,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z
);

// Update an existing region
int schematic_update_region(SchematicWrapper* schematic, const char* name, const DefinitionRegionWrapper* region);

// Modify regions in-place on the schematic
int schematic_definition_region_add_bounds(
    SchematicWrapper* schematic, const char* name,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z
);
int schematic_definition_region_add_point(SchematicWrapper* schematic, const char* name, int x, int y, int z);
int schematic_definition_region_set_metadata(SchematicWrapper* schematic, const char* name, const char* key, const char* value);
int schematic_definition_region_shift(SchematicWrapper* schematic, const char* name, int dx, int dy, int dz);
```

### Standalone Region Operations

Modify standalone `DefinitionRegionWrapper` instances.

```c
// Mutating (modify in place)
int definitionregion_add_bounds(DefinitionRegionWrapper* ptr, int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
int definitionregion_add_point(DefinitionRegionWrapper* ptr, int x, int y, int z);
int definitionregion_set_metadata(DefinitionRegionWrapper* ptr, const char* key, const char* value);
int definitionregion_set_metadata_mut(DefinitionRegionWrapper* ptr, const char* key, const char* value);
int definitionregion_set_color(DefinitionRegionWrapper* ptr, uint32_t color);
int definitionregion_shift(DefinitionRegionWrapper* ptr, int dx, int dy, int dz);
int definitionregion_expand(DefinitionRegionWrapper* ptr, int x, int y, int z);
int definitionregion_contract(DefinitionRegionWrapper* ptr, int amount);
int definitionregion_simplify(DefinitionRegionWrapper* ptr);
int definitionregion_merge(DefinitionRegionWrapper* ptr, const DefinitionRegionWrapper* other);
int definitionregion_union_into(DefinitionRegionWrapper* ptr, const DefinitionRegionWrapper* other);

// Immutable (return new instances, caller must free)
DefinitionRegionWrapper* definitionregion_shifted(const DefinitionRegionWrapper* ptr, int dx, int dy, int dz);
DefinitionRegionWrapper* definitionregion_expanded(const DefinitionRegionWrapper* ptr, int x, int y, int z);
DefinitionRegionWrapper* definitionregion_contracted(const DefinitionRegionWrapper* ptr, int amount);
```

### Querying

```c
// Returns 1=yes, 0=no, -1=null
int definitionregion_is_empty(const DefinitionRegionWrapper* ptr);
int definitionregion_contains(const DefinitionRegionWrapper* ptr, int x, int y, int z);
int definitionregion_is_contiguous(const DefinitionRegionWrapper* ptr);

// Returns value or -1 on error
int definitionregion_volume(const DefinitionRegionWrapper* ptr);
int definitionregion_connected_components(const DefinitionRegionWrapper* ptr);
int definitionregion_box_count(const DefinitionRegionWrapper* ptr);

// Returns IntArray [min_x, min_y, min_z, max_x, max_y, max_z]. Caller must free_int_array().
IntArray definitionregion_get_bounds(const DefinitionRegionWrapper* ptr);

// Returns IntArray [width, height, length]
IntArray definitionregion_dimensions(const DefinitionRegionWrapper* ptr);

// Returns IntArray [x, y, z]
IntArray definitionregion_center(const DefinitionRegionWrapper* ptr);

// Returns CFloatArray [x, y, z]
CFloatArray definitionregion_center_f32(const DefinitionRegionWrapper* ptr);

// Returns flat IntArray [x0,y0,z0,x1,y1,z1,...]
IntArray definitionregion_positions(const DefinitionRegionWrapper* ptr);
IntArray definitionregion_positions_sorted(const DefinitionRegionWrapper* ptr);

// Returns CBoundingBox struct
CBoundingBox definitionregion_get_box(const DefinitionRegionWrapper* ptr, size_t index);

// Returns flat IntArray [min_x,min_y,min_z,max_x,max_y,max_z,...]
IntArray definitionregion_get_boxes(const DefinitionRegionWrapper* ptr);

// Metadata. Caller must free_string() / free_string_array().
char* definitionregion_get_metadata(const DefinitionRegionWrapper* ptr, const char* key);
StringArray definitionregion_get_all_metadata(const DefinitionRegionWrapper* ptr);  // "key=value" format
StringArray definitionregion_metadata_keys(const DefinitionRegionWrapper* ptr);
```

### Set Operations

All return new `DefinitionRegionWrapper*` instances. Caller must `definitionregion_free()`.

```c
DefinitionRegionWrapper* definitionregion_union(const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_intersect(const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_intersected(const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_subtract(const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);
DefinitionRegionWrapper* definitionregion_subtracted(const DefinitionRegionWrapper* a, const DefinitionRegionWrapper* b);

// Returns 1=intersects, 0=no, -1=null
int definitionregion_intersects_bounds(
    const DefinitionRegionWrapper* ptr,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z
);
```

### Filtering

```c
// Keep positions containing a specific block. Returns new region.
DefinitionRegionWrapper* definitionregion_filter_by_block(
    const DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic,
    const char* block_name
);

// Remove positions with a block (in place)
int definitionregion_exclude_block(
    DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic,
    const char* block_name
);

// Filter by block properties (JSON). Returns new region.
DefinitionRegionWrapper* definitionregion_filter_by_properties(
    const DefinitionRegionWrapper* ptr,
    const SchematicWrapper* schematic,
    const char* properties_json
);

// Add a filter string
int definitionregion_add_filter(DefinitionRegionWrapper* ptr, const char* filter);

// Get all blocks in the region. Caller must free_block_array().
CBlockArray definitionregion_blocks(const DefinitionRegionWrapper* ptr, const SchematicWrapper* schematic);
```

### Manipulation

```c
// Sync a standalone region back to a schematic
int definitionregion_sync(const DefinitionRegionWrapper* ptr, SchematicWrapper* schematic, const char* name);
```

---

## Building Tools

### Shapes

```c
ShapeWrapper* shape_sphere(float cx, float cy, float cz, float radius);
ShapeWrapper* shape_cuboid(int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
void shape_free(ShapeWrapper* ptr);
```

### Brushes

```c
// Solid single block
BrushWrapper* brush_solid(const char* block_name);

// RGB color (auto-maps to closest Minecraft block)
BrushWrapper* brush_color(unsigned char r, unsigned char g, unsigned char b);

// Linear gradient between two colored points. space: 0=RGB, 1=Oklab
BrushWrapper* brush_linear_gradient(
    int x1, int y1, int z1, unsigned char r1, unsigned char g1, unsigned char b1,
    int x2, int y2, int z2, unsigned char r2, unsigned char g2, unsigned char b2,
    int space
);

// Lambertian shaded brush with light direction
BrushWrapper* brush_shaded(
    unsigned char r, unsigned char g, unsigned char b,
    float lx, float ly, float lz
);

// 4-corner bilinear gradient (origin + U axis + V axis + 4 corner colors)
BrushWrapper* brush_bilinear_gradient(
    int ox, int oy, int oz,
    int ux, int uy, int uz,
    int vx, int vy, int vz,
    unsigned char r00, unsigned char g00, unsigned char b00,
    unsigned char r10, unsigned char g10, unsigned char b10,
    unsigned char r01, unsigned char g01, unsigned char b01,
    unsigned char r11, unsigned char g11, unsigned char b11,
    int space
);

// IDW point gradient. positions: [x0,y0,z0,...], colors: [r0,g0,b0,...], count: number of points
BrushWrapper* brush_point_gradient(
    const int* positions,
    const unsigned char* colors,
    size_t count,
    float falloff,
    int space
);

void brush_free(BrushWrapper* ptr);
```

### Fill

```c
// Apply brush to shape in schematic
int buildingtool_fill(SchematicWrapper* schematic, const ShapeWrapper* shape, const BrushWrapper* brush);

// Simple cuboid/sphere fill (no brush needed)
int schematic_fill_cuboid(
    SchematicWrapper* schematic,
    int min_x, int min_y, int min_z,
    int max_x, int max_y, int max_z,
    const char* block_name
);

int schematic_fill_sphere(
    SchematicWrapper* schematic,
    float cx, float cy, float cz,
    float radius,
    const char* block_name
);
```

---

## Schematic Builder

ASCII art layer-based schematic construction.

```c
SchematicBuilderWrapper* schematicbuilder_new(void);
void schematicbuilder_free(SchematicBuilderWrapper* ptr);

// Set name
int schematicbuilder_name(SchematicBuilderWrapper* ptr, const char* name);

// Map a character to a block type
int schematicbuilder_map(SchematicBuilderWrapper* ptr, char ch, const char* block);

// Set layers as a JSON array of string arrays
int schematicbuilder_layers(SchematicBuilderWrapper* ptr, const char* layers_json);

// Build the schematic. Returns NULL on error.
SchematicWrapper* schematicbuilder_build(SchematicBuilderWrapper* ptr);

// Build with detailed error reporting.
// On success: returns NULL and sets *out to the SchematicWrapper.
// On error: returns error string (caller must free_string()), *out is NULL.
char* schematicbuilder_build_with_error(SchematicBuilderWrapper* ptr, SchematicWrapper** out);

// Create from a named template
SchematicBuilderWrapper* schematicbuilder_from_template(const char* template_name);
```

---

## Sign Text & Insign

```c
// Extract sign text. Returns JSON array string. Caller must free_string().
char* schematic_extract_signs(const SchematicWrapper* schematic);

// Compile Insign annotations. Returns JSON or NULL. Caller must free_string().
char* schematic_compile_insign(const SchematicWrapper* schematic);
```

---

## Debug & Utility

```c
char* schematic_debug_info(const SchematicWrapper* schematic);
char* schematic_print(const SchematicWrapper* schematic);
char* schematic_print_schematic(const SchematicWrapper* schematic);
char* debug_schematic(const SchematicWrapper* schematic);
char* debug_json_schematic(const SchematicWrapper* schematic);
```

All return heap-allocated strings. Caller must `free_string()`.

---

## Simulation (feature-gated)

> All simulation functions require the `simulation` feature flag at compile time.

### MchprsWorld

```c
// Create simulation world from schematic
MchprsWorldWrapper* mchprs_world_new(const SchematicWrapper* schematic);

// Create with options: optimize (0/1), io_only (0/1)
MchprsWorldWrapper* mchprs_world_new_with_options(
    const SchematicWrapper* schematic,
    int optimize,
    int io_only
);

// Create with custom IO positions: flat array [x0,y0,z0,x1,y1,z1,...], count = number of positions
MchprsWorldWrapper* mchprs_world_new_with_custom_io(
    const SchematicWrapper* schematic,
    int optimize, int io_only,
    const int* custom_io_positions,
    int custom_io_count
);

void mchprs_world_free(MchprsWorldWrapper* world);

// Simulation control
int mchprs_world_tick(MchprsWorldWrapper* world, uint32_t ticks);
int mchprs_world_flush(MchprsWorldWrapper* world);
int mchprs_world_sync_to_schematic(MchprsWorldWrapper* world);
SchematicWrapper* mchprs_world_get_schematic(const MchprsWorldWrapper* world);

// Block interaction
int mchprs_world_on_use_block(MchprsWorldWrapper* world, int x, int y, int z);
int mchprs_world_set_lever_power(MchprsWorldWrapper* world, int x, int y, int z, int powered);

// Querying (returns 1/0 or signal level, -1 on null)
int mchprs_world_get_lever_power(const MchprsWorldWrapper* world, int x, int y, int z);
int mchprs_world_is_lit(const MchprsWorldWrapper* world, int x, int y, int z);

// Signal strength (0-15)
int mchprs_world_set_signal_strength(MchprsWorldWrapper* world, int x, int y, int z, uint8_t strength);
uint8_t mchprs_world_get_signal_strength(const MchprsWorldWrapper* world, int x, int y, int z);
```

### Value Types

Typed values for circuit I/O.

```c
ValueWrapper* value_from_u32(uint32_t v);
ValueWrapper* value_from_i32(int32_t v);
ValueWrapper* value_from_f32(float v);
ValueWrapper* value_from_bool(int v);        // 0 or 1
ValueWrapper* value_from_string(const char* s);

uint32_t value_as_u32(const ValueWrapper* v);
int32_t  value_as_i32(const ValueWrapper* v);
float    value_as_f32(const ValueWrapper* v);
int      value_as_bool(const ValueWrapper* v);
char*    value_as_string(const ValueWrapper* v);  // Caller must free_string()

char* value_type_name(const ValueWrapper* v);     // Caller must free_string()
void  value_free(ValueWrapper* ptr);
```

### IoType

Data type definitions for I/O ports.

```c
IoTypeWrapper* io_type_unsigned_int(size_t bits);
IoTypeWrapper* io_type_signed_int(size_t bits);
IoTypeWrapper* io_type_float32(void);
IoTypeWrapper* io_type_boolean(void);
IoTypeWrapper* io_type_ascii(size_t chars);
void io_type_free(IoTypeWrapper* ptr);
```

### LayoutFunction

Bit-to-position mapping strategies.

```c
LayoutFunctionWrapper* layout_function_one_to_one(void);
LayoutFunctionWrapper* layout_function_packed4(void);
LayoutFunctionWrapper* layout_function_custom(const size_t* mapping, size_t len);
LayoutFunctionWrapper* layout_function_row_major(size_t rows, size_t cols, size_t bits_per_element);
LayoutFunctionWrapper* layout_function_column_major(size_t rows, size_t cols, size_t bits_per_element);
LayoutFunctionWrapper* layout_function_scanline(size_t width, size_t height, size_t bits_per_pixel);
void layout_function_free(LayoutFunctionWrapper* ptr);
```

### OutputCondition

Predicates for conditional execution.

```c
OutputConditionWrapper* output_condition_equals(const ValueWrapper* value);
OutputConditionWrapper* output_condition_not_equals(const ValueWrapper* value);
OutputConditionWrapper* output_condition_greater_than(const ValueWrapper* value);
OutputConditionWrapper* output_condition_less_than(const ValueWrapper* value);
OutputConditionWrapper* output_condition_bitwise_and(uint32_t mask);
void output_condition_free(OutputConditionWrapper* ptr);
```

### ExecutionMode

Controls circuit execution behavior.

```c
ExecutionModeWrapper* execution_mode_fixed_ticks(uint32_t ticks);

ExecutionModeWrapper* execution_mode_until_condition(
    const char* output_name,
    const OutputConditionWrapper* condition,
    uint32_t max_ticks,
    uint32_t check_interval
);

ExecutionModeWrapper* execution_mode_until_change(uint32_t max_ticks, uint32_t check_interval);
ExecutionModeWrapper* execution_mode_until_stable(uint32_t stable_ticks, uint32_t max_ticks);
void execution_mode_free(ExecutionModeWrapper* ptr);
```

### SortStrategy

Position sorting for I/O bit assignment.

```c
SortStrategyWrapper* sort_strategy_yxz(void);  // Default
SortStrategyWrapper* sort_strategy_xyz(void);
SortStrategyWrapper* sort_strategy_zyx(void);
void sort_strategy_free(SortStrategyWrapper* ptr);
```

> Note: The FFI currently exposes three basic sort strategies. For more advanced sorting (descending, distance-based), use the WASM or Python bindings.
