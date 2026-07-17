#ifndef BuildingTool_H
#define BuildingTool_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Brush.d.h"
#include "NucleationError.d.h"
#include "Schematic.d.h"
#include "Shape.d.h"

#include "BuildingTool.d.h"






void BuildingTool_fill(Schematic* schematic, const Shape* shape, const Brush* brush);

void BuildingTool_rstack(Schematic* schematic, const Shape* shape, const Brush* brush, size_t count, int32_t offset_x, int32_t offset_y, int32_t offset_z);

void BuildingTool_fill_only_air(Schematic* schematic, const Shape* shape, const Brush* brush);

typedef struct BuildingTool_fill_replacing_result {union { NucleationError err;}; bool is_ok;} BuildingTool_fill_replacing_result;
BuildingTool_fill_replacing_result BuildingTool_fill_replacing(Schematic* schematic, const Shape* shape, const Brush* brush, DiplomatStringView targets_json);

void BuildingTool_destroy(BuildingTool* self);





#endif // BuildingTool_H
