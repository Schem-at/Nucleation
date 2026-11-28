#ifndef MaterialCondition_HPP
#define MaterialCondition_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::MaterialCondition* MaterialCondition_always(void);

    diplomat::capi::MaterialCondition* MaterialCondition_and_with(const diplomat::capi::MaterialCondition* self, const diplomat::capi::MaterialCondition* other);

    diplomat::capi::MaterialCondition* MaterialCondition_or_with(const diplomat::capi::MaterialCondition* self, const diplomat::capi::MaterialCondition* other);

    diplomat::capi::MaterialCondition* MaterialCondition_negated(const diplomat::capi::MaterialCondition* self);

    diplomat::capi::MaterialCondition* MaterialCondition_tag(uint16_t material_id);

    typedef struct MaterialCondition_channel_range_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_channel_range_result;
    MaterialCondition_channel_range_result MaterialCondition_channel_range(diplomat::capi::FieldChannel channel, bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_field_range_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_field_range_result;
    MaterialCondition_field_range_result MaterialCondition_field_range(const diplomat::capi::Field* field, bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_y_range_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_range_result;
    MaterialCondition_y_range_result MaterialCondition_y_range(bool has_min, float min, bool has_max, float max);

    diplomat::capi::MaterialCondition* MaterialCondition_depth_below_surface(bool has_min, int32_t min, bool has_max, int32_t max);

    typedef struct MaterialCondition_slope_range_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_slope_range_result;
    MaterialCondition_slope_range_result MaterialCondition_slope_range(bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_curvature_range_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_curvature_range_result;
    MaterialCondition_curvature_range_result MaterialCondition_curvature_range(bool has_min, float min, bool has_max, float max);

    typedef struct MaterialCondition_normal_dot_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_normal_dot_result;
    MaterialCondition_normal_dot_result MaterialCondition_normal_dot(float dx, float dy, float dz, bool has_min, float min, bool has_max, float max);

    diplomat::capi::MaterialCondition* MaterialCondition_exposed_face(uint8_t mask, bool require_all);

    diplomat::capi::MaterialCondition* MaterialCondition_exposed_top(void);

    diplomat::capi::MaterialCondition* MaterialCondition_exposed_bottom(void);

    diplomat::capi::MaterialCondition* MaterialCondition_exposed_side(void);

    diplomat::capi::MaterialCondition* MaterialCondition_exposed_any(void);

    diplomat::capi::MaterialCondition* MaterialCondition_interior(void);

    typedef struct MaterialCondition_y_above_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_above_result;
    MaterialCondition_y_above_result MaterialCondition_y_above(float min_y);

    typedef struct MaterialCondition_y_below_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_below_result;
    MaterialCondition_y_below_result MaterialCondition_y_below(float max_y);

    typedef struct MaterialCondition_y_between_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_y_between_result;
    MaterialCondition_y_between_result MaterialCondition_y_between(float min_y, float max_y);

    typedef struct MaterialCondition_slope_below_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_slope_below_result;
    MaterialCondition_slope_below_result MaterialCondition_slope_below(float max_slope);

    typedef struct MaterialCondition_slope_above_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_slope_above_result;
    MaterialCondition_slope_above_result MaterialCondition_slope_above(float min_slope);

    diplomat::capi::MaterialCondition* MaterialCondition_depth_between(int32_t min_depth, int32_t max_depth);

    typedef struct MaterialCondition_chance_result {union {diplomat::capi::MaterialCondition* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MaterialCondition_chance_result;
    MaterialCondition_chance_result MaterialCondition_chance(float probability, int32_t seed);

    void MaterialCondition_destroy(MaterialCondition* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<MaterialCondition> MaterialCondition::always() {
    auto result = diplomat::capi::MaterialCondition_always();
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::and_with(const MaterialCondition& other) const {
    auto result = diplomat::capi::MaterialCondition_and_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::or_with(const MaterialCondition& other) const {
    auto result = diplomat::capi::MaterialCondition_or_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::negated() const {
    auto result = diplomat::capi::MaterialCondition_negated(this->AsFFI());
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::tag(uint16_t material_id) {
    auto result = diplomat::capi::MaterialCondition_tag(material_id);
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::channel_range(FieldChannel channel, bool has_min, float min, bool has_max, float max) {
    auto result = diplomat::capi::MaterialCondition_channel_range(channel.AsFFI(),
        has_min,
        min,
        has_max,
        max);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::field_range(const Field& field, bool has_min, float min, bool has_max, float max) {
    auto result = diplomat::capi::MaterialCondition_field_range(field.AsFFI(),
        has_min,
        min,
        has_max,
        max);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::y_range(bool has_min, float min, bool has_max, float max) {
    auto result = diplomat::capi::MaterialCondition_y_range(has_min,
        min,
        has_max,
        max);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::depth_below_surface(bool has_min, int32_t min, bool has_max, int32_t max) {
    auto result = diplomat::capi::MaterialCondition_depth_below_surface(has_min,
        min,
        has_max,
        max);
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::slope_range(bool has_min, float min, bool has_max, float max) {
    auto result = diplomat::capi::MaterialCondition_slope_range(has_min,
        min,
        has_max,
        max);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::curvature_range(bool has_min, float min, bool has_max, float max) {
    auto result = diplomat::capi::MaterialCondition_curvature_range(has_min,
        min,
        has_max,
        max);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::normal_dot(float dx, float dy, float dz, bool has_min, float min, bool has_max, float max) {
    auto result = diplomat::capi::MaterialCondition_normal_dot(dx,
        dy,
        dz,
        has_min,
        min,
        has_max,
        max);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::exposed_face(uint8_t mask, bool require_all) {
    auto result = diplomat::capi::MaterialCondition_exposed_face(mask,
        require_all);
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::exposed_top() {
    auto result = diplomat::capi::MaterialCondition_exposed_top();
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::exposed_bottom() {
    auto result = diplomat::capi::MaterialCondition_exposed_bottom();
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::exposed_side() {
    auto result = diplomat::capi::MaterialCondition_exposed_side();
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::exposed_any() {
    auto result = diplomat::capi::MaterialCondition_exposed_any();
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::interior() {
    auto result = diplomat::capi::MaterialCondition_interior();
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::y_above(float min_y) {
    auto result = diplomat::capi::MaterialCondition_y_above(min_y);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::y_below(float max_y) {
    auto result = diplomat::capi::MaterialCondition_y_below(max_y);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::y_between(float min_y, float max_y) {
    auto result = diplomat::capi::MaterialCondition_y_between(min_y,
        max_y);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::slope_below(float max_slope) {
    auto result = diplomat::capi::MaterialCondition_slope_below(max_slope);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::slope_above(float min_slope) {
    auto result = diplomat::capi::MaterialCondition_slope_above(min_slope);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<MaterialCondition> MaterialCondition::depth_between(int32_t min_depth, int32_t max_depth) {
    auto result = diplomat::capi::MaterialCondition_depth_between(min_depth,
        max_depth);
    return std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> MaterialCondition::chance(float probability, int32_t seed) {
    auto result = diplomat::capi::MaterialCondition_chance(probability,
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Ok<std::unique_ptr<MaterialCondition>>(std::unique_ptr<MaterialCondition>(MaterialCondition::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::MaterialCondition* MaterialCondition::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MaterialCondition*>(this);
}

inline diplomat::capi::MaterialCondition* MaterialCondition::AsFFI() {
    return reinterpret_cast<diplomat::capi::MaterialCondition*>(this);
}

inline const MaterialCondition* MaterialCondition::FromFFI(const diplomat::capi::MaterialCondition* ptr) {
    return reinterpret_cast<const MaterialCondition*>(ptr);
}

inline MaterialCondition* MaterialCondition::FromFFI(diplomat::capi::MaterialCondition* ptr) {
    return reinterpret_cast<MaterialCondition*>(ptr);
}

inline void MaterialCondition::operator delete(void* ptr) {
    diplomat::capi::MaterialCondition_destroy(reinterpret_cast<diplomat::capi::MaterialCondition*>(ptr));
}


#endif // MaterialCondition_HPP
