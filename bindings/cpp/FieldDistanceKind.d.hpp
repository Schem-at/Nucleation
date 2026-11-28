#ifndef FieldDistanceKind_D_HPP
#define FieldDistanceKind_D_HPP

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
    enum FieldDistanceKind {
      FieldDistanceKind_Exact = 0,
      FieldDistanceKind_LowerBound = 1,
      FieldDistanceKind_Estimate = 2,
      FieldDistanceKind_Implicit = 3,
    };

    typedef struct FieldDistanceKind_option {union { FieldDistanceKind ok; }; bool is_ok; } FieldDistanceKind_option;
} // namespace capi
} // namespace

/**
 * Distance semantics of a {@link Field}.
 */
class FieldDistanceKind {
public:
    enum Value {
        Exact = 0,
        LowerBound = 1,
        Estimate = 2,
        Implicit = 3,
    };

    FieldDistanceKind(): value(Value::Exact) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldDistanceKind(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::FieldDistanceKind AsFFI() const;
    inline static FieldDistanceKind FromFFI(diplomat::capi::FieldDistanceKind c_enum);
private:
    Value value;
};


#endif // FieldDistanceKind_D_HPP
