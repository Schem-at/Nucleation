#ifndef FieldChannel_D_HPP
#define FieldChannel_D_HPP

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
    enum FieldChannel {
      FieldChannel_WorldX = 0,
      FieldChannel_WorldY = 1,
      FieldChannel_WorldZ = 2,
      FieldChannel_Distance = 3,
      FieldChannel_DepthBelowSurface = 4,
      FieldChannel_Slope = 5,
      FieldChannel_NormalY = 6,
      FieldChannel_Curvature = 7,
    };

    typedef struct FieldChannel_option {union { FieldChannel ok; }; bool is_ok; } FieldChannel_option;
} // namespace capi
} // namespace

/**
 * Built-in voxel channels readable by material gradients, ramps, and conditions.
 */
class FieldChannel {
public:
    enum Value {
        WorldX = 0,
        WorldY = 1,
        WorldZ = 2,
        Distance = 3,
        DepthBelowSurface = 4,
        Slope = 5,
        NormalY = 6,
        Curvature = 7,
    };

    FieldChannel(): value(Value::WorldX) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldChannel(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::FieldChannel AsFFI() const;
    inline static FieldChannel FromFFI(diplomat::capi::FieldChannel c_enum);
private:
    Value value;
};


#endif // FieldChannel_D_HPP
