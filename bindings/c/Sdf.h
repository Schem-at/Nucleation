#ifndef Sdf_H
#define Sdf_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Field3.d.h"
#include "FieldProgram.d.h"
#include "NucleationError.d.h"
#include "Schematic.d.h"
#include "SdfAxis.d.h"
#include "SdfBounds.d.h"
#include "SdfCellMode.d.h"
#include "SdfNormal.d.h"
#include "Shape.d.h"

#include "Sdf.d.h"






typedef struct Sdf_sphere_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_sphere_result;
Sdf_sphere_result Sdf_sphere(float radius);

typedef struct Sdf_box_shape_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_box_shape_result;
Sdf_box_shape_result Sdf_box_shape(float half_x, float half_y, float half_z, float rounding);

typedef struct Sdf_ellipsoid_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_ellipsoid_result;
Sdf_ellipsoid_result Sdf_ellipsoid(float radius_x, float radius_y, float radius_z);

typedef struct Sdf_torus_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_torus_result;
Sdf_torus_result Sdf_torus(float major_radius, float minor_radius);

typedef struct Sdf_capped_torus_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_capped_torus_result;
Sdf_capped_torus_result Sdf_capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

typedef struct Sdf_link_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_link_result;
Sdf_link_result Sdf_link(float major_radius, float minor_radius, float half_length);

typedef struct Sdf_capsule_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_capsule_result;
Sdf_capsule_result Sdf_capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

typedef struct Sdf_round_cone_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_round_cone_result;
Sdf_round_cone_result Sdf_round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

typedef struct Sdf_solid_angle_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_solid_angle_result;
Sdf_solid_angle_result Sdf_solid_angle(float radius, float angle_degrees);

typedef struct Sdf_cut_sphere_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_cut_sphere_result;
Sdf_cut_sphere_result Sdf_cut_sphere(float radius, float height);

typedef struct Sdf_cut_hollow_sphere_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_cut_hollow_sphere_result;
Sdf_cut_hollow_sphere_result Sdf_cut_hollow_sphere(float radius, float height, float thickness);

typedef struct Sdf_capped_cylinder_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_capped_cylinder_result;
Sdf_capped_cylinder_result Sdf_capped_cylinder(float radius, float half_height);

typedef struct Sdf_infinite_cylinder_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_infinite_cylinder_result;
Sdf_infinite_cylinder_result Sdf_infinite_cylinder(float radius);

typedef struct Sdf_capped_cone_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_capped_cone_result;
Sdf_capped_cone_result Sdf_capped_cone(float half_height, float bottom_radius, float top_radius);

typedef struct Sdf_plane_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_plane_result;
Sdf_plane_result Sdf_plane(float normal_x, float normal_y, float normal_z, float offset);

typedef struct Sdf_octahedron_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_octahedron_result;
Sdf_octahedron_result Sdf_octahedron(float size);

typedef struct Sdf_hex_prism_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_hex_prism_result;
Sdf_hex_prism_result Sdf_hex_prism(float radius, float half_height);

typedef struct Sdf_super_prism_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_super_prism_result;
Sdf_super_prism_result Sdf_super_prism(float half_x, float half_y, float half_z, float exponent);

typedef struct Sdf_box_frame_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_box_frame_result;
Sdf_box_frame_result Sdf_box_frame(float half_x, float half_y, float half_z, float thickness);

typedef struct Sdf_infinite_cone_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_infinite_cone_result;
Sdf_infinite_cone_result Sdf_infinite_cone(float angle_degrees);

typedef struct Sdf_square_pyramid_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_square_pyramid_result;
Sdf_square_pyramid_result Sdf_square_pyramid(float half_base, float height);

typedef struct Sdf_cells_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_cells_result;
Sdf_cells_result Sdf_cells(float frequency, int32_t seed, float jitter, SdfCellMode mode, float threshold);

Sdf* Sdf_union_with(const Sdf* self, const Sdf* other);

Sdf* Sdf_intersection_with(const Sdf* self, const Sdf* other);

Sdf* Sdf_subtract(const Sdf* self, const Sdf* other);

typedef struct Sdf_smooth_union_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_smooth_union_result;
Sdf_smooth_union_result Sdf_smooth_union(const Sdf* self, const Sdf* other, float radius);

typedef struct Sdf_smooth_subtract_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_smooth_subtract_result;
Sdf_smooth_subtract_result Sdf_smooth_subtract(const Sdf* self, const Sdf* other, float radius);

typedef struct Sdf_smooth_intersection_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_smooth_intersection_result;
Sdf_smooth_intersection_result Sdf_smooth_intersection(const Sdf* self, const Sdf* other, float radius);

typedef struct Sdf_rounded_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_rounded_result;
Sdf_rounded_result Sdf_rounded(const Sdf* self, float radius);

typedef struct Sdf_shell_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_shell_result;
Sdf_shell_result Sdf_shell(const Sdf* self, float thickness);

Sdf* Sdf_xor_with(const Sdf* self, const Sdf* other);

typedef struct Sdf_elongate_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_elongate_result;
Sdf_elongate_result Sdf_elongate(const Sdf* self, float half_x, float half_y, float half_z);

typedef struct Sdf_translate_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_translate_result;
Sdf_translate_result Sdf_translate(const Sdf* self, float x, float y, float z);

typedef struct Sdf_rotate_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_rotate_result;
Sdf_rotate_result Sdf_rotate(const Sdf* self, float x_degrees, float y_degrees, float z_degrees);

typedef struct Sdf_scale_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_scale_result;
Sdf_scale_result Sdf_scale(const Sdf* self, float factor);

Sdf* Sdf_mirror(const Sdf* self, SdfAxis axis);

typedef struct Sdf_twist_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_twist_result;
Sdf_twist_result Sdf_twist(const Sdf* self, float amount);

typedef struct Sdf_bend_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_bend_result;
Sdf_bend_result Sdf_bend(const Sdf* self, float amount);

typedef struct Sdf_repeat_infinite_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_repeat_infinite_result;
Sdf_repeat_infinite_result Sdf_repeat_infinite(const Sdf* self, float spacing_x, float spacing_y, float spacing_z);

typedef struct Sdf_repeat_counted_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_repeat_counted_result;
Sdf_repeat_counted_result Sdf_repeat_counted(const Sdf* self, float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z);

typedef struct Sdf_repeat_points_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_repeat_points_result;
Sdf_repeat_points_result Sdf_repeat_points(const Sdf* self, DiplomatF32View offsets);

typedef struct Sdf_displace_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_displace_result;
Sdf_displace_result Sdf_displace(const Sdf* self, float amplitude, float frequency, int32_t seed, uint32_t octaves);

typedef struct Sdf_offset_by_field_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_offset_by_field_result;
Sdf_offset_by_field_result Sdf_offset_by_field(const Sdf* self, const Field3* field, float amplitude);

typedef struct Sdf_warp_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_warp_result;
Sdf_warp_result Sdf_warp(const Sdf* self, float amplitude, float frequency, int32_t seed);

float Sdf_eval_at(const Sdf* self, float x, float y, float z);

typedef struct Sdf_normal_result {union {SdfNormal ok; NucleationError err;}; bool is_ok;} Sdf_normal_result;
Sdf_normal_result Sdf_normal(const Sdf* self, float x, float y, float z, float epsilon);

typedef struct Sdf_bounds_result {union {SdfBounds ok; NucleationError err;}; bool is_ok;} Sdf_bounds_result;
Sdf_bounds_result Sdf_bounds(const Sdf* self);

typedef struct Sdf_to_shape_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Sdf_to_shape_result;
Sdf_to_shape_result Sdf_to_shape(const Sdf* self);

typedef struct Sdf_to_shape_bounded_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Sdf_to_shape_bounded_result;
Sdf_to_shape_bounded_result Sdf_to_shape_bounded(const Sdf* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Sdf_from_json_string_result {union {Sdf* ok; NucleationError err;}; bool is_ok;} Sdf_from_json_string_result;
Sdf_from_json_string_result Sdf_from_json_string(DiplomatStringView json);

typedef struct Sdf_to_json_result {union { NucleationError err;}; bool is_ok;} Sdf_to_json_result;
Sdf_to_json_result Sdf_to_json(const Sdf* self, DiplomatWrite* write);

Sdf* Sdf_from_program(const FieldProgram* program);

typedef struct Sdf_schematic_from_sdf_auto_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_auto_result;
Sdf_schematic_from_sdf_auto_result Sdf_schematic_from_sdf_auto(DiplomatStringView sdf_json, DiplomatStringView rules_json);

typedef struct Sdf_schematic_from_sdf_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_result;
Sdf_schematic_from_sdf_result Sdf_schematic_from_sdf(DiplomatStringView sdf_json, DiplomatStringView rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Sdf_eval_result {union {float ok; NucleationError err;}; bool is_ok;} Sdf_eval_result;
Sdf_eval_result Sdf_eval(DiplomatStringView sdf_json, float x, float y, float z);

void Sdf_destroy(Sdf* self);





#endif // Sdf_H
