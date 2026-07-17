#ifndef Brush_D_HPP
#define Brush_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Palette; }
class Palette;
class InterpolationSpace;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Brush;
} // namespace capi
} // namespace

/**
 * Decides which block goes at each point of a filled shape. Wraps `BrushEnum`.
 */
class Brush {
public:

  /**
   * Every point becomes `block_name` (a block-state string).
   */
  inline static diplomat::result<std::unique_ptr<Brush>, NucleationError> solid(std::string_view block_name);

  /**
   * Nearest-block-to-RGB-color brush.
   */
  inline static std::unique_ptr<Brush> color(uint8_t r, uint8_t g, uint8_t b);

  /**
   * Linear color gradient between two anchored points.
   */
  inline static std::unique_ptr<Brush> linear_gradient(int32_t x1, int32_t y1, int32_t z1, uint8_t r1, uint8_t g1, uint8_t b1, int32_t x2, int32_t y2, int32_t z2, uint8_t r2, uint8_t g2, uint8_t b2, InterpolationSpace space);

  /**
   * Base color shaded by surface normal against light direction
   * (`lx`, `ly`, `lz`).
   */
  inline static std::unique_ptr<Brush> shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz);

  /**
   * Bilinear gradient over the patch `origin + s*u + t*v` with corner colors
   * c00/c10/c01/c11.
   */
  inline static std::unique_ptr<Brush> bilinear_gradient(int32_t ox, int32_t oy, int32_t oz, int32_t ux, int32_t uy, int32_t uz, int32_t vx, int32_t vy, int32_t vz, uint8_t r00, uint8_t g00, uint8_t b00, uint8_t r10, uint8_t g10, uint8_t b10, uint8_t r01, uint8_t g01, uint8_t b01, uint8_t r11, uint8_t g11, uint8_t b11, InterpolationSpace space);

  /**
   * Inverse-distance-weighted gradient between colored anchor points.
   * `positions` is flat `[x0, y0, z0, x1, …]` and `colors` is flat
   * `[r0, g0, b0, r1, …]`; both must describe the same non-zero number of
   * points (`positions.len() == colors.len()`, a multiple of 3).
   */
  inline static diplomat::result<std::unique_ptr<Brush>, NucleationError> point_gradient(diplomat::span<const int32_t> positions, diplomat::span<const uint8_t> colors, float falloff, InterpolationSpace space);

  /**
   * Use `palette` for this brush's color→block snapping instead of the
   * default all-blocks palette. No-op for `solid` brushes, which place
   * a fixed block state. Set it before filling; the palette is shared,
   * not copied.
   */
  inline void set_palette(const Palette& palette);

  /**
   * Gradient along a parametric curve: `stops` holds the curve parameters in
   * `[0, 1]` and `colors` the matching flat RGB triples
   * (`colors.len() == stops.len() * 3`, `stops` non-empty).
   */
  inline static diplomat::result<std::unique_ptr<Brush>, NucleationError> curve_gradient(diplomat::span<const float> stops, diplomat::span<const uint8_t> colors, InterpolationSpace space);

    inline const diplomat::capi::Brush* AsFFI() const;
    inline diplomat::capi::Brush* AsFFI();
    inline static const Brush* FromFFI(const diplomat::capi::Brush* ptr);
    inline static Brush* FromFFI(diplomat::capi::Brush* ptr);
    inline static void operator delete(void* ptr);
private:
    Brush() = delete;
    Brush(const Brush&) = delete;
    Brush(Brush&&) noexcept = delete;
    Brush operator=(const Brush&) = delete;
    Brush operator=(Brush&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Brush_D_HPP
