#ifndef Brush_HPP
#define Brush_HPP

#include "Brush.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "InterpolationSpace.hpp"
#include "NucleationError.hpp"
#include "Palette.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Brush_solid_result {union {diplomat::capi::Brush* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Brush_solid_result;
    Brush_solid_result Brush_solid(diplomat::capi::DiplomatStringView block_name);

    diplomat::capi::Brush* Brush_color(uint8_t r, uint8_t g, uint8_t b);

    diplomat::capi::Brush* Brush_linear_gradient(int32_t x1, int32_t y1, int32_t z1, uint8_t r1, uint8_t g1, uint8_t b1, int32_t x2, int32_t y2, int32_t z2, uint8_t r2, uint8_t g2, uint8_t b2, diplomat::capi::InterpolationSpace space);

    diplomat::capi::Brush* Brush_shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz);

    diplomat::capi::Brush* Brush_bilinear_gradient(int32_t ox, int32_t oy, int32_t oz, int32_t ux, int32_t uy, int32_t uz, int32_t vx, int32_t vy, int32_t vz, uint8_t r00, uint8_t g00, uint8_t b00, uint8_t r10, uint8_t g10, uint8_t b10, uint8_t r01, uint8_t g01, uint8_t b01, uint8_t r11, uint8_t g11, uint8_t b11, diplomat::capi::InterpolationSpace space);

    typedef struct Brush_point_gradient_result {union {diplomat::capi::Brush* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Brush_point_gradient_result;
    Brush_point_gradient_result Brush_point_gradient(diplomat::capi::DiplomatI32View positions, diplomat::capi::DiplomatU8View colors, float falloff, diplomat::capi::InterpolationSpace space);

    diplomat::capi::Brush* Brush_spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b);

    void Brush_set_palette(diplomat::capi::Brush* self, const diplomat::capi::Palette* palette);

    typedef struct Brush_curve_gradient_result {union {diplomat::capi::Brush* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Brush_curve_gradient_result;
    Brush_curve_gradient_result Brush_curve_gradient(diplomat::capi::DiplomatF32View stops, diplomat::capi::DiplomatU8View colors, diplomat::capi::InterpolationSpace space);

    void Brush_destroy(Brush* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Brush>, NucleationError> Brush::solid(std::string_view block_name) {
    auto result = diplomat::capi::Brush_solid({block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Brush>, NucleationError>(diplomat::Ok<std::unique_ptr<Brush>>(std::unique_ptr<Brush>(Brush::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Brush>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Brush> Brush::color(uint8_t r, uint8_t g, uint8_t b) {
    auto result = diplomat::capi::Brush_color(r,
        g,
        b);
    return std::unique_ptr<Brush>(Brush::FromFFI(result));
}

inline std::unique_ptr<Brush> Brush::linear_gradient(int32_t x1, int32_t y1, int32_t z1, uint8_t r1, uint8_t g1, uint8_t b1, int32_t x2, int32_t y2, int32_t z2, uint8_t r2, uint8_t g2, uint8_t b2, InterpolationSpace space) {
    auto result = diplomat::capi::Brush_linear_gradient(x1,
        y1,
        z1,
        r1,
        g1,
        b1,
        x2,
        y2,
        z2,
        r2,
        g2,
        b2,
        space.AsFFI());
    return std::unique_ptr<Brush>(Brush::FromFFI(result));
}

inline std::unique_ptr<Brush> Brush::shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz) {
    auto result = diplomat::capi::Brush_shaded(r,
        g,
        b,
        lx,
        ly,
        lz);
    return std::unique_ptr<Brush>(Brush::FromFFI(result));
}

inline std::unique_ptr<Brush> Brush::bilinear_gradient(int32_t ox, int32_t oy, int32_t oz, int32_t ux, int32_t uy, int32_t uz, int32_t vx, int32_t vy, int32_t vz, uint8_t r00, uint8_t g00, uint8_t b00, uint8_t r10, uint8_t g10, uint8_t b10, uint8_t r01, uint8_t g01, uint8_t b01, uint8_t r11, uint8_t g11, uint8_t b11, InterpolationSpace space) {
    auto result = diplomat::capi::Brush_bilinear_gradient(ox,
        oy,
        oz,
        ux,
        uy,
        uz,
        vx,
        vy,
        vz,
        r00,
        g00,
        b00,
        r10,
        g10,
        b10,
        r01,
        g01,
        b01,
        r11,
        g11,
        b11,
        space.AsFFI());
    return std::unique_ptr<Brush>(Brush::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Brush>, NucleationError> Brush::point_gradient(diplomat::span<const int32_t> positions, diplomat::span<const uint8_t> colors, float falloff, InterpolationSpace space) {
    auto result = diplomat::capi::Brush_point_gradient({positions.data(), positions.size()},
        {colors.data(), colors.size()},
        falloff,
        space.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Brush>, NucleationError>(diplomat::Ok<std::unique_ptr<Brush>>(std::unique_ptr<Brush>(Brush::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Brush>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<Brush> Brush::spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b) {
    auto result = diplomat::capi::Brush_spotlight(px,
        py,
        pz,
        dx,
        dy,
        dz,
        cone_angle_deg,
        r,
        g,
        b);
    return std::unique_ptr<Brush>(Brush::FromFFI(result));
}

inline void Brush::set_palette(const Palette& palette) {
    diplomat::capi::Brush_set_palette(this->AsFFI(),
        palette.AsFFI());
}

inline diplomat::result<std::unique_ptr<Brush>, NucleationError> Brush::curve_gradient(diplomat::span<const float> stops, diplomat::span<const uint8_t> colors, InterpolationSpace space) {
    auto result = diplomat::capi::Brush_curve_gradient({stops.data(), stops.size()},
        {colors.data(), colors.size()},
        space.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Brush>, NucleationError>(diplomat::Ok<std::unique_ptr<Brush>>(std::unique_ptr<Brush>(Brush::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Brush>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Brush* Brush::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Brush*>(this);
}

inline diplomat::capi::Brush* Brush::AsFFI() {
    return reinterpret_cast<diplomat::capi::Brush*>(this);
}

inline const Brush* Brush::FromFFI(const diplomat::capi::Brush* ptr) {
    return reinterpret_cast<const Brush*>(ptr);
}

inline Brush* Brush::FromFFI(diplomat::capi::Brush* ptr) {
    return reinterpret_cast<Brush*>(ptr);
}

inline void Brush::operator delete(void* ptr) {
    diplomat::capi::Brush_destroy(reinterpret_cast<diplomat::capi::Brush*>(ptr));
}


#endif // Brush_HPP
