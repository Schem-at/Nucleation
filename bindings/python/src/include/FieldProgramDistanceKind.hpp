#ifndef NUCLEATION_FieldProgramDistanceKind_HPP
#define NUCLEATION_FieldProgramDistanceKind_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::FieldProgramDistanceKind nucleation::FieldProgramDistanceKind::AsFFI() const {
    return static_cast<nucleation::capi::FieldProgramDistanceKind>(value);
}

inline nucleation::FieldProgramDistanceKind nucleation::FieldProgramDistanceKind::FromFFI(nucleation::capi::FieldProgramDistanceKind c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldProgramDistanceKind_Exact:
        case nucleation::capi::FieldProgramDistanceKind_LowerBound:
        case nucleation::capi::FieldProgramDistanceKind_Estimate:
        case nucleation::capi::FieldProgramDistanceKind_Implicit:
            return static_cast<nucleation::FieldProgramDistanceKind::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldProgramDistanceKind_HPP
