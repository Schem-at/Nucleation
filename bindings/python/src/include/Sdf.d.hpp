#ifndef NUCLEATION_Sdf_D_HPP
#define NUCLEATION_Sdf_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct Field3; }
class Field3;
namespace capi { struct FieldProgram; }
class FieldProgram;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct Sdf; }
class Sdf;
namespace capi { struct Shape; }
class Shape;
struct SdfBounds;
struct SdfNormal;
class NucleationError;
class SdfAxis;
class SdfCellMode;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Sdf;
} // namespace capi
} // namespace

namespace nucleation {
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

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> sphere(float radius);

  /**
   * Axis-aligned rounded box, centered at the origin.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> box_shape(float half_x, float half_y, float half_z, float rounding);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> ellipsoid(float radius_x, float radius_y, float radius_z);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> torus(float major_radius, float minor_radius);

  /**
   * Torus ring cut down to an arc. `cap_angle_degrees` is the half-aperture
   * in `(0, 180]`, measured from +X and mirrored across X; `180` is a full
   * torus.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

  /**
   * Chain-link shape: a torus stretched along Z by `half_length` and
   * capped by two half-tori. `half_length: 0` is a plain torus.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> link(float major_radius, float minor_radius, float half_length);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

  /**
   * Convex hull of two spheres: a capsule with a linear taper between
   * `r1` (at `a`) and `r2` (at `b`) instead of one constant radius.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

  /**
   * Sphere of `radius` intersected with an infinite cone of
   * half-aperture `angle_degrees` (in `(0, 180)`) from the +Y axis,
   * apex at the origin.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> solid_angle(float radius, float angle_degrees);

  /**
   * Sphere cut by the plane `y = height`, keeping the cap above it
   * (a dome). `height` must be strictly between `-radius` and
   * `radius`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> cut_sphere(float radius, float height);

  /**
   * Open (hollow) shell of `cut_sphere`'s dome: just the spherical
   * cap surface, offset by `thickness`, with no flat floor.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> cut_hollow_sphere(float radius, float height, float thickness);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> capped_cylinder(float radius, float half_height);

  /**
   * Exact Y-axis cylinder with infinite extent. Sampling requires explicit bounds.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> infinite_cylinder(float radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> capped_cone(float half_height, float bottom_radius, float top_radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> plane(float normal_x, float normal_y, float normal_z, float offset);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> octahedron(float size);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> hex_prism(float radius, float half_height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> super_prism(float half_x, float half_y, float half_z, float exponent);

  /**
   * Hollow wireframe box: only the 12 edge beams are solid.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> box_frame(float half_x, float half_y, float half_z, float thickness);

  /**
   * Exact but unbounded Y-axis infinite cone: apex at the origin,
   * single nappe opening along +Y, half-aperture `angle_degrees`
   * strictly in `(0, 90)`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> infinite_cone(float angle_degrees);

  /**
   * Square-base pyramid, vertically centered: base (half-extent
   * `half_base`) at `y = -height/2`, apex at `y = height/2`. `height`
   * must be at least the smallest positive normal `f32`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> square_pyramid(float half_base, float height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> cells(float frequency, int32_t seed, float jitter, nucleation::SdfCellMode mode, float threshold);

  inline std::unique_ptr<nucleation::Sdf> union_with(const nucleation::Sdf& other) const;

  inline std::unique_ptr<nucleation::Sdf> intersection_with(const nucleation::Sdf& other) const;

  inline std::unique_ptr<nucleation::Sdf> subtract(const nucleation::Sdf& other) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> smooth_union(const nucleation::Sdf& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> smooth_subtract(const nucleation::Sdf& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> smooth_intersection(const nucleation::Sdf& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> rounded(float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> shell(float thickness) const;

  /**
   * Symmetric difference (XOR): solid where exactly one of `self`/
   * `other` is solid.
   */
  inline std::unique_ptr<nucleation::Sdf> xor_with(const nucleation::Sdf& other) const;

  /**
   * Stretches this graph with IQ's origin-centered `opElongate` fold.
   * Exactness requires a suitable origin-centered, reflection-symmetric
   * child; off-center/asymmetric children are mirrored and produce only
   * an estimate. Half-lengths must be finite and non-negative, with at
   * least one strictly positive.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> elongate(float half_x, float half_y, float half_z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> translate(float x, float y, float z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> rotate(float x_degrees, float y_degrees, float z_degrees) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> scale(float factor) const;

  inline std::unique_ptr<nucleation::Sdf> mirror(nucleation::SdfAxis axis) const;

  /**
   * Twists this graph about the Y axis by `amount` radians per unit
   * Y (IQ's `opTwist`). *Distorted*: not guaranteed exact even when
   * `self` is.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> twist(float amount) const;

  /**
   * Cheaply bends this graph by `amount` radians per unit X (IQ's
   * `opCheapBend`). *Distorted*: not guaranteed exact even when
   * `self` is.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> bend(float amount) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const;

  /**
   * Finite rigid instances of this graph at arbitrary XYZ offsets.
   * `offsets` is flat `[x0, y0, z0, x1, y1, z1, ...]` and may contain
   * at most 4096 points.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> repeat_points(nucleation::diplomat::span<const float> offsets) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> displace(float amplitude, float frequency, int32_t seed, uint32_t octaves) const;

  /**
   * Offset this surface by a reusable scalar field. The resulting zero
   * set is generally an approximate field, not an exact distance field.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> offset_by_field(const nucleation::Field3& field, float amplitude) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> warp(float amplitude, float frequency, int32_t seed) const;

  inline float eval_at(float x, float y, float z) const;

  inline nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError> normal(float x, float y, float z, float epsilon) const;

  /**
   * Conservative finite bounds, or `NotFound` for an unbounded graph
   * (a bare `plane` or `infinite_cylinder` has no finite extent).
   */
  inline nucleation::diplomat::result<nucleation::SdfBounds, nucleation::NucleationError> bounds() const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> to_shape() const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> from_json_string(std::string_view json);

  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_json_write(W& writeable_output) const;

  /**
   * Wrap a validated {@link FieldProgram} as an `Sdf` graph (cloning it,
   * with its own explicit bounds and distance-kind metadata), so it
   * composes with every other combinator.
   */
  inline static std::unique_ptr<nucleation::Sdf> from_program(const nucleation::FieldProgram& program);

  /**
   * Legacy JSON-first terrain helper. Prefer typed constructors and
   * `to_shape()` with `BuildingTool.fill()` for new code.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> schematic_from_sdf_auto(std::string_view sdf_json, std::string_view rules_json);

  /**
   * Legacy JSON-first terrain helper with optional explicit bounds.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Legacy JSON-first evaluator. Prefer `Sdf.from_json_string(...).eval_at(...)`.
   */
  inline static nucleation::diplomat::result<float, nucleation::NucleationError> eval(std::string_view sdf_json, float x, float y, float z);

    inline const nucleation::capi::Sdf* AsFFI() const;
    inline nucleation::capi::Sdf* AsFFI();
    inline static const nucleation::Sdf* FromFFI(const nucleation::capi::Sdf* ptr);
    inline static nucleation::Sdf* FromFFI(nucleation::capi::Sdf* ptr);
    inline static void operator delete(void* ptr);
private:
    Sdf() = delete;
    Sdf(const nucleation::Sdf&) = delete;
    Sdf(nucleation::Sdf&&) noexcept = delete;
    Sdf operator=(const nucleation::Sdf&) = delete;
    Sdf operator=(nucleation::Sdf&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Sdf_D_HPP
