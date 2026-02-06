/**
 * nucleation.h - C header for Nucleation FFI bindings
 *
 * High-performance Minecraft schematic parser, transformer, and simulator.
 *
 * All opaque types are forward-declared as structs. Pass them as pointers.
 * Return values of 0 indicate success; -1 indicates a null pointer argument;
 * -2 indicates an internal error.
 */

#ifndef NUCLEATION_H
#define NUCLEATION_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================== */
/* Opaque Types                                                               */
/* ========================================================================== */

typedef struct SchematicWrapper SchematicWrapper;
typedef struct BlockStateWrapper BlockStateWrapper;
typedef struct DefinitionRegionWrapper DefinitionRegionWrapper;
typedef struct ShapeWrapper ShapeWrapper;
typedef struct BrushWrapper BrushWrapper;
typedef struct SchematicBuilderWrapper SchematicBuilderWrapper;

/* ========================================================================== */
/* C-Compatible Data Structures                                               */
/* ========================================================================== */

typedef struct {
    unsigned char *data;
    size_t len;
} ByteArray;

typedef struct {
    char **data;
    size_t len;
} StringArray;

typedef struct {
    int *data;
    size_t len;
} IntArray;

typedef struct {
    float *data;
    size_t len;
} CFloatArray;

typedef struct {
    char *key;
    char *value;
} CProperty;

typedef struct {
    CProperty *data;
    size_t len;
} CPropertyArray;

typedef struct {
    int x;
    int y;
    int z;
    char *name;
    char *properties_json;
} CBlock;

typedef struct {
    CBlock *data;
    size_t len;
} CBlockArray;

typedef struct {
    char *id;
    int x;
    int y;
    int z;
    char *nbt_json;
} CBlockEntity;

typedef struct {
    CBlockEntity *data;
    size_t len;
} CBlockEntityArray;

typedef struct {
    int chunk_x;
    int chunk_y;
    int chunk_z;
    CBlockArray blocks;
} CChunk;

typedef struct {
    CChunk *data;
    size_t len;
} CChunkArray;

typedef struct {
    int min_x;
    int min_y;
    int min_z;
    int max_x;
    int max_y;
    int max_z;
} CBoundingBox;

/* ========================================================================== */
/* Memory Management                                                          */
/* ========================================================================== */

void free_byte_array(ByteArray array);
void free_string_array(StringArray array);
void free_int_array(IntArray array);
void free_float_array(CFloatArray array);
void free_string(char *string);
void free_property_array(CPropertyArray array);
void free_block_array(CBlockArray array);
void free_block_entity_array(CBlockEntityArray array);
void free_chunk_array(CChunkArray array);

/* ========================================================================== */
/* Schematic Lifecycle                                                        */
/* ========================================================================== */

SchematicWrapper *schematic_new(void);
void schematic_free(SchematicWrapper *schematic);

/* ========================================================================== */
/* Schematic I/O                                                              */
/* ========================================================================== */

int schematic_from_data(SchematicWrapper *schematic,
                        const unsigned char *data, size_t data_len);
int schematic_from_litematic(SchematicWrapper *schematic,
                             const unsigned char *data, size_t data_len);
ByteArray schematic_to_litematic(const SchematicWrapper *schematic);
int schematic_from_schematic(SchematicWrapper *schematic,
                             const unsigned char *data, size_t data_len);
ByteArray schematic_to_schematic(const SchematicWrapper *schematic);

/* ========================================================================== */
/* Format Management                                                          */
/* ========================================================================== */

ByteArray schematic_save_as(const SchematicWrapper *schematic,
                            const char *format, const char *version);
StringArray schematic_get_supported_import_formats(void);
StringArray schematic_get_supported_export_formats(void);
StringArray schematic_get_format_versions(const char *format);
char *schematic_get_default_format_version(const char *format);
ByteArray schematic_to_schematic_version(const SchematicWrapper *schematic,
                                         const char *version);
StringArray schematic_get_available_schematic_versions(void);

/* ========================================================================== */
/* Block Manipulation                                                         */
/* ========================================================================== */

int schematic_set_block(SchematicWrapper *schematic,
                        int x, int y, int z, const char *block_name);
int schematic_set_block_with_properties(SchematicWrapper *schematic,
                                        int x, int y, int z,
                                        const char *block_name,
                                        const CProperty *properties,
                                        size_t properties_len);
int schematic_set_block_from_string(SchematicWrapper *schematic,
                                    int x, int y, int z,
                                    const char *block_string);
int schematic_set_block_with_nbt(SchematicWrapper *schematic,
                                 int x, int y, int z,
                                 const char *block_name,
                                 const char *nbt_json);
int schematic_set_block_in_region(SchematicWrapper *schematic,
                                  const char *region_name,
                                  int x, int y, int z,
                                  const char *block_name);
int schematic_copy_region(SchematicWrapper *target,
                          const SchematicWrapper *source,
                          int min_x, int min_y, int min_z,
                          int max_x, int max_y, int max_z,
                          int target_x, int target_y, int target_z,
                          const char **excluded_blocks,
                          size_t excluded_blocks_len);

/* ========================================================================== */
/* Block & Entity Accessors                                                   */
/* ========================================================================== */

char *schematic_get_block(const SchematicWrapper *schematic,
                          int x, int y, int z);
char *schematic_get_block_string(const SchematicWrapper *schematic,
                                 int x, int y, int z);
BlockStateWrapper *schematic_get_block_with_properties(
    const SchematicWrapper *schematic, int x, int y, int z);
CBlockEntity *schematic_get_block_entity(const SchematicWrapper *schematic,
                                         int x, int y, int z);
CBlockEntityArray schematic_get_all_block_entities(
    const SchematicWrapper *schematic);
CBlockArray schematic_get_all_blocks(const SchematicWrapper *schematic);
CBlockArray schematic_get_chunk_blocks(const SchematicWrapper *schematic,
                                       int offset_x, int offset_y,
                                       int offset_z, int width,
                                       int height, int length);

/* ========================================================================== */
/* Chunking                                                                   */
/* ========================================================================== */

CChunkArray schematic_get_chunks(const SchematicWrapper *schematic,
                                 int chunk_width, int chunk_height,
                                 int chunk_length);
CChunkArray schematic_get_chunks_with_strategy(
    const SchematicWrapper *schematic,
    int chunk_width, int chunk_height, int chunk_length,
    const char *strategy, float camera_x, float camera_y, float camera_z);

/* ========================================================================== */
/* Metadata & Dimensions                                                      */
/* ========================================================================== */

IntArray schematic_get_dimensions(const SchematicWrapper *schematic);
IntArray schematic_get_allocated_dimensions(const SchematicWrapper *schematic);
IntArray schematic_get_tight_dimensions(const SchematicWrapper *schematic);
IntArray schematic_get_tight_bounds_min(const SchematicWrapper *schematic);
IntArray schematic_get_tight_bounds_max(const SchematicWrapper *schematic);
int schematic_get_block_count(const SchematicWrapper *schematic);
int schematic_get_volume(const SchematicWrapper *schematic);
StringArray schematic_get_region_names(const SchematicWrapper *schematic);
CBoundingBox schematic_get_bounding_box(const SchematicWrapper *schematic);
CBoundingBox schematic_get_region_bounding_box(
    const SchematicWrapper *schematic, const char *region_name);

/* ========================================================================== */
/* Palette                                                                    */
/* ========================================================================== */

StringArray schematic_get_palette(const SchematicWrapper *schematic);
char *schematic_get_all_palettes(const SchematicWrapper *schematic);
StringArray schematic_get_default_region_palette(
    const SchematicWrapper *schematic);
StringArray schematic_get_palette_from_region(
    const SchematicWrapper *schematic, const char *region_name);

/* ========================================================================== */
/* Transformations                                                            */
/* ========================================================================== */

int schematic_flip_x(SchematicWrapper *schematic);
int schematic_flip_y(SchematicWrapper *schematic);
int schematic_flip_z(SchematicWrapper *schematic);
int schematic_rotate_x(SchematicWrapper *schematic, int degrees);
int schematic_rotate_y(SchematicWrapper *schematic, int degrees);
int schematic_rotate_z(SchematicWrapper *schematic, int degrees);
int schematic_flip_region_x(SchematicWrapper *schematic,
                            const char *region_name);
int schematic_flip_region_y(SchematicWrapper *schematic,
                            const char *region_name);
int schematic_flip_region_z(SchematicWrapper *schematic,
                            const char *region_name);
int schematic_rotate_region_x(SchematicWrapper *schematic,
                              const char *region_name, int degrees);
int schematic_rotate_region_y(SchematicWrapper *schematic,
                              const char *region_name, int degrees);
int schematic_rotate_region_z(SchematicWrapper *schematic,
                              const char *region_name, int degrees);

/* ========================================================================== */
/* Building (Fill)                                                            */
/* ========================================================================== */

int schematic_fill_cuboid(SchematicWrapper *schematic,
                          int min_x, int min_y, int min_z,
                          int max_x, int max_y, int max_z,
                          const char *block_name);
int schematic_fill_sphere(SchematicWrapper *schematic,
                          int cx, int cy, int cz, float radius,
                          const char *block_name);

/* ========================================================================== */
/* InSign                                                                     */
/* ========================================================================== */

char *schematic_extract_signs(const SchematicWrapper *schematic);
char *schematic_compile_insign(const SchematicWrapper *schematic);

/* ========================================================================== */
/* Debug & Print                                                              */
/* ========================================================================== */

char *schematic_debug_info(const SchematicWrapper *schematic);
char *schematic_print(const SchematicWrapper *schematic);
char *schematic_print_schematic(const SchematicWrapper *schematic);
char *debug_schematic(const SchematicWrapper *schematic);
char *debug_json_schematic(const SchematicWrapper *schematic);

/* ========================================================================== */
/* Definition Region Management (on Schematic)                                */
/* ========================================================================== */

int schematic_add_definition_region(SchematicWrapper *schematic,
                                    const char *name,
                                    const DefinitionRegionWrapper *region);
DefinitionRegionWrapper *schematic_get_definition_region(
    const SchematicWrapper *schematic, const char *name);
int schematic_remove_definition_region(SchematicWrapper *schematic,
                                       const char *name);
StringArray schematic_get_definition_region_names(
    const SchematicWrapper *schematic);
int schematic_create_definition_region(SchematicWrapper *schematic,
                                       const char *name);
int schematic_create_definition_region_from_point(
    SchematicWrapper *schematic, const char *name, int x, int y, int z);
int schematic_create_definition_region_from_bounds(
    SchematicWrapper *schematic, const char *name,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
DefinitionRegionWrapper *schematic_create_region(
    SchematicWrapper *schematic, const char *name,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
int schematic_update_region(SchematicWrapper *schematic, const char *name,
                            const DefinitionRegionWrapper *region);
int schematic_definition_region_add_bounds(
    SchematicWrapper *schematic, const char *name,
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
int schematic_definition_region_add_point(SchematicWrapper *schematic,
                                          const char *name,
                                          int x, int y, int z);
int schematic_definition_region_set_metadata(SchematicWrapper *schematic,
                                             const char *name,
                                             const char *key,
                                             const char *value);
int schematic_definition_region_shift(SchematicWrapper *schematic,
                                      const char *name,
                                      int dx, int dy, int dz);

/* ========================================================================== */
/* BlockState Wrapper                                                         */
/* ========================================================================== */

BlockStateWrapper *blockstate_new(const char *name);
void blockstate_free(BlockStateWrapper *bs);
BlockStateWrapper *blockstate_with_property(BlockStateWrapper *block_state,
                                            const char *key,
                                            const char *value);
char *blockstate_get_name(const BlockStateWrapper *block_state);
CPropertyArray blockstate_get_properties(
    const BlockStateWrapper *block_state);

/* ========================================================================== */
/* DefinitionRegion Wrapper                                                   */
/* ========================================================================== */

/* Lifecycle */
DefinitionRegionWrapper *definitionregion_new(void);
void definitionregion_free(DefinitionRegionWrapper *ptr);

/* Constructors */
DefinitionRegionWrapper *definitionregion_from_bounds(
    int min_x, int min_y, int min_z, int max_x, int max_y, int max_z);
DefinitionRegionWrapper *definitionregion_from_positions(
    const int *positions, size_t positions_len);
DefinitionRegionWrapper *definitionregion_from_bounding_boxes(
    const int *boxes, size_t boxes_len);

/* Mutators */
int definitionregion_add_bounds(DefinitionRegionWrapper *ptr,
                                int min_x, int min_y, int min_z,
                                int max_x, int max_y, int max_z);
int definitionregion_add_point(DefinitionRegionWrapper *ptr,
                               int x, int y, int z);
int definitionregion_set_metadata(DefinitionRegionWrapper *ptr,
                                  const char *key, const char *value);
int definitionregion_set_metadata_mut(DefinitionRegionWrapper *ptr,
                                      const char *key, const char *value);
int definitionregion_set_color(DefinitionRegionWrapper *ptr, uint32_t color);
int definitionregion_shift(DefinitionRegionWrapper *ptr,
                           int dx, int dy, int dz);
int definitionregion_expand(DefinitionRegionWrapper *ptr,
                            int x, int y, int z);
int definitionregion_contract(DefinitionRegionWrapper *ptr, int amount);
int definitionregion_simplify(DefinitionRegionWrapper *ptr);
int definitionregion_merge(DefinitionRegionWrapper *ptr,
                           const DefinitionRegionWrapper *other);
int definitionregion_union_into(DefinitionRegionWrapper *ptr,
                                const DefinitionRegionWrapper *other);
int definitionregion_add_filter(DefinitionRegionWrapper *ptr,
                                const char *filter);
int definitionregion_exclude_block(DefinitionRegionWrapper *ptr,
                                   const SchematicWrapper *schematic,
                                   const char *block_name);
int definitionregion_sync(const DefinitionRegionWrapper *ptr,
                          SchematicWrapper *schematic, const char *name);

/* Accessors */
char *definitionregion_get_metadata(const DefinitionRegionWrapper *ptr,
                                    const char *key);
char *definitionregion_get_all_metadata(const DefinitionRegionWrapper *ptr);
StringArray definitionregion_metadata_keys(
    const DefinitionRegionWrapper *ptr);
int definitionregion_is_empty(const DefinitionRegionWrapper *ptr);
int definitionregion_volume(const DefinitionRegionWrapper *ptr);
int definitionregion_contains(const DefinitionRegionWrapper *ptr,
                              int x, int y, int z);
int definitionregion_is_contiguous(const DefinitionRegionWrapper *ptr);
int definitionregion_connected_components(
    const DefinitionRegionWrapper *ptr);
int definitionregion_box_count(const DefinitionRegionWrapper *ptr);
CBoundingBox definitionregion_get_bounds(
    const DefinitionRegionWrapper *ptr);
CBoundingBox definitionregion_get_box(const DefinitionRegionWrapper *ptr,
                                      size_t index);
IntArray definitionregion_get_boxes(const DefinitionRegionWrapper *ptr);
IntArray definitionregion_dimensions(const DefinitionRegionWrapper *ptr);
IntArray definitionregion_center(const DefinitionRegionWrapper *ptr);
CFloatArray definitionregion_center_f32(const DefinitionRegionWrapper *ptr);
IntArray definitionregion_positions(const DefinitionRegionWrapper *ptr);
IntArray definitionregion_positions_sorted(
    const DefinitionRegionWrapper *ptr);
int definitionregion_intersects_bounds(const DefinitionRegionWrapper *ptr,
                                       int min_x, int min_y, int min_z,
                                       int max_x, int max_y, int max_z);
CBlockArray definitionregion_blocks(const DefinitionRegionWrapper *ptr,
                                    const SchematicWrapper *schematic);

/* Set operations (return new regions - caller must free) */
DefinitionRegionWrapper *definitionregion_intersect(
    const DefinitionRegionWrapper *a, const DefinitionRegionWrapper *b);
DefinitionRegionWrapper *definitionregion_union(
    const DefinitionRegionWrapper *a, const DefinitionRegionWrapper *b);
DefinitionRegionWrapper *definitionregion_subtract(
    const DefinitionRegionWrapper *a, const DefinitionRegionWrapper *b);
DefinitionRegionWrapper *definitionregion_intersected(
    const DefinitionRegionWrapper *a, const DefinitionRegionWrapper *b);
DefinitionRegionWrapper *definitionregion_subtracted(
    const DefinitionRegionWrapper *a, const DefinitionRegionWrapper *b);

/* Immutable copies / transforms (return new regions - caller must free) */
DefinitionRegionWrapper *definitionregion_shifted(
    const DefinitionRegionWrapper *ptr, int dx, int dy, int dz);
DefinitionRegionWrapper *definitionregion_expanded(
    const DefinitionRegionWrapper *ptr, int x, int y, int z);
DefinitionRegionWrapper *definitionregion_contracted(
    const DefinitionRegionWrapper *ptr, int amount);
DefinitionRegionWrapper *definitionregion_copy(
    const DefinitionRegionWrapper *ptr);
DefinitionRegionWrapper *definitionregion_clone_region(
    const DefinitionRegionWrapper *ptr);
DefinitionRegionWrapper *definitionregion_filter_by_block(
    const DefinitionRegionWrapper *ptr, const SchematicWrapper *schematic,
    const char *block_name);
DefinitionRegionWrapper *definitionregion_filter_by_properties(
    const DefinitionRegionWrapper *ptr, const SchematicWrapper *schematic,
    const char *properties_json);

/* ========================================================================== */
/* Shape / Brush / BuildingTool                                               */
/* ========================================================================== */

/* Shapes */
ShapeWrapper *shape_sphere(int cx, int cy, int cz, float radius);
ShapeWrapper *shape_cuboid(int min_x, int min_y, int min_z,
                           int max_x, int max_y, int max_z);
void shape_free(ShapeWrapper *ptr);

/* Brushes */
BrushWrapper *brush_solid(const char *block_name);
BrushWrapper *brush_color(unsigned char r, unsigned char g, unsigned char b);
BrushWrapper *brush_linear_gradient(int x1, int y1, int z1,
                                    unsigned char r1, unsigned char g1,
                                    unsigned char b1,
                                    int x2, int y2, int z2,
                                    unsigned char r2, unsigned char g2,
                                    unsigned char b2, int space);
BrushWrapper *brush_shaded(unsigned char r, unsigned char g, unsigned char b,
                           float lx, float ly, float lz);
BrushWrapper *brush_bilinear_gradient(
    int ox, int oy, int oz, int ux, int uy, int uz, int vx, int vy, int vz,
    unsigned char r00, unsigned char g00, unsigned char b00,
    unsigned char r10, unsigned char g10, unsigned char b10,
    unsigned char r01, unsigned char g01, unsigned char b01,
    unsigned char r11, unsigned char g11, unsigned char b11, int space);
BrushWrapper *brush_point_gradient(const int *positions,
                                   const unsigned char *colors,
                                   size_t count, float falloff, int space);
void brush_free(BrushWrapper *ptr);

/* BuildingTool */
int buildingtool_fill(SchematicWrapper *schematic,
                      const ShapeWrapper *shape, const BrushWrapper *brush);

/* ========================================================================== */
/* SchematicBuilder                                                           */
/* ========================================================================== */

SchematicBuilderWrapper *schematicbuilder_new(void);
void schematicbuilder_free(SchematicBuilderWrapper *ptr);
int schematicbuilder_name(SchematicBuilderWrapper *ptr, const char *name);
int schematicbuilder_map(SchematicBuilderWrapper *ptr, char ch,
                         const char *block);
int schematicbuilder_layers(SchematicBuilderWrapper *ptr,
                            const char *layers_json);
SchematicWrapper *schematicbuilder_build(SchematicBuilderWrapper *ptr);
SchematicBuilderWrapper *schematicbuilder_from_template(
    const char *template_str);

#ifdef __cplusplus
}
#endif

#endif /* NUCLEATION_H */
