#ifndef VoxelModel_H
#define VoxelModel_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Palette.d.h"
#include "Schematic.d.h"

#include "VoxelModel.d.h"






typedef struct VoxelModel_plan_json_result {union { NucleationError err;}; bool is_ok;} VoxelModel_plan_json_result;
VoxelModel_plan_json_result VoxelModel_plan_json(const VoxelModel* self, DiplomatStringView options_json, DiplomatWrite* write);

typedef struct VoxelModel_to_schematic_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} VoxelModel_to_schematic_result;
VoxelModel_to_schematic_result VoxelModel_to_schematic(const VoxelModel* self, DiplomatStringView options_json, const Palette* palette, DiplomatStringView name);

void VoxelModel_destroy(VoxelModel* self);





#endif // VoxelModel_H
