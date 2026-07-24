#ifndef FieldProgram_H
#define FieldProgram_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "FieldProgramDistanceKind.d.h"
#include "NucleationError.d.h"
#include "SdfBounds.d.h"
#include "SdfNormal.d.h"

#include "FieldProgram.d.h"






typedef struct FieldProgram_from_json_string_result {union {FieldProgram* ok; NucleationError err;}; bool is_ok;} FieldProgram_from_json_string_result;
FieldProgram_from_json_string_result FieldProgram_from_json_string(DiplomatStringView json);

typedef struct FieldProgram_to_json_result {union { NucleationError err;}; bool is_ok;} FieldProgram_to_json_result;
FieldProgram_to_json_result FieldProgram_to_json(const FieldProgram* self, DiplomatWrite* write);

float FieldProgram_eval_at(const FieldProgram* self, float x, float y, float z);

typedef struct FieldProgram_gradient_result {union {SdfNormal ok; NucleationError err;}; bool is_ok;} FieldProgram_gradient_result;
FieldProgram_gradient_result FieldProgram_gradient(const FieldProgram* self, float x, float y, float z, float epsilon);

SdfBounds FieldProgram_bounds(const FieldProgram* self);

FieldProgramDistanceKind FieldProgram_distance_kind(const FieldProgram* self);

void FieldProgram_destroy(FieldProgram* self);





#endif // FieldProgram_H
