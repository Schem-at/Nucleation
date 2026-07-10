#ifndef NUCLEATION_LayoutFunction_D_HPP
#define NUCLEATION_LayoutFunction_D_HPP

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
namespace capi { struct LayoutFunction; }
class LayoutFunction;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct LayoutFunction;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Maps logical bits to physical positions (PORTING rule 10).
 */
class LayoutFunction {
public:

  inline static std::unique_ptr<nucleation::LayoutFunction> one_to_one();

  inline static std::unique_ptr<nucleation::LayoutFunction> packed4();

  /**
   * Custom bit-to-position mapping.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::LayoutFunction>, nucleation::NucleationError> custom(nucleation::diplomat::span<const uint32_t> mapping);

  inline static std::unique_ptr<nucleation::LayoutFunction> row_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

  inline static std::unique_ptr<nucleation::LayoutFunction> column_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

  inline static std::unique_ptr<nucleation::LayoutFunction> scanline(uint32_t width, uint32_t height, uint32_t bits_per_pixel);

    inline const nucleation::capi::LayoutFunction* AsFFI() const;
    inline nucleation::capi::LayoutFunction* AsFFI();
    inline static const nucleation::LayoutFunction* FromFFI(const nucleation::capi::LayoutFunction* ptr);
    inline static nucleation::LayoutFunction* FromFFI(nucleation::capi::LayoutFunction* ptr);
    inline static void operator delete(void* ptr);
private:
    LayoutFunction() = delete;
    LayoutFunction(const nucleation::LayoutFunction&) = delete;
    LayoutFunction(nucleation::LayoutFunction&&) noexcept = delete;
    LayoutFunction operator=(const nucleation::LayoutFunction&) = delete;
    LayoutFunction operator=(nucleation::LayoutFunction&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_LayoutFunction_D_HPP
