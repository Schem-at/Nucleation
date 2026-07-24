#ifndef NUCLEATION_FieldProgramUnaryOp_HPP
#define NUCLEATION_FieldProgramUnaryOp_HPP

#include "FieldProgramUnaryOp.d.hpp"

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

inline nucleation::capi::FieldProgramUnaryOp nucleation::FieldProgramUnaryOp::AsFFI() const {
    return static_cast<nucleation::capi::FieldProgramUnaryOp>(value);
}

inline nucleation::FieldProgramUnaryOp nucleation::FieldProgramUnaryOp::FromFFI(nucleation::capi::FieldProgramUnaryOp c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldProgramUnaryOp_Neg:
        case nucleation::capi::FieldProgramUnaryOp_Abs:
        case nucleation::capi::FieldProgramUnaryOp_Sqrt:
        case nucleation::capi::FieldProgramUnaryOp_Log:
        case nucleation::capi::FieldProgramUnaryOp_Sin:
        case nucleation::capi::FieldProgramUnaryOp_Cos:
        case nucleation::capi::FieldProgramUnaryOp_Acos:
        case nucleation::capi::FieldProgramUnaryOp_VecX:
        case nucleation::capi::FieldProgramUnaryOp_VecY:
        case nucleation::capi::FieldProgramUnaryOp_VecZ:
        case nucleation::capi::FieldProgramUnaryOp_Length:
        case nucleation::capi::FieldProgramUnaryOp_Normalize:
            return static_cast<nucleation::FieldProgramUnaryOp::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldProgramUnaryOp_HPP
