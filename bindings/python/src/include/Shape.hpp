#ifndef NUCLEATION_Shape_HPP
#define NUCLEATION_Shape_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::Shape* Shape_sphere(float cx, float cy, float cz, float radius);

    nucleation::capi::Shape* Shape_cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    nucleation::capi::Shape* Shape_ellipsoid(int32_t cx, int32_t cy, int32_t cz, float rx, float ry, float rz);

    nucleation::capi::Shape* Shape_cylinder(float bx, float by, float bz, float ax, float ay, float az, float radius, float height);

    nucleation::capi::Shape* Shape_cylinder_between(float x1, float y1, float z1, float x2, float y2, float z2, float radius);

    nucleation::capi::Shape* Shape_cone(float ax, float ay, float az, float dx, float dy, float dz, float radius, float height);

    nucleation::capi::Shape* Shape_torus(float cx, float cy, float cz, float major_r, float minor_r, float ax, float ay, float az);

    nucleation::capi::Shape* Shape_pyramid(float bx, float by, float bz, float half_w, float half_d, float height, float ax, float ay, float az);

    nucleation::capi::Shape* Shape_disk(float cx, float cy, float cz, float radius, float nx, float ny, float nz, float thickness);

    nucleation::capi::Shape* Shape_plane(float ox, float oy, float oz, float ux, float uy, float uz, float vx, float vy, float vz, float u_ext, float v_ext, float thickness);

    nucleation::capi::Shape* Shape_triangle(float ax, float ay, float az, float bx, float by, float bz, float cx, float cy, float cz, float thickness);

    nucleation::capi::Shape* Shape_line(float x1, float y1, float z1, float x2, float y2, float z2, float thickness);

    typedef struct Shape_bezier_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Shape_bezier_result;
    Shape_bezier_result Shape_bezier(nucleation::diplomat::capi::DiplomatF32View control_points, float thickness, uint32_t resolution);

    typedef struct Shape_sdf_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Shape_sdf_result;
    Shape_sdf_result Shape_sdf(nucleation::diplomat::capi::DiplomatStringView sdf_json);

    typedef struct Shape_sdf_bounded_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Shape_sdf_bounded_result;
    Shape_sdf_bounded_result Shape_sdf_bounded(nucleation::diplomat::capi::DiplomatStringView sdf_json, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    nucleation::capi::Shape* Shape_hollow(const nucleation::capi::Shape* self, uint32_t thickness);

    nucleation::capi::Shape* Shape_union_with(const nucleation::capi::Shape* self, const nucleation::capi::Shape* other);

    nucleation::capi::Shape* Shape_intersection_with(const nucleation::capi::Shape* self, const nucleation::capi::Shape* other);

    nucleation::capi::Shape* Shape_difference_with(const nucleation::capi::Shape* self, const nucleation::capi::Shape* other);

    void Shape_destroy(Shape* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::sphere(float cx, float cy, float cz, float radius) {
    auto result = nucleation::capi::Shape_sphere(cx,
        cy,
        cz,
        radius);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::Shape_cuboid(min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::ellipsoid(int32_t cx, int32_t cy, int32_t cz, float rx, float ry, float rz) {
    auto result = nucleation::capi::Shape_ellipsoid(cx,
        cy,
        cz,
        rx,
        ry,
        rz);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::cylinder(float bx, float by, float bz, float ax, float ay, float az, float radius, float height) {
    auto result = nucleation::capi::Shape_cylinder(bx,
        by,
        bz,
        ax,
        ay,
        az,
        radius,
        height);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::cylinder_between(float x1, float y1, float z1, float x2, float y2, float z2, float radius) {
    auto result = nucleation::capi::Shape_cylinder_between(x1,
        y1,
        z1,
        x2,
        y2,
        z2,
        radius);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::cone(float ax, float ay, float az, float dx, float dy, float dz, float radius, float height) {
    auto result = nucleation::capi::Shape_cone(ax,
        ay,
        az,
        dx,
        dy,
        dz,
        radius,
        height);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::torus(float cx, float cy, float cz, float major_r, float minor_r, float ax, float ay, float az) {
    auto result = nucleation::capi::Shape_torus(cx,
        cy,
        cz,
        major_r,
        minor_r,
        ax,
        ay,
        az);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::pyramid(float bx, float by, float bz, float half_w, float half_d, float height, float ax, float ay, float az) {
    auto result = nucleation::capi::Shape_pyramid(bx,
        by,
        bz,
        half_w,
        half_d,
        height,
        ax,
        ay,
        az);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::disk(float cx, float cy, float cz, float radius, float nx, float ny, float nz, float thickness) {
    auto result = nucleation::capi::Shape_disk(cx,
        cy,
        cz,
        radius,
        nx,
        ny,
        nz,
        thickness);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::plane(float ox, float oy, float oz, float ux, float uy, float uz, float vx, float vy, float vz, float u_ext, float v_ext, float thickness) {
    auto result = nucleation::capi::Shape_plane(ox,
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
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::triangle(float ax, float ay, float az, float bx, float by, float bz, float cx, float cy, float cz, float thickness) {
    auto result = nucleation::capi::Shape_triangle(ax,
        ay,
        az,
        bx,
        by,
        bz,
        cx,
        cy,
        cz,
        thickness);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::line(float x1, float y1, float z1, float x2, float y2, float z2, float thickness) {
    auto result = nucleation::capi::Shape_line(x1,
        y1,
        z1,
        x2,
        y2,
        z2,
        thickness);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Shape::bezier(nucleation::diplomat::span<const float> control_points, float thickness, uint32_t resolution) {
    auto result = nucleation::capi::Shape_bezier({control_points.data(), control_points.size()},
        thickness,
        resolution);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Shape::sdf(std::string_view sdf_json) {
    auto result = nucleation::capi::Shape_sdf({sdf_json.data(), sdf_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Shape::sdf_bounded(std::string_view sdf_json, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::Shape_sdf_bounded({sdf_json.data(), sdf_json.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::hollow(uint32_t thickness) const {
    auto result = nucleation::capi::Shape_hollow(this->AsFFI(),
        thickness);
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::union_with(const nucleation::Shape& other) const {
    auto result = nucleation::capi::Shape_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::intersection_with(const nucleation::Shape& other) const {
    auto result = nucleation::capi::Shape_intersection_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline std::unique_ptr<nucleation::Shape> nucleation::Shape::difference_with(const nucleation::Shape& other) const {
    auto result = nucleation::capi::Shape_difference_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result));
}

inline const nucleation::capi::Shape* nucleation::Shape::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Shape*>(this);
}

inline nucleation::capi::Shape* nucleation::Shape::AsFFI() {
    return reinterpret_cast<nucleation::capi::Shape*>(this);
}

inline const nucleation::Shape* nucleation::Shape::FromFFI(const nucleation::capi::Shape* ptr) {
    return reinterpret_cast<const nucleation::Shape*>(ptr);
}

inline nucleation::Shape* nucleation::Shape::FromFFI(nucleation::capi::Shape* ptr) {
    return reinterpret_cast<nucleation::Shape*>(ptr);
}

inline void nucleation::Shape::operator delete(void* ptr) {
    nucleation::capi::Shape_destroy(reinterpret_cast<nucleation::capi::Shape*>(ptr));
}


#endif // NUCLEATION_Shape_HPP
