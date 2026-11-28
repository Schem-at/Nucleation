#ifndef VoxelPipeline_H
#define VoxelPipeline_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Field.d.h"
#include "Material.d.h"
#include "NucleationError.d.h"
#include "ScatterRule.d.h"
#include "Schematic.d.h"

#include "VoxelPipeline.d.h"






VoxelPipeline* VoxelPipeline_create(const Field* geometry, const Material* material);

typedef struct VoxelPipeline_add_fluid_result {union { NucleationError err;}; bool is_ok;} VoxelPipeline_add_fluid_result;
VoxelPipeline_add_fluid_result VoxelPipeline_add_fluid(VoxelPipeline* self, const Field* field, DiplomatStringView block);

typedef struct VoxelPipeline_add_fluid_level_result {union { NucleationError err;}; bool is_ok;} VoxelPipeline_add_fluid_level_result;
VoxelPipeline_add_fluid_level_result VoxelPipeline_add_fluid_level(VoxelPipeline* self, float y_level, DiplomatStringView block);

void VoxelPipeline_add_scatter(VoxelPipeline* self, const ScatterRule* rule);

void VoxelPipeline_set_fill_replace(VoxelPipeline* self);

void VoxelPipeline_set_fill_keep_existing(VoxelPipeline* self);

typedef struct VoxelPipeline_set_fill_replacing_result {union { NucleationError err;}; bool is_ok;} VoxelPipeline_set_fill_replacing_result;
VoxelPipeline_set_fill_replacing_result VoxelPipeline_set_fill_replacing(VoxelPipeline* self, DiplomatStringView targets_json);

typedef struct VoxelPipeline_to_schematic_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} VoxelPipeline_to_schematic_result;
VoxelPipeline_to_schematic_result VoxelPipeline_to_schematic(const VoxelPipeline* self, DiplomatStringView name);

typedef struct VoxelPipeline_to_schematic_bounded_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} VoxelPipeline_to_schematic_bounded_result;
VoxelPipeline_to_schematic_bounded_result VoxelPipeline_to_schematic_bounded(const VoxelPipeline* self, DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct VoxelPipeline_apply_to_schematic_result {union { NucleationError err;}; bool is_ok;} VoxelPipeline_apply_to_schematic_result;
VoxelPipeline_apply_to_schematic_result VoxelPipeline_apply_to_schematic(const VoxelPipeline* self, Schematic* schematic);

typedef struct VoxelPipeline_apply_to_schematic_bounded_result {union { NucleationError err;}; bool is_ok;} VoxelPipeline_apply_to_schematic_bounded_result;
VoxelPipeline_apply_to_schematic_bounded_result VoxelPipeline_apply_to_schematic_bounded(const VoxelPipeline* self, Schematic* schematic, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

void VoxelPipeline_destroy(VoxelPipeline* self);





#endif // VoxelPipeline_H
