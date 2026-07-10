#ifndef DefinitionRegion_H
#define DefinitionRegion_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "BlockPos.d.h"
#include "Dimensions.d.h"
#include "NucleationError.d.h"
#include "RegionBounds.d.h"
#include "Schematic.d.h"

#include "DefinitionRegion.d.h"






DefinitionRegion* DefinitionRegion_create(void);

DefinitionRegion* DefinitionRegion_from_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct DefinitionRegion_from_positions_result {union {DefinitionRegion* ok; NucleationError err;}; bool is_ok;} DefinitionRegion_from_positions_result;
DefinitionRegion_from_positions_result DefinitionRegion_from_positions(DiplomatI32View positions);

typedef struct DefinitionRegion_from_bounding_boxes_result {union {DefinitionRegion* ok; NucleationError err;}; bool is_ok;} DefinitionRegion_from_bounding_boxes_result;
DefinitionRegion_from_bounding_boxes_result DefinitionRegion_from_bounding_boxes(DiplomatI32View boxes);

void DefinitionRegion_add_bounds(DefinitionRegion* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

void DefinitionRegion_add_point(DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

typedef struct DefinitionRegion_set_metadata_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_set_metadata_result;
DefinitionRegion_set_metadata_result DefinitionRegion_set_metadata(DefinitionRegion* self, DiplomatStringView key, DiplomatStringView value);

typedef struct DefinitionRegion_get_metadata_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_get_metadata_result;
DefinitionRegion_get_metadata_result DefinitionRegion_get_metadata(const DefinitionRegion* self, DiplomatStringView key, DiplomatWrite* write);

typedef struct DefinitionRegion_all_metadata_json_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_all_metadata_json_result;
DefinitionRegion_all_metadata_json_result DefinitionRegion_all_metadata_json(const DefinitionRegion* self, DiplomatWrite* write);

typedef struct DefinitionRegion_metadata_keys_json_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_metadata_keys_json_result;
DefinitionRegion_metadata_keys_json_result DefinitionRegion_metadata_keys_json(const DefinitionRegion* self, DiplomatWrite* write);

typedef struct DefinitionRegion_add_filter_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_add_filter_result;
DefinitionRegion_add_filter_result DefinitionRegion_add_filter(DefinitionRegion* self, DiplomatStringView filter);

bool DefinitionRegion_is_empty(const DefinitionRegion* self);

uint64_t DefinitionRegion_volume(const DefinitionRegion* self);

bool DefinitionRegion_contains(const DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

void DefinitionRegion_shift(DefinitionRegion* self, int32_t dx, int32_t dy, int32_t dz);

void DefinitionRegion_expand(DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

void DefinitionRegion_contract(DefinitionRegion* self, int32_t amount);

DefinitionRegion* DefinitionRegion_intersected(const DefinitionRegion* self, const DefinitionRegion* other);

DefinitionRegion* DefinitionRegion_union_with(const DefinitionRegion* self, const DefinitionRegion* other);

DefinitionRegion* DefinitionRegion_subtracted(const DefinitionRegion* self, const DefinitionRegion* other);

void DefinitionRegion_merge(DefinitionRegion* self, const DefinitionRegion* other);

void DefinitionRegion_union_into(DefinitionRegion* self, const DefinitionRegion* other);

typedef struct DefinitionRegion_bounds_result {union {RegionBounds ok; NucleationError err;}; bool is_ok;} DefinitionRegion_bounds_result;
DefinitionRegion_bounds_result DefinitionRegion_bounds(const DefinitionRegion* self);

Dimensions DefinitionRegion_dimensions(const DefinitionRegion* self);

typedef struct DefinitionRegion_center_result {union {BlockPos ok; NucleationError err;}; bool is_ok;} DefinitionRegion_center_result;
DefinitionRegion_center_result DefinitionRegion_center(const DefinitionRegion* self);

typedef struct DefinitionRegion_center_f32_json_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_center_f32_json_result;
DefinitionRegion_center_f32_json_result DefinitionRegion_center_f32_json(const DefinitionRegion* self, DiplomatWrite* write);

typedef struct DefinitionRegion_positions_json_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_positions_json_result;
DefinitionRegion_positions_json_result DefinitionRegion_positions_json(const DefinitionRegion* self, DiplomatWrite* write);

typedef struct DefinitionRegion_positions_sorted_json_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_positions_sorted_json_result;
DefinitionRegion_positions_sorted_json_result DefinitionRegion_positions_sorted_json(const DefinitionRegion* self, DiplomatWrite* write);

uint32_t DefinitionRegion_box_count(const DefinitionRegion* self);

typedef struct DefinitionRegion_get_box_result {union {RegionBounds ok; NucleationError err;}; bool is_ok;} DefinitionRegion_get_box_result;
DefinitionRegion_get_box_result DefinitionRegion_get_box(const DefinitionRegion* self, uint32_t index);

typedef struct DefinitionRegion_boxes_json_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_boxes_json_result;
DefinitionRegion_boxes_json_result DefinitionRegion_boxes_json(const DefinitionRegion* self, DiplomatWrite* write);

bool DefinitionRegion_is_contiguous(const DefinitionRegion* self);

uint32_t DefinitionRegion_connected_components(const DefinitionRegion* self);

void DefinitionRegion_simplify(DefinitionRegion* self);

typedef struct DefinitionRegion_filter_by_block_result {union {DefinitionRegion* ok; NucleationError err;}; bool is_ok;} DefinitionRegion_filter_by_block_result;
DefinitionRegion_filter_by_block_result DefinitionRegion_filter_by_block(const DefinitionRegion* self, const Schematic* schematic, DiplomatStringView block_name);

typedef struct DefinitionRegion_filter_by_properties_result {union {DefinitionRegion* ok; NucleationError err;}; bool is_ok;} DefinitionRegion_filter_by_properties_result;
DefinitionRegion_filter_by_properties_result DefinitionRegion_filter_by_properties(const DefinitionRegion* self, const Schematic* schematic, DiplomatStringView properties_json);

typedef struct DefinitionRegion_exclude_block_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_exclude_block_result;
DefinitionRegion_exclude_block_result DefinitionRegion_exclude_block(DefinitionRegion* self, const Schematic* schematic, DiplomatStringView block_name);

bool DefinitionRegion_intersects_bounds(const DefinitionRegion* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

DefinitionRegion* DefinitionRegion_shifted(const DefinitionRegion* self, int32_t dx, int32_t dy, int32_t dz);

DefinitionRegion* DefinitionRegion_expanded(const DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

DefinitionRegion* DefinitionRegion_contracted(const DefinitionRegion* self, int32_t amount);

DefinitionRegion* DefinitionRegion_copy(const DefinitionRegion* self);

void DefinitionRegion_set_color(DefinitionRegion* self, uint32_t color);

typedef struct DefinitionRegion_blocks_json_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_blocks_json_result;
DefinitionRegion_blocks_json_result DefinitionRegion_blocks_json(const DefinitionRegion* self, const Schematic* schematic, DiplomatWrite* write);

typedef struct DefinitionRegion_sync_result {union { NucleationError err;}; bool is_ok;} DefinitionRegion_sync_result;
DefinitionRegion_sync_result DefinitionRegion_sync(const DefinitionRegion* self, Schematic* schematic, DiplomatStringView name);

void DefinitionRegion_destroy(DefinitionRegion* self);





#endif // DefinitionRegion_H
