#ifndef Value_H
#define Value_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "Value.d.h"






Value* Value_from_u32(uint32_t v);

Value* Value_from_i32(int32_t v);

Value* Value_from_f32(float v);

Value* Value_from_bool(bool v);

typedef struct Value_from_string_result {union {Value* ok; NucleationError err;}; bool is_ok;} Value_from_string_result;
Value_from_string_result Value_from_string(DiplomatStringView s);

typedef struct Value_as_u32_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} Value_as_u32_result;
Value_as_u32_result Value_as_u32(const Value* self);

typedef struct Value_as_i32_result {union {int32_t ok; NucleationError err;}; bool is_ok;} Value_as_i32_result;
Value_as_i32_result Value_as_i32(const Value* self);

typedef struct Value_as_f32_result {union {float ok; NucleationError err;}; bool is_ok;} Value_as_f32_result;
Value_as_f32_result Value_as_f32(const Value* self);

typedef struct Value_as_bool_result {union {bool ok; NucleationError err;}; bool is_ok;} Value_as_bool_result;
Value_as_bool_result Value_as_bool(const Value* self);

typedef struct Value_as_string_result {union { NucleationError err;}; bool is_ok;} Value_as_string_result;
Value_as_string_result Value_as_string(const Value* self, DiplomatWrite* write);

void Value_type_name(const Value* self, DiplomatWrite* write);

void Value_destroy(Value* self);





#endif // Value_H
