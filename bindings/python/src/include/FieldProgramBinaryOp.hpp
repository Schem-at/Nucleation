#ifndef NUCLEATION_FieldProgramBinaryOp_HPP
#define NUCLEATION_FieldProgramBinaryOp_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::FieldProgramBinaryOp nucleation::FieldProgramBinaryOp::AsFFI() const {
    return static_cast<nucleation::capi::FieldProgramBinaryOp>(value);
}

inline nucleation::FieldProgramBinaryOp nucleation::FieldProgramBinaryOp::FromFFI(nucleation::capi::FieldProgramBinaryOp c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldProgramBinaryOp_Add:
        case nucleation::capi::FieldProgramBinaryOp_Sub:
        case nucleation::capi::FieldProgramBinaryOp_Mul:
        case nucleation::capi::FieldProgramBinaryOp_Div:
        case nucleation::capi::FieldProgramBinaryOp_Min:
        case nucleation::capi::FieldProgramBinaryOp_Max:
        case nucleation::capi::FieldProgramBinaryOp_Pow:
        case nucleation::capi::FieldProgramBinaryOp_Atan2:
        case nucleation::capi::FieldProgramBinaryOp_Lt:
        case nucleation::capi::FieldProgramBinaryOp_Le:
        case nucleation::capi::FieldProgramBinaryOp_Gt:
        case nucleation::capi::FieldProgramBinaryOp_Ge:
        case nucleation::capi::FieldProgramBinaryOp_Eq:
        case nucleation::capi::FieldProgramBinaryOp_Dot:
        case nucleation::capi::FieldProgramBinaryOp_Cross:
        case nucleation::capi::FieldProgramBinaryOp_Scale:
            return static_cast<nucleation::FieldProgramBinaryOp::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldProgramBinaryOp_HPP
