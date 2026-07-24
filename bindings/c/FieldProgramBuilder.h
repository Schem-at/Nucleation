#ifndef FieldProgramBuilder_H
#define FieldProgramBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "FieldProgram.d.h"
#include "FieldProgramBinaryOp.d.h"
#include "FieldProgramDistanceKind.d.h"
#include "FieldProgramUnaryOp.d.h"
#include "FieldProgramValueType.d.h"
#include "NucleationError.d.h"

#include "FieldProgramBuilder.d.h"






FieldProgramBuilder* FieldProgramBuilder_create(void);

typedef struct FieldProgramBuilder_add_slot_result {union {uint16_t ok; NucleationError err;}; bool is_ok;} FieldProgramBuilder_add_slot_result;
FieldProgramBuilder_add_slot_result FieldProgramBuilder_add_slot(FieldProgramBuilder* self, FieldProgramValueType value_type);

typedef struct FieldProgramBuilder_push_const_scalar_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_scalar_result;
FieldProgramBuilder_push_const_scalar_result FieldProgramBuilder_push_const_scalar(FieldProgramBuilder* self, float value);

typedef struct FieldProgramBuilder_push_const_vec3_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_vec3_result;
FieldProgramBuilder_push_const_vec3_result FieldProgramBuilder_push_const_vec3(FieldProgramBuilder* self, float x, float y, float z);

typedef struct FieldProgramBuilder_push_const_bool_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_bool_result;
FieldProgramBuilder_push_const_bool_result FieldProgramBuilder_push_const_bool(FieldProgramBuilder* self, bool value);

typedef struct FieldProgramBuilder_push_pos_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_pos_result;
FieldProgramBuilder_push_pos_result FieldProgramBuilder_push_pos(FieldProgramBuilder* self);

typedef struct FieldProgramBuilder_load_local_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_load_local_result;
FieldProgramBuilder_load_local_result FieldProgramBuilder_load_local(FieldProgramBuilder* self, uint16_t slot);

typedef struct FieldProgramBuilder_store_local_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_store_local_result;
FieldProgramBuilder_store_local_result FieldProgramBuilder_store_local(FieldProgramBuilder* self, uint16_t slot);

typedef struct FieldProgramBuilder_pop_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_pop_result;
FieldProgramBuilder_pop_result FieldProgramBuilder_pop(FieldProgramBuilder* self);

typedef struct FieldProgramBuilder_unary_op_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_unary_op_result;
FieldProgramBuilder_unary_op_result FieldProgramBuilder_unary_op(FieldProgramBuilder* self, FieldProgramUnaryOp op);

typedef struct FieldProgramBuilder_binary_op_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_binary_op_result;
FieldProgramBuilder_binary_op_result FieldProgramBuilder_binary_op(FieldProgramBuilder* self, FieldProgramBinaryOp op);

typedef struct FieldProgramBuilder_clamp_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_clamp_result;
FieldProgramBuilder_clamp_result FieldProgramBuilder_clamp(FieldProgramBuilder* self);

typedef struct FieldProgramBuilder_select_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_select_result;
FieldProgramBuilder_select_result FieldProgramBuilder_select(FieldProgramBuilder* self);

typedef struct FieldProgramBuilder_make_vec3_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_make_vec3_result;
FieldProgramBuilder_make_vec3_result FieldProgramBuilder_make_vec3(FieldProgramBuilder* self);

typedef struct FieldProgramBuilder_break_if_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_break_if_result;
FieldProgramBuilder_break_if_result FieldProgramBuilder_break_if(FieldProgramBuilder* self);

typedef struct FieldProgramBuilder_begin_repeat_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_begin_repeat_result;
FieldProgramBuilder_begin_repeat_result FieldProgramBuilder_begin_repeat(FieldProgramBuilder* self, uint32_t count);

typedef struct FieldProgramBuilder_end_repeat_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_end_repeat_result;
FieldProgramBuilder_end_repeat_result FieldProgramBuilder_end_repeat(FieldProgramBuilder* self);

typedef struct FieldProgramBuilder_set_output_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_output_result;
FieldProgramBuilder_set_output_result FieldProgramBuilder_set_output(FieldProgramBuilder* self, uint16_t slot);

typedef struct FieldProgramBuilder_set_bounds_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_bounds_result;
FieldProgramBuilder_set_bounds_result FieldProgramBuilder_set_bounds(FieldProgramBuilder* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

typedef struct FieldProgramBuilder_set_distance_kind_result {union { NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_distance_kind_result;
FieldProgramBuilder_set_distance_kind_result FieldProgramBuilder_set_distance_kind(FieldProgramBuilder* self, FieldProgramDistanceKind kind);

typedef struct FieldProgramBuilder_build_result {union {FieldProgram* ok; NucleationError err;}; bool is_ok;} FieldProgramBuilder_build_result;
FieldProgramBuilder_build_result FieldProgramBuilder_build(FieldProgramBuilder* self);

void FieldProgramBuilder_destroy(FieldProgramBuilder* self);





#endif // FieldProgramBuilder_H
