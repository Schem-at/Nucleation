#ifndef Sdf_HPP
#define Sdf_HPP

#include "Sdf.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Field3.hpp"
#include "FieldProgram.hpp"
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "SdfAxis.hpp"
#include "SdfBounds.hpp"
#include "SdfCellMode.hpp"
#include "SdfNormal.hpp"
#include "Shape.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Sdf_sphere_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_sphere_result;
    Sdf_sphere_result Sdf_sphere(float radius);

    typedef struct Sdf_box_shape_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_box_shape_result;
    Sdf_box_shape_result Sdf_box_shape(float half_x, float half_y, float half_z, float rounding);

    typedef struct Sdf_ellipsoid_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_ellipsoid_result;
    Sdf_ellipsoid_result Sdf_ellipsoid(float radius_x, float radius_y, float radius_z);

    typedef struct Sdf_torus_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_torus_result;
    Sdf_torus_result Sdf_torus(float major_radius, float minor_radius);

    typedef struct Sdf_capped_torus_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_capped_torus_result;
    Sdf_capped_torus_result Sdf_capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

    typedef struct Sdf_link_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_link_result;
    Sdf_link_result Sdf_link(float major_radius, float minor_radius, float half_length);

    typedef struct Sdf_capsule_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_capsule_result;
    Sdf_capsule_result Sdf_capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

    typedef struct Sdf_round_cone_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_round_cone_result;
    Sdf_round_cone_result Sdf_round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

    typedef struct Sdf_solid_angle_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_solid_angle_result;
    Sdf_solid_angle_result Sdf_solid_angle(float radius, float angle_degrees);

    typedef struct Sdf_cut_sphere_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_cut_sphere_result;
    Sdf_cut_sphere_result Sdf_cut_sphere(float radius, float height);

    typedef struct Sdf_cut_hollow_sphere_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_cut_hollow_sphere_result;
    Sdf_cut_hollow_sphere_result Sdf_cut_hollow_sphere(float radius, float height, float thickness);

    typedef struct Sdf_capped_cylinder_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_capped_cylinder_result;
    Sdf_capped_cylinder_result Sdf_capped_cylinder(float radius, float half_height);

    typedef struct Sdf_infinite_cylinder_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_infinite_cylinder_result;
    Sdf_infinite_cylinder_result Sdf_infinite_cylinder(float radius);

    typedef struct Sdf_capped_cone_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_capped_cone_result;
    Sdf_capped_cone_result Sdf_capped_cone(float half_height, float bottom_radius, float top_radius);

    typedef struct Sdf_plane_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_plane_result;
    Sdf_plane_result Sdf_plane(float normal_x, float normal_y, float normal_z, float offset);

    typedef struct Sdf_octahedron_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_octahedron_result;
    Sdf_octahedron_result Sdf_octahedron(float size);

    typedef struct Sdf_hex_prism_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_hex_prism_result;
    Sdf_hex_prism_result Sdf_hex_prism(float radius, float half_height);

    typedef struct Sdf_super_prism_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_super_prism_result;
    Sdf_super_prism_result Sdf_super_prism(float half_x, float half_y, float half_z, float exponent);

    typedef struct Sdf_box_frame_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_box_frame_result;
    Sdf_box_frame_result Sdf_box_frame(float half_x, float half_y, float half_z, float thickness);

    typedef struct Sdf_infinite_cone_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_infinite_cone_result;
    Sdf_infinite_cone_result Sdf_infinite_cone(float angle_degrees);

    typedef struct Sdf_square_pyramid_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_square_pyramid_result;
    Sdf_square_pyramid_result Sdf_square_pyramid(float half_base, float height);

    typedef struct Sdf_cells_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_cells_result;
    Sdf_cells_result Sdf_cells(float frequency, int32_t seed, float jitter, diplomat::capi::SdfCellMode mode, float threshold);

    diplomat::capi::Sdf* Sdf_union_with(const diplomat::capi::Sdf* self, const diplomat::capi::Sdf* other);

    diplomat::capi::Sdf* Sdf_intersection_with(const diplomat::capi::Sdf* self, const diplomat::capi::Sdf* other);

    diplomat::capi::Sdf* Sdf_subtract(const diplomat::capi::Sdf* self, const diplomat::capi::Sdf* other);

    typedef struct Sdf_smooth_union_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_smooth_union_result;
    Sdf_smooth_union_result Sdf_smooth_union(const diplomat::capi::Sdf* self, const diplomat::capi::Sdf* other, float radius);

    typedef struct Sdf_smooth_subtract_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_smooth_subtract_result;
    Sdf_smooth_subtract_result Sdf_smooth_subtract(const diplomat::capi::Sdf* self, const diplomat::capi::Sdf* other, float radius);

    typedef struct Sdf_smooth_intersection_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_smooth_intersection_result;
    Sdf_smooth_intersection_result Sdf_smooth_intersection(const diplomat::capi::Sdf* self, const diplomat::capi::Sdf* other, float radius);

    typedef struct Sdf_rounded_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_rounded_result;
    Sdf_rounded_result Sdf_rounded(const diplomat::capi::Sdf* self, float radius);

    typedef struct Sdf_shell_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_shell_result;
    Sdf_shell_result Sdf_shell(const diplomat::capi::Sdf* self, float thickness);

    diplomat::capi::Sdf* Sdf_xor_with(const diplomat::capi::Sdf* self, const diplomat::capi::Sdf* other);

    typedef struct Sdf_elongate_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_elongate_result;
    Sdf_elongate_result Sdf_elongate(const diplomat::capi::Sdf* self, float half_x, float half_y, float half_z);

    typedef struct Sdf_translate_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_translate_result;
    Sdf_translate_result Sdf_translate(const diplomat::capi::Sdf* self, float x, float y, float z);

    typedef struct Sdf_rotate_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_rotate_result;
    Sdf_rotate_result Sdf_rotate(const diplomat::capi::Sdf* self, float x_degrees, float y_degrees, float z_degrees);

    typedef struct Sdf_scale_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_scale_result;
    Sdf_scale_result Sdf_scale(const diplomat::capi::Sdf* self, float factor);

    diplomat::capi::Sdf* Sdf_mirror(const diplomat::capi::Sdf* self, diplomat::capi::SdfAxis axis);

    typedef struct Sdf_twist_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_twist_result;
    Sdf_twist_result Sdf_twist(const diplomat::capi::Sdf* self, float amount);

    typedef struct Sdf_bend_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_bend_result;
    Sdf_bend_result Sdf_bend(const diplomat::capi::Sdf* self, float amount);

    typedef struct Sdf_repeat_infinite_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_repeat_infinite_result;
    Sdf_repeat_infinite_result Sdf_repeat_infinite(const diplomat::capi::Sdf* self, float spacing_x, float spacing_y, float spacing_z);

    typedef struct Sdf_repeat_counted_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_repeat_counted_result;
    Sdf_repeat_counted_result Sdf_repeat_counted(const diplomat::capi::Sdf* self, float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z);

    typedef struct Sdf_repeat_points_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_repeat_points_result;
    Sdf_repeat_points_result Sdf_repeat_points(const diplomat::capi::Sdf* self, diplomat::capi::DiplomatF32View offsets);

    typedef struct Sdf_displace_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_displace_result;
    Sdf_displace_result Sdf_displace(const diplomat::capi::Sdf* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

    typedef struct Sdf_offset_by_field_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_offset_by_field_result;
    Sdf_offset_by_field_result Sdf_offset_by_field(const diplomat::capi::Sdf* self, const diplomat::capi::Field3* field, float amplitude);

    typedef struct Sdf_warp_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_warp_result;
    Sdf_warp_result Sdf_warp(const diplomat::capi::Sdf* self, float amplitude, float frequency, int32_t seed);

    float Sdf_eval_at(const diplomat::capi::Sdf* self, float x, float y, float z);

    typedef struct Sdf_normal_result {union {diplomat::capi::SdfNormal ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_normal_result;
    Sdf_normal_result Sdf_normal(const diplomat::capi::Sdf* self, float x, float y, float z, float epsilon);

    typedef struct Sdf_bounds_result {union {diplomat::capi::SdfBounds ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_bounds_result;
    Sdf_bounds_result Sdf_bounds(const diplomat::capi::Sdf* self);

    typedef struct Sdf_to_shape_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_to_shape_result;
    Sdf_to_shape_result Sdf_to_shape(const diplomat::capi::Sdf* self);

    typedef struct Sdf_to_shape_bounded_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_to_shape_bounded_result;
    Sdf_to_shape_bounded_result Sdf_to_shape_bounded(const diplomat::capi::Sdf* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Sdf_from_json_string_result {union {diplomat::capi::Sdf* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_from_json_string_result;
    Sdf_from_json_string_result Sdf_from_json_string(diplomat::capi::DiplomatStringView json);

    typedef struct Sdf_to_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_to_json_result;
    Sdf_to_json_result Sdf_to_json(const diplomat::capi::Sdf* self, diplomat::capi::DiplomatWrite* write);

    diplomat::capi::Sdf* Sdf_from_program(const diplomat::capi::FieldProgram* program);

    typedef struct Sdf_schematic_from_sdf_auto_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_auto_result;
    Sdf_schematic_from_sdf_auto_result Sdf_schematic_from_sdf_auto(diplomat::capi::DiplomatStringView sdf_json, diplomat::capi::DiplomatStringView rules_json);

    typedef struct Sdf_schematic_from_sdf_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_result;
    Sdf_schematic_from_sdf_result Sdf_schematic_from_sdf(diplomat::capi::DiplomatStringView sdf_json, diplomat::capi::DiplomatStringView rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Sdf_eval_result {union {float ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_eval_result;
    Sdf_eval_result Sdf_eval(diplomat::capi::DiplomatStringView sdf_json, float x, float y, float z);

    void Sdf_destroy(Sdf* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::sphere(float radius) {
    auto result = diplomat::capi::Sdf_sphere(radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::box_shape(float half_x, float half_y, float half_z, float rounding) {
    auto result = diplomat::capi::Sdf_box_shape(half_x,
        half_y,
        half_z,
        rounding);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::ellipsoid(float radius_x, float radius_y, float radius_z) {
    auto result = diplomat::capi::Sdf_ellipsoid(radius_x,
        radius_y,
        radius_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::torus(float major_radius, float minor_radius) {
    auto result = diplomat::capi::Sdf_torus(major_radius,
        minor_radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::capped_torus(float major_radius, float minor_radius, float cap_angle_degrees) {
    auto result = diplomat::capi::Sdf_capped_torus(major_radius,
        minor_radius,
        cap_angle_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::link(float major_radius, float minor_radius, float half_length) {
    auto result = diplomat::capi::Sdf_link(major_radius,
        minor_radius,
        half_length);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::capsule(float ax, float ay, float az, float bx, float by, float bz, float radius) {
    auto result = diplomat::capi::Sdf_capsule(ax,
        ay,
        az,
        bx,
        by,
        bz,
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2) {
    auto result = diplomat::capi::Sdf_round_cone(ax,
        ay,
        az,
        bx,
        by,
        bz,
        r1,
        r2);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::solid_angle(float radius, float angle_degrees) {
    auto result = diplomat::capi::Sdf_solid_angle(radius,
        angle_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::cut_sphere(float radius, float height) {
    auto result = diplomat::capi::Sdf_cut_sphere(radius,
        height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::cut_hollow_sphere(float radius, float height, float thickness) {
    auto result = diplomat::capi::Sdf_cut_hollow_sphere(radius,
        height,
        thickness);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::capped_cylinder(float radius, float half_height) {
    auto result = diplomat::capi::Sdf_capped_cylinder(radius,
        half_height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::infinite_cylinder(float radius) {
    auto result = diplomat::capi::Sdf_infinite_cylinder(radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::capped_cone(float half_height, float bottom_radius, float top_radius) {
    auto result = diplomat::capi::Sdf_capped_cone(half_height,
        bottom_radius,
        top_radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::plane(float normal_x, float normal_y, float normal_z, float offset) {
    auto result = diplomat::capi::Sdf_plane(normal_x,
        normal_y,
        normal_z,
        offset);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::octahedron(float size) {
    auto result = diplomat::capi::Sdf_octahedron(size);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::hex_prism(float radius, float half_height) {
    auto result = diplomat::capi::Sdf_hex_prism(radius,
        half_height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::super_prism(float half_x, float half_y, float half_z, float exponent) {
    auto result = diplomat::capi::Sdf_super_prism(half_x,
        half_y,
        half_z,
        exponent);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::box_frame(float half_x, float half_y, float half_z, float thickness) {
    auto result = diplomat::capi::Sdf_box_frame(half_x,
        half_y,
        half_z,
        thickness);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::infinite_cone(float angle_degrees) {
    auto result = diplomat::capi::Sdf_infinite_cone(angle_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::square_pyramid(float half_base, float height) {
    auto result = diplomat::capi::Sdf_square_pyramid(half_base,
        height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::cells(float frequency, int32_t seed, float jitter, SdfCellMode mode, float threshold) {
    auto result = diplomat::capi::Sdf_cells(frequency,
        seed,
        jitter,
        mode.AsFFI(),
        threshold);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Sdf> Sdf::union_with(const Sdf& other) const {
    auto result = diplomat::capi::Sdf_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Sdf>(Sdf::FromFFI(result));
}

inline std::unique_ptr<Sdf> Sdf::intersection_with(const Sdf& other) const {
    auto result = diplomat::capi::Sdf_intersection_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Sdf>(Sdf::FromFFI(result));
}

inline std::unique_ptr<Sdf> Sdf::subtract(const Sdf& other) const {
    auto result = diplomat::capi::Sdf_subtract(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Sdf>(Sdf::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::smooth_union(const Sdf& other, float radius) const {
    auto result = diplomat::capi::Sdf_smooth_union(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::smooth_subtract(const Sdf& other, float radius) const {
    auto result = diplomat::capi::Sdf_smooth_subtract(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::smooth_intersection(const Sdf& other, float radius) const {
    auto result = diplomat::capi::Sdf_smooth_intersection(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::rounded(float radius) const {
    auto result = diplomat::capi::Sdf_rounded(this->AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::shell(float thickness) const {
    auto result = diplomat::capi::Sdf_shell(this->AsFFI(),
        thickness);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Sdf> Sdf::xor_with(const Sdf& other) const {
    auto result = diplomat::capi::Sdf_xor_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Sdf>(Sdf::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::elongate(float half_x, float half_y, float half_z) const {
    auto result = diplomat::capi::Sdf_elongate(this->AsFFI(),
        half_x,
        half_y,
        half_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::translate(float x, float y, float z) const {
    auto result = diplomat::capi::Sdf_translate(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::rotate(float x_degrees, float y_degrees, float z_degrees) const {
    auto result = diplomat::capi::Sdf_rotate(this->AsFFI(),
        x_degrees,
        y_degrees,
        z_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::scale(float factor) const {
    auto result = diplomat::capi::Sdf_scale(this->AsFFI(),
        factor);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Sdf> Sdf::mirror(SdfAxis axis) const {
    auto result = diplomat::capi::Sdf_mirror(this->AsFFI(),
        axis.AsFFI());
    return std::unique_ptr<Sdf>(Sdf::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::twist(float amount) const {
    auto result = diplomat::capi::Sdf_twist(this->AsFFI(),
        amount);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::bend(float amount) const {
    auto result = diplomat::capi::Sdf_bend(this->AsFFI(),
        amount);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const {
    auto result = diplomat::capi::Sdf_repeat_infinite(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const {
    auto result = diplomat::capi::Sdf_repeat_counted(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z,
        count_x,
        count_y,
        count_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::repeat_points(diplomat::span<const float> offsets) const {
    auto result = diplomat::capi::Sdf_repeat_points(this->AsFFI(),
        {offsets.data(), offsets.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::displace(float amplitude, float frequency, int32_t seed, uint32_t octaves) const {
    auto result = diplomat::capi::Sdf_displace(this->AsFFI(),
        amplitude,
        frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::offset_by_field(const Field3& field, float amplitude) const {
    auto result = diplomat::capi::Sdf_offset_by_field(this->AsFFI(),
        field.AsFFI(),
        amplitude);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::warp(float amplitude, float frequency, int32_t seed) const {
    auto result = diplomat::capi::Sdf_warp(this->AsFFI(),
        amplitude,
        frequency,
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline float Sdf::eval_at(float x, float y, float z) const {
    auto result = diplomat::capi::Sdf_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline diplomat::result<SdfNormal, NucleationError> Sdf::normal(float x, float y, float z, float epsilon) const {
    auto result = diplomat::capi::Sdf_normal(this->AsFFI(),
        x,
        y,
        z,
        epsilon);
    return result.is_ok ? diplomat::result<SdfNormal, NucleationError>(diplomat::Ok<SdfNormal>(SdfNormal::FromFFI(result.ok))) : diplomat::result<SdfNormal, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<SdfBounds, NucleationError> Sdf::bounds() const {
    auto result = diplomat::capi::Sdf_bounds(this->AsFFI());
    return result.is_ok ? diplomat::result<SdfBounds, NucleationError>(diplomat::Ok<SdfBounds>(SdfBounds::FromFFI(result.ok))) : diplomat::result<SdfBounds, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Sdf::to_shape() const {
    auto result = diplomat::capi::Sdf_to_shape(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Sdf::to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = diplomat::capi::Sdf_to_shape_bounded(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> Sdf::from_json_string(std::string_view json) {
    auto result = diplomat::capi::Sdf_from_json_string({json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Ok<std::unique_ptr<Sdf>>(std::unique_ptr<Sdf>(Sdf::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Sdf>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Sdf::to_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Sdf_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Sdf::to_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Sdf_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Sdf> Sdf::from_program(const FieldProgram& program) {
    auto result = diplomat::capi::Sdf_from_program(program.AsFFI());
    return std::unique_ptr<Sdf>(Sdf::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Sdf::schematic_from_sdf_auto(std::string_view sdf_json, std::string_view rules_json) {
    auto result = diplomat::capi::Sdf_schematic_from_sdf_auto({sdf_json.data(), sdf_json.size()},
        {rules_json.data(), rules_json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Sdf::schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::Sdf_schematic_from_sdf({sdf_json.data(), sdf_json.size()},
        {rules_json.data(), rules_json.size()},
        has_bounds,
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<float, NucleationError> Sdf::eval(std::string_view sdf_json, float x, float y, float z) {
    auto result = diplomat::capi::Sdf_eval({sdf_json.data(), sdf_json.size()},
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<float, NucleationError>(diplomat::Ok<float>(result.ok)) : diplomat::result<float, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Sdf* Sdf::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Sdf*>(this);
}

inline diplomat::capi::Sdf* Sdf::AsFFI() {
    return reinterpret_cast<diplomat::capi::Sdf*>(this);
}

inline const Sdf* Sdf::FromFFI(const diplomat::capi::Sdf* ptr) {
    return reinterpret_cast<const Sdf*>(ptr);
}

inline Sdf* Sdf::FromFFI(diplomat::capi::Sdf* ptr) {
    return reinterpret_cast<Sdf*>(ptr);
}

inline void Sdf::operator delete(void* ptr) {
    diplomat::capi::Sdf_destroy(reinterpret_cast<diplomat::capi::Sdf*>(ptr));
}


#endif // Sdf_HPP
