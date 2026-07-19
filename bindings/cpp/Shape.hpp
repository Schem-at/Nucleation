#ifndef Shape_HPP
#define Shape_HPP

#include "Shape.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::Shape* Shape_sphere(float cx, float cy, float cz, float radius);

    diplomat::capi::Shape* Shape_cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Shape_polygon_prism_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Shape_polygon_prism_result;
    Shape_polygon_prism_result Shape_polygon_prism(diplomat::capi::DiplomatStringView polygon_json, int32_t y_min, int32_t y_max);

    diplomat::capi::Shape* Shape_ellipsoid(int32_t cx, int32_t cy, int32_t cz, float rx, float ry, float rz);

    diplomat::capi::Shape* Shape_cylinder(float bx, float by, float bz, float ax, float ay, float az, float radius, float height);

    diplomat::capi::Shape* Shape_cylinder_between(float x1, float y1, float z1, float x2, float y2, float z2, float radius);

    diplomat::capi::Shape* Shape_cone(float ax, float ay, float az, float dx, float dy, float dz, float radius, float height);

    diplomat::capi::Shape* Shape_torus(float cx, float cy, float cz, float major_r, float minor_r, float ax, float ay, float az);

    diplomat::capi::Shape* Shape_pyramid(float bx, float by, float bz, float half_w, float half_d, float height, float ax, float ay, float az);

    diplomat::capi::Shape* Shape_disk(float cx, float cy, float cz, float radius, float nx, float ny, float nz, float thickness);

    diplomat::capi::Shape* Shape_plane(float ox, float oy, float oz, float ux, float uy, float uz, float vx, float vy, float vz, float u_ext, float v_ext, float thickness);

    diplomat::capi::Shape* Shape_triangle(float ax, float ay, float az, float bx, float by, float bz, float cx, float cy, float cz, float thickness);

    diplomat::capi::Shape* Shape_line(float x1, float y1, float z1, float x2, float y2, float z2, float thickness);

    typedef struct Shape_bezier_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Shape_bezier_result;
    Shape_bezier_result Shape_bezier(diplomat::capi::DiplomatF32View control_points, float thickness, uint32_t resolution);

    typedef struct Shape_sdf_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Shape_sdf_result;
    Shape_sdf_result Shape_sdf(diplomat::capi::DiplomatStringView sdf_json);

    typedef struct Shape_sdf_bounded_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Shape_sdf_bounded_result;
    Shape_sdf_bounded_result Shape_sdf_bounded(diplomat::capi::DiplomatStringView sdf_json, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    diplomat::capi::Shape* Shape_hollow(const diplomat::capi::Shape* self, uint32_t thickness);

    diplomat::capi::Shape* Shape_union_with(const diplomat::capi::Shape* self, const diplomat::capi::Shape* other);

    diplomat::capi::Shape* Shape_intersection_with(const diplomat::capi::Shape* self, const diplomat::capi::Shape* other);

    diplomat::capi::Shape* Shape_difference_with(const diplomat::capi::Shape* self, const diplomat::capi::Shape* other);

    void Shape_destroy(Shape* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<Shape> Shape::sphere(float cx, float cy, float cz, float radius) {
    auto result = diplomat::capi::Shape_sphere(cx,
        cy,
        cz,
        radius);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::Shape_cuboid(min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Shape::polygon_prism(std::string_view polygon_json, int32_t y_min, int32_t y_max) {
    auto result = diplomat::capi::Shape_polygon_prism({polygon_json.data(), polygon_json.size()},
        y_min,
        y_max);
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Shape> Shape::ellipsoid(int32_t cx, int32_t cy, int32_t cz, float rx, float ry, float rz) {
    auto result = diplomat::capi::Shape_ellipsoid(cx,
        cy,
        cz,
        rx,
        ry,
        rz);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::cylinder(float bx, float by, float bz, float ax, float ay, float az, float radius, float height) {
    auto result = diplomat::capi::Shape_cylinder(bx,
        by,
        bz,
        ax,
        ay,
        az,
        radius,
        height);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::cylinder_between(float x1, float y1, float z1, float x2, float y2, float z2, float radius) {
    auto result = diplomat::capi::Shape_cylinder_between(x1,
        y1,
        z1,
        x2,
        y2,
        z2,
        radius);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::cone(float ax, float ay, float az, float dx, float dy, float dz, float radius, float height) {
    auto result = diplomat::capi::Shape_cone(ax,
        ay,
        az,
        dx,
        dy,
        dz,
        radius,
        height);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::torus(float cx, float cy, float cz, float major_r, float minor_r, float ax, float ay, float az) {
    auto result = diplomat::capi::Shape_torus(cx,
        cy,
        cz,
        major_r,
        minor_r,
        ax,
        ay,
        az);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::pyramid(float bx, float by, float bz, float half_w, float half_d, float height, float ax, float ay, float az) {
    auto result = diplomat::capi::Shape_pyramid(bx,
        by,
        bz,
        half_w,
        half_d,
        height,
        ax,
        ay,
        az);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::disk(float cx, float cy, float cz, float radius, float nx, float ny, float nz, float thickness) {
    auto result = diplomat::capi::Shape_disk(cx,
        cy,
        cz,
        radius,
        nx,
        ny,
        nz,
        thickness);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::plane(float ox, float oy, float oz, float ux, float uy, float uz, float vx, float vy, float vz, float u_ext, float v_ext, float thickness) {
    auto result = diplomat::capi::Shape_plane(ox,
        oy,
        oz,
        ux,
        uy,
        uz,
        vx,
        vy,
        vz,
        u_ext,
        v_ext,
        thickness);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::triangle(float ax, float ay, float az, float bx, float by, float bz, float cx, float cy, float cz, float thickness) {
    auto result = diplomat::capi::Shape_triangle(ax,
        ay,
        az,
        bx,
        by,
        bz,
        cx,
        cy,
        cz,
        thickness);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::line(float x1, float y1, float z1, float x2, float y2, float z2, float thickness) {
    auto result = diplomat::capi::Shape_line(x1,
        y1,
        z1,
        x2,
        y2,
        z2,
        thickness);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Shape::bezier(diplomat::span<const float> control_points, float thickness, uint32_t resolution) {
    auto result = diplomat::capi::Shape_bezier({control_points.data(), control_points.size()},
        thickness,
        resolution);
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Shape::sdf(std::string_view sdf_json) {
    auto result = diplomat::capi::Shape_sdf({sdf_json.data(), sdf_json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Shape::sdf_bounded(std::string_view sdf_json, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::Shape_sdf_bounded({sdf_json.data(), sdf_json.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Shape> Shape::hollow(uint32_t thickness) const {
    auto result = diplomat::capi::Shape_hollow(this->AsFFI(),
        thickness);
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::union_with(const Shape& other) const {
    auto result = diplomat::capi::Shape_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::intersection_with(const Shape& other) const {
    auto result = diplomat::capi::Shape_intersection_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline std::unique_ptr<Shape> Shape::difference_with(const Shape& other) const {
    auto result = diplomat::capi::Shape_difference_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<Shape>(Shape::FromFFI(result));
}

inline const diplomat::capi::Shape* Shape::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Shape*>(this);
}

inline diplomat::capi::Shape* Shape::AsFFI() {
    return reinterpret_cast<diplomat::capi::Shape*>(this);
}

inline const Shape* Shape::FromFFI(const diplomat::capi::Shape* ptr) {
    return reinterpret_cast<const Shape*>(ptr);
}

inline Shape* Shape::FromFFI(diplomat::capi::Shape* ptr) {
    return reinterpret_cast<Shape*>(ptr);
}

inline void Shape::operator delete(void* ptr) {
    diplomat::capi::Shape_destroy(reinterpret_cast<diplomat::capi::Shape*>(ptr));
}


#endif // Shape_HPP
