#ifndef Material_HPP
#define Material_HPP

#include "Material.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Brush.hpp"
#include "Field.hpp"
#include "FieldChannel.hpp"
#include "InterpolationSpace.hpp"
#include "NucleationError.hpp"
#include "Palette.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Material_solid_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_solid_result;
    Material_solid_result Material_solid(diplomat::capi::DiplomatStringView block);

    diplomat::capi::Material* Material_skip(void);

    diplomat::capi::Material* Material_color(uint8_t r, uint8_t g, uint8_t b, const diplomat::capi::Palette* palette);

    diplomat::capi::Material* Material_from_brush(const diplomat::capi::Brush* brush);

    typedef struct Material_channel_gradient_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_channel_gradient_result;
    Material_channel_gradient_result Material_channel_gradient(diplomat::capi::FieldChannel channel, diplomat::capi::DiplomatF32View stops, diplomat::capi::DiplomatU8View colors, float lo, float hi, const diplomat::capi::Palette* palette, diplomat::capi::InterpolationSpace space);

    typedef struct Material_field_gradient_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_field_gradient_result;
    Material_field_gradient_result Material_field_gradient(const diplomat::capi::Field* field, diplomat::capi::DiplomatF32View stops, diplomat::capi::DiplomatU8View colors, float lo, float hi, const diplomat::capi::Palette* palette, diplomat::capi::InterpolationSpace space);

    typedef struct Material_channel_ramp_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_channel_ramp_result;
    Material_channel_ramp_result Material_channel_ramp(diplomat::capi::FieldChannel channel, float lo, float hi, diplomat::capi::DiplomatStringView blocks_json, bool dither);

    typedef struct Material_field_ramp_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_field_ramp_result;
    Material_field_ramp_result Material_field_ramp(const diplomat::capi::Field* field, float lo, float hi, diplomat::capi::DiplomatStringView blocks_json, bool dither);

    typedef struct Material_weighted_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_weighted_result;
    Material_weighted_result Material_weighted(diplomat::capi::DiplomatStringView blocks_json, diplomat::capi::DiplomatF32View weights, int32_t seed);

    typedef struct Material_shaded_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_shaded_result;
    Material_shaded_result Material_shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz, float ambient, const diplomat::capi::Palette* palette);

    typedef struct Material_spotlight_result {union {diplomat::capi::Material* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Material_spotlight_result;
    Material_spotlight_result Material_spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b, float ambient, const diplomat::capi::Palette* palette);

    void Material_destroy(Material* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::solid(std::string_view block) {
    auto result = diplomat::capi::Material_solid({block.data(), block.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Material> Material::skip() {
    auto result = diplomat::capi::Material_skip();
    return std::unique_ptr<Material>(Material::FromFFI(result));
}

inline std::unique_ptr<Material> Material::color(uint8_t r, uint8_t g, uint8_t b, const Palette& palette) {
    auto result = diplomat::capi::Material_color(r,
        g,
        b,
        palette.AsFFI());
    return std::unique_ptr<Material>(Material::FromFFI(result));
}

inline std::unique_ptr<Material> Material::from_brush(const Brush& brush) {
    auto result = diplomat::capi::Material_from_brush(brush.AsFFI());
    return std::unique_ptr<Material>(Material::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::channel_gradient(FieldChannel channel, diplomat::span<const float> stops, diplomat::span<const uint8_t> colors, float lo, float hi, const Palette& palette, InterpolationSpace space) {
    auto result = diplomat::capi::Material_channel_gradient(channel.AsFFI(),
        {stops.data(), stops.size()},
        {colors.data(), colors.size()},
        lo,
        hi,
        palette.AsFFI(),
        space.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::field_gradient(const Field& field, diplomat::span<const float> stops, diplomat::span<const uint8_t> colors, float lo, float hi, const Palette& palette, InterpolationSpace space) {
    auto result = diplomat::capi::Material_field_gradient(field.AsFFI(),
        {stops.data(), stops.size()},
        {colors.data(), colors.size()},
        lo,
        hi,
        palette.AsFFI(),
        space.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::channel_ramp(FieldChannel channel, float lo, float hi, std::string_view blocks_json, bool dither) {
    auto result = diplomat::capi::Material_channel_ramp(channel.AsFFI(),
        lo,
        hi,
        {blocks_json.data(), blocks_json.size()},
        dither);
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::field_ramp(const Field& field, float lo, float hi, std::string_view blocks_json, bool dither) {
    auto result = diplomat::capi::Material_field_ramp(field.AsFFI(),
        lo,
        hi,
        {blocks_json.data(), blocks_json.size()},
        dither);
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::weighted(std::string_view blocks_json, diplomat::span<const float> weights, int32_t seed) {
    auto result = diplomat::capi::Material_weighted({blocks_json.data(), blocks_json.size()},
        {weights.data(), weights.size()},
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz, float ambient, const Palette& palette) {
    auto result = diplomat::capi::Material_shaded(r,
        g,
        b,
        lx,
        ly,
        lz,
        ambient,
        palette.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Material>, NucleationError> Material::spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b, float ambient, const Palette& palette) {
    auto result = diplomat::capi::Material_spotlight(px,
        py,
        pz,
        dx,
        dy,
        dz,
        cone_angle_deg,
        r,
        g,
        b,
        ambient,
        palette.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Ok<std::unique_ptr<Material>>(std::unique_ptr<Material>(Material::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Material>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Material* Material::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Material*>(this);
}

inline diplomat::capi::Material* Material::AsFFI() {
    return reinterpret_cast<diplomat::capi::Material*>(this);
}

inline const Material* Material::FromFFI(const diplomat::capi::Material* ptr) {
    return reinterpret_cast<const Material*>(ptr);
}

inline Material* Material::FromFFI(diplomat::capi::Material* ptr) {
    return reinterpret_cast<Material*>(ptr);
}

inline void Material::operator delete(void* ptr) {
    diplomat::capi::Material_destroy(reinterpret_cast<diplomat::capi::Material*>(ptr));
}


#endif // Material_HPP
