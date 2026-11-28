#ifndef FieldExprBuilder_H
#define FieldExprBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Field.d.h"
#include "FieldDistanceKind.d.h"
#include "NucleationError.d.h"

#include "FieldExprBuilder.d.h"






FieldExprBuilder* FieldExprBuilder_create(void);

typedef struct FieldExprBuilder_bind_result {union { NucleationError err;}; bool is_ok;} FieldExprBuilder_bind_result;
FieldExprBuilder_bind_result FieldExprBuilder_bind(FieldExprBuilder* self, DiplomatStringView name, const Field* field);

typedef struct FieldExprBuilder_set_bounds_result {union { NucleationError err;}; bool is_ok;} FieldExprBuilder_set_bounds_result;
FieldExprBuilder_set_bounds_result FieldExprBuilder_set_bounds(FieldExprBuilder* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

void FieldExprBuilder_set_distance_kind(FieldExprBuilder* self, FieldDistanceKind kind);

typedef struct FieldExprBuilder_compile_result {union {Field* ok; NucleationError err;}; bool is_ok;} FieldExprBuilder_compile_result;
FieldExprBuilder_compile_result FieldExprBuilder_compile(const FieldExprBuilder* self, DiplomatStringView source);

void FieldExprBuilder_destroy(FieldExprBuilder* self);





#endif // FieldExprBuilder_H
