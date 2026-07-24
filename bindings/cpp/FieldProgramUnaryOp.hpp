#ifndef FieldProgramUnaryOp_HPP
#define FieldProgramUnaryOp_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace

inline diplomat::capi::FieldProgramUnaryOp FieldProgramUnaryOp::AsFFI() const {
    return static_cast<diplomat::capi::FieldProgramUnaryOp>(value);
}

inline FieldProgramUnaryOp FieldProgramUnaryOp::FromFFI(diplomat::capi::FieldProgramUnaryOp c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldProgramUnaryOp_Neg:
        case diplomat::capi::FieldProgramUnaryOp_Abs:
        case diplomat::capi::FieldProgramUnaryOp_Sqrt:
        case diplomat::capi::FieldProgramUnaryOp_Log:
        case diplomat::capi::FieldProgramUnaryOp_Sin:
        case diplomat::capi::FieldProgramUnaryOp_Cos:
        case diplomat::capi::FieldProgramUnaryOp_Acos:
        case diplomat::capi::FieldProgramUnaryOp_VecX:
        case diplomat::capi::FieldProgramUnaryOp_VecY:
        case diplomat::capi::FieldProgramUnaryOp_VecZ:
        case diplomat::capi::FieldProgramUnaryOp_Length:
        case diplomat::capi::FieldProgramUnaryOp_Normalize:
            return static_cast<FieldProgramUnaryOp::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldProgramUnaryOp_HPP
