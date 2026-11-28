#ifndef NUCLEATION_FieldDistanceKind_HPP
#define NUCLEATION_FieldDistanceKind_HPP

#include "FieldDistanceKind.d.hpp"

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

inline nucleation::capi::FieldDistanceKind nucleation::FieldDistanceKind::AsFFI() const {
    return static_cast<nucleation::capi::FieldDistanceKind>(value);
}

inline nucleation::FieldDistanceKind nucleation::FieldDistanceKind::FromFFI(nucleation::capi::FieldDistanceKind c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldDistanceKind_Exact:
        case nucleation::capi::FieldDistanceKind_LowerBound:
        case nucleation::capi::FieldDistanceKind_Estimate:
        case nucleation::capi::FieldDistanceKind_Implicit:
            return static_cast<nucleation::FieldDistanceKind::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldDistanceKind_HPP
