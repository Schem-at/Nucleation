#ifndef LayoutFunction_D_HPP
#define LayoutFunction_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct LayoutFunction;
} // namespace capi
} // namespace

/**
 * Maps logical bits to physical positions (PORTING rule 10).
 */
class LayoutFunction {
public:

  /**
   * One bit per position: signal strength 0 = false, 15 = true.
   * The most common layout for traditional redstone circuits.
   */
  inline static std::unique_ptr<LayoutFunction> one_to_one();

  /**
   * Four bits per position, packed as a hex nibble (0-15) with the
   * lowest bit first; uses `ceil(bits / 4)` positions (4x denser than
   * one-to-one).
   */
  inline static std::unique_ptr<LayoutFunction> packed4();

  /**
   * Custom bit-to-position mapping.
   */
  inline static diplomat::result<std::unique_ptr<LayoutFunction>, NucleationError> custom(diplomat::span<const uint32_t> mapping);

  /**
   * 2D matrix layout, elements laid out row by row; element bits are
   * packed 4 per position (nibbles).
   */
  inline static std::unique_ptr<LayoutFunction> row_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

  /**
   * 2D matrix layout, elements laid out column by column; element bits
   * are packed 4 per position (nibbles).
   */
  inline static std::unique_ptr<LayoutFunction> column_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

  /**
   * Screen layout: pixels laid out row by row, left to right; pixel
   * bits are packed 4 per position (nibbles).
   */
  inline static std::unique_ptr<LayoutFunction> scanline(uint32_t width, uint32_t height, uint32_t bits_per_pixel);

    inline const diplomat::capi::LayoutFunction* AsFFI() const;
    inline diplomat::capi::LayoutFunction* AsFFI();
    inline static const LayoutFunction* FromFFI(const diplomat::capi::LayoutFunction* ptr);
    inline static LayoutFunction* FromFFI(diplomat::capi::LayoutFunction* ptr);
    inline static void operator delete(void* ptr);
private:
    LayoutFunction() = delete;
    LayoutFunction(const LayoutFunction&) = delete;
    LayoutFunction(LayoutFunction&&) noexcept = delete;
    LayoutFunction operator=(const LayoutFunction&) = delete;
    LayoutFunction operator=(LayoutFunction&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // LayoutFunction_D_HPP
