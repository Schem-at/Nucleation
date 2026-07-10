#ifndef InterpolationSpace_HPP
#define InterpolationSpace_HPP

#include "InterpolationSpace.d.hpp"

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

inline diplomat::capi::InterpolationSpace InterpolationSpace::AsFFI() const {
    return static_cast<diplomat::capi::InterpolationSpace>(value);
}

inline InterpolationSpace InterpolationSpace::FromFFI(diplomat::capi::InterpolationSpace c_enum) {
    switch (c_enum) {
        case diplomat::capi::InterpolationSpace_Rgb:
        case diplomat::capi::InterpolationSpace_Oklab:
            return static_cast<InterpolationSpace::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // InterpolationSpace_HPP
