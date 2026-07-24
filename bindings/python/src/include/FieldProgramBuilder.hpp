#ifndef NUCLEATION_FieldProgramBuilder_HPP
#define NUCLEATION_FieldProgramBuilder_HPP

#include "FieldProgramBuilder.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "FieldProgram.hpp"
#include "FieldProgramBinaryOp.hpp"
#include "FieldProgramDistanceKind.hpp"
#include "FieldProgramUnaryOp.hpp"
#include "FieldProgramValueType.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::FieldProgramBuilder* FieldProgramBuilder_create(void);

    typedef struct FieldProgramBuilder_add_slot_result {union {uint16_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_add_slot_result;
    FieldProgramBuilder_add_slot_result FieldProgramBuilder_add_slot(nucleation::capi::FieldProgramBuilder* self, nucleation::capi::FieldProgramValueType value_type);

    typedef struct FieldProgramBuilder_push_const_scalar_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_scalar_result;
    FieldProgramBuilder_push_const_scalar_result FieldProgramBuilder_push_const_scalar(nucleation::capi::FieldProgramBuilder* self, float value);

    typedef struct FieldProgramBuilder_push_const_vec3_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_vec3_result;
    FieldProgramBuilder_push_const_vec3_result FieldProgramBuilder_push_const_vec3(nucleation::capi::FieldProgramBuilder* self, float x, float y, float z);

    typedef struct FieldProgramBuilder_push_const_bool_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_bool_result;
    FieldProgramBuilder_push_const_bool_result FieldProgramBuilder_push_const_bool(nucleation::capi::FieldProgramBuilder* self, bool value);

    typedef struct FieldProgramBuilder_push_pos_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_pos_result;
    FieldProgramBuilder_push_pos_result FieldProgramBuilder_push_pos(nucleation::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_load_local_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_load_local_result;
    FieldProgramBuilder_load_local_result FieldProgramBuilder_load_local(nucleation::capi::FieldProgramBuilder* self, uint16_t slot);

    typedef struct FieldProgramBuilder_store_local_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_store_local_result;
    FieldProgramBuilder_store_local_result FieldProgramBuilder_store_local(nucleation::capi::FieldProgramBuilder* self, uint16_t slot);

    typedef struct FieldProgramBuilder_pop_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_pop_result;
    FieldProgramBuilder_pop_result FieldProgramBuilder_pop(nucleation::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_unary_op_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_unary_op_result;
    FieldProgramBuilder_unary_op_result FieldProgramBuilder_unary_op(nucleation::capi::FieldProgramBuilder* self, nucleation::capi::FieldProgramUnaryOp op);

    typedef struct FieldProgramBuilder_binary_op_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_binary_op_result;
    FieldProgramBuilder_binary_op_result FieldProgramBuilder_binary_op(nucleation::capi::FieldProgramBuilder* self, nucleation::capi::FieldProgramBinaryOp op);

    typedef struct FieldProgramBuilder_clamp_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_clamp_result;
    FieldProgramBuilder_clamp_result FieldProgramBuilder_clamp(nucleation::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_select_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_select_result;
    FieldProgramBuilder_select_result FieldProgramBuilder_select(nucleation::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_make_vec3_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_make_vec3_result;
    FieldProgramBuilder_make_vec3_result FieldProgramBuilder_make_vec3(nucleation::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_break_if_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_break_if_result;
    FieldProgramBuilder_break_if_result FieldProgramBuilder_break_if(nucleation::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_begin_repeat_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_begin_repeat_result;
    FieldProgramBuilder_begin_repeat_result FieldProgramBuilder_begin_repeat(nucleation::capi::FieldProgramBuilder* self, uint32_t count);

    typedef struct FieldProgramBuilder_end_repeat_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_end_repeat_result;
    FieldProgramBuilder_end_repeat_result FieldProgramBuilder_end_repeat(nucleation::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_set_output_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_output_result;
    FieldProgramBuilder_set_output_result FieldProgramBuilder_set_output(nucleation::capi::FieldProgramBuilder* self, uint16_t slot);

    typedef struct FieldProgramBuilder_set_bounds_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_bounds_result;
    FieldProgramBuilder_set_bounds_result FieldProgramBuilder_set_bounds(nucleation::capi::FieldProgramBuilder* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

    typedef struct FieldProgramBuilder_set_distance_kind_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_distance_kind_result;
    FieldProgramBuilder_set_distance_kind_result FieldProgramBuilder_set_distance_kind(nucleation::capi::FieldProgramBuilder* self, nucleation::capi::FieldProgramDistanceKind kind);

    typedef struct FieldProgramBuilder_build_result {union {nucleation::capi::FieldProgram* ok; nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_build_result;
    FieldProgramBuilder_build_result FieldProgramBuilder_build(nucleation::capi::FieldProgramBuilder* self);

    void FieldProgramBuilder_destroy(FieldProgramBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::FieldProgramBuilder> nucleation::FieldProgramBuilder::create() {
    auto result = nucleation::capi::FieldProgramBuilder_create();
    return std::unique_ptr<nucleation::FieldProgramBuilder>(nucleation::FieldProgramBuilder::FromFFI(result));
}

inline nucleation::diplomat::result<uint16_t, nucleation::NucleationError> nucleation::FieldProgramBuilder::add_slot(nucleation::FieldProgramValueType value_type) {
    auto result = nucleation::capi::FieldProgramBuilder_add_slot(this->AsFFI(),
        value_type.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<uint16_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint16_t>(result.ok)) : nucleation::diplomat::result<uint16_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::push_const_scalar(float value) {
    auto result = nucleation::capi::FieldProgramBuilder_push_const_scalar(this->AsFFI(),
        value);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::push_const_vec3(float x, float y, float z) {
    auto result = nucleation::capi::FieldProgramBuilder_push_const_vec3(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::push_const_bool(bool value) {
    auto result = nucleation::capi::FieldProgramBuilder_push_const_bool(this->AsFFI(),
        value);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::push_pos() {
    auto result = nucleation::capi::FieldProgramBuilder_push_pos(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::load_local(uint16_t slot) {
    auto result = nucleation::capi::FieldProgramBuilder_load_local(this->AsFFI(),
        slot);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::store_local(uint16_t slot) {
    auto result = nucleation::capi::FieldProgramBuilder_store_local(this->AsFFI(),
        slot);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::pop() {
    auto result = nucleation::capi::FieldProgramBuilder_pop(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::unary_op(nucleation::FieldProgramUnaryOp op) {
    auto result = nucleation::capi::FieldProgramBuilder_unary_op(this->AsFFI(),
        op.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::binary_op(nucleation::FieldProgramBinaryOp op) {
    auto result = nucleation::capi::FieldProgramBuilder_binary_op(this->AsFFI(),
        op.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::clamp() {
    auto result = nucleation::capi::FieldProgramBuilder_clamp(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::select() {
    auto result = nucleation::capi::FieldProgramBuilder_select(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::make_vec3() {
    auto result = nucleation::capi::FieldProgramBuilder_make_vec3(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::break_if() {
    auto result = nucleation::capi::FieldProgramBuilder_break_if(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::begin_repeat(uint32_t count) {
    auto result = nucleation::capi::FieldProgramBuilder_begin_repeat(this->AsFFI(),
        count);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::end_repeat() {
    auto result = nucleation::capi::FieldProgramBuilder_end_repeat(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::set_output(uint16_t slot) {
    auto result = nucleation::capi::FieldProgramBuilder_set_output(this->AsFFI(),
        slot);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) {
    auto result = nucleation::capi::FieldProgramBuilder_set_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgramBuilder::set_distance_kind(nucleation::FieldProgramDistanceKind kind) {
    auto result = nucleation::capi::FieldProgramBuilder_set_distance_kind(this->AsFFI(),
        kind.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError> nucleation::FieldProgramBuilder::build() {
    auto result = nucleation::capi::FieldProgramBuilder_build(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::FieldProgram>>(std::unique_ptr<nucleation::FieldProgram>(nucleation::FieldProgram::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::FieldProgramBuilder* nucleation::FieldProgramBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::FieldProgramBuilder*>(this);
}

inline nucleation::capi::FieldProgramBuilder* nucleation::FieldProgramBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::FieldProgramBuilder*>(this);
}

inline const nucleation::FieldProgramBuilder* nucleation::FieldProgramBuilder::FromFFI(const nucleation::capi::FieldProgramBuilder* ptr) {
    return reinterpret_cast<const nucleation::FieldProgramBuilder*>(ptr);
}

inline nucleation::FieldProgramBuilder* nucleation::FieldProgramBuilder::FromFFI(nucleation::capi::FieldProgramBuilder* ptr) {
    return reinterpret_cast<nucleation::FieldProgramBuilder*>(ptr);
}

inline void nucleation::FieldProgramBuilder::operator delete(void* ptr) {
    nucleation::capi::FieldProgramBuilder_destroy(reinterpret_cast<nucleation::capi::FieldProgramBuilder*>(ptr));
}


#endif // NUCLEATION_FieldProgramBuilder_HPP
