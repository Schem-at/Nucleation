#ifndef FieldProgramBuilder_HPP
#define FieldProgramBuilder_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::FieldProgramBuilder* FieldProgramBuilder_create(void);

    typedef struct FieldProgramBuilder_add_slot_result {union {uint16_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_add_slot_result;
    FieldProgramBuilder_add_slot_result FieldProgramBuilder_add_slot(diplomat::capi::FieldProgramBuilder* self, diplomat::capi::FieldProgramValueType value_type);

    typedef struct FieldProgramBuilder_push_const_scalar_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_scalar_result;
    FieldProgramBuilder_push_const_scalar_result FieldProgramBuilder_push_const_scalar(diplomat::capi::FieldProgramBuilder* self, float value);

    typedef struct FieldProgramBuilder_push_const_vec3_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_vec3_result;
    FieldProgramBuilder_push_const_vec3_result FieldProgramBuilder_push_const_vec3(diplomat::capi::FieldProgramBuilder* self, float x, float y, float z);

    typedef struct FieldProgramBuilder_push_const_bool_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_const_bool_result;
    FieldProgramBuilder_push_const_bool_result FieldProgramBuilder_push_const_bool(diplomat::capi::FieldProgramBuilder* self, bool value);

    typedef struct FieldProgramBuilder_push_pos_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_push_pos_result;
    FieldProgramBuilder_push_pos_result FieldProgramBuilder_push_pos(diplomat::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_load_local_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_load_local_result;
    FieldProgramBuilder_load_local_result FieldProgramBuilder_load_local(diplomat::capi::FieldProgramBuilder* self, uint16_t slot);

    typedef struct FieldProgramBuilder_store_local_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_store_local_result;
    FieldProgramBuilder_store_local_result FieldProgramBuilder_store_local(diplomat::capi::FieldProgramBuilder* self, uint16_t slot);

    typedef struct FieldProgramBuilder_pop_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_pop_result;
    FieldProgramBuilder_pop_result FieldProgramBuilder_pop(diplomat::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_unary_op_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_unary_op_result;
    FieldProgramBuilder_unary_op_result FieldProgramBuilder_unary_op(diplomat::capi::FieldProgramBuilder* self, diplomat::capi::FieldProgramUnaryOp op);

    typedef struct FieldProgramBuilder_binary_op_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_binary_op_result;
    FieldProgramBuilder_binary_op_result FieldProgramBuilder_binary_op(diplomat::capi::FieldProgramBuilder* self, diplomat::capi::FieldProgramBinaryOp op);

    typedef struct FieldProgramBuilder_clamp_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_clamp_result;
    FieldProgramBuilder_clamp_result FieldProgramBuilder_clamp(diplomat::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_select_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_select_result;
    FieldProgramBuilder_select_result FieldProgramBuilder_select(diplomat::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_make_vec3_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_make_vec3_result;
    FieldProgramBuilder_make_vec3_result FieldProgramBuilder_make_vec3(diplomat::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_break_if_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_break_if_result;
    FieldProgramBuilder_break_if_result FieldProgramBuilder_break_if(diplomat::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_begin_repeat_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_begin_repeat_result;
    FieldProgramBuilder_begin_repeat_result FieldProgramBuilder_begin_repeat(diplomat::capi::FieldProgramBuilder* self, uint32_t count);

    typedef struct FieldProgramBuilder_end_repeat_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_end_repeat_result;
    FieldProgramBuilder_end_repeat_result FieldProgramBuilder_end_repeat(diplomat::capi::FieldProgramBuilder* self);

    typedef struct FieldProgramBuilder_set_output_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_output_result;
    FieldProgramBuilder_set_output_result FieldProgramBuilder_set_output(diplomat::capi::FieldProgramBuilder* self, uint16_t slot);

    typedef struct FieldProgramBuilder_set_bounds_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_bounds_result;
    FieldProgramBuilder_set_bounds_result FieldProgramBuilder_set_bounds(diplomat::capi::FieldProgramBuilder* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

    typedef struct FieldProgramBuilder_set_distance_kind_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_set_distance_kind_result;
    FieldProgramBuilder_set_distance_kind_result FieldProgramBuilder_set_distance_kind(diplomat::capi::FieldProgramBuilder* self, diplomat::capi::FieldProgramDistanceKind kind);

    typedef struct FieldProgramBuilder_build_result {union {diplomat::capi::FieldProgram* ok; diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgramBuilder_build_result;
    FieldProgramBuilder_build_result FieldProgramBuilder_build(diplomat::capi::FieldProgramBuilder* self);

    void FieldProgramBuilder_destroy(FieldProgramBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<FieldProgramBuilder> FieldProgramBuilder::create() {
    auto result = diplomat::capi::FieldProgramBuilder_create();
    return std::unique_ptr<FieldProgramBuilder>(FieldProgramBuilder::FromFFI(result));
}

inline diplomat::result<uint16_t, NucleationError> FieldProgramBuilder::add_slot(FieldProgramValueType value_type) {
    auto result = diplomat::capi::FieldProgramBuilder_add_slot(this->AsFFI(),
        value_type.AsFFI());
    return result.is_ok ? diplomat::result<uint16_t, NucleationError>(diplomat::Ok<uint16_t>(result.ok)) : diplomat::result<uint16_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::push_const_scalar(float value) {
    auto result = diplomat::capi::FieldProgramBuilder_push_const_scalar(this->AsFFI(),
        value);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::push_const_vec3(float x, float y, float z) {
    auto result = diplomat::capi::FieldProgramBuilder_push_const_vec3(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::push_const_bool(bool value) {
    auto result = diplomat::capi::FieldProgramBuilder_push_const_bool(this->AsFFI(),
        value);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::push_pos() {
    auto result = diplomat::capi::FieldProgramBuilder_push_pos(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::load_local(uint16_t slot) {
    auto result = diplomat::capi::FieldProgramBuilder_load_local(this->AsFFI(),
        slot);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::store_local(uint16_t slot) {
    auto result = diplomat::capi::FieldProgramBuilder_store_local(this->AsFFI(),
        slot);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::pop() {
    auto result = diplomat::capi::FieldProgramBuilder_pop(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::unary_op(FieldProgramUnaryOp op) {
    auto result = diplomat::capi::FieldProgramBuilder_unary_op(this->AsFFI(),
        op.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::binary_op(FieldProgramBinaryOp op) {
    auto result = diplomat::capi::FieldProgramBuilder_binary_op(this->AsFFI(),
        op.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::clamp() {
    auto result = diplomat::capi::FieldProgramBuilder_clamp(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::select() {
    auto result = diplomat::capi::FieldProgramBuilder_select(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::make_vec3() {
    auto result = diplomat::capi::FieldProgramBuilder_make_vec3(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::break_if() {
    auto result = diplomat::capi::FieldProgramBuilder_break_if(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::begin_repeat(uint32_t count) {
    auto result = diplomat::capi::FieldProgramBuilder_begin_repeat(this->AsFFI(),
        count);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::end_repeat() {
    auto result = diplomat::capi::FieldProgramBuilder_end_repeat(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::set_output(uint16_t slot) {
    auto result = diplomat::capi::FieldProgramBuilder_set_output(this->AsFFI(),
        slot);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) {
    auto result = diplomat::capi::FieldProgramBuilder_set_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldProgramBuilder::set_distance_kind(FieldProgramDistanceKind kind) {
    auto result = diplomat::capi::FieldProgramBuilder_set_distance_kind(this->AsFFI(),
        kind.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<FieldProgram>, NucleationError> FieldProgramBuilder::build() {
    auto result = diplomat::capi::FieldProgramBuilder_build(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<FieldProgram>, NucleationError>(diplomat::Ok<std::unique_ptr<FieldProgram>>(std::unique_ptr<FieldProgram>(FieldProgram::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<FieldProgram>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::FieldProgramBuilder* FieldProgramBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::FieldProgramBuilder*>(this);
}

inline diplomat::capi::FieldProgramBuilder* FieldProgramBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::FieldProgramBuilder*>(this);
}

inline const FieldProgramBuilder* FieldProgramBuilder::FromFFI(const diplomat::capi::FieldProgramBuilder* ptr) {
    return reinterpret_cast<const FieldProgramBuilder*>(ptr);
}

inline FieldProgramBuilder* FieldProgramBuilder::FromFFI(diplomat::capi::FieldProgramBuilder* ptr) {
    return reinterpret_cast<FieldProgramBuilder*>(ptr);
}

inline void FieldProgramBuilder::operator delete(void* ptr) {
    diplomat::capi::FieldProgramBuilder_destroy(reinterpret_cast<diplomat::capi::FieldProgramBuilder*>(ptr));
}


#endif // FieldProgramBuilder_HPP
