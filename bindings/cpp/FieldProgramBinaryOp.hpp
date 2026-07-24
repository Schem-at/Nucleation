#ifndef FieldProgramBinaryOp_HPP
#define FieldProgramBinaryOp_HPP

#include "FieldProgramBinaryOp.d.hpp"

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

inline diplomat::capi::FieldProgramBinaryOp FieldProgramBinaryOp::AsFFI() const {
    return static_cast<diplomat::capi::FieldProgramBinaryOp>(value);
}

inline FieldProgramBinaryOp FieldProgramBinaryOp::FromFFI(diplomat::capi::FieldProgramBinaryOp c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldProgramBinaryOp_Add:
        case diplomat::capi::FieldProgramBinaryOp_Sub:
        case diplomat::capi::FieldProgramBinaryOp_Mul:
        case diplomat::capi::FieldProgramBinaryOp_Div:
        case diplomat::capi::FieldProgramBinaryOp_Min:
        case diplomat::capi::FieldProgramBinaryOp_Max:
        case diplomat::capi::FieldProgramBinaryOp_Pow:
        case diplomat::capi::FieldProgramBinaryOp_Atan2:
        case diplomat::capi::FieldProgramBinaryOp_Lt:
        case diplomat::capi::FieldProgramBinaryOp_Le:
        case diplomat::capi::FieldProgramBinaryOp_Gt:
        case diplomat::capi::FieldProgramBinaryOp_Ge:
        case diplomat::capi::FieldProgramBinaryOp_Eq:
        case diplomat::capi::FieldProgramBinaryOp_Dot:
        case diplomat::capi::FieldProgramBinaryOp_Cross:
        case diplomat::capi::FieldProgramBinaryOp_Scale:
            return static_cast<FieldProgramBinaryOp::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldProgramBinaryOp_HPP
