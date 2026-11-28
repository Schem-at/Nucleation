#ifndef NUCLEATION_FieldAxis_HPP
#define NUCLEATION_FieldAxis_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::FieldAxis nucleation::FieldAxis::AsFFI() const {
    return static_cast<nucleation::capi::FieldAxis>(value);
}

inline nucleation::FieldAxis nucleation::FieldAxis::FromFFI(nucleation::capi::FieldAxis c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldAxis_X:
        case nucleation::capi::FieldAxis_Y:
        case nucleation::capi::FieldAxis_Z:
            return static_cast<nucleation::FieldAxis::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldAxis_HPP
