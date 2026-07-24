#ifndef Shape_H
#define Shape_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Curve3D.d.h"
#include "NucleationError.d.h"

#include "Shape.d.h"






typedef struct Shape_tube_along_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Shape_tube_along_result;
Shape_tube_along_result Shape_tube_along(const Curve3D* curve, double radius);

Shape* Shape_sphere(float cx, float cy, float cz, float radius);

Shape* Shape_cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Shape_polygon_prism_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Shape_polygon_prism_result;
Shape_polygon_prism_result Shape_polygon_prism(DiplomatStringView polygon_json, int32_t y_min, int32_t y_max);

Shape* Shape_ellipsoid(int32_t cx, int32_t cy, int32_t cz, float rx, float ry, float rz);

Shape* Shape_cylinder(float bx, float by, float bz, float ax, float ay, float az, float radius, float height);

Shape* Shape_cylinder_between(float x1, float y1, float z1, float x2, float y2, float z2, float radius);

Shape* Shape_cone(float ax, float ay, float az, float dx, float dy, float dz, float radius, float height);

Shape* Shape_torus(float cx, float cy, float cz, float major_r, float minor_r, float ax, float ay, float az);

Shape* Shape_pyramid(float bx, float by, float bz, float half_w, float half_d, float height, float ax, float ay, float az);

Shape* Shape_disk(float cx, float cy, float cz, float radius, float nx, float ny, float nz, float thickness);

Shape* Shape_plane(float ox, float oy, float oz, float ux, float uy, float uz, float vx, float vy, float vz, float u_ext, float v_ext, float thickness);

Shape* Shape_triangle(float ax, float ay, float az, float bx, float by, float bz, float cx, float cy, float cz, float thickness);

Shape* Shape_line(float x1, float y1, float z1, float x2, float y2, float z2, float thickness);

typedef struct Shape_bezier_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Shape_bezier_result;
Shape_bezier_result Shape_bezier(DiplomatF32View control_points, float thickness, uint32_t resolution);

typedef struct Shape_sdf_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Shape_sdf_result;
Shape_sdf_result Shape_sdf(DiplomatStringView sdf_json);

typedef struct Shape_sdf_bounded_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Shape_sdf_bounded_result;
Shape_sdf_bounded_result Shape_sdf_bounded(DiplomatStringView sdf_json, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

Shape* Shape_hollow(const Shape* self, uint32_t thickness);

Shape* Shape_union_with(const Shape* self, const Shape* other);

Shape* Shape_intersection_with(const Shape* self, const Shape* other);

Shape* Shape_difference_with(const Shape* self, const Shape* other);

void Shape_destroy(Shape* self);





#endif // Shape_H
