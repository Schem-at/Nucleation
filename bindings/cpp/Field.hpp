#ifndef Field_HPP
#define Field_HPP

#include "Field.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "FieldAxis.hpp"
#include "FieldBounds.hpp"
#include "FieldCellMode.hpp"
#include "FieldDistanceKind.hpp"
#include "FieldNoiseBasis.hpp"
#include "FieldNormal.hpp"
#include "FieldRange.hpp"
#include "FieldSample.hpp"
#include "NucleationError.hpp"
#include "Shape.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Field_constant_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_constant_result;
    Field_constant_result Field_constant(float value);

    diplomat::capi::Field* Field_coord_x(void);

    diplomat::capi::Field* Field_coord_y(void);

    diplomat::capi::Field* Field_coord_z(void);

    typedef struct Field_sphere_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_sphere_result;
    Field_sphere_result Field_sphere(float radius);

    typedef struct Field_box_shape_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_box_shape_result;
    Field_box_shape_result Field_box_shape(float half_x, float half_y, float half_z, float rounding);

    typedef struct Field_ellipsoid_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_ellipsoid_result;
    Field_ellipsoid_result Field_ellipsoid(float radius_x, float radius_y, float radius_z);

    typedef struct Field_torus_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_torus_result;
    Field_torus_result Field_torus(float major_radius, float minor_radius);

    typedef struct Field_capped_torus_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_capped_torus_result;
    Field_capped_torus_result Field_capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

    typedef struct Field_link_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_link_result;
    Field_link_result Field_link(float major_radius, float minor_radius, float half_length);

    typedef struct Field_capsule_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_capsule_result;
    Field_capsule_result Field_capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

    typedef struct Field_round_cone_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_round_cone_result;
    Field_round_cone_result Field_round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

    typedef struct Field_solid_angle_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_solid_angle_result;
    Field_solid_angle_result Field_solid_angle(float radius, float angle_degrees);

    typedef struct Field_cut_sphere_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_cut_sphere_result;
    Field_cut_sphere_result Field_cut_sphere(float radius, float height);

    typedef struct Field_cut_hollow_sphere_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_cut_hollow_sphere_result;
    Field_cut_hollow_sphere_result Field_cut_hollow_sphere(float radius, float height, float thickness);

    typedef struct Field_capped_cylinder_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_capped_cylinder_result;
    Field_capped_cylinder_result Field_capped_cylinder(float radius, float half_height);

    typedef struct Field_infinite_cylinder_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_infinite_cylinder_result;
    Field_infinite_cylinder_result Field_infinite_cylinder(float radius);

    typedef struct Field_capped_cone_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_capped_cone_result;
    Field_capped_cone_result Field_capped_cone(float half_height, float bottom_radius, float top_radius);

    typedef struct Field_plane_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_plane_result;
    Field_plane_result Field_plane(float normal_x, float normal_y, float normal_z, float offset);

    typedef struct Field_octahedron_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_octahedron_result;
    Field_octahedron_result Field_octahedron(float size);

    typedef struct Field_hex_prism_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_hex_prism_result;
    Field_hex_prism_result Field_hex_prism(float radius, float half_height);

    typedef struct Field_super_prism_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_super_prism_result;
    Field_super_prism_result Field_super_prism(float half_x, float half_y, float half_z, float exponent);

    typedef struct Field_box_frame_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_box_frame_result;
    Field_box_frame_result Field_box_frame(float half_x, float half_y, float half_z, float thickness);

    typedef struct Field_infinite_cone_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_infinite_cone_result;
    Field_infinite_cone_result Field_infinite_cone(float angle_degrees);

    typedef struct Field_square_pyramid_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_square_pyramid_result;
    Field_square_pyramid_result Field_square_pyramid(float half_base, float height);

    typedef struct Field_heightmap_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_heightmap_result;
    Field_heightmap_result Field_heightmap(float base_y, const diplomat::capi::Field* height);

    typedef struct Field_value_noise2_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_value_noise2_result;
    Field_value_noise2_result Field_value_noise2(float frequency, int32_t seed);

    typedef struct Field_value_noise3_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_value_noise3_result;
    Field_value_noise3_result Field_value_noise3(float frequency, int32_t seed);

    typedef struct Field_perlin2_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_perlin2_result;
    Field_perlin2_result Field_perlin2(float frequency, int32_t seed);

    typedef struct Field_perlin3_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_perlin3_result;
    Field_perlin3_result Field_perlin3(float frequency, int32_t seed);

    typedef struct Field_fbm2_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_fbm2_result;
    Field_fbm2_result Field_fbm2(diplomat::capi::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

    typedef struct Field_fbm3_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_fbm3_result;
    Field_fbm3_result Field_fbm3(diplomat::capi::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

    typedef struct Field_value_fbm3_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_value_fbm3_result;
    Field_value_fbm3_result Field_value_fbm3(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_perlin_fbm3_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_perlin_fbm3_result;
    Field_perlin_fbm3_result Field_perlin_fbm3(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_ridged_fbm3_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_ridged_fbm3_result;
    Field_ridged_fbm3_result Field_ridged_fbm3(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_perlin_fbm2_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_perlin_fbm2_result;
    Field_perlin_fbm2_result Field_perlin_fbm2(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_cells_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_cells_result;
    Field_cells_result Field_cells(float frequency, int32_t seed, float jitter, diplomat::capi::FieldCellMode mode, float threshold);

    diplomat::capi::Field* Field_neg(const diplomat::capi::Field* self);

    diplomat::capi::Field* Field_abs(const diplomat::capi::Field* self);

    diplomat::capi::Field* Field_sqrt(const diplomat::capi::Field* self);

    diplomat::capi::Field* Field_sin(const diplomat::capi::Field* self);

    diplomat::capi::Field* Field_cos(const diplomat::capi::Field* self);

    diplomat::capi::Field* Field_saturate(const diplomat::capi::Field* self);

    diplomat::capi::Field* Field_add(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_sub(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_mul(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_div(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_min_with(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_max_with(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    typedef struct Field_add_const_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_add_const_result;
    Field_add_const_result Field_add_const(const diplomat::capi::Field* self, float value);

    typedef struct Field_mul_const_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_mul_const_result;
    Field_mul_const_result Field_mul_const(const diplomat::capi::Field* self, float value);

    typedef struct Field_clamp_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_clamp_result;
    Field_clamp_result Field_clamp(const diplomat::capi::Field* self, float lo, float hi);

    typedef struct Field_smoothstep_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_smoothstep_result;
    Field_smoothstep_result Field_smoothstep(const diplomat::capi::Field* self, float edge0, float edge1);

    typedef struct Field_step_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_step_result;
    Field_step_result Field_step(const diplomat::capi::Field* self, float edge);

    typedef struct Field_remap_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_remap_result;
    Field_remap_result Field_remap(const diplomat::capi::Field* self, float in_lo, float in_hi, float out_lo, float out_hi, bool clamp);

    diplomat::capi::Field* Field_lerp_with(const diplomat::capi::Field* self, const diplomat::capi::Field* other, const diplomat::capi::Field* t);

    diplomat::capi::Field* Field_select(const diplomat::capi::Field* cond, const diplomat::capi::Field* then_field, const diplomat::capi::Field* else_field);

    diplomat::capi::Field* Field_union_with(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_intersection_with(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_subtract(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    diplomat::capi::Field* Field_xor_with(const diplomat::capi::Field* self, const diplomat::capi::Field* other);

    typedef struct Field_smooth_union_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_smooth_union_result;
    Field_smooth_union_result Field_smooth_union(const diplomat::capi::Field* self, const diplomat::capi::Field* other, float radius);

    typedef struct Field_smooth_subtract_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_smooth_subtract_result;
    Field_smooth_subtract_result Field_smooth_subtract(const diplomat::capi::Field* self, const diplomat::capi::Field* other, float radius);

    typedef struct Field_smooth_intersection_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_smooth_intersection_result;
    Field_smooth_intersection_result Field_smooth_intersection(const diplomat::capi::Field* self, const diplomat::capi::Field* other, float radius);

    typedef struct Field_chamfer_union_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_chamfer_union_result;
    Field_chamfer_union_result Field_chamfer_union(const diplomat::capi::Field* self, const diplomat::capi::Field* other, float radius);

    typedef struct Field_chamfer_subtract_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_chamfer_subtract_result;
    Field_chamfer_subtract_result Field_chamfer_subtract(const diplomat::capi::Field* self, const diplomat::capi::Field* other, float radius);

    typedef struct Field_chamfer_intersection_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_chamfer_intersection_result;
    Field_chamfer_intersection_result Field_chamfer_intersection(const diplomat::capi::Field* self, const diplomat::capi::Field* other, float radius);

    typedef struct Field_steps_union_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_steps_union_result;
    Field_steps_union_result Field_steps_union(const diplomat::capi::Field* self, const diplomat::capi::Field* other, float radius, float steps);

    typedef struct Field_rounded_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_rounded_result;
    Field_rounded_result Field_rounded(const diplomat::capi::Field* self, float radius);

    typedef struct Field_shell_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_shell_result;
    Field_shell_result Field_shell(const diplomat::capi::Field* self, float thickness);

    typedef struct Field_elongate_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_elongate_result;
    Field_elongate_result Field_elongate(const diplomat::capi::Field* self, float half_x, float half_y, float half_z);

    typedef struct Field_displace_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_displace_result;
    Field_displace_result Field_displace(const diplomat::capi::Field* self, const diplomat::capi::Field* offset, float amplitude);

    typedef struct Field_displace_noise_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_displace_noise_result;
    Field_displace_noise_result Field_displace_noise(const diplomat::capi::Field* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_translate_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_translate_result;
    Field_translate_result Field_translate(const diplomat::capi::Field* self, float x, float y, float z);

    typedef struct Field_rotate_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_rotate_result;
    Field_rotate_result Field_rotate(const diplomat::capi::Field* self, float x_degrees, float y_degrees, float z_degrees);

    typedef struct Field_scale_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_scale_result;
    Field_scale_result Field_scale(const diplomat::capi::Field* self, float factor);

    typedef struct Field_scale_xyz_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_scale_xyz_result;
    Field_scale_xyz_result Field_scale_xyz(const diplomat::capi::Field* self, float sx, float sy, float sz);

    diplomat::capi::Field* Field_mirror(const diplomat::capi::Field* self, diplomat::capi::FieldAxis axis);

    typedef struct Field_mirror_plane_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_mirror_plane_result;
    Field_mirror_plane_result Field_mirror_plane(const diplomat::capi::Field* self, float nx, float ny, float nz, float offset);

    typedef struct Field_repeat_infinite_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_repeat_infinite_result;
    Field_repeat_infinite_result Field_repeat_infinite(const diplomat::capi::Field* self, float spacing_x, float spacing_y, float spacing_z);

    typedef struct Field_repeat_counted_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_repeat_counted_result;
    Field_repeat_counted_result Field_repeat_counted(const diplomat::capi::Field* self, float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z);

    typedef struct Field_repeat_polar_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_repeat_polar_result;
    Field_repeat_polar_result Field_repeat_polar(const diplomat::capi::Field* self, uint32_t count);

    typedef struct Field_repeat_points_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_repeat_points_result;
    Field_repeat_points_result Field_repeat_points(const diplomat::capi::Field* self, diplomat::capi::DiplomatF32View offsets);

    typedef struct Field_twist_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_twist_result;
    Field_twist_result Field_twist(const diplomat::capi::Field* self, float amount);

    typedef struct Field_bend_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_bend_result;
    Field_bend_result Field_bend(const diplomat::capi::Field* self, float amount);

    typedef struct Field_warp_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_warp_result;
    Field_warp_result Field_warp(const diplomat::capi::Field* self, const diplomat::capi::Field* warp_x, const diplomat::capi::Field* warp_y, const diplomat::capi::Field* warp_z, float amplitude);

    typedef struct Field_warp_noise_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_warp_noise_result;
    Field_warp_noise_result Field_warp_noise(const diplomat::capi::Field* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_tag_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_tag_result;
    Field_tag_result Field_tag(const diplomat::capi::Field* self, uint16_t material_id);

    typedef struct Field_bounded_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_bounded_result;
    Field_bounded_result Field_bounded(const diplomat::capi::Field* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

    typedef struct Field_expr_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_expr_result;
    Field_expr_result Field_expr(diplomat::capi::DiplomatStringView source);

    float Field_eval_at(const diplomat::capi::Field* self, float x, float y, float z);

    diplomat::capi::FieldSample Field_sample_tagged(const diplomat::capi::Field* self, float x, float y, float z);

    typedef struct Field_normal_result {union {diplomat::capi::FieldNormal ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_normal_result;
    Field_normal_result Field_normal(const diplomat::capi::Field* self, float x, float y, float z, float epsilon);

    typedef struct Field_bounds_result {union {diplomat::capi::FieldBounds ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_bounds_result;
    Field_bounds_result Field_bounds(const diplomat::capi::Field* self);

    typedef struct Field_output_range_result {union {diplomat::capi::FieldRange ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_output_range_result;
    Field_output_range_result Field_output_range(const diplomat::capi::Field* self);

    diplomat::capi::FieldDistanceKind Field_distance_kind(const diplomat::capi::Field* self);

    typedef struct Field_to_shape_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_to_shape_result;
    Field_to_shape_result Field_to_shape(const diplomat::capi::Field* self);

    typedef struct Field_to_shape_bounded_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_to_shape_bounded_result;
    Field_to_shape_bounded_result Field_to_shape_bounded(const diplomat::capi::Field* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Field_from_json_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field_from_json_result;
    Field_from_json_result Field_from_json(diplomat::capi::DiplomatStringView json);

    typedef struct Field_to_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Field_to_json_result;
    Field_to_json_result Field_to_json(const diplomat::capi::Field* self, diplomat::capi::DiplomatWrite* write);

    void Field_destroy(Field* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::constant(float value) {
    auto result = diplomat::capi::Field_constant(value);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Field> Field::coord_x() {
    auto result = diplomat::capi::Field_coord_x();
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::coord_y() {
    auto result = diplomat::capi::Field_coord_y();
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::coord_z() {
    auto result = diplomat::capi::Field_coord_z();
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::sphere(float radius) {
    auto result = diplomat::capi::Field_sphere(radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::box_shape(float half_x, float half_y, float half_z, float rounding) {
    auto result = diplomat::capi::Field_box_shape(half_x,
        half_y,
        half_z,
        rounding);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::ellipsoid(float radius_x, float radius_y, float radius_z) {
    auto result = diplomat::capi::Field_ellipsoid(radius_x,
        radius_y,
        radius_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::torus(float major_radius, float minor_radius) {
    auto result = diplomat::capi::Field_torus(major_radius,
        minor_radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::capped_torus(float major_radius, float minor_radius, float cap_angle_degrees) {
    auto result = diplomat::capi::Field_capped_torus(major_radius,
        minor_radius,
        cap_angle_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::link(float major_radius, float minor_radius, float half_length) {
    auto result = diplomat::capi::Field_link(major_radius,
        minor_radius,
        half_length);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::capsule(float ax, float ay, float az, float bx, float by, float bz, float radius) {
    auto result = diplomat::capi::Field_capsule(ax,
        ay,
        az,
        bx,
        by,
        bz,
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2) {
    auto result = diplomat::capi::Field_round_cone(ax,
        ay,
        az,
        bx,
        by,
        bz,
        r1,
        r2);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::solid_angle(float radius, float angle_degrees) {
    auto result = diplomat::capi::Field_solid_angle(radius,
        angle_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::cut_sphere(float radius, float height) {
    auto result = diplomat::capi::Field_cut_sphere(radius,
        height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::cut_hollow_sphere(float radius, float height, float thickness) {
    auto result = diplomat::capi::Field_cut_hollow_sphere(radius,
        height,
        thickness);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::capped_cylinder(float radius, float half_height) {
    auto result = diplomat::capi::Field_capped_cylinder(radius,
        half_height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::infinite_cylinder(float radius) {
    auto result = diplomat::capi::Field_infinite_cylinder(radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::capped_cone(float half_height, float bottom_radius, float top_radius) {
    auto result = diplomat::capi::Field_capped_cone(half_height,
        bottom_radius,
        top_radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::plane(float normal_x, float normal_y, float normal_z, float offset) {
    auto result = diplomat::capi::Field_plane(normal_x,
        normal_y,
        normal_z,
        offset);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::octahedron(float size) {
    auto result = diplomat::capi::Field_octahedron(size);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::hex_prism(float radius, float half_height) {
    auto result = diplomat::capi::Field_hex_prism(radius,
        half_height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::super_prism(float half_x, float half_y, float half_z, float exponent) {
    auto result = diplomat::capi::Field_super_prism(half_x,
        half_y,
        half_z,
        exponent);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::box_frame(float half_x, float half_y, float half_z, float thickness) {
    auto result = diplomat::capi::Field_box_frame(half_x,
        half_y,
        half_z,
        thickness);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::infinite_cone(float angle_degrees) {
    auto result = diplomat::capi::Field_infinite_cone(angle_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::square_pyramid(float half_base, float height) {
    auto result = diplomat::capi::Field_square_pyramid(half_base,
        height);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::heightmap(float base_y, const Field& height) {
    auto result = diplomat::capi::Field_heightmap(base_y,
        height.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::value_noise2(float frequency, int32_t seed) {
    auto result = diplomat::capi::Field_value_noise2(frequency,
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::value_noise3(float frequency, int32_t seed) {
    auto result = diplomat::capi::Field_value_noise3(frequency,
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::perlin2(float frequency, int32_t seed) {
    auto result = diplomat::capi::Field_perlin2(frequency,
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::perlin3(float frequency, int32_t seed) {
    auto result = diplomat::capi::Field_perlin3(frequency,
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::fbm2(FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged) {
    auto result = diplomat::capi::Field_fbm2(basis.AsFFI(),
        frequency,
        seed,
        octaves,
        lacunarity,
        gain,
        ridged);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::fbm3(FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged) {
    auto result = diplomat::capi::Field_fbm3(basis.AsFFI(),
        frequency,
        seed,
        octaves,
        lacunarity,
        gain,
        ridged);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::value_fbm3(float frequency, int32_t seed, uint32_t octaves) {
    auto result = diplomat::capi::Field_value_fbm3(frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::perlin_fbm3(float frequency, int32_t seed, uint32_t octaves) {
    auto result = diplomat::capi::Field_perlin_fbm3(frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::ridged_fbm3(float frequency, int32_t seed, uint32_t octaves) {
    auto result = diplomat::capi::Field_ridged_fbm3(frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::perlin_fbm2(float frequency, int32_t seed, uint32_t octaves) {
    auto result = diplomat::capi::Field_perlin_fbm2(frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::cells(float frequency, int32_t seed, float jitter, FieldCellMode mode, float threshold) {
    auto result = diplomat::capi::Field_cells(frequency,
        seed,
        jitter,
        mode.AsFFI(),
        threshold);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Field> Field::neg() const {
    auto result = diplomat::capi::Field_neg(this->AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::abs() const {
    auto result = diplomat::capi::Field_abs(this->AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::sqrt() const {
    auto result = diplomat::capi::Field_sqrt(this->AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::sin() const {
    auto result = diplomat::capi::Field_sin(this->AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::cos() const {
    auto result = diplomat::capi::Field_cos(this->AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::saturate() const {
    auto result = diplomat::capi::Field_saturate(this->AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::add(const Field& other) const {
    auto result = diplomat::capi::Field_add(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::sub(const Field& other) const {
    auto result = diplomat::capi::Field_sub(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::mul(const Field& other) const {
    auto result = diplomat::capi::Field_mul(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::div(const Field& other) const {
    auto result = diplomat::capi::Field_div(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::min_with(const Field& other) const {
    auto result = diplomat::capi::Field_min_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::max_with(const Field& other) const {
    auto result = diplomat::capi::Field_max_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::add_const(float value) const {
    auto result = diplomat::capi::Field_add_const(this->AsFFI(),
        value);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::mul_const(float value) const {
    auto result = diplomat::capi::Field_mul_const(this->AsFFI(),
        value);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::clamp(float lo, float hi) const {
    auto result = diplomat::capi::Field_clamp(this->AsFFI(),
        lo,
        hi);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::smoothstep(float edge0, float edge1) const {
    auto result = diplomat::capi::Field_smoothstep(this->AsFFI(),
        edge0,
        edge1);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::step(float edge) const {
    auto result = diplomat::capi::Field_step(this->AsFFI(),
        edge);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::remap(float in_lo, float in_hi, float out_lo, float out_hi, bool clamp) const {
    auto result = diplomat::capi::Field_remap(this->AsFFI(),
        in_lo,
        in_hi,
        out_lo,
        out_hi,
        clamp);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Field> Field::lerp_with(const Field& other, const Field& t) const {
    auto result = diplomat::capi::Field_lerp_with(this->AsFFI(),
        other.AsFFI(),
        t.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::select(const Field& cond, const Field& then_field, const Field& else_field) {
    auto result = diplomat::capi::Field_select(cond.AsFFI(),
        then_field.AsFFI(),
        else_field.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::union_with(const Field& other) const {
    auto result = diplomat::capi::Field_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::intersection_with(const Field& other) const {
    auto result = diplomat::capi::Field_intersection_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::subtract(const Field& other) const {
    auto result = diplomat::capi::Field_subtract(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline std::unique_ptr<Field> Field::xor_with(const Field& other) const {
    auto result = diplomat::capi::Field_xor_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::smooth_union(const Field& other, float radius) const {
    auto result = diplomat::capi::Field_smooth_union(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::smooth_subtract(const Field& other, float radius) const {
    auto result = diplomat::capi::Field_smooth_subtract(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::smooth_intersection(const Field& other, float radius) const {
    auto result = diplomat::capi::Field_smooth_intersection(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::chamfer_union(const Field& other, float radius) const {
    auto result = diplomat::capi::Field_chamfer_union(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::chamfer_subtract(const Field& other, float radius) const {
    auto result = diplomat::capi::Field_chamfer_subtract(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::chamfer_intersection(const Field& other, float radius) const {
    auto result = diplomat::capi::Field_chamfer_intersection(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::steps_union(const Field& other, float radius, float steps) const {
    auto result = diplomat::capi::Field_steps_union(this->AsFFI(),
        other.AsFFI(),
        radius,
        steps);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::rounded(float radius) const {
    auto result = diplomat::capi::Field_rounded(this->AsFFI(),
        radius);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::shell(float thickness) const {
    auto result = diplomat::capi::Field_shell(this->AsFFI(),
        thickness);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::elongate(float half_x, float half_y, float half_z) const {
    auto result = diplomat::capi::Field_elongate(this->AsFFI(),
        half_x,
        half_y,
        half_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::displace(const Field& offset, float amplitude) const {
    auto result = diplomat::capi::Field_displace(this->AsFFI(),
        offset.AsFFI(),
        amplitude);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::displace_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const {
    auto result = diplomat::capi::Field_displace_noise(this->AsFFI(),
        amplitude,
        frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::translate(float x, float y, float z) const {
    auto result = diplomat::capi::Field_translate(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::rotate(float x_degrees, float y_degrees, float z_degrees) const {
    auto result = diplomat::capi::Field_rotate(this->AsFFI(),
        x_degrees,
        y_degrees,
        z_degrees);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::scale(float factor) const {
    auto result = diplomat::capi::Field_scale(this->AsFFI(),
        factor);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::scale_xyz(float sx, float sy, float sz) const {
    auto result = diplomat::capi::Field_scale_xyz(this->AsFFI(),
        sx,
        sy,
        sz);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Field> Field::mirror(FieldAxis axis) const {
    auto result = diplomat::capi::Field_mirror(this->AsFFI(),
        axis.AsFFI());
    return std::unique_ptr<Field>(Field::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::mirror_plane(float nx, float ny, float nz, float offset) const {
    auto result = diplomat::capi::Field_mirror_plane(this->AsFFI(),
        nx,
        ny,
        nz,
        offset);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const {
    auto result = diplomat::capi::Field_repeat_infinite(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const {
    auto result = diplomat::capi::Field_repeat_counted(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z,
        count_x,
        count_y,
        count_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::repeat_polar(uint32_t count) const {
    auto result = diplomat::capi::Field_repeat_polar(this->AsFFI(),
        count);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::repeat_points(diplomat::span<const float> offsets) const {
    auto result = diplomat::capi::Field_repeat_points(this->AsFFI(),
        {offsets.data(), offsets.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::twist(float amount) const {
    auto result = diplomat::capi::Field_twist(this->AsFFI(),
        amount);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::bend(float amount) const {
    auto result = diplomat::capi::Field_bend(this->AsFFI(),
        amount);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::warp(const Field& warp_x, const Field& warp_y, const Field& warp_z, float amplitude) const {
    auto result = diplomat::capi::Field_warp(this->AsFFI(),
        warp_x.AsFFI(),
        warp_y.AsFFI(),
        warp_z.AsFFI(),
        amplitude);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::warp_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const {
    auto result = diplomat::capi::Field_warp_noise(this->AsFFI(),
        amplitude,
        frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::tag(uint16_t material_id) const {
    auto result = diplomat::capi::Field_tag(this->AsFFI(),
        material_id);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::bounded(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) const {
    auto result = diplomat::capi::Field_bounded(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::expr(std::string_view source) {
    auto result = diplomat::capi::Field_expr({source.data(), source.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline float Field::eval_at(float x, float y, float z) const {
    auto result = diplomat::capi::Field_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline FieldSample Field::sample_tagged(float x, float y, float z) const {
    auto result = diplomat::capi::Field_sample_tagged(this->AsFFI(),
        x,
        y,
        z);
    return FieldSample::FromFFI(result);
}

inline diplomat::result<FieldNormal, NucleationError> Field::normal(float x, float y, float z, float epsilon) const {
    auto result = diplomat::capi::Field_normal(this->AsFFI(),
        x,
        y,
        z,
        epsilon);
    return result.is_ok ? diplomat::result<FieldNormal, NucleationError>(diplomat::Ok<FieldNormal>(FieldNormal::FromFFI(result.ok))) : diplomat::result<FieldNormal, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<FieldBounds, NucleationError> Field::bounds() const {
    auto result = diplomat::capi::Field_bounds(this->AsFFI());
    return result.is_ok ? diplomat::result<FieldBounds, NucleationError>(diplomat::Ok<FieldBounds>(FieldBounds::FromFFI(result.ok))) : diplomat::result<FieldBounds, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<FieldRange, NucleationError> Field::output_range() const {
    auto result = diplomat::capi::Field_output_range(this->AsFFI());
    return result.is_ok ? diplomat::result<FieldRange, NucleationError>(diplomat::Ok<FieldRange>(FieldRange::FromFFI(result.ok))) : diplomat::result<FieldRange, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline FieldDistanceKind Field::distance_kind() const {
    auto result = diplomat::capi::Field_distance_kind(this->AsFFI());
    return FieldDistanceKind::FromFFI(result);
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Field::to_shape() const {
    auto result = diplomat::capi::Field_to_shape(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Field::to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = diplomat::capi::Field_to_shape_bounded(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> Field::from_json(std::string_view json) {
    auto result = diplomat::capi::Field_from_json({json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Field::to_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Field_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Field::to_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Field_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Field* Field::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Field*>(this);
}

inline diplomat::capi::Field* Field::AsFFI() {
    return reinterpret_cast<diplomat::capi::Field*>(this);
}

inline const Field* Field::FromFFI(const diplomat::capi::Field* ptr) {
    return reinterpret_cast<const Field*>(ptr);
}

inline Field* Field::FromFFI(diplomat::capi::Field* ptr) {
    return reinterpret_cast<Field*>(ptr);
}

inline void Field::operator delete(void* ptr) {
    diplomat::capi::Field_destroy(reinterpret_cast<diplomat::capi::Field*>(ptr));
}


#endif // Field_HPP
