#ifndef NUCLEATION_MaterialCondition_HPP
#define NUCLEATION_MaterialCondition_HPP

#include "MaterialCondition.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Field.hpp"
#include "FieldChannel.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::MaterialCondition* MaterialCondition_always(void);

    nucleation::capi::MaterialCondition* MaterialCondition_and_with(const nucleation::capi::MaterialCondition* self, const nucleation::capi::MaterialCondition* other);

    nucleation::capi::MaterialCondition* MaterialCondition_or_with(const nucleation::capi::MaterialCondition* self, const nucleation::capi::MaterialCondition* other);

    nucleation::capi::MaterialCondition* MaterialCondition_negated(const nucleation::capi::MaterialCondition* self);

    nucleation::capi::MaterialCondition* MaterialCondition_tag(uint16_t material_id);

    typedef struct MaterialCondition_channel_range_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_channel_range_result;
    MaterialCondition_channel_range_result MaterialCondition_channel_range(nucleation::capi::FieldChannel channel, bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_field_range_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_field_range_result;
    MaterialCondition_field_range_result MaterialCondition_field_range(const nucleation::capi::Field* field, bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_y_range_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_range_result;
    MaterialCondition_y_range_result MaterialCondition_y_range(bool has_min, float min, bool has_max, float max);

    nucleation::capi::MaterialCondition* MaterialCondition_depth_below_surface(bool has_min, int32_t min, bool has_max, int32_t max);

    typedef struct MaterialCondition_slope_range_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_slope_range_result;
    MaterialCondition_slope_range_result MaterialCondition_slope_range(bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_curvature_range_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_curvature_range_result;
    MaterialCondition_curvature_range_result MaterialCondition_curvature_range(bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_normal_dot_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_normal_dot_result;
    MaterialCondition_normal_dot_result MaterialCondition_normal_dot(float dx, float dy, float dz, bool has_min, float min, bool has_max, float max);

    nucleation::capi::MaterialCondition* MaterialCondition_exposed_face(uint8_t mask, bool require_all);

    nucleation::capi::MaterialCondition* MaterialCondition_exposed_top(void);

    nucleation::capi::MaterialCondition* MaterialCondition_exposed_bottom(void);

    nucleation::capi::MaterialCondition* MaterialCondition_exposed_side(void);

    nucleation::capi::MaterialCondition* MaterialCondition_exposed_any(void);

    nucleation::capi::MaterialCondition* MaterialCondition_interior(void);

    typedef struct MaterialCondition_y_above_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_above_result;
    MaterialCondition_y_above_result MaterialCondition_y_above(float min_y);

    typedef struct MaterialCondition_y_below_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_below_result;
    MaterialCondition_y_below_result MaterialCondition_y_below(float max_y);

    typedef struct MaterialCondition_y_between_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_between_result;
    MaterialCondition_y_between_result MaterialCondition_y_between(float min_y, float max_y);

    typedef struct MaterialCondition_slope_below_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_slope_below_result;
    MaterialCondition_slope_below_result MaterialCondition_slope_below(float max_slope);

    typedef struct MaterialCondition_slope_above_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_slope_above_result;
    MaterialCondition_slope_above_result MaterialCondition_slope_above(float min_slope);

    nucleation::capi::MaterialCondition* MaterialCondition_depth_between(int32_t min_depth, int32_t max_depth);

    typedef struct MaterialCondition_chance_result {union {nucleation::capi::MaterialCondition* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MaterialCondition_chance_result;
    MaterialCondition_chance_result MaterialCondition_chance(float probability, int32_t seed);

    void MaterialCondition_destroy(MaterialCondition* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::always() {
    auto result = nucleation::capi::MaterialCondition_always();
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::and_with(const nucleation::MaterialCondition& other) const {
    auto result = nucleation::capi::MaterialCondition_and_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::or_with(const nucleation::MaterialCondition& other) const {
    auto result = nucleation::capi::MaterialCondition_or_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::negated() const {
    auto result = nucleation::capi::MaterialCondition_negated(this->AsFFI());
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::tag(uint16_t material_id) {
    auto result = nucleation::capi::MaterialCondition_tag(material_id);
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::channel_range(nucleation::FieldChannel channel, bool has_min, float min, bool has_max, float max) {
    auto result = nucleation::capi::MaterialCondition_channel_range(channel.AsFFI(),
        has_min,
        min,
        has_max,
        max);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::field_range(const nucleation::Field& field, bool has_min, float min, bool has_max, float max) {
    auto result = nucleation::capi::MaterialCondition_field_range(field.AsFFI(),
        has_min,
        min,
        has_max,
        max);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::y_range(bool has_min, float min, bool has_max, float max) {
    auto result = nucleation::capi::MaterialCondition_y_range(has_min,
        min,
        has_max,
        max);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::depth_below_surface(bool has_min, int32_t min, bool has_max, int32_t max) {
    auto result = nucleation::capi::MaterialCondition_depth_below_surface(has_min,
        min,
        has_max,
        max);
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::slope_range(bool has_min, float min, bool has_max, float max) {
    auto result = nucleation::capi::MaterialCondition_slope_range(has_min,
        min,
        has_max,
        max);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::curvature_range(bool has_min, float min, bool has_max, float max) {
    auto result = nucleation::capi::MaterialCondition_curvature_range(has_min,
        min,
        has_max,
        max);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::normal_dot(float dx, float dy, float dz, bool has_min, float min, bool has_max, float max) {
    auto result = nucleation::capi::MaterialCondition_normal_dot(dx,
        dy,
        dz,
        has_min,
        min,
        has_max,
        max);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::exposed_face(uint8_t mask, bool require_all) {
    auto result = nucleation::capi::MaterialCondition_exposed_face(mask,
        require_all);
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::exposed_top() {
    auto result = nucleation::capi::MaterialCondition_exposed_top();
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::exposed_bottom() {
    auto result = nucleation::capi::MaterialCondition_exposed_bottom();
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::exposed_side() {
    auto result = nucleation::capi::MaterialCondition_exposed_side();
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::exposed_any() {
    auto result = nucleation::capi::MaterialCondition_exposed_any();
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::interior() {
    auto result = nucleation::capi::MaterialCondition_interior();
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::y_above(float min_y) {
    auto result = nucleation::capi::MaterialCondition_y_above(min_y);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::y_below(float max_y) {
    auto result = nucleation::capi::MaterialCondition_y_below(max_y);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::y_between(float min_y, float max_y) {
    auto result = nucleation::capi::MaterialCondition_y_between(min_y,
        max_y);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::slope_below(float max_slope) {
    auto result = nucleation::capi::MaterialCondition_slope_below(max_slope);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::slope_above(float min_slope) {
    auto result = nucleation::capi::MaterialCondition_slope_above(min_slope);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::MaterialCondition> nucleation::MaterialCondition::depth_between(int32_t min_depth, int32_t max_depth) {
    auto result = nucleation::capi::MaterialCondition_depth_between(min_depth,
        max_depth);
    return std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> nucleation::MaterialCondition::chance(float probability, int32_t seed) {
    auto result = nucleation::capi::MaterialCondition_chance(probability,
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MaterialCondition>>(std::unique_ptr<nucleation::MaterialCondition>(nucleation::MaterialCondition::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::MaterialCondition* nucleation::MaterialCondition::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MaterialCondition*>(this);
}

inline nucleation::capi::MaterialCondition* nucleation::MaterialCondition::AsFFI() {
    return reinterpret_cast<nucleation::capi::MaterialCondition*>(this);
}

inline const nucleation::MaterialCondition* nucleation::MaterialCondition::FromFFI(const nucleation::capi::MaterialCondition* ptr) {
    return reinterpret_cast<const nucleation::MaterialCondition*>(ptr);
}

inline nucleation::MaterialCondition* nucleation::MaterialCondition::FromFFI(nucleation::capi::MaterialCondition* ptr) {
    return reinterpret_cast<nucleation::MaterialCondition*>(ptr);
}

inline void nucleation::MaterialCondition::operator delete(void* ptr) {
    nucleation::capi::MaterialCondition_destroy(reinterpret_cast<nucleation::capi::MaterialCondition*>(ptr));
}


#endif // NUCLEATION_MaterialCondition_HPP
