#ifndef NUCLEATION_SdfAxis_HPP
#define NUCLEATION_SdfAxis_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::SdfAxis nucleation::SdfAxis::AsFFI() const {
    return static_cast<nucleation::capi::SdfAxis>(value);
}

inline nucleation::SdfAxis nucleation::SdfAxis::FromFFI(nucleation::capi::SdfAxis c_enum) {
    switch (c_enum) {
        case nucleation::capi::SdfAxis_X:
        case nucleation::capi::SdfAxis_Y:
        case nucleation::capi::SdfAxis_Z:
            return static_cast<nucleation::SdfAxis::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_SdfAxis_HPP
