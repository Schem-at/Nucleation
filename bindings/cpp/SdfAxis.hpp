#ifndef SdfAxis_HPP
#define SdfAxis_HPP

#include "SdfAxis.d.hpp"

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

inline diplomat::capi::SdfAxis SdfAxis::AsFFI() const {
    return static_cast<diplomat::capi::SdfAxis>(value);
}

inline SdfAxis SdfAxis::FromFFI(diplomat::capi::SdfAxis c_enum) {
    switch (c_enum) {
        case diplomat::capi::SdfAxis_X:
        case diplomat::capi::SdfAxis_Y:
        case diplomat::capi::SdfAxis_Z:
            return static_cast<SdfAxis::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // SdfAxis_HPP
