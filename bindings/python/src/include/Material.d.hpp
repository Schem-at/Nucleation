#ifndef NUCLEATION_Material_D_HPP
#define NUCLEATION_Material_D_HPP

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
namespace capi { struct Brush; }
class Brush;
namespace capi { struct Field; }
class Field;
namespace capi { struct Material; }
class Material;
namespace capi { struct Palette; }
class Palette;
class FieldChannel;
class InterpolationSpace;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Material;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Composable block shader for {@link VoxelPipeline}.
 */
class Material {
public:

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> solid(std::string_view block);

  inline static std::unique_ptr<nucleation::Material> skip();

  inline static std::unique_ptr<nucleation::Material> color(uint8_t r, uint8_t g, uint8_t b, const nucleation::Palette& palette);

  inline static std::unique_ptr<nucleation::Material> from_brush(const nucleation::Brush& brush);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> channel_gradient(nucleation::FieldChannel channel, nucleation::diplomat::span<const float> stops, nucleation::diplomat::span<const uint8_t> colors, float lo, float hi, const nucleation::Palette& palette, nucleation::InterpolationSpace space);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> field_gradient(const nucleation::Field& field, nucleation::diplomat::span<const float> stops, nucleation::diplomat::span<const uint8_t> colors, float lo, float hi, const nucleation::Palette& palette, nucleation::InterpolationSpace space);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> channel_ramp(nucleation::FieldChannel channel, float lo, float hi, std::string_view blocks_json, bool dither);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> field_ramp(const nucleation::Field& field, float lo, float hi, std::string_view blocks_json, bool dither);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> weighted(std::string_view blocks_json, nucleation::diplomat::span<const float> weights, int32_t seed);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz, float ambient, const nucleation::Palette& palette);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b, float ambient, const nucleation::Palette& palette);

    inline const nucleation::capi::Material* AsFFI() const;
    inline nucleation::capi::Material* AsFFI();
    inline static const nucleation::Material* FromFFI(const nucleation::capi::Material* ptr);
    inline static nucleation::Material* FromFFI(nucleation::capi::Material* ptr);
    inline static void operator delete(void* ptr);
private:
    Material() = delete;
    Material(const nucleation::Material&) = delete;
    Material(nucleation::Material&&) noexcept = delete;
    Material operator=(const nucleation::Material&) = delete;
    Material operator=(nucleation::Material&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Material_D_HPP
