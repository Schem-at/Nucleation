#ifndef NUCLEATION_SdfAxis_D_HPP
#define NUCLEATION_SdfAxis_D_HPP

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
    enum SdfAxis {
      SdfAxis_X = 0,
      SdfAxis_Y = 1,
      SdfAxis_Z = 2,
    };

    typedef struct SdfAxis_option {union { SdfAxis ok; }; bool is_ok; } SdfAxis_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Axis used by mirror operations.
 */
class SdfAxis {
public:
    enum Value {
        X = 0,
        Y = 1,
        Z = 2,
    };

    SdfAxis(): value(Value::X) {}

    // Implicit conversions between enum and ::Value
    constexpr SdfAxis(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::SdfAxis AsFFI() const;
    inline static nucleation::SdfAxis FromFFI(nucleation::capi::SdfAxis c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_SdfAxis_D_HPP
