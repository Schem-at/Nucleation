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

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> capped_cylinder(float radius, float half_height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> capped_cone(float half_height, float bottom_radius, float top_radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> plane(float normal_x, float normal_y, float normal_z, float offset);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> octahedron(float size);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> hex_prism(float radius, float half_height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> super_prism(float half_x, float half_y, float half_z, float exponent);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> cells(float frequency, int32_t seed, float jitter, nucleation::SdfCellMode mode, float threshold);

  inline std::unique_ptr<nucleation::Sdf> union_with(const nucleation::Sdf& other) const;

  inline std::unique_ptr<nucleation::Sdf> intersection_with(const nucleation::Sdf& other) const;

  inline std::unique_ptr<nucleation::Sdf> subtract(const nucleation::Sdf& other) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> smooth_union(const nucleation::Sdf& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> smooth_subtract(const nucleation::Sdf& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> smooth_intersection(const nucleation::Sdf& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> rounded(float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> shell(float thickness) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> translate(float x, float y, float z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> rotate(float x_degrees, float y_degrees, float z_degrees) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> scale(float factor) const;

  inline std::unique_ptr<nucleation::Sdf> mirror(nucleation::SdfAxis axis) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> displace(float amplitude, float frequency, int32_t seed, uint32_t octaves) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> warp(float amplitude, float frequency, int32_t seed) const;

  inline float eval_at(float x, float y, float z) const;

  inline nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError> normal(float x, float y, float z, float epsilon) const;

  inline nucleation::diplomat::result<nucleation::SdfBounds, nucleation::NucleationError> bounds() const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> to_shape() const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Sdf>, nucleation::NucleationError> from_json_string(std::string_view json);

  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_json_write(W& writeable_output) const;

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
