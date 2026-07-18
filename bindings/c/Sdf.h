#ifndef Sdf_H
#define Sdf_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Sdf.d.h"






typedef struct Sdf_schematic_from_sdf_auto_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_auto_result;
Sdf_schematic_from_sdf_auto_result Sdf_schematic_from_sdf_auto(DiplomatStringView sdf_json, DiplomatStringView rules_json);

typedef struct Sdf_schematic_from_sdf_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_result;
Sdf_schematic_from_sdf_result Sdf_schematic_from_sdf(DiplomatStringView sdf_json, DiplomatStringView rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Sdf_eval_result {union {float ok; NucleationError err;}; bool is_ok;} Sdf_eval_result;
Sdf_eval_result Sdf_eval(DiplomatStringView sdf_json, float x, float y, float z);

void Sdf_destroy(Sdf* self);





#endif // Sdf_H
