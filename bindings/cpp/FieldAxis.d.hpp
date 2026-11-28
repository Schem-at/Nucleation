#ifndef FieldAxis_D_HPP
#define FieldAxis_D_HPP

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
    enum FieldAxis {
      FieldAxis_X = 0,
      FieldAxis_Y = 1,
      FieldAxis_Z = 2,
    };

    typedef struct FieldAxis_option {union { FieldAxis ok; }; bool is_ok; } FieldAxis_option;
} // namespace capi
} // namespace

/**
 * Axis selector for mirror operations.
 */
class FieldAxis {
public:
    enum Value {
        X = 0,
        Y = 1,
        Z = 2,
    };

    FieldAxis(): value(Value::X) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldAxis(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::FieldAxis AsFFI() const;
    inline static FieldAxis FromFFI(diplomat::capi::FieldAxis c_enum);
private:
    Value value;
};


#endif // FieldAxis_D_HPP
