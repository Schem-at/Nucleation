#ifndef Material_D_HPP
#define Material_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Brush; }
class Brush;
namespace diplomat::capi { struct Field; }
class Field;
namespace diplomat::capi { struct Palette; }
class Palette;
class FieldChannel;
class InterpolationSpace;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Material;
} // namespace capi
} // namespace

/**
 * Composable block shader for {@link VoxelPipeline}.
 */
class Material {
public:

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> solid(std::string_view block);

  inline static std::unique_ptr<Material> skip();

  inline static std::unique_ptr<Material> color(uint8_t r, uint8_t g, uint8_t b, const Palette& palette);

  inline static std::unique_ptr<Material> from_brush(const Brush& brush);

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> channel_gradient(FieldChannel channel, diplomat::span<const float> stops, diplomat::span<const uint8_t> colors, float lo, float hi, const Palette& palette, InterpolationSpace space);

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> field_gradient(const Field& field, diplomat::span<const float> stops, diplomat::span<const uint8_t> colors, float lo, float hi, const Palette& palette, InterpolationSpace space);

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> channel_ramp(FieldChannel channel, float lo, float hi, std::string_view blocks_json, bool dither);

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> field_ramp(const Field& field, float lo, float hi, std::string_view blocks_json, bool dither);

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> weighted(std::string_view blocks_json, diplomat::span<const float> weights, int32_t seed);

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz, float ambient, const Palette& palette);

  inline static diplomat::result<std::unique_ptr<Material>, NucleationError> spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b, float ambient, const Palette& palette);

    inline const diplomat::capi::Material* AsFFI() const;
    inline diplomat::capi::Material* AsFFI();
    inline static const Material* FromFFI(const diplomat::capi::Material* ptr);
    inline static Material* FromFFI(diplomat::capi::Material* ptr);
    inline static void operator delete(void* ptr);
private:
    Material() = delete;
    Material(const Material&) = delete;
    Material(Material&&) noexcept = delete;
    Material operator=(const Material&) = delete;
    Material operator=(Material&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Material_D_HPP
