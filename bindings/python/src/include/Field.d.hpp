#ifndef NUCLEATION_Field_D_HPP
#define NUCLEATION_Field_D_HPP

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
namespace capi { struct Field; }
class Field;
namespace capi { struct Shape; }
class Shape;
struct FieldBounds;
struct FieldNormal;
struct FieldRange;
struct FieldSample;
class FieldAxis;
class FieldCellMode;
class FieldDistanceKind;
class FieldNoiseBasis;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Field;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Unified immutable spatial field / SDF DAG node.
 *
 * Primitives, CSG (hard, smooth, chamfer, stepped), transforms, domain
 * warps, procedural noise, arbitrary math, material tags, and sandboxed
 * infix expressions all compose freely and share storage across consumers.
 */
class Field {
public:

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> constant(float value);

  inline static std::unique_ptr<nucleation::Field> coord_x();

  inline static std::unique_ptr<nucleation::Field> coord_y();

  inline static std::unique_ptr<nucleation::Field> coord_z();

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> sphere(float radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> box_shape(float half_x, float half_y, float half_z, float rounding);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> ellipsoid(float radius_x, float radius_y, float radius_z);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> torus(float major_radius, float minor_radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> link(float major_radius, float minor_radius, float half_length);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> solid_angle(float radius, float angle_degrees);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> cut_sphere(float radius, float height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> cut_hollow_sphere(float radius, float height, float thickness);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> capped_cylinder(float radius, float half_height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> infinite_cylinder(float radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> capped_cone(float half_height, float bottom_radius, float top_radius);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> plane(float normal_x, float normal_y, float normal_z, float offset);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> octahedron(float size);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> hex_prism(float radius, float half_height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> super_prism(float half_x, float half_y, float half_z, float exponent);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> box_frame(float half_x, float half_y, float half_z, float thickness);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> infinite_cone(float angle_degrees);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> square_pyramid(float half_base, float height);

  /**
   * Heightmap terrain surface: solid where `y <= base_y + height(x, 0, z)`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> heightmap(float base_y, const nucleation::Field& height);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> value_noise2(float frequency, int32_t seed);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> value_noise3(float frequency, int32_t seed);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> perlin2(float frequency, int32_t seed);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> perlin3(float frequency, int32_t seed);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> fbm2(nucleation::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> fbm3(nucleation::FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> value_fbm3(float frequency, int32_t seed, uint32_t octaves);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> perlin_fbm3(float frequency, int32_t seed, uint32_t octaves);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> ridged_fbm3(float frequency, int32_t seed, uint32_t octaves);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> perlin_fbm2(float frequency, int32_t seed, uint32_t octaves);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> cells(float frequency, int32_t seed, float jitter, nucleation::FieldCellMode mode, float threshold);

  inline std::unique_ptr<nucleation::Field> neg() const;

  inline std::unique_ptr<nucleation::Field> abs() const;

  inline std::unique_ptr<nucleation::Field> sqrt() const;

  inline std::unique_ptr<nucleation::Field> sin() const;

  inline std::unique_ptr<nucleation::Field> cos() const;

  inline std::unique_ptr<nucleation::Field> saturate() const;

  inline std::unique_ptr<nucleation::Field> add(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> sub(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> mul(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> div(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> min_with(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> max_with(const nucleation::Field& other) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> add_const(float value) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> mul_const(float value) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> clamp(float lo, float hi) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> smoothstep(float edge0, float edge1) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> step(float edge) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> remap(float in_lo, float in_hi, float out_lo, float out_hi, bool clamp) const;

  inline std::unique_ptr<nucleation::Field> lerp_with(const nucleation::Field& other, const nucleation::Field& t) const;

  inline static std::unique_ptr<nucleation::Field> select(const nucleation::Field& cond, const nucleation::Field& then_field, const nucleation::Field& else_field);

  inline std::unique_ptr<nucleation::Field> union_with(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> intersection_with(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> subtract(const nucleation::Field& other) const;

  inline std::unique_ptr<nucleation::Field> xor_with(const nucleation::Field& other) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> smooth_union(const nucleation::Field& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> smooth_subtract(const nucleation::Field& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> smooth_intersection(const nucleation::Field& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> chamfer_union(const nucleation::Field& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> chamfer_subtract(const nucleation::Field& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> chamfer_intersection(const nucleation::Field& other, float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> steps_union(const nucleation::Field& other, float radius, float steps) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> rounded(float radius) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> shell(float thickness) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> elongate(float half_x, float half_y, float half_z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> displace(const nucleation::Field& offset, float amplitude) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> displace_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> translate(float x, float y, float z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> rotate(float x_degrees, float y_degrees, float z_degrees) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> scale(float factor) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> scale_xyz(float sx, float sy, float sz) const;

  inline std::unique_ptr<nucleation::Field> mirror(nucleation::FieldAxis axis) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> mirror_plane(float nx, float ny, float nz, float offset) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> repeat_polar(uint32_t count) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> repeat_points(nucleation::diplomat::span<const float> offsets) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> twist(float amount) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> bend(float amount) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> warp(const nucleation::Field& warp_x, const nucleation::Field& warp_y, const nucleation::Field& warp_z, float amplitude) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> warp_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> tag(uint16_t material_id) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> bounded(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) const;

  /**
   * Parse a sandboxed infix expression (supports `x, y, z, p`, math,
   * noise, primitives, `let`, `if/else`, and bounded `for` loops).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> expr(std::string_view source);

  inline float eval_at(float x, float y, float z) const;

  inline nucleation::FieldSample sample_tagged(float x, float y, float z) const;

  inline nucleation::diplomat::result<nucleation::FieldNormal, nucleation::NucleationError> normal(float x, float y, float z, float epsilon) const;

  inline nucleation::diplomat::result<nucleation::FieldBounds, nucleation::NucleationError> bounds() const;

  inline nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError> output_range() const;

  inline nucleation::FieldDistanceKind distance_kind() const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> to_shape() const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> from_json(std::string_view json);

  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_json_write(W& writeable_output) const;

    inline const nucleation::capi::Field* AsFFI() const;
    inline nucleation::capi::Field* AsFFI();
    inline static const nucleation::Field* FromFFI(const nucleation::capi::Field* ptr);
    inline static nucleation::Field* FromFFI(nucleation::capi::Field* ptr);
    inline static void operator delete(void* ptr);
private:
    Field() = delete;
    Field(const nucleation::Field&) = delete;
    Field(nucleation::Field&&) noexcept = delete;
    Field operator=(const nucleation::Field&) = delete;
    Field operator=(nucleation::Field&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Field_D_HPP
