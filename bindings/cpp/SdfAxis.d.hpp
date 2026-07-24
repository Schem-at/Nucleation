#ifndef SdfAxis_D_HPP
#define SdfAxis_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    enum SdfAxis {
      SdfAxis_X = 0,
      SdfAxis_Y = 1,
      SdfAxis_Z = 2,
    };

    typedef struct SdfAxis_option {union { SdfAxis ok; }; bool is_ok; } SdfAxis_option;
} // namespace capi
} // namespace

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

    inline diplomat::capi::SdfAxis AsFFI() const;
    inline static SdfAxis FromFFI(diplomat::capi::SdfAxis c_enum);
private:
    Value value;
};


#endif // SdfAxis_D_HPP
