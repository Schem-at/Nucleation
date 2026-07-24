#ifndef FieldProgramValueType_HPP
#define FieldProgramValueType_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace

inline diplomat::capi::FieldProgramValueType FieldProgramValueType::AsFFI() const {
    return static_cast<diplomat::capi::FieldProgramValueType>(value);
}

inline FieldProgramValueType FieldProgramValueType::FromFFI(diplomat::capi::FieldProgramValueType c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldProgramValueType_Scalar:
        case diplomat::capi::FieldProgramValueType_Vec3:
        case diplomat::capi::FieldProgramValueType_Bool:
            return static_cast<FieldProgramValueType::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldProgramValueType_HPP
