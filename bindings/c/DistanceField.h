#ifndef DistanceField_H
#define DistanceField_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Schematic.d.h"

#include "DistanceField.d.h"






DistanceField* DistanceField_from_schematic(const Schematic* schematic);

int32_t DistanceField_depth(const DistanceField* self, int32_t x, int32_t y, int32_t z);

float DistanceField_slope(const DistanceField* self, int32_t x, int32_t y, int32_t z);

void DistanceField_normal_json(const DistanceField* self, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

void DistanceField_destroy(DistanceField* self);





#endif // DistanceField_H
