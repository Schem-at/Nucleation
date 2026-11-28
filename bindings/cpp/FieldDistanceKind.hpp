#ifndef FieldDistanceKind_HPP
#define FieldDistanceKind_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace

inline diplomat::capi::FieldDistanceKind FieldDistanceKind::AsFFI() const {
    return static_cast<diplomat::capi::FieldDistanceKind>(value);
}

inline FieldDistanceKind FieldDistanceKind::FromFFI(diplomat::capi::FieldDistanceKind c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldDistanceKind_Exact:
        case diplomat::capi::FieldDistanceKind_LowerBound:
        case diplomat::capi::FieldDistanceKind_Estimate:
        case diplomat::capi::FieldDistanceKind_Implicit:
            return static_cast<FieldDistanceKind::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldDistanceKind_HPP
