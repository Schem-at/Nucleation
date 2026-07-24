#ifndef NUCLEATION_FieldProgramUnaryOp_D_HPP
#define NUCLEATION_FieldProgramUnaryOp_D_HPP

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
    enum FieldProgramUnaryOp {
      FieldProgramUnaryOp_Neg = 0,
      FieldProgramUnaryOp_Abs = 1,
      FieldProgramUnaryOp_Sqrt = 2,
      FieldProgramUnaryOp_Log = 3,
      FieldProgramUnaryOp_Sin = 4,
      FieldProgramUnaryOp_Cos = 5,
      FieldProgramUnaryOp_Acos = 6,
      FieldProgramUnaryOp_VecX = 7,
      FieldProgramUnaryOp_VecY = 8,
      FieldProgramUnaryOp_VecZ = 9,
      FieldProgramUnaryOp_Length = 10,
      FieldProgramUnaryOp_Normalize = 11,
    };

    typedef struct FieldProgramUnaryOp_option {union { FieldProgramUnaryOp ok; }; bool is_ok; } FieldProgramUnaryOp_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Unary field-program operations (see `crate::sdf::UnaryOp`).
 */
class FieldProgramUnaryOp {
public:
    enum Value {
        Neg = 0,
        Abs = 1,
        Sqrt = 2,
        Log = 3,
        Sin = 4,
        Cos = 5,
        Acos = 6,
        VecX = 7,
        VecY = 8,
        VecZ = 9,
        Length = 10,
        Normalize = 11,
    };

    FieldProgramUnaryOp(): value(Value::Neg) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldProgramUnaryOp(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::FieldProgramUnaryOp AsFFI() const;
    inline static nucleation::FieldProgramUnaryOp FromFFI(nucleation::capi::FieldProgramUnaryOp c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_FieldProgramUnaryOp_D_HPP
