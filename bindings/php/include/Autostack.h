#ifndef Autostack_H
#define Autostack_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Autostack.d.h"






void Autostack_detect_structures(const Schematic* schematic, DiplomatWrite* write);

void Autostack_detect_structures_graph(const Schematic* schematic, DiplomatWrite* write);

typedef struct Autostack_resize_1d_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Autostack_resize_1d_result;
Autostack_resize_1d_result Autostack_resize_1d(const Schematic* schematic, int32_t vx, int32_t vy, int32_t vz, uint32_t units);

typedef struct Autostack_resize_2d_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Autostack_resize_2d_result;
Autostack_resize_2d_result Autostack_resize_2d(const Schematic* schematic, int32_t v1x, int32_t v1y, int32_t v1z, int32_t v2x, int32_t v2y, int32_t v2z, uint32_t n1, uint32_t n2);

void Autostack_destroy(Autostack* self);





#endif // Autostack_H
