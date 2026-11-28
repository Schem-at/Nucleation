#ifndef NUCLEATION_Field_HPP
#define NUCLEATION_Field_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Field_constant_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_constant_result;
    Field_constant_result Field_constant(float value);

    nucleation::capi::Field* Field_coord_x(void);

    nucleation::capi::Field* Field_coord_y(void);

    nucleation::capi::Field* Field_coord_z(void);

    typedef struct Field_sphere_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_sphere_result;
    Field_sphere_result Field_sphere(float radius);

    typedef struct Field_box_shape_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_box_shape_result;
    Field_box_shape_result Field_box_shape(float half_x, float half_y, float half_z, float rounding);

    typedef struct Field_ellipsoid_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_ellipsoid_result;
    Field_ellipsoid_result Field_ellipsoid(float radius_x, float radius_y, float radius_z);

    typedef struct Field_torus_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_torus_result;
    Field_torus_result Field_torus(float major_radius, float minor_radius);

    typedef struct Field_capped_torus_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_capped_torus_result;
    Field_capped_torus_result Field_capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

    typedef struct Field_link_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_link_result;
    Field_link_result Field_link(float major_radius, float minor_radius, float half_length);

    typedef struct Field_capsule_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_capsule_result;
    Field_capsule_result Field_capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

    typedef struct Field_round_cone_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_round_cone_result;
    Field_round_cone_result Field_round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

    typedef struct Field_solid_angle_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_solid_angle_result;
    Field_solid_angle_result Field_solid_angle(float radius, float angle_degrees);

    typedef struct Field_cut_sphere_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_cut_sphere_result;
    Field_cut_sphere_result Field_cut_sphere(float radius, float height);

    typedef struct Field_cut_hollow_sphere_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_cut_hollow_sphere_result;
    Field_cut_hollow_sphere_result Field_cut_hollow_sphere(float radius, float height, float thickness);

    typedef struct Field_capped_cylinder_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_capped_cylinder_result;
    Field_capped_cylinder_result Field_capped_cylinder(float radius, float half_height);

    typedef struct Field_infinite_cylinder_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_infinite_cylinder_result;
    Field_infinite_cylinder_result Field_infinite_cylinder(float radius);

    typedef struct Field_capped_cone_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_capped_cone_result;
    Field_capped_cone_result Field_capped_cone(float half_height, float bottom_radius, float top_radius);

    typedef struct Field_plane_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_plane_result;
    Field_plane_result Field_plane(float normal_x, float normal_y, float normal_z, float offset);

    typedef struct Field_octahedron_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_octahedron_result;
    Field_octahedron_result Field_octahedron(float size);

    typedef struct Field_hex_prism_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_hex_prism_result;
    Field_hex_prism_result Field_hex_prism(float radius, float half_height);

    typedef struct Field_super_prism_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_super_prism_result;
    Field_super_prism_result Field_super_prism(float half_x, float half_y, float half_z, float exponent);

    typedef struct Field_box_frame_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_box_frame_result;
    Field_box_frame_result Field_box_frame(float half_x, float half_y, float half_z, float thickness);

    typedef struct Field_infinite_cone_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_infinite_cone_result;
    Field_infinite_cone_result Field_infinite_cone(float angle_degrees);

    typedef struct Field_square_pyramid_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_square_pyramid_result;
    Field_square_pyramid_result Field_square_pyramid(float half_base, float height);

    typedef struct Field_heightmap_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_heightmap_result;
    Field_heightmap_result Field_heightmap(float base_y, const nucleation::capi::Field* height);

    typedef struct Field_value_noise2_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_value_noise2_result;
    Field_value_noise2_result Field_value_noise2(float frequency, int32_t seed);

    typedef struct Field_value_noise3_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_value_noise3_result;
    Field_value_noise3_result Field_value_noise3(float frequency, int32_t seed);

    typedef struct Field_perlin2_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_perlin2_result;
    Field_perlin2_result Field_perlin2(float frequency, int32_t seed);

    typedef struct Field_perlin3_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_perlin3_result;
    Field_perlin3_result Field_perlin3(float frequency, int32_t seed);

    typedef struct Field_fbm2_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_fbm2_result;
    Field_fbm2_result Field_fbm2(nucleation::capi::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

    typedef struct Field_fbm3_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_fbm3_result;
    Field_fbm3_result Field_fbm3(nucleation::capi::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

    typedef struct Field_value_fbm3_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_value_fbm3_result;
    Field_value_fbm3_result Field_value_fbm3(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_perlin_fbm3_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_perlin_fbm3_result;
    Field_perlin_fbm3_result Field_perlin_fbm3(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_ridged_fbm3_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_ridged_fbm3_result;
    Field_ridged_fbm3_result Field_ridged_fbm3(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_perlin_fbm2_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_perlin_fbm2_result;
    Field_perlin_fbm2_result Field_perlin_fbm2(float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_cells_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_cells_result;
    Field_cells_result Field_cells(float frequency, int32_t seed, float jitter, nucleation::capi::FieldCellMode mode, float threshold);

    nucleation::capi::Field* Field_neg(const nucleation::capi::Field* self);

    nucleation::capi::Field* Field_abs(const nucleation::capi::Field* self);

    nucleation::capi::Field* Field_sqrt(const nucleation::capi::Field* self);

    nucleation::capi::Field* Field_sin(const nucleation::capi::Field* self);

    nucleation::capi::Field* Field_cos(const nucleation::capi::Field* self);

    nucleation::capi::Field* Field_saturate(const nucleation::capi::Field* self);

    nucleation::capi::Field* Field_add(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_sub(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_mul(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_div(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_min_with(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_max_with(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    typedef struct Field_add_const_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_add_const_result;
    Field_add_const_result Field_add_const(const nucleation::capi::Field* self, float value);

    typedef struct Field_mul_const_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_mul_const_result;
    Field_mul_const_result Field_mul_const(const nucleation::capi::Field* self, float value);

    typedef struct Field_clamp_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_clamp_result;
    Field_clamp_result Field_clamp(const nucleation::capi::Field* self, float lo, float hi);

    typedef struct Field_smoothstep_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_smoothstep_result;
    Field_smoothstep_result Field_smoothstep(const nucleation::capi::Field* self, float edge0, float edge1);

    typedef struct Field_step_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_step_result;
    Field_step_result Field_step(const nucleation::capi::Field* self, float edge);

    typedef struct Field_remap_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_remap_result;
    Field_remap_result Field_remap(const nucleation::capi::Field* self, float in_lo, float in_hi, float out_lo, float out_hi, bool clamp);

    nucleation::capi::Field* Field_lerp_with(const nucleation::capi::Field* self, const nucleation::capi::Field* other, const nucleation::capi::Field* t);

    nucleation::capi::Field* Field_select(const nucleation::capi::Field* cond, const nucleation::capi::Field* then_field, const nucleation::capi::Field* else_field);

    nucleation::capi::Field* Field_union_with(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_intersection_with(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_subtract(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    nucleation::capi::Field* Field_xor_with(const nucleation::capi::Field* self, const nucleation::capi::Field* other);

    typedef struct Field_smooth_union_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_smooth_union_result;
    Field_smooth_union_result Field_smooth_union(const nucleation::capi::Field* self, const nucleation::capi::Field* other, float radius);

    typedef struct Field_smooth_subtract_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_smooth_subtract_result;
    Field_smooth_subtract_result Field_smooth_subtract(const nucleation::capi::Field* self, const nucleation::capi::Field* other, float radius);

    typedef struct Field_smooth_intersection_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_smooth_intersection_result;
    Field_smooth_intersection_result Field_smooth_intersection(const nucleation::capi::Field* self, const nucleation::capi::Field* other, float radius);

    typedef struct Field_chamfer_union_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_chamfer_union_result;
    Field_chamfer_union_result Field_chamfer_union(const nucleation::capi::Field* self, const nucleation::capi::Field* other, float radius);

    typedef struct Field_chamfer_subtract_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_chamfer_subtract_result;
    Field_chamfer_subtract_result Field_chamfer_subtract(const nucleation::capi::Field* self, const nucleation::capi::Field* other, float radius);

    typedef struct Field_chamfer_intersection_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_chamfer_intersection_result;
    Field_chamfer_intersection_result Field_chamfer_intersection(const nucleation::capi::Field* self, const nucleation::capi::Field* other, float radius);

    typedef struct Field_steps_union_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_steps_union_result;
    Field_steps_union_result Field_steps_union(const nucleation::capi::Field* self, const nucleation::capi::Field* other, float radius, float steps);

    typedef struct Field_rounded_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_rounded_result;
    Field_rounded_result Field_rounded(const nucleation::capi::Field* self, float radius);

    typedef struct Field_shell_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_shell_result;
    Field_shell_result Field_shell(const nucleation::capi::Field* self, float thickness);

    typedef struct Field_elongate_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_elongate_result;
    Field_elongate_result Field_elongate(const nucleation::capi::Field* self, float half_x, float half_y, float half_z);

    typedef struct Field_displace_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_displace_result;
    Field_displace_result Field_displace(const nucleation::capi::Field* self, const nucleation::capi::Field* offset, float amplitude);

    typedef struct Field_displace_noise_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_displace_noise_result;
    Field_displace_noise_result Field_displace_noise(const nucleation::capi::Field* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_translate_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_translate_result;
    Field_translate_result Field_translate(const nucleation::capi::Field* self, float x, float y, float z);

    typedef struct Field_rotate_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_rotate_result;
    Field_rotate_result Field_rotate(const nucleation::capi::Field* self, float x_degrees, float y_degrees, float z_degrees);

    typedef struct Field_scale_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_scale_result;
    Field_scale_result Field_scale(const nucleation::capi::Field* self, float factor);

    typedef struct Field_scale_xyz_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_scale_xyz_result;
    Field_scale_xyz_result Field_scale_xyz(const nucleation::capi::Field* self, float sx, float sy, float sz);

    nucleation::capi::Field* Field_mirror(const nucleation::capi::Field* self, nucleation::capi::FieldAxis axis);

    typedef struct Field_mirror_plane_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_mirror_plane_result;
    Field_mirror_plane_result Field_mirror_plane(const nucleation::capi::Field* self, float nx, float ny, float nz, float offset);

    typedef struct Field_repeat_infinite_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_repeat_infinite_result;
    Field_repeat_infinite_result Field_repeat_infinite(const nucleation::capi::Field* self, float spacing_x, float spacing_y, float spacing_z);

    typedef struct Field_repeat_counted_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_repeat_counted_result;
    Field_repeat_counted_result Field_repeat_counted(const nucleation::capi::Field* self, float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z);

    typedef struct Field_repeat_polar_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_repeat_polar_result;
    Field_repeat_polar_result Field_repeat_polar(const nucleation::capi::Field* self, uint32_t count);

    typedef struct Field_repeat_points_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_repeat_points_result;
    Field_repeat_points_result Field_repeat_points(const nucleation::capi::Field* self, nucleation::diplomat::capi::DiplomatF32View offsets);

    typedef struct Field_twist_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_twist_result;
    Field_twist_result Field_twist(const nucleation::capi::Field* self, float amount);

    typedef struct Field_bend_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_bend_result;
    Field_bend_result Field_bend(const nucleation::capi::Field* self, float amount);

    typedef struct Field_warp_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_warp_result;
    Field_warp_result Field_warp(const nucleation::capi::Field* self, const nucleation::capi::Field* warp_x, const nucleation::capi::Field* warp_y, const nucleation::capi::Field* warp_z, float amplitude);

    typedef struct Field_warp_noise_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_warp_noise_result;
    Field_warp_noise_result Field_warp_noise(const nucleation::capi::Field* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

    typedef struct Field_tag_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_tag_result;
    Field_tag_result Field_tag(const nucleation::capi::Field* self, uint16_t material_id);

    typedef struct Field_bounded_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_bounded_result;
    Field_bounded_result Field_bounded(const nucleation::capi::Field* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

    typedef struct Field_expr_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_expr_result;
    Field_expr_result Field_expr(nucleation::diplomat::capi::DiplomatStringView source);

    float Field_eval_at(const nucleation::capi::Field* self, float x, float y, float z);

    nucleation::capi::FieldSample Field_sample_tagged(const nucleation::capi::Field* self, float x, float y, float z);

    typedef struct Field_normal_result {union {nucleation::capi::FieldNormal ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_normal_result;
    Field_normal_result Field_normal(const nucleation::capi::Field* self, float x, float y, float z, float epsilon);

    typedef struct Field_bounds_result {union {nucleation::capi::FieldBounds ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_bounds_result;
    Field_bounds_result Field_bounds(const nucleation::capi::Field* self);

    typedef struct Field_output_range_result {union {nucleation::capi::FieldRange ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_output_range_result;
    Field_output_range_result Field_output_range(const nucleation::capi::Field* self);

    nucleation::capi::FieldDistanceKind Field_distance_kind(const nucleation::capi::Field* self);

    typedef struct Field_to_shape_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_to_shape_result;
    Field_to_shape_result Field_to_shape(const nucleation::capi::Field* self);

    typedef struct Field_to_shape_bounded_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_to_shape_bounded_result;
    Field_to_shape_bounded_result Field_to_shape_bounded(const nucleation::capi::Field* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Field_from_json_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field_from_json_result;
    Field_from_json_result Field_from_json(nucleation::diplomat::capi::DiplomatStringView json);

    typedef struct Field_to_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Field_to_json_result;
    Field_to_json_result Field_to_json(const nucleation::capi::Field* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Field_destroy(Field* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::constant(float value) {
    auto result = nucleation::capi::Field_constant(value);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::coord_x() {
    auto result = nucleation::capi::Field_coord_x();
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::coord_y() {
    auto result = nucleation::capi::Field_coord_y();
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::coord_z() {
    auto result = nucleation::capi::Field_coord_z();
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::sphere(float radius) {
    auto result = nucleation::capi::Field_sphere(radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::box_shape(float half_x, float half_y, float half_z, float rounding) {
    auto result = nucleation::capi::Field_box_shape(half_x,
        half_y,
        half_z,
        rounding);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::ellipsoid(float radius_x, float radius_y, float radius_z) {
    auto result = nucleation::capi::Field_ellipsoid(radius_x,
        radius_y,
        radius_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::torus(float major_radius, float minor_radius) {
    auto result = nucleation::capi::Field_torus(major_radius,
        minor_radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::capped_torus(float major_radius, float minor_radius, float cap_angle_degrees) {
    auto result = nucleation::capi::Field_capped_torus(major_radius,
        minor_radius,
        cap_angle_degrees);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::link(float major_radius, float minor_radius, float half_length) {
    auto result = nucleation::capi::Field_link(major_radius,
        minor_radius,
        half_length);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::capsule(float ax, float ay, float az, float bx, float by, float bz, float radius) {
    auto result = nucleation::capi::Field_capsule(ax,
        ay,
        az,
        bx,
        by,
        bz,
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2) {
    auto result = nucleation::capi::Field_round_cone(ax,
        ay,
        az,
        bx,
        by,
        bz,
        r1,
        r2);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::solid_angle(float radius, float angle_degrees) {
    auto result = nucleation::capi::Field_solid_angle(radius,
        angle_degrees);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::cut_sphere(float radius, float height) {
    auto result = nucleation::capi::Field_cut_sphere(radius,
        height);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::cut_hollow_sphere(float radius, float height, float thickness) {
    auto result = nucleation::capi::Field_cut_hollow_sphere(radius,
        height,
        thickness);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::capped_cylinder(float radius, float half_height) {
    auto result = nucleation::capi::Field_capped_cylinder(radius,
        half_height);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::infinite_cylinder(float radius) {
    auto result = nucleation::capi::Field_infinite_cylinder(radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::capped_cone(float half_height, float bottom_radius, float top_radius) {
    auto result = nucleation::capi::Field_capped_cone(half_height,
        bottom_radius,
        top_radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::plane(float normal_x, float normal_y, float normal_z, float offset) {
    auto result = nucleation::capi::Field_plane(normal_x,
        normal_y,
        normal_z,
        offset);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::octahedron(float size) {
    auto result = nucleation::capi::Field_octahedron(size);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::hex_prism(float radius, float half_height) {
    auto result = nucleation::capi::Field_hex_prism(radius,
        half_height);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::super_prism(float half_x, float half_y, float half_z, float exponent) {
    auto result = nucleation::capi::Field_super_prism(half_x,
        half_y,
        half_z,
        exponent);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::box_frame(float half_x, float half_y, float half_z, float thickness) {
    auto result = nucleation::capi::Field_box_frame(half_x,
        half_y,
        half_z,
        thickness);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::infinite_cone(float angle_degrees) {
    auto result = nucleation::capi::Field_infinite_cone(angle_degrees);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::square_pyramid(float half_base, float height) {
    auto result = nucleation::capi::Field_square_pyramid(half_base,
        height);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::heightmap(float base_y, const nucleation::Field& height) {
    auto result = nucleation::capi::Field_heightmap(base_y,
        height.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::value_noise2(float frequency, int32_t seed) {
    auto result = nucleation::capi::Field_value_noise2(frequency,
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::value_noise3(float frequency, int32_t seed) {
    auto result = nucleation::capi::Field_value_noise3(frequency,
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::perlin2(float frequency, int32_t seed) {
    auto result = nucleation::capi::Field_perlin2(frequency,
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::perlin3(float frequency, int32_t seed) {
    auto result = nucleation::capi::Field_perlin3(frequency,
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::fbm2(nucleation::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged) {
    auto result = nucleation::capi::Field_fbm2(basis.AsFFI(),
        frequency,
        seed,
        octaves,
        lacunarity,
        gain,
        ridged);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::fbm3(nucleation::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged) {
    auto result = nucleation::capi::Field_fbm3(basis.AsFFI(),
        frequency,
        seed,
        octaves,
        lacunarity,
        gain,
        ridged);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::value_fbm3(float frequency, int32_t seed, uint32_t octaves) {
    auto result = nucleation::capi::Field_value_fbm3(frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::perlin_fbm3(float frequency, int32_t seed, uint32_t octaves) {
    auto result = nucleation::capi::Field_perlin_fbm3(frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::ridged_fbm3(float frequency, int32_t seed, uint32_t octaves) {
    auto result = nucleation::capi::Field_ridged_fbm3(frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::perlin_fbm2(float frequency, int32_t seed, uint32_t octaves) {
    auto result = nucleation::capi::Field_perlin_fbm2(frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::cells(float frequency, int32_t seed, float jitter, nucleation::FieldCellMode mode, float threshold) {
    auto result = nucleation::capi::Field_cells(frequency,
        seed,
        jitter,
        mode.AsFFI(),
        threshold);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::neg() const {
    auto result = nucleation::capi::Field_neg(this->AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::abs() const {
    auto result = nucleation::capi::Field_abs(this->AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::sqrt() const {
    auto result = nucleation::capi::Field_sqrt(this->AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::sin() const {
    auto result = nucleation::capi::Field_sin(this->AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::cos() const {
    auto result = nucleation::capi::Field_cos(this->AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::saturate() const {
    auto result = nucleation::capi::Field_saturate(this->AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::add(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_add(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::sub(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_sub(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::mul(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_mul(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::div(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_div(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::min_with(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_min_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::max_with(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_max_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::add_const(float value) const {
    auto result = nucleation::capi::Field_add_const(this->AsFFI(),
        value);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::mul_const(float value) const {
    auto result = nucleation::capi::Field_mul_const(this->AsFFI(),
        value);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::clamp(float lo, float hi) const {
    auto result = nucleation::capi::Field_clamp(this->AsFFI(),
        lo,
        hi);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::smoothstep(float edge0, float edge1) const {
    auto result = nucleation::capi::Field_smoothstep(this->AsFFI(),
        edge0,
        edge1);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::step(float edge) const {
    auto result = nucleation::capi::Field_step(this->AsFFI(),
        edge);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::remap(float in_lo, float in_hi, float out_lo, float out_hi, bool clamp) const {
    auto result = nucleation::capi::Field_remap(this->AsFFI(),
        in_lo,
        in_hi,
        out_lo,
        out_hi,
        clamp);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::lerp_with(const nucleation::Field& other, const nucleation::Field& t) const {
    auto result = nucleation::capi::Field_lerp_with(this->AsFFI(),
        other.AsFFI(),
        t.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::select(const nucleation::Field& cond, const nucleation::Field& then_field, const nucleation::Field& else_field) {
    auto result = nucleation::capi::Field_select(cond.AsFFI(),
        then_field.AsFFI(),
        else_field.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::union_with(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::intersection_with(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_intersection_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::subtract(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_subtract(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::xor_with(const nucleation::Field& other) const {
    auto result = nucleation::capi::Field_xor_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::smooth_union(const nucleation::Field& other, float radius) const {
    auto result = nucleation::capi::Field_smooth_union(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::smooth_subtract(const nucleation::Field& other, float radius) const {
    auto result = nucleation::capi::Field_smooth_subtract(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::smooth_intersection(const nucleation::Field& other, float radius) const {
    auto result = nucleation::capi::Field_smooth_intersection(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::chamfer_union(const nucleation::Field& other, float radius) const {
    auto result = nucleation::capi::Field_chamfer_union(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::chamfer_subtract(const nucleation::Field& other, float radius) const {
    auto result = nucleation::capi::Field_chamfer_subtract(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::chamfer_intersection(const nucleation::Field& other, float radius) const {
    auto result = nucleation::capi::Field_chamfer_intersection(this->AsFFI(),
        other.AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::steps_union(const nucleation::Field& other, float radius, float steps) const {
    auto result = nucleation::capi::Field_steps_union(this->AsFFI(),
        other.AsFFI(),
        radius,
        steps);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::rounded(float radius) const {
    auto result = nucleation::capi::Field_rounded(this->AsFFI(),
        radius);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::shell(float thickness) const {
    auto result = nucleation::capi::Field_shell(this->AsFFI(),
        thickness);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::elongate(float half_x, float half_y, float half_z) const {
    auto result = nucleation::capi::Field_elongate(this->AsFFI(),
        half_x,
        half_y,
        half_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::displace(const nucleation::Field& offset, float amplitude) const {
    auto result = nucleation::capi::Field_displace(this->AsFFI(),
        offset.AsFFI(),
        amplitude);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::displace_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const {
    auto result = nucleation::capi::Field_displace_noise(this->AsFFI(),
        amplitude,
        frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::translate(float x, float y, float z) const {
    auto result = nucleation::capi::Field_translate(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::rotate(float x_degrees, float y_degrees, float z_degrees) const {
    auto result = nucleation::capi::Field_rotate(this->AsFFI(),
        x_degrees,
        y_degrees,
        z_degrees);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::scale(float factor) const {
    auto result = nucleation::capi::Field_scale(this->AsFFI(),
        factor);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::scale_xyz(float sx, float sy, float sz) const {
    auto result = nucleation::capi::Field_scale_xyz(this->AsFFI(),
        sx,
        sy,
        sz);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Field> nucleation::Field::mirror(nucleation::FieldAxis axis) const {
    auto result = nucleation::capi::Field_mirror(this->AsFFI(),
        axis.AsFFI());
    return std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::mirror_plane(float nx, float ny, float nz, float offset) const {
    auto result = nucleation::capi::Field_mirror_plane(this->AsFFI(),
        nx,
        ny,
        nz,
        offset);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const {
    auto result = nucleation::capi::Field_repeat_infinite(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const {
    auto result = nucleation::capi::Field_repeat_counted(this->AsFFI(),
        spacing_x,
        spacing_y,
        spacing_z,
        count_x,
        count_y,
        count_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::repeat_polar(uint32_t count) const {
    auto result = nucleation::capi::Field_repeat_polar(this->AsFFI(),
        count);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::repeat_points(nucleation::diplomat::span<const float> offsets) const {
    auto result = nucleation::capi::Field_repeat_points(this->AsFFI(),
        {offsets.data(), offsets.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::twist(float amount) const {
    auto result = nucleation::capi::Field_twist(this->AsFFI(),
        amount);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::bend(float amount) const {
    auto result = nucleation::capi::Field_bend(this->AsFFI(),
        amount);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::warp(const nucleation::Field& warp_x, const nucleation::Field& warp_y, const nucleation::Field& warp_z, float amplitude) const {
    auto result = nucleation::capi::Field_warp(this->AsFFI(),
        warp_x.AsFFI(),
        warp_y.AsFFI(),
        warp_z.AsFFI(),
        amplitude);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::warp_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const {
    auto result = nucleation::capi::Field_warp_noise(this->AsFFI(),
        amplitude,
        frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::tag(uint16_t material_id) const {
    auto result = nucleation::capi::Field_tag(this->AsFFI(),
        material_id);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::bounded(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) const {
    auto result = nucleation::capi::Field_bounded(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::expr(std::string_view source) {
    auto result = nucleation::capi::Field_expr({source.data(), source.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline float nucleation::Field::eval_at(float x, float y, float z) const {
    auto result = nucleation::capi::Field_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline nucleation::FieldSample nucleation::Field::sample_tagged(float x, float y, float z) const {
    auto result = nucleation::capi::Field_sample_tagged(this->AsFFI(),
        x,
        y,
        z);
    return nucleation::FieldSample::FromFFI(result);
}

inline nucleation::diplomat::result<nucleation::FieldNormal, nucleation::NucleationError> nucleation::Field::normal(float x, float y, float z, float epsilon) const {
    auto result = nucleation::capi::Field_normal(this->AsFFI(),
        x,
        y,
        z,
        epsilon);
    return result.is_ok ? nucleation::diplomat::result<nucleation::FieldNormal, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::FieldNormal>(nucleation::FieldNormal::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::FieldNormal, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::FieldBounds, nucleation::NucleationError> nucleation::Field::bounds() const {
    auto result = nucleation::capi::Field_bounds(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::FieldBounds, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::FieldBounds>(nucleation::FieldBounds::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::FieldBounds, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError> nucleation::Field::output_range() const {
    auto result = nucleation::capi::Field_output_range(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::FieldRange>(nucleation::FieldRange::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::FieldDistanceKind nucleation::Field::distance_kind() const {
    auto result = nucleation::capi::Field_distance_kind(this->AsFFI());
    return nucleation::FieldDistanceKind::FromFFI(result);
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Field::to_shape() const {
    auto result = nucleation::capi::Field_to_shape(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Field::to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = nucleation::capi::Field_to_shape_bounded(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::Field::from_json(std::string_view json) {
    auto result = nucleation::capi::Field_from_json({json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Field::to_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Field_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Field::to_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Field_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Field* nucleation::Field::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Field*>(this);
}

inline nucleation::capi::Field* nucleation::Field::AsFFI() {
    return reinterpret_cast<nucleation::capi::Field*>(this);
}

inline const nucleation::Field* nucleation::Field::FromFFI(const nucleation::capi::Field* ptr) {
    return reinterpret_cast<const nucleation::Field*>(ptr);
}

inline nucleation::Field* nucleation::Field::FromFFI(nucleation::capi::Field* ptr) {
    return reinterpret_cast<nucleation::Field*>(ptr);
}

inline void nucleation::Field::operator delete(void* ptr) {
    nucleation::capi::Field_destroy(reinterpret_cast<nucleation::capi::Field*>(ptr));
}


#endif // NUCLEATION_Field_HPP
