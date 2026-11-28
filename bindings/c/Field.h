#ifndef Field_H
#define Field_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "FieldAxis.d.h"
#include "FieldBounds.d.h"
#include "FieldCellMode.d.h"
#include "FieldDistanceKind.d.h"
#include "FieldNoiseBasis.d.h"
#include "FieldNormal.d.h"
#include "FieldRange.d.h"
#include "FieldSample.d.h"
#include "NucleationError.d.h"
#include "Shape.d.h"

#include "Field.d.h"






typedef struct Field_constant_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_constant_result;
Field_constant_result Field_constant(float value);

Field* Field_coord_x(void);

Field* Field_coord_y(void);

Field* Field_coord_z(void);

typedef struct Field_sphere_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_sphere_result;
Field_sphere_result Field_sphere(float radius);

typedef struct Field_box_shape_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_box_shape_result;
Field_box_shape_result Field_box_shape(float half_x, float half_y, float half_z, float rounding);

typedef struct Field_ellipsoid_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_ellipsoid_result;
Field_ellipsoid_result Field_ellipsoid(float radius_x, float radius_y, float radius_z);

typedef struct Field_torus_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_torus_result;
Field_torus_result Field_torus(float major_radius, float minor_radius);

typedef struct Field_capped_torus_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_capped_torus_result;
Field_capped_torus_result Field_capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

typedef struct Field_link_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_link_result;
Field_link_result Field_link(float major_radius, float minor_radius, float half_length);

typedef struct Field_capsule_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_capsule_result;
Field_capsule_result Field_capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

typedef struct Field_round_cone_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_round_cone_result;
Field_round_cone_result Field_round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

typedef struct Field_solid_angle_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_solid_angle_result;
Field_solid_angle_result Field_solid_angle(float radius, float angle_degrees);

typedef struct Field_cut_sphere_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_cut_sphere_result;
Field_cut_sphere_result Field_cut_sphere(float radius, float height);

typedef struct Field_cut_hollow_sphere_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_cut_hollow_sphere_result;
Field_cut_hollow_sphere_result Field_cut_hollow_sphere(float radius, float height, float thickness);

typedef struct Field_capped_cylinder_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_capped_cylinder_result;
Field_capped_cylinder_result Field_capped_cylinder(float radius, float half_height);

typedef struct Field_infinite_cylinder_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_infinite_cylinder_result;
Field_infinite_cylinder_result Field_infinite_cylinder(float radius);

typedef struct Field_capped_cone_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_capped_cone_result;
Field_capped_cone_result Field_capped_cone(float half_height, float bottom_radius, float top_radius);

typedef struct Field_plane_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_plane_result;
Field_plane_result Field_plane(float normal_x, float normal_y, float normal_z, float offset);

typedef struct Field_octahedron_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_octahedron_result;
Field_octahedron_result Field_octahedron(float size);

typedef struct Field_hex_prism_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_hex_prism_result;
Field_hex_prism_result Field_hex_prism(float radius, float half_height);

typedef struct Field_super_prism_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_super_prism_result;
Field_super_prism_result Field_super_prism(float half_x, float half_y, float half_z, float exponent);

typedef struct Field_box_frame_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_box_frame_result;
Field_box_frame_result Field_box_frame(float half_x, float half_y, float half_z, float thickness);

typedef struct Field_infinite_cone_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_infinite_cone_result;
Field_infinite_cone_result Field_infinite_cone(float angle_degrees);

typedef struct Field_square_pyramid_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_square_pyramid_result;
Field_square_pyramid_result Field_square_pyramid(float half_base, float height);

typedef struct Field_heightmap_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_heightmap_result;
Field_heightmap_result Field_heightmap(float base_y, const Field* height);

typedef struct Field_value_noise2_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_value_noise2_result;
Field_value_noise2_result Field_value_noise2(float frequency, int32_t seed);

typedef struct Field_value_noise3_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_value_noise3_result;
Field_value_noise3_result Field_value_noise3(float frequency, int32_t seed);

typedef struct Field_perlin2_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_perlin2_result;
Field_perlin2_result Field_perlin2(float frequency, int32_t seed);

typedef struct Field_perlin3_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_perlin3_result;
Field_perlin3_result Field_perlin3(float frequency, int32_t seed);

typedef struct Field_fbm2_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_fbm2_result;
Field_fbm2_result Field_fbm2(FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

typedef struct Field_fbm3_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_fbm3_result;
Field_fbm3_result Field_fbm3(FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

typedef struct Field_value_fbm3_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_value_fbm3_result;
Field_value_fbm3_result Field_value_fbm3(float frequency, int32_t seed, uint32_t octaves);

typedef struct Field_perlin_fbm3_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_perlin_fbm3_result;
Field_perlin_fbm3_result Field_perlin_fbm3(float frequency, int32_t seed, uint32_t octaves);

typedef struct Field_ridged_fbm3_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_ridged_fbm3_result;
Field_ridged_fbm3_result Field_ridged_fbm3(float frequency, int32_t seed, uint32_t octaves);

typedef struct Field_perlin_fbm2_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_perlin_fbm2_result;
Field_perlin_fbm2_result Field_perlin_fbm2(float frequency, int32_t seed, uint32_t octaves);

typedef struct Field_cells_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_cells_result;
Field_cells_result Field_cells(float frequency, int32_t seed, float jitter, FieldCellMode mode, float threshold);

Field* Field_neg(const Field* self);

Field* Field_abs(const Field* self);

Field* Field_sqrt(const Field* self);

Field* Field_sin(const Field* self);

Field* Field_cos(const Field* self);

Field* Field_saturate(const Field* self);

Field* Field_add(const Field* self, const Field* other);

Field* Field_sub(const Field* self, const Field* other);

Field* Field_mul(const Field* self, const Field* other);

Field* Field_div(const Field* self, const Field* other);

Field* Field_min_with(const Field* self, const Field* other);

Field* Field_max_with(const Field* self, const Field* other);

typedef struct Field_add_const_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_add_const_result;
Field_add_const_result Field_add_const(const Field* self, float value);

typedef struct Field_mul_const_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_mul_const_result;
Field_mul_const_result Field_mul_const(const Field* self, float value);

typedef struct Field_clamp_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_clamp_result;
Field_clamp_result Field_clamp(const Field* self, float lo, float hi);

typedef struct Field_smoothstep_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_smoothstep_result;
Field_smoothstep_result Field_smoothstep(const Field* self, float edge0, float edge1);

typedef struct Field_step_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_step_result;
Field_step_result Field_step(const Field* self, float edge);

typedef struct Field_remap_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_remap_result;
Field_remap_result Field_remap(const Field* self, float in_lo, float in_hi, float out_lo, float out_hi, bool clamp);

Field* Field_lerp_with(const Field* self, const Field* other, const Field* t);

Field* Field_select(const Field* cond, const Field* then_field, const Field* else_field);

Field* Field_union_with(const Field* self, const Field* other);

Field* Field_intersection_with(const Field* self, const Field* other);

Field* Field_subtract(const Field* self, const Field* other);

Field* Field_xor_with(const Field* self, const Field* other);

typedef struct Field_smooth_union_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_smooth_union_result;
Field_smooth_union_result Field_smooth_union(const Field* self, const Field* other, float radius);

typedef struct Field_smooth_subtract_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_smooth_subtract_result;
Field_smooth_subtract_result Field_smooth_subtract(const Field* self, const Field* other, float radius);

typedef struct Field_smooth_intersection_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_smooth_intersection_result;
Field_smooth_intersection_result Field_smooth_intersection(const Field* self, const Field* other, float radius);

typedef struct Field_chamfer_union_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_chamfer_union_result;
Field_chamfer_union_result Field_chamfer_union(const Field* self, const Field* other, float radius);

typedef struct Field_chamfer_subtract_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_chamfer_subtract_result;
Field_chamfer_subtract_result Field_chamfer_subtract(const Field* self, const Field* other, float radius);

typedef struct Field_chamfer_intersection_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_chamfer_intersection_result;
Field_chamfer_intersection_result Field_chamfer_intersection(const Field* self, const Field* other, float radius);

typedef struct Field_steps_union_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_steps_union_result;
Field_steps_union_result Field_steps_union(const Field* self, const Field* other, float radius, float steps);

typedef struct Field_rounded_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_rounded_result;
Field_rounded_result Field_rounded(const Field* self, float radius);

typedef struct Field_shell_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_shell_result;
Field_shell_result Field_shell(const Field* self, float thickness);

typedef struct Field_elongate_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_elongate_result;
Field_elongate_result Field_elongate(const Field* self, float half_x, float half_y, float half_z);

typedef struct Field_displace_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_displace_result;
Field_displace_result Field_displace(const Field* self, const Field* offset, float amplitude);

typedef struct Field_displace_noise_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_displace_noise_result;
Field_displace_noise_result Field_displace_noise(const Field* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

typedef struct Field_translate_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_translate_result;
Field_translate_result Field_translate(const Field* self, float x, float y, float z);

typedef struct Field_rotate_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_rotate_result;
Field_rotate_result Field_rotate(const Field* self, float x_degrees, float y_degrees, float z_degrees);

typedef struct Field_scale_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_scale_result;
Field_scale_result Field_scale(const Field* self, float factor);

typedef struct Field_scale_xyz_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_scale_xyz_result;
Field_scale_xyz_result Field_scale_xyz(const Field* self, float sx, float sy, float sz);

Field* Field_mirror(const Field* self, FieldAxis axis);

typedef struct Field_mirror_plane_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_mirror_plane_result;
Field_mirror_plane_result Field_mirror_plane(const Field* self, float nx, float ny, float nz, float offset);

typedef struct Field_repeat_infinite_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_repeat_infinite_result;
Field_repeat_infinite_result Field_repeat_infinite(const Field* self, float spacing_x, float spacing_y, float spacing_z);

typedef struct Field_repeat_counted_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_repeat_counted_result;
Field_repeat_counted_result Field_repeat_counted(const Field* self, float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z);

typedef struct Field_repeat_polar_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_repeat_polar_result;
Field_repeat_polar_result Field_repeat_polar(const Field* self, uint32_t count);

typedef struct Field_repeat_points_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_repeat_points_result;
Field_repeat_points_result Field_repeat_points(const Field* self, DiplomatF32View offsets);

typedef struct Field_twist_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_twist_result;
Field_twist_result Field_twist(const Field* self, float amount);

typedef struct Field_bend_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_bend_result;
Field_bend_result Field_bend(const Field* self, float amount);

typedef struct Field_warp_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_warp_result;
Field_warp_result Field_warp(const Field* self, const Field* warp_x, const Field* warp_y, const Field* warp_z, float amplitude);

typedef struct Field_warp_noise_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_warp_noise_result;
Field_warp_noise_result Field_warp_noise(const Field* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

typedef struct Field_tag_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_tag_result;
Field_tag_result Field_tag(const Field* self, uint16_t material_id);

typedef struct Field_bounded_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_bounded_result;
Field_bounded_result Field_bounded(const Field* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

typedef struct Field_expr_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_expr_result;
Field_expr_result Field_expr(DiplomatStringView source);

float Field_eval_at(const Field* self, float x, float y, float z);

FieldSample Field_sample_tagged(const Field* self, float x, float y, float z);

typedef struct Field_normal_result {union {FieldNormal ok; NucleationError err;}; bool is_ok;} Field_normal_result;
Field_normal_result Field_normal(const Field* self, float x, float y, float z, float epsilon);

typedef struct Field_bounds_result {union {FieldBounds ok; NucleationError err;}; bool is_ok;} Field_bounds_result;
Field_bounds_result Field_bounds(const Field* self);

typedef struct Field_output_range_result {union {FieldRange ok; NucleationError err;}; bool is_ok;} Field_output_range_result;
Field_output_range_result Field_output_range(const Field* self);

FieldDistanceKind Field_distance_kind(const Field* self);

typedef struct Field_to_shape_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Field_to_shape_result;
Field_to_shape_result Field_to_shape(const Field* self);

typedef struct Field_to_shape_bounded_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Field_to_shape_bounded_result;
Field_to_shape_bounded_result Field_to_shape_bounded(const Field* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Field_from_json_result {union {Field* ok; NucleationError err;}; bool is_ok;} Field_from_json_result;
Field_from_json_result Field_from_json(DiplomatStringView json);

typedef struct Field_to_json_result {union { NucleationError err;}; bool is_ok;} Field_to_json_result;
Field_to_json_result Field_to_json(const Field* self, DiplomatWrite* write);

void Field_destroy(Field* self);





#endif // Field_H
