#ifndef NUCLEATION_FieldProgramDistanceKind_D_HPP
#define NUCLEATION_FieldProgramDistanceKind_D_HPP

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
    enum FieldProgramDistanceKind {
      FieldProgramDistanceKind_Exact = 0,
      FieldProgramDistanceKind_LowerBound = 1,
      FieldProgramDistanceKind_Estimate = 2,
      FieldProgramDistanceKind_Implicit = 3,
    };

    typedef struct FieldProgramDistanceKind_option {union { FieldProgramDistanceKind ok; }; bool is_ok; } FieldProgramDistanceKind_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * What kind of distance a field program's output represents.
 */
class FieldProgramDistanceKind {
public:
    enum Value {
        Exact = 0,
        LowerBound = 1,
        Estimate = 2,
        Implicit = 3,
    };

    FieldProgramDistanceKind(): value(Value::Exact) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldProgramDistanceKind(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::FieldProgramDistanceKind AsFFI() const;
    inline static nucleation::FieldProgramDistanceKind FromFFI(nucleation::capi::FieldProgramDistanceKind c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_FieldProgramDistanceKind_D_HPP
