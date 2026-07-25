#ifndef CellularSdfConfig_H
#define CellularSdfConfig_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "CellularSdfConfig.d.h"






typedef struct CellularSdfConfig_create_result {union {CellularSdfConfig* ok; NucleationError err;}; bool is_ok;} CellularSdfConfig_create_result;
CellularSdfConfig_create_result CellularSdfConfig_create(int32_t cell_size_x, int32_t cell_size_z, uint64_t seed, float max_jitter_x, float max_jitter_z, float max_yaw_degrees, float min_scale, float max_scale, int32_t min_y_offset, int32_t max_y_offset, uint32_t presence_numerator, uint32_t presence_denominator, uint64_t feature_salt);

void CellularSdfConfig_destroy(CellularSdfConfig* self);





#endif // CellularSdfConfig_H
