#ifndef FieldProgramDistanceKind_HPP
#define FieldProgramDistanceKind_HPP

#include "FieldProgramDistanceKind.d.hpp"

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

inline diplomat::capi::FieldProgramDistanceKind FieldProgramDistanceKind::AsFFI() const {
    return static_cast<diplomat::capi::FieldProgramDistanceKind>(value);
}

inline FieldProgramDistanceKind FieldProgramDistanceKind::FromFFI(diplomat::capi::FieldProgramDistanceKind c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldProgramDistanceKind_Exact:
        case diplomat::capi::FieldProgramDistanceKind_LowerBound:
        case diplomat::capi::FieldProgramDistanceKind_Estimate:
        case diplomat::capi::FieldProgramDistanceKind_Implicit:
            return static_cast<FieldProgramDistanceKind::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldProgramDistanceKind_HPP
