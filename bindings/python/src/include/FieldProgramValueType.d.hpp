#ifndef NUCLEATION_FieldProgramValueType_D_HPP
#define NUCLEATION_FieldProgramValueType_D_HPP

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
    enum FieldProgramValueType {
      FieldProgramValueType_Scalar = 0,
      FieldProgramValueType_Vec3 = 1,
      FieldProgramValueType_Bool = 2,
    };

    typedef struct FieldProgramValueType_option {union { FieldProgramValueType ok; }; bool is_ok; } FieldProgramValueType_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * The type of a value on a {@link FieldProgramBuilder}'s stack or in a slot.
 */
class FieldProgramValueType {
public:
    enum Value {
        Scalar = 0,
        Vec3 = 1,
        Bool = 2,
    };

    FieldProgramValueType(): value(Value::Scalar) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldProgramValueType(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::FieldProgramValueType AsFFI() const;
    inline static nucleation::FieldProgramValueType FromFFI(nucleation::capi::FieldProgramValueType c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_FieldProgramValueType_D_HPP
