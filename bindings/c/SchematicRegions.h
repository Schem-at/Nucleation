#ifndef SchematicRegions_H
#define SchematicRegions_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "DefinitionRegion.d.h"
#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "SchematicRegions.d.h"






typedef struct SchematicRegions_add_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_add_result;
SchematicRegions_add_result SchematicRegions_add(Schematic* schematic, DiplomatStringView name, const DefinitionRegion* region);

typedef struct SchematicRegions_update_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_update_result;
SchematicRegions_update_result SchematicRegions_update(Schematic* schematic, DiplomatStringView name, const DefinitionRegion* region);

typedef struct SchematicRegions_get_result {union {DefinitionRegion* ok; NucleationError err;}; bool is_ok;} SchematicRegions_get_result;
SchematicRegions_get_result SchematicRegions_get(const Schematic* schematic, DiplomatStringView name);

typedef struct SchematicRegions_remove_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_remove_result;
SchematicRegions_remove_result SchematicRegions_remove(Schematic* schematic, DiplomatStringView name);

typedef struct SchematicRegions_names_json_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_names_json_result;
SchematicRegions_names_json_result SchematicRegions_names_json(const Schematic* schematic, DiplomatWrite* write);

typedef struct SchematicRegions_create_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_create_result;
SchematicRegions_create_result SchematicRegions_create(Schematic* schematic, DiplomatStringView name);

typedef struct SchematicRegions_create_from_point_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_create_from_point_result;
SchematicRegions_create_from_point_result SchematicRegions_create_from_point(Schematic* schematic, DiplomatStringView name, int32_t x, int32_t y, int32_t z);

typedef struct SchematicRegions_create_from_bounds_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_create_from_bounds_result;
SchematicRegions_create_from_bounds_result SchematicRegions_create_from_bounds(Schematic* schematic, DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct SchematicRegions_create_region_result {union {DefinitionRegion* ok; NucleationError err;}; bool is_ok;} SchematicRegions_create_region_result;
SchematicRegions_create_region_result SchematicRegions_create_region(Schematic* schematic, DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct SchematicRegions_add_bounds_to_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_add_bounds_to_result;
SchematicRegions_add_bounds_to_result SchematicRegions_add_bounds_to(Schematic* schematic, DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct SchematicRegions_add_point_to_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_add_point_to_result;
SchematicRegions_add_point_to_result SchematicRegions_add_point_to(Schematic* schematic, DiplomatStringView name, int32_t x, int32_t y, int32_t z);

typedef struct SchematicRegions_set_metadata_on_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_set_metadata_on_result;
SchematicRegions_set_metadata_on_result SchematicRegions_set_metadata_on(Schematic* schematic, DiplomatStringView name, DiplomatStringView key, DiplomatStringView value);

typedef struct SchematicRegions_shift_region_result {union { NucleationError err;}; bool is_ok;} SchematicRegions_shift_region_result;
SchematicRegions_shift_region_result SchematicRegions_shift_region(Schematic* schematic, DiplomatStringView name, int32_t dx, int32_t dy, int32_t dz);

void SchematicRegions_destroy(SchematicRegions* self);





#endif // SchematicRegions_H
