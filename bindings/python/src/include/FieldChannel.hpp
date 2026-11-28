#ifndef NUCLEATION_FieldChannel_HPP
#define NUCLEATION_FieldChannel_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::FieldChannel nucleation::FieldChannel::AsFFI() const {
    return static_cast<nucleation::capi::FieldChannel>(value);
}

inline nucleation::FieldChannel nucleation::FieldChannel::FromFFI(nucleation::capi::FieldChannel c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldChannel_WorldX:
        case nucleation::capi::FieldChannel_WorldY:
        case nucleation::capi::FieldChannel_WorldZ:
        case nucleation::capi::FieldChannel_Distance:
        case nucleation::capi::FieldChannel_DepthBelowSurface:
        case nucleation::capi::FieldChannel_Slope:
        case nucleation::capi::FieldChannel_NormalY:
        case nucleation::capi::FieldChannel_Curvature:
            return static_cast<nucleation::FieldChannel::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldChannel_HPP
