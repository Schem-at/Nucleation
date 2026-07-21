#ifndef Shape_D_HPP
#define Shape_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Curve3D; }
class Curve3D;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Shape;
} // namespace capi
} // namespace

/**
 * A solid region of blocks: primitives (sphere, cuboid, …) and boolean
 * combinations thereof. Wraps `ShapeEnum`.
 */
class Shape {
public:

  /**
   * Thicken a sampled 3D curve into a parametric tube with the given radius.
   */
  inline static diplomat::result<std::unique_ptr<Shape>, NucleationError> tube_along(const Curve3D& curve, double radius);

  /**
   * Sphere centered at (`cx`, `cy`, `cz`) (truncated to block coordinates,
   * matching the old `shape_sphere`).
   */
  inline static std::unique_ptr<Shape> sphere(float cx, float cy, float cz, float radius);

  /**
   * Axis-aligned box spanning the two corners (inclusive).
   */
  inline static std::unique_ptr<Shape> cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * A closed 2D polygon extruded between two Y levels (inclusive). The
   * footprint is `polygon_json`, a JSON array of `[x, z]` world-coordinate
   * pairs in order (winding does not matter; the ring closes implicitly);
   * any simple polygon works, concave ones included. This is the shape
   * behind extruded building footprints, lake outlines, and plot fills.
   * Errors with `Parse` on invalid JSON and `InvalidArgument` on non-UTF-8
   * or fewer than three vertices.
   */
  inline static diplomat::result<std::unique_ptr<Shape>, NucleationError> polygon_prism(std::string_view polygon_json, int32_t y_min, int32_t y_max);

  /**
   * Ellipsoid centered at (`cx`, `cy`, `cz`) with per-axis radii.
   */
  inline static std::unique_ptr<Shape> ellipsoid(int32_t cx, int32_t cy, int32_t cz, float rx, float ry, float rz);

  /**
   * Cylinder from base point along an axis vector.
   */
  inline static std::unique_ptr<Shape> cylinder(float bx, float by, float bz, float ax, float ay, float az, float radius, float height);

  /**
   * Cylinder spanning the segment between two points.
   */
  inline static std::unique_ptr<Shape> cylinder_between(float x1, float y1, float z1, float x2, float y2, float z2, float radius);

  /**
   * Cone with apex at (`ax`, `ay`, `az`) opening along direction (`dx`, `dy`, `dz`).
   */
  inline static std::unique_ptr<Shape> cone(float ax, float ay, float az, float dx, float dy, float dz, float radius, float height);

  /**
   * Torus centered at (`cx`, `cy`, `cz`) with the given major/minor radii and
   * axis (`ax`, `ay`, `az`).
   */
  inline static std::unique_ptr<Shape> torus(float cx, float cy, float cz, float major_r, float minor_r, float ax, float ay, float az);

  /**
   * Rectangular pyramid: base center, half-extents, height, up-axis.
   */
  inline static std::unique_ptr<Shape> pyramid(float bx, float by, float bz, float half_w, float half_d, float height, float ax, float ay, float az);

  /**
   * Flat disk: center, radius, plane normal, thickness.
   */
  inline static std::unique_ptr<Shape> disk(float cx, float cy, float cz, float radius, float nx, float ny, float nz, float thickness);

  /**
   * Finite plane patch: origin, two spanning vectors `u`/`v`, extents along
   * each, thickness.
   */
  inline static std::unique_ptr<Shape> plane(float ox, float oy, float oz, float ux, float uy, float uz, float vx, float vy, float vz, float u_ext, float v_ext, float thickness);

  /**
   * Filled triangle between three vertices, thickened by `thickness`.
   */
  inline static std::unique_ptr<Shape> triangle(float ax, float ay, float az, float bx, float by, float bz, float cx, float cy, float cz, float thickness);

  /**
   * Line segment between two points, thickened by `thickness`.
   */
  inline static std::unique_ptr<Shape> line(float x1, float y1, float z1, float x2, float y2, float z2, float thickness);

  /**
   * Bézier curve through `control_points` (flat `[x0, y0, z0, x1, y1, z1, …]`,
   * so the length must be a non-zero multiple of 3), thickened by `thickness`
   * and sampled at `resolution` steps.
   */
  inline static diplomat::result<std::unique_ptr<Shape>, NucleationError> bezier(diplomat::span<const float> control_points, float thickness, uint32_t resolution);

  /**
   * Any SDF tree as a Shape: the same JSON the terrain sampler takes
   * (primitives, smooth booleans, noise — see the SDF guide) becomes
   * fillable with every brush, combinable with other shapes, and
   * usable in masked fills. Normals come from the field gradient, so
   * the shaded brush shades smooth blends smoothly. Errors with
   * `Parse` on invalid JSON and `InvalidArgument` for unbounded trees
   * (use `sdf_bounded`).
   */
  inline static diplomat::result<std::unique_ptr<Shape>, NucleationError> sdf(std::string_view sdf_json);

  /**
   * Like `sdf`, with explicit sampling bounds (inclusive block
   * coordinates) for unbounded trees such as planes.
   */
  inline static diplomat::result<std::unique_ptr<Shape>, NucleationError> sdf_bounded(std::string_view sdf_json, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Hollowed-out copy of this shape with the given wall thickness (clones the
   * input, like the old `shape_hollow`).
   */
  inline std::unique_ptr<Shape> hollow(uint32_t thickness) const;

  /**
   * Boolean union of this shape and `other` (clones both inputs).
   */
  inline std::unique_ptr<Shape> union_with(const Shape& other) const;

  /**
   * Boolean intersection of this shape and `other` (clones both inputs).
   */
  inline std::unique_ptr<Shape> intersection_with(const Shape& other) const;

  /**
   * Boolean difference: this shape minus `other` (clones both inputs).
   */
  inline std::unique_ptr<Shape> difference_with(const Shape& other) const;

    inline const diplomat::capi::Shape* AsFFI() const;
    inline diplomat::capi::Shape* AsFFI();
    inline static const Shape* FromFFI(const diplomat::capi::Shape* ptr);
    inline static Shape* FromFFI(diplomat::capi::Shape* ptr);
    inline static void operator delete(void* ptr);
private:
    Shape() = delete;
    Shape(const Shape&) = delete;
    Shape(Shape&&) noexcept = delete;
    Shape operator=(const Shape&) = delete;
    Shape operator=(Shape&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Shape_D_HPP
