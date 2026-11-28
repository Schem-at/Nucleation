#ifndef FieldChannel_HPP
#define FieldChannel_HPP

#include "FieldChannel.d.hpp"

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

} // namespace capi
} // namespace

inline diplomat::capi::FieldChannel FieldChannel::AsFFI() const {
    return static_cast<diplomat::capi::FieldChannel>(value);
}

inline FieldChannel FieldChannel::FromFFI(diplomat::capi::FieldChannel c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldChannel_WorldX:
        case diplomat::capi::FieldChannel_WorldY:
        case diplomat::capi::FieldChannel_WorldZ:
        case diplomat::capi::FieldChannel_Distance:
        case diplomat::capi::FieldChannel_DepthBelowSurface:
        case diplomat::capi::FieldChannel_Slope:
        case diplomat::capi::FieldChannel_NormalY:
        case diplomat::capi::FieldChannel_Curvature:
            return static_cast<FieldChannel::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldChannel_HPP
