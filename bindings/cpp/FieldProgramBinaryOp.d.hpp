#ifndef FieldProgramBinaryOp_D_HPP
#define FieldProgramBinaryOp_D_HPP

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
    enum FieldProgramBinaryOp {
      FieldProgramBinaryOp_Add = 0,
      FieldProgramBinaryOp_Sub = 1,
      FieldProgramBinaryOp_Mul = 2,
      FieldProgramBinaryOp_Div = 3,
      FieldProgramBinaryOp_Min = 4,
      FieldProgramBinaryOp_Max = 5,
      FieldProgramBinaryOp_Pow = 6,
      FieldProgramBinaryOp_Atan2 = 7,
      FieldProgramBinaryOp_Lt = 8,
      FieldProgramBinaryOp_Le = 9,
      FieldProgramBinaryOp_Gt = 10,
      FieldProgramBinaryOp_Ge = 11,
      FieldProgramBinaryOp_Eq = 12,
      FieldProgramBinaryOp_Dot = 13,
      FieldProgramBinaryOp_Cross = 14,
      FieldProgramBinaryOp_Scale = 15,
    };

    typedef struct FieldProgramBinaryOp_option {union { FieldProgramBinaryOp ok; }; bool is_ok; } FieldProgramBinaryOp_option;
} // namespace capi
} // namespace

/**
 * Binary field-program operations (see `crate::sdf::BinaryOp`). `Add`
 * and `Sub` accept either two scalars or two vec3s.
 */
class FieldProgramBinaryOp {
public:
    enum Value {
        Add = 0,
        Sub = 1,
        Mul = 2,
        Div = 3,
        Min = 4,
        Max = 5,
        Pow = 6,
        Atan2 = 7,
        Lt = 8,
        Le = 9,
        Gt = 10,
        Ge = 11,
        Eq = 12,
        Dot = 13,
        Cross = 14,
        /**
         * `vec3 * scalar`, componentwise.
         */
        Scale = 15,
    };

    FieldProgramBinaryOp(): value(Value::Add) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldProgramBinaryOp(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::FieldProgramBinaryOp AsFFI() const;
    inline static FieldProgramBinaryOp FromFFI(diplomat::capi::FieldProgramBinaryOp c_enum);
private:
    Value value;
};


#endif // FieldProgramBinaryOp_D_HPP
