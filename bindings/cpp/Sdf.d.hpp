#ifndef Sdf_D_HPP
#define Sdf_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct FieldProgram; }
class FieldProgram;
namespace diplomat::capi { struct Schematic; }
class Schematic;
namespace diplomat::capi { struct Shape; }
class Shape;
struct SdfBounds;
struct SdfNormal;
class NucleationError;
class SdfAxis;
class SdfCellMode;




namespace diplomat {
namespace capi {
    struct Sdf;
} // namespace capi
} // namespace

/**
 * An immutable, composable signed-distance-field expression graph.
 *
 * Primitive constructors and every combinator return a new graph, so values
 * can be shared safely between Flow nodes and across Kotlin/Java, JavaScript,
 * and Python bindings. JSON is retained only for explicit import/export and
 * the legacy sampling helpers.
 */
class Sdf {
public:

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> sphere(float radius);

  /**
   * Axis-aligned rounded box, centered at the origin.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> box_shape(float half_x, float half_y, float half_z, float rounding);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> ellipsoid(float radius_x, float radius_y, float radius_z);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> torus(float major_radius, float minor_radius);

  /**
   * Torus ring cut down to an arc. `cap_angle_degrees` is the half-aperture
   * in `(0, 180]`, measured from +X and mirrored across X; `180` is a full
   * torus.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

  /**
   * Chain-link shape: a torus stretched along Z by `half_length` and
   * capped by two half-tori. `half_length: 0` is a plain torus.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> link(float major_radius, float minor_radius, float half_length);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

  /**
   * Convex hull of two spheres: a capsule with a linear taper between
   * `r1` (at `a`) and `r2` (at `b`) instead of one constant radius.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

  /**
   * Sphere of `radius` intersected with an infinite cone of
   * half-aperture `angle_degrees` (in `(0, 180)`) from the +Y axis,
   * apex at the origin.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> solid_angle(float radius, float angle_degrees);

  /**
   * Sphere cut by the plane `y = height`, keeping the cap above it
   * (a dome). `height` must be strictly between `-radius` and
   * `radius`.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> cut_sphere(float radius, float height);

  /**
   * Open (hollow) shell of `cut_sphere`'s dome: just the spherical
   * cap surface, offset by `thickness`, with no flat floor.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> cut_hollow_sphere(float radius, float height, float thickness);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> capped_cylinder(float radius, float half_height);

  /**
   * Exact Y-axis cylinder with infinite extent. Sampling requires explicit bounds.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> infinite_cylinder(float radius);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> capped_cone(float half_height, float bottom_radius, float top_radius);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> plane(float normal_x, float normal_y, float normal_z, float offset);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> octahedron(float size);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> hex_prism(float radius, float half_height);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> super_prism(float half_x, float half_y, float half_z, float exponent);

  /**
   * Hollow wireframe box: only the 12 edge beams are solid.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> box_frame(float half_x, float half_y, float half_z, float thickness);

  /**
   * Exact but unbounded Y-axis infinite cone: apex at the origin,
   * single nappe opening along +Y, half-aperture `angle_degrees`
   * strictly in `(0, 90)`.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> infinite_cone(float angle_degrees);

  /**
   * Square-base pyramid, vertically centered: base (half-extent
   * `half_base`) at `y = -height/2`, apex at `y = height/2`. `height`
   * must be at least the smallest positive normal `f32`.
   */
  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> square_pyramid(float half_base, float height);

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> cells(float frequency, int32_t seed, float jitter, SdfCellMode mode, float threshold);

  inline std::unique_ptr<Sdf> union_with(const Sdf& other) const;

  inline std::unique_ptr<Sdf> intersection_with(const Sdf& other) const;

  inline std::unique_ptr<Sdf> subtract(const Sdf& other) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> smooth_union(const Sdf& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> smooth_subtract(const Sdf& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> smooth_intersection(const Sdf& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> rounded(float radius) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> shell(float thickness) const;

  /**
   * Symmetric difference (XOR): solid where exactly one of `self`/
   * `other` is solid.
   */
  inline std::unique_ptr<Sdf> xor_with(const Sdf& other) const;

  /**
   * Stretches this graph with IQ's origin-centered `opElongate` fold.
   * Exactness requires a suitable origin-centered, reflection-symmetric
   * child; off-center/asymmetric children are mirrored and produce only
   * an estimate. Half-lengths must be finite and non-negative, with at
   * least one strictly positive.
   */
  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> elongate(float half_x, float half_y, float half_z) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> translate(float x, float y, float z) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> rotate(float x_degrees, float y_degrees, float z_degrees) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> scale(float factor) const;

  inline std::unique_ptr<Sdf> mirror(SdfAxis axis) const;

  /**
   * Twists this graph about the Y axis by `amount` radians per unit
   * Y (IQ's `opTwist`). *Distorted*: not guaranteed exact even when
   * `self` is.
   */
  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> twist(float amount) const;

  /**
   * Cheaply bends this graph by `amount` radians per unit X (IQ's
   * `opCheapBend`). *Distorted*: not guaranteed exact even when
   * `self` is.
   */
  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> bend(float amount) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> displace(float amplitude, float frequency, int32_t seed, uint32_t octaves) const;

  inline diplomat::result<std::unique_ptr<Sdf>, NucleationError> warp(float amplitude, float frequency, int32_t seed) const;

  inline float eval_at(float x, float y, float z) const;

  inline diplomat::result<SdfNormal, NucleationError> normal(float x, float y, float z, float epsilon) const;

  inline diplomat::result<SdfBounds, NucleationError> bounds() const;

  inline diplomat::result<std::unique_ptr<Shape>, NucleationError> to_shape() const;

  inline diplomat::result<std::unique_ptr<Shape>, NucleationError> to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  inline static diplomat::result<std::unique_ptr<Sdf>, NucleationError> from_json_string(std::string_view json);

  inline diplomat::result<std::string, NucleationError> to_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_json_write(W& writeable_output) const;

  /**
   * Wrap a validated {@link FieldProgram} as an `Sdf` graph (cloning it,
   * with its own explicit bounds and distance-kind metadata), so it
   * composes with every other combinator.
   */
  inline static std::unique_ptr<Sdf> from_program(const FieldProgram& program);

  /**
   * Legacy JSON-first terrain helper. Prefer typed constructors and
   * `to_shape()` with `BuildingTool.fill()` for new code.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> schematic_from_sdf_auto(std::string_view sdf_json, std::string_view rules_json);

  /**
   * Legacy JSON-first terrain helper with optional explicit bounds.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Legacy JSON-first evaluator. Prefer `Sdf.from_json_string(...).eval_at(...)`.
   */
  inline static diplomat::result<float, NucleationError> eval(std::string_view sdf_json, float x, float y, float z);

    inline const diplomat::capi::Sdf* AsFFI() const;
    inline diplomat::capi::Sdf* AsFFI();
    inline static const Sdf* FromFFI(const diplomat::capi::Sdf* ptr);
    inline static Sdf* FromFFI(diplomat::capi::Sdf* ptr);
    inline static void operator delete(void* ptr);
private:
    Sdf() = delete;
    Sdf(const Sdf&) = delete;
    Sdf(Sdf&&) noexcept = delete;
    Sdf operator=(const Sdf&) = delete;
    Sdf operator=(Sdf&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Sdf_D_HPP
