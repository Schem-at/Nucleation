#ifndef NUCLEATION_FieldProgramValueType_HPP
#define NUCLEATION_FieldProgramValueType_HPP

#include "FieldProgramValueType.d.hpp"

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

inline nucleation::capi::FieldProgramValueType nucleation::FieldProgramValueType::AsFFI() const {
    return static_cast<nucleation::capi::FieldProgramValueType>(value);
}

inline nucleation::FieldProgramValueType nucleation::FieldProgramValueType::FromFFI(nucleation::capi::FieldProgramValueType c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldProgramValueType_Scalar:
        case nucleation::capi::FieldProgramValueType_Vec3:
        case nucleation::capi::FieldProgramValueType_Bool:
            return static_cast<nucleation::FieldProgramValueType::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldProgramValueType_HPP
