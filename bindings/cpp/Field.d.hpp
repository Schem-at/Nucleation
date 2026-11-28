#ifndef Field_D_HPP
#define Field_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
namespace diplomat::capi { struct Shape; }
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




namespace diplomat {
namespace capi {
    struct Field;
} // namespace capi
} // namespace

/**
 * Unified immutable spatial field / SDF DAG node.
 *
 * Primitives, CSG (hard, smooth, chamfer, stepped), transforms, domain
 * warps, procedural noise, arbitrary math, material tags, and sandboxed
 * infix expressions all compose freely and share storage across consumers.
 */
class Field {
public:

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> constant(float value);

  inline static std::unique_ptr<Field> coord_x();

  inline static std::unique_ptr<Field> coord_y();

  inline static std::unique_ptr<Field> coord_z();

  /**
   * Convert any existing Minecraft schematic into a smooth, composable
   * 3D signed distance field via exact 3D Euclidean Distance Transform.
   */
  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> from_schematic(const Schematic& schematic, uint32_t pad);

  /**
   * Convert binary glTF (`.glb`) bytes into a composable 3D signed
   * distance field fitted to `max_dim` blocks along its longest axis.
   */
  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> from_glb_bytes(diplomat::span<const uint8_t> bytes, uint32_t max_dim, uint32_t pad);

  /**
   * Convert a Wavefront `.obj` string into a composable 3D signed
   * distance field fitted to `max_dim` blocks along its longest axis.
   */
  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> from_obj_str(std::string_view obj_text, uint32_t max_dim, uint32_t pad);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> sphere(float radius);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> box_shape(float half_x, float half_y, float half_z, float rounding);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> ellipsoid(float radius_x, float radius_y, float radius_z);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> torus(float major_radius, float minor_radius);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> capped_torus(float major_radius, float minor_radius, float cap_angle_degrees);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> link(float major_radius, float minor_radius, float half_length);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> capsule(float ax, float ay, float az, float bx, float by, float bz, float radius);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> round_cone(float ax, float ay, float az, float bx, float by, float bz, float r1, float r2);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> solid_angle(float radius, float angle_degrees);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> cut_sphere(float radius, float height);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> cut_hollow_sphere(float radius, float height, float thickness);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> capped_cylinder(float radius, float half_height);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> infinite_cylinder(float radius);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> capped_cone(float half_height, float bottom_radius, float top_radius);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> plane(float normal_x, float normal_y, float normal_z, float offset);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> octahedron(float size);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> hex_prism(float radius, float half_height);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> super_prism(float half_x, float half_y, float half_z, float exponent);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> box_frame(float half_x, float half_y, float half_z, float thickness);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> infinite_cone(float angle_degrees);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> square_pyramid(float half_base, float height);

  /**
   * Heightmap terrain surface: solid where `y <= base_y + height(x, 0, z)`.
   */
  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> heightmap(float base_y, const Field& height);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> value_noise2(float frequency, int32_t seed);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> value_noise3(float frequency, int32_t seed);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> perlin2(float frequency, int32_t seed);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> perlin3(float frequency, int32_t seed);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> fbm2(FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> fbm3(FieldNoiseBasis basis, float frequency, int32_t seed, uint32_t octaves, float lacunarity, float gain, bool ridged);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> value_fbm3(float frequency, int32_t seed, uint32_t octaves);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> perlin_fbm3(float frequency, int32_t seed, uint32_t octaves);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> ridged_fbm3(float frequency, int32_t seed, uint32_t octaves);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> perlin_fbm2(float frequency, int32_t seed, uint32_t octaves);

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> cells(float frequency, int32_t seed, float jitter, FieldCellMode mode, float threshold);

  inline std::unique_ptr<Field> neg() const;

  inline std::unique_ptr<Field> abs() const;

  inline std::unique_ptr<Field> sqrt() const;

  inline std::unique_ptr<Field> sin() const;

  inline std::unique_ptr<Field> cos() const;

  inline std::unique_ptr<Field> saturate() const;

  inline std::unique_ptr<Field> add(const Field& other) const;

  inline std::unique_ptr<Field> sub(const Field& other) const;

  inline std::unique_ptr<Field> mul(const Field& other) const;

  inline std::unique_ptr<Field> div(const Field& other) const;

  inline std::unique_ptr<Field> min_with(const Field& other) const;

  inline std::unique_ptr<Field> max_with(const Field& other) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> add_const(float value) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> mul_const(float value) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> clamp(float lo, float hi) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> smoothstep(float edge0, float edge1) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> step(float edge) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> remap(float in_lo, float in_hi, float out_lo, float out_hi, bool clamp) const;

  inline std::unique_ptr<Field> lerp_with(const Field& other, const Field& t) const;

  inline static std::unique_ptr<Field> select(const Field& cond, const Field& then_field, const Field& else_field);

  inline std::unique_ptr<Field> union_with(const Field& other) const;

  inline std::unique_ptr<Field> intersection_with(const Field& other) const;

  inline std::unique_ptr<Field> subtract(const Field& other) const;

  inline std::unique_ptr<Field> xor_with(const Field& other) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> smooth_union(const Field& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> smooth_subtract(const Field& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> smooth_intersection(const Field& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> chamfer_union(const Field& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> chamfer_subtract(const Field& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> chamfer_intersection(const Field& other, float radius) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> steps_union(const Field& other, float radius, float steps) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> rounded(float radius) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> shell(float thickness) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> elongate(float half_x, float half_y, float half_z) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> displace(const Field& offset, float amplitude) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> displace_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> translate(float x, float y, float z) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> rotate(float x_degrees, float y_degrees, float z_degrees) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> scale(float factor) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> scale_xyz(float sx, float sy, float sz) const;

  inline std::unique_ptr<Field> mirror(FieldAxis axis) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> mirror_plane(float nx, float ny, float nz, float offset) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> repeat_infinite(float spacing_x, float spacing_y, float spacing_z) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> repeat_counted(float spacing_x, float spacing_y, float spacing_z, uint32_t count_x, uint32_t count_y, uint32_t count_z) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> repeat_polar(uint32_t count) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> repeat_points(diplomat::span<const float> offsets) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> twist(float amount) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> bend(float amount) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> warp(const Field& warp_x, const Field& warp_y, const Field& warp_z, float amplitude) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> warp_noise(float amplitude, float frequency, int32_t seed, uint32_t octaves) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> tag(uint16_t material_id) const;

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> bounded(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) const;

  /**
   * Parse a sandboxed infix expression (supports `x, y, z, p`, math,
   * noise, primitives, `let`, `if/else`, and bounded `for` loops).
   */
  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> expr(std::string_view source);

  inline float eval_at(float x, float y, float z) const;

  inline FieldSample sample_tagged(float x, float y, float z) const;

  inline diplomat::result<FieldNormal, NucleationError> normal(float x, float y, float z, float epsilon) const;

  inline diplomat::result<FieldBounds, NucleationError> bounds() const;

  inline diplomat::result<FieldRange, NucleationError> output_range() const;

  inline FieldDistanceKind distance_kind() const;

  inline diplomat::result<std::unique_ptr<Shape>, NucleationError> to_shape() const;

  inline diplomat::result<std::unique_ptr<Shape>, NucleationError> to_shape_bounded(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  inline static diplomat::result<std::unique_ptr<Field>, NucleationError> from_json(std::string_view json);

  inline diplomat::result<std::string, NucleationError> to_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_json_write(W& writeable_output) const;

    inline const diplomat::capi::Field* AsFFI() const;
    inline diplomat::capi::Field* AsFFI();
    inline static const Field* FromFFI(const diplomat::capi::Field* ptr);
    inline static Field* FromFFI(diplomat::capi::Field* ptr);
    inline static void operator delete(void* ptr);
private:
    Field() = delete;
    Field(const Field&) = delete;
    Field(Field&&) noexcept = delete;
    Field operator=(const Field&) = delete;
    Field operator=(Field&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Field_D_HPP
