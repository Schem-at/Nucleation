#ifndef NUCLEATION_Sdf_HPP
#define NUCLEATION_Sdf_HPP

#include "Sdf.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "SdfAxis.hpp"
#include "SdfBounds.hpp"
#include "SdfCellMode.hpp"
#include "SdfNormal.hpp"
#include "Shape.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Sdf_sphere_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_sphere_result;
    Sdf_sphere_result Sdf_sphere(float radius);

    typedef struct Sdf_box_shape_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_box_shape_result;
    Sdf_box_shape_result Sdf_box_shape(float half_x, float half_y, float half_z, float rounding);

    typedef struct Sdf_ellipsoid_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_ellipsoid_result;
    Sdf_ellipsoid_result Sdf_ellipsoid(float radius_x, float radius_y, float radius_z);

    typedef struct Sdf_torus_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_torus_result;
    Sdf_torus_result Sdf_torus(float major_radius, float minor_radius);

    typedef struct Sdf_capsule_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_capsule_result;
    Sdf_capsule_result Sdf_capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

    typedef struct Sdf_capped_cylinder_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_capped_cylinder_result;
    Sdf_capped_cylinder_result Sdf_capped_cylinder(float radius, float half_height);

    typedef struct Sdf_capped_cone_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_capped_cone_result;
    Sdf_capped_cone_result Sdf_capped_cone(float half_height, float bottom_radius, float top_radius);

    typedef struct Sdf_plane_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_plane_result;
    Sdf_plane_result Sdf_plane(float normal_x, float normal_y, float normal_z, float offset);

    typedef struct Sdf_octahedron_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_octahedron_result;
    Sdf_octahedron_result Sdf_octahedron(float size);

    typedef struct Sdf_hex_prism_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_hex_prism_result;
    Sdf_hex_prism_result Sdf_hex_prism(float radius, float half_height);

    typedef struct Sdf_super_prism_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_super_prism_result;
    Sdf_super_prism_result Sdf_super_prism(float half_x, float half_y, float half_z, float exponent);

    typedef struct Sdf_cells_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_cells_result;
    Sdf_cells_result Sdf_cells(float frequency, int32_t seed, float jitter, nucleation::capi::SdfCellMode mode, float threshold);

    nucleation::capi::Sdf* Sdf_union_with(const nucleation::capi::Sdf* self, const nucleation::capi::Sdf* other);

    nucleation::capi::Sdf* Sdf_intersection_with(const nucleation::capi::Sdf* self, const nucleation::capi::Sdf* other);

    nucleation::capi::Sdf* Sdf_subtract(const nucleation::capi::Sdf* self, const nucleation::capi::Sdf* other);

    typedef struct Sdf_smooth_union_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_smooth_union_result;
    Sdf_smooth_union_result Sdf_smooth_union(const nucleation::capi::Sdf* self, const nucleation::capi::Sdf* other, float radius);

    typedef struct Sdf_smooth_subtract_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_smooth_subtract_result;
    Sdf_smooth_subtract_result Sdf_smooth_subtract(const nucleation::capi::Sdf* self, const nucleation::capi::Sdf* other, float radius);

    typedef struct Sdf_smooth_intersection_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_smooth_intersection_result;
    Sdf_smooth_intersection_result Sdf_smooth_intersection(const nucleation::capi::Sdf* self, const nucleation::capi::Sdf* other, float radius);

    typedef struct Sdf_rounded_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_rounded_result;
    Sdf_rounded_result Sdf_rounded(const nucleation::capi::Sdf* self, float radius);

    typedef struct Sdf_shell_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_shell_result;
    Sdf_shell_result Sdf_shell(const nucleation::capi::Sdf* self, float thickness);

    typedef struct Sdf_translate_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_translate_result;
    Sdf_translate_result Sdf_translate(const nucleation::capi::Sdf* self, float x, float y, float z);

    typedef struct Sdf_rotate_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_rotate_result;
    Sdf_rotate_result Sdf_rotate(const nucleation::capi::Sdf* self, float x_degrees, float y_degrees, float z_degrees);

    typedef struct Sdf_scale_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_scale_result;
    Sdf_scale_result Sdf_scale(const nucleation::capi::Sdf* self, float factor);

    nucleation::capi::Sdf* Sdf_mirror(const nucleation::capi::Sdf* self, nucleation::capi::SdfAxis axis);

    typedef struct Sdf_repeat_infinite_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_repeat_infinite_result;
    Sdf_repeat_infinite_result Sdf_repeat_infinite(const nucleation::capi::Sdf* self, float spacing_x, float spacing_y, float spacing_z);

    typedef struct Sdf_repeat_counted_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_repeat_counted_result;
    Sdf_repeat_counted_result Sdf_repeat_counted(const nucleation::capi::Sdf* self, float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z);

    typedef struct Sdf_displace_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_displace_result;
    Sdf_displace_result Sdf_displace(const nucleation::capi::Sdf* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

    typedef struct Sdf_warp_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_warp_result;
    Sdf_warp_result Sdf_warp(const nucleation::capi::Sdf* self, float amplitude, float frequency, int32_t seed);

    float Sdf_eval_at(const nucleation::capi::Sdf* self, float x, float y, float z);

    typedef struct Sdf_normal_result {union {nucleation::capi::SdfNormal ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_normal_result;
    Sdf_normal_result Sdf_normal(const nucleation::capi::Sdf* self, float x, float y, float z, float epsilon);

    typedef struct Sdf_bounds_result {union {nucleation::capi::SdfBounds ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_bounds_result;
    Sdf_bounds_result Sdf_bounds(const nucleation::capi::Sdf* self);

    typedef struct Sdf_to_shape_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_to_shape_result;
    Sdf_to_shape_result Sdf_to_shape(const nucleation::capi::Sdf* self);

    typedef struct Sdf_to_shape_bounded_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_to_shape_bounded_result;
    Sdf_to_shape_bounded_result Sdf_to_shape_bounded(const nucleation::capi::Sdf* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Sdf_from_json_string_result {union {nucleation::capi::Sdf* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_from_json_string_result;
    Sdf_from_json_string_result Sdf_from_json_string(nucleation::diplomat::capi::DiplomatStringView json);

    typedef struct Sdf_to_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_to_json_result;
    Sdf_to_json_result Sdf_to_json(const nucleation::capi::Sdf* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Sdf_schematic_from_sdf_auto_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_auto_result;
    Sdf_schematic_from_sdf_auto_result Sdf_schematic_from_sdf_auto(nucleation::diplomat::capi::DiplomatStringView sdf_json, nucleation::diplomat::capi::DiplomatStringView rules_json);

    typedef struct Sdf_schematic_from_sdf_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_result;
    Sdf_schematic_from_sdf_result Sdf_schematic_from_sdf(nucleation::diplomat::capi::DiplomatStringView sdf_json, nucleation::diplomat::capi::DiplomatStringView rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Sdf_eval_result {union {float ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_eval_result;
    Sdf_eval_result Sdf_eval(nucleation::diplomat::capi::DiplomatStringView sdf_json, float x, float y, float z);

    void Sdf_destroy(Sdf* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::sphere(float radius) {
    auto result = nucleation::capi::Sdf_sphere(radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::box_shape(float half_x, float half_y, float half_z, float rounding) {
    auto result = nucleation::capi::Sdf_box_shape(half_x,
        half_y,
        half_z,
        rounding);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::ellipsoid(float radius_x, float radius_y, float radius_z) {
    auto result = nucleation::capi::Sdf_ellipsoid(radius_x,
        radius_y,
        radius_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::torus(float major_radius, float minor_radius) {
    auto result = nucleation::capi::Sdf_torus(major_radius,
        minor_radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::capsule(float ax, float ay, float az, float bx, float by, float bz, float radius) {
    auto result = nucleation::capi::Sdf_capsule(ax,
        ay,
        az,
        bx,
        by,
        bz,
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::capped_cylinder(float radius, float half_height) {
    auto result = nucleation::capi::Sdf_capped_cylinder(radius,
        half_height);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::capped_cone(float half_height, float bottom_radius, float top_radius) {
    auto result = nucleation::capi::Sdf_capped_cone(half_height,
        bottom_radius,
        top_radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::plane(float normal_x, float normal_y, float normal_z, float offset) {
    auto result = nucleation::capi::Sdf_plane(normal_x,
        normal_y,
        normal_z,
        offset);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::octahedron(float size) {
    auto result = nucleation::capi::Sdf_octahedron(size);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::hex_prism(float radius, float half_height) {
    auto result = nucleation::capi::Sdf_hex_prism(radius,
        half_height);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::super_prism(float half_x, float half_y, float half_z, float exponent) {
    auto result = nucleation::capi::Sdf_super_prism(half_x,
        half_y,
        half_z,
        exponent);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::cells(float frequency, int32_t seed, float jitter, nucleation::SdfCellMode mode, float threshold) {
    auto result = nucleation::capi::Sdf_cells(frequency,
        seed,
        jitter,
        mode.AsFFI(),
        threshold);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Sdf> nucleation::Sdf::union_with(const nucleation::Sdf& other) const {
    auto result = nucleation::capi::Sdf_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result));
}

inline std::unique_ptr<nucleation::Sdf> nucleation::Sdf::intersection_with(const nucleation::Sdf& other) const {
    auto result = nucleation::capi::Sdf_intersection_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result));
}

inline std::unique_ptr<nucleation::Sdf> nucleation::Sdf::subtract(const nucleation::Sdf& other) const {
    auto result = nucleation::capi::Sdf_subtract(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::smooth_union(const nucleation::Sdf& other, float radius) const {
    auto result = nucleation::capi::Sdf_smooth_union(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::smooth_subtract(const nucleation::Sdf& other, float radius) const {
    auto result = nucleation::capi::Sdf_smooth_subtract(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::smooth_intersection(const nucleation::Sdf& other, float radius) const {
    auto result = nucleation::capi::Sdf_smooth_intersection(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::rounded(float radius) const {
    auto result = nucleation::capi::Sdf_rounded(this->AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::shell(float thickness) const {
    auto result = nucleation::capi::Sdf_shell(this->AsFFI(),
        thickness);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::translate(float x, float y, float z) const {
    auto result = nucleation::capi::Sdf_translate(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::rotate(float x_degrees, float y_degrees, float z_degrees) const {
    auto result = nucleation::capi::Sdf_rotate(this->AsFFI(),
        x_degrees,
        y_degrees,
        z_degrees);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::scale(float factor) const {
    auto result = nucleation::capi::Sdf_scale(this->AsFFI(),
        factor);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Sdf> nucleation::Sdf::mirror(nucleation::SdfAxis axis) const {
    auto result = nucleation::capi::Sdf_mirror(this->AsFFI(),
        axis.AsFFI());
    return std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const {
    auto result = nucleation::capi::Sdf_repeat_infinite(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const {
    auto result = nucleation::capi::Sdf_repeat_counted(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z,
        count_x,
        count_y,
        count_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::displace(float amplitude, float frequency, int32_t seed, uint32_t octaves) const {
    auto result = nucleation::capi::Sdf_displace(this->AsFFI(),
        amplitude,
        frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::warp(float amplitude, float frequency, int32_t seed) const {
    auto result = nucleation::capi::Sdf_warp(this->AsFFI(),
        amplitude,
        frequency,
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline float nucleation::Sdf::eval_at(float x, float y, float z) const {
    auto result = nucleation::capi::Sdf_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError> nucleation::Sdf::normal(float x, float y, float z, float epsilon) const {
    auto result = nucleation::capi::Sdf_normal(this->AsFFI(),
        x,
        y,
        z,
        epsilon);
    return result.is_ok ? nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::SdfNormal>(nucleation::SdfNormal::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::SdfBounds, nucleation::NucleationError> nucleation::Sdf::bounds() const {
    auto result = nucleation::capi::Sdf_bounds(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::SdfBounds, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::SdfBounds>(nucleation::SdfBounds::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::SdfBounds, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Sdf::to_shape() const {
    auto result = nucleation::capi::Sdf_to_shape(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Sdf::to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = nucleation::capi::Sdf_to_shape_bounded(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> nucleation::Sdf::from_json_string(std::string_view json) {
    auto result = nucleation::capi::Sdf_from_json_string({json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Sdf>>(std::unique_ptr<nucleation::Sdf>(nucleation::Sdf::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Sdf::to_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Sdf_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Sdf::to_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Sdf_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Sdf::schematic_from_sdf_auto(std::string_view sdf_json, std::string_view rules_json) {
    auto result = nucleation::capi::Sdf_schematic_from_sdf_auto({sdf_json.data(), sdf_json.size()},
        {rules_json.data(), rules_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Sdf::schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::Sdf_schematic_from_sdf({sdf_json.data(), sdf_json.size()},
        {rules_json.data(), rules_json.size()},
        has_bounds,
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<float, nucleation::NucleationError> nucleation::Sdf::eval(std::string_view sdf_json, float x, float y, float z) {
    auto result = nucleation::capi::Sdf_eval({sdf_json.data(), sdf_json.size()},
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Ok<float>(result.ok)) : nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Sdf* nucleation::Sdf::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Sdf*>(this);
}

inline nucleation::capi::Sdf* nucleation::Sdf::AsFFI() {
    return reinterpret_cast<nucleation::capi::Sdf*>(this);
}

inline const nucleation::Sdf* nucleation::Sdf::FromFFI(const nucleation::capi::Sdf* ptr) {
    return reinterpret_cast<const nucleation::Sdf*>(ptr);
}

inline nucleation::Sdf* nucleation::Sdf::FromFFI(nucleation::capi::Sdf* ptr) {
    return reinterpret_cast<nucleation::Sdf*>(ptr);
}

inline void nucleation::Sdf::operator delete(void* ptr) {
    nucleation::capi::Sdf_destroy(reinterpret_cast<nucleation::capi::Sdf*>(ptr));
}


#endif // NUCLEATION_Sdf_HPP
