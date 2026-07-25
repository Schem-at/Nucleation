#ifndef Field3_H
#define Field3_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "FieldRange.d.h"
#include "NucleationError.d.h"

#include "Field3.d.h"






typedef struct Field3_value_noise_fbm_result {union {Field3* ok; NucleationError err;}; bool is_ok;} Field3_value_noise_fbm_result;
Field3_value_noise_fbm_result Field3_value_noise_fbm(float frequency, int32_t seed, uint32_t octaves);

float Field3_eval_at(const Field3* self, float x, float y, float z);

typedef struct Field3_output_range_result {union {FieldRange ok; NucleationError err;}; bool is_ok;} Field3_output_range_result;
Field3_output_range_result Field3_output_range(const Field3* self);

typedef struct Field3_from_json_string_result {union {Field3* ok; NucleationError err;}; bool is_ok;} Field3_from_json_string_result;
Field3_from_json_string_result Field3_from_json_string(DiplomatStringView json);

typedef struct Field3_to_json_result {union { NucleationError err;}; bool is_ok;} Field3_to_json_result;
Field3_to_json_result Field3_to_json(const Field3* self, DiplomatWrite* write);

void Field3_destroy(Field3* self);





#endif // Field3_H
