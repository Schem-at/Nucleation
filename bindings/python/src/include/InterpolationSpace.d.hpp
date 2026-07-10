#ifndef NUCLEATION_InterpolationSpace_D_HPP
#define NUCLEATION_InterpolationSpace_D_HPP

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
namespace capi {
    enum InterpolationSpace {
      InterpolationSpace_Rgb = 0,
      InterpolationSpace_Oklab = 1,
    };

    typedef struct InterpolationSpace_option {union { InterpolationSpace ok; }; bool is_ok; } InterpolationSpace_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Color interpolation space for gradient brushes. The old ABI passed this as
 * `space: c_int` (`1` = Oklab, anything else = RGB).
 */
class InterpolationSpace {
public:
    enum Value {
        Rgb = 0,
        Oklab = 1,
    };

    InterpolationSpace(): value(Value::Rgb) {}

    // Implicit conversions between enum and ::Value
    constexpr InterpolationSpace(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::InterpolationSpace AsFFI() const;
    inline static nucleation::InterpolationSpace FromFFI(nucleation::capi::InterpolationSpace c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_InterpolationSpace_D_HPP
