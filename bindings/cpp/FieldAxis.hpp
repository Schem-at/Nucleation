#ifndef FieldAxis_HPP
#define FieldAxis_HPP

#include "FieldAxis.d.hpp"

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

inline diplomat::capi::FieldAxis FieldAxis::AsFFI() const {
    return static_cast<diplomat::capi::FieldAxis>(value);
}

inline FieldAxis FieldAxis::FromFFI(diplomat::capi::FieldAxis c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldAxis_X:
        case diplomat::capi::FieldAxis_Y:
        case diplomat::capi::FieldAxis_Z:
            return static_cast<FieldAxis::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldAxis_HPP
