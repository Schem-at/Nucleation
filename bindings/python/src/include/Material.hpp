#ifndef NUCLEATION_Material_HPP
#define NUCLEATION_Material_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Material_solid_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_solid_result;
    Material_solid_result Material_solid(nucleation::diplomat::capi::DiplomatStringView block);

    nucleation::capi::Material* Material_skip(void);

    nucleation::capi::Material* Material_color(uint8_t r, uint8_t g, uint8_t b, const nucleation::capi::Palette* palette);

    nucleation::capi::Material* Material_from_brush(const nucleation::capi::Brush* brush);

    typedef struct Material_channel_gradient_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_channel_gradient_result;
    Material_channel_gradient_result Material_channel_gradient(nucleation::capi::FieldChannel channel, nucleation::diplomat::capi::DiplomatF32View stops, nucleation::diplomat::capi::DiplomatU8View colors, float lo, float hi, const nucleation::capi::Palette* palette, nucleation::capi::InterpolationSpace space);

    typedef struct Material_field_gradient_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_field_gradient_result;
    Material_field_gradient_result Material_field_gradient(const nucleation::capi::Field* field, nucleation::diplomat::capi::DiplomatF32View stops, nucleation::diplomat::capi::DiplomatU8View colors, float lo, float hi, const nucleation::capi::Palette* palette, nucleation::capi::InterpolationSpace space);

    typedef struct Material_channel_ramp_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_channel_ramp_result;
    Material_channel_ramp_result Material_channel_ramp(nucleation::capi::FieldChannel channel, float lo, float hi, nucleation::diplomat::capi::DiplomatStringView blocks_json, bool dither);

    typedef struct Material_field_ramp_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_field_ramp_result;
    Material_field_ramp_result Material_field_ramp(const nucleation::capi::Field* field, float lo, float hi, nucleation::diplomat::capi::DiplomatStringView blocks_json, bool dither);

    typedef struct Material_weighted_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_weighted_result;
    Material_weighted_result Material_weighted(nucleation::diplomat::capi::DiplomatStringView blocks_json, nucleation::diplomat::capi::DiplomatF32View weights, int32_t seed);

    typedef struct Material_shaded_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_shaded_result;
    Material_shaded_result Material_shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz, float ambient, const nucleation::capi::Palette* palette);

    typedef struct Material_spotlight_result {union {nucleation::capi::Material* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Material_spotlight_result;
    Material_spotlight_result Material_spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b, float ambient, const nucleation::capi::Palette* palette);

    void Material_destroy(Material* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::solid(std::string_view block) {
    auto result = nucleation::capi::Material_solid({block.data(), block.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::Material> nucleation::Material::skip() {
    auto result = nucleation::capi::Material_skip();
    return std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result));
}

inline std::unique_ptr<nucleation::Material> nucleation::Material::color(uint8_t r, uint8_t g, uint8_t b, const nucleation::Palette& palette) {
    auto result = nucleation::capi::Material_color(r,
        g,
        b,
        palette.AsFFI());
    return std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result));
}

inline std::unique_ptr<nucleation::Material> nucleation::Material::from_brush(const nucleation::Brush& brush) {
    auto result = nucleation::capi::Material_from_brush(brush.AsFFI());
    return std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::channel_gradient(nucleation::FieldChannel channel, nucleation::diplomat::span<const float> stops, nucleation::diplomat::span<const uint8_t> colors, float lo, float hi, const nucleation::Palette& palette, nucleation::InterpolationSpace space) {
    auto result = nucleation::capi::Material_channel_gradient(channel.AsFFI(),
        {stops.data(), stops.size()},
        {colors.data(), colors.size()},
        lo,
        hi,
        palette.AsFFI(),
        space.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::field_gradient(const nucleation::Field& field, nucleation::diplomat::span<const float> stops, nucleation::diplomat::span<const uint8_t> colors, float lo, float hi, const nucleation::Palette& palette, nucleation::InterpolationSpace space) {
    auto result = nucleation::capi::Material_field_gradient(field.AsFFI(),
        {stops.data(), stops.size()},
        {colors.data(), colors.size()},
        lo,
        hi,
        palette.AsFFI(),
        space.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::channel_ramp(nucleation::FieldChannel channel, float lo, float hi, std::string_view blocks_json, bool dither) {
    auto result = nucleation::capi::Material_channel_ramp(channel.AsFFI(),
        lo,
        hi,
        {blocks_json.data(), blocks_json.size()},
        dither);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::field_ramp(const nucleation::Field& field, float lo, float hi, std::string_view blocks_json, bool dither) {
    auto result = nucleation::capi::Material_field_ramp(field.AsFFI(),
        lo,
        hi,
        {blocks_json.data(), blocks_json.size()},
        dither);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::weighted(std::string_view blocks_json, nucleation::diplomat::span<const float> weights, int32_t seed) {
    auto result = nucleation::capi::Material_weighted({blocks_json.data(), blocks_json.size()},
        {weights.data(), weights.size()},
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz, float ambient, const nucleation::Palette& palette) {
    auto result = nucleation::capi::Material_shaded(r,
        g,
        b,
        lx,
        ly,
        lz,
        ambient,
        palette.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError> nucleation::Material::spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b, float ambient, const nucleation::Palette& palette) {
    auto result = nucleation::capi::Material_spotlight(px,
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
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Material>>(std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Material>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Material* nucleation::Material::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Material*>(this);
}

inline nucleation::capi::Material* nucleation::Material::AsFFI() {
    return reinterpret_cast<nucleation::capi::Material*>(this);
}

inline const nucleation::Material* nucleation::Material::FromFFI(const nucleation::capi::Material* ptr) {
    return reinterpret_cast<const nucleation::Material*>(ptr);
}

inline nucleation::Material* nucleation::Material::FromFFI(nucleation::capi::Material* ptr) {
    return reinterpret_cast<nucleation::Material*>(ptr);
}

inline void nucleation::Material::operator delete(void* ptr) {
    nucleation::capi::Material_destroy(reinterpret_cast<nucleation::capi::Material*>(ptr));
}


#endif // NUCLEATION_Material_HPP
