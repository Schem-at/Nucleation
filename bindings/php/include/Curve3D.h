#ifndef Curve3D_H
#define Curve3D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "Curve3D.d.h"






typedef struct Curve3D_from_points_result {union {Curve3D* ok; NucleationError err;}; bool is_ok;} Curve3D_from_points_result;
Curve3D_from_points_result Curve3D_from_points(DiplomatF64View coordinates, bool closed);

uint32_t Curve3D_point_count(const Curve3D* self);

bool Curve3D_is_closed(const Curve3D* self);

void Curve3D_destroy(Curve3D* self);





#endif // Curve3D_H
