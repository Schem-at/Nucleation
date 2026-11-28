#ifndef NUCLEATION_FieldCellMode_D_HPP
#define NUCLEATION_FieldCellMode_D_HPP

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
    enum FieldCellMode {
      FieldCellMode_F1 = 0,
      FieldCellMode_F2 = 1,
      FieldCellMode_F2MinusF1 = 2,
      FieldCellMode_CellValue = 3,
    };

    typedef struct FieldCellMode_option {union { FieldCellMode ok; }; bool is_ok; } FieldCellMode_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Cellular / Worley noise output mode.
 */
class FieldCellMode {
public:
    enum Value {
        F1 = 0,
        F2 = 1,
        F2MinusF1 = 2,
        CellValue = 3,
    };

    FieldCellMode(): value(Value::F1) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldCellMode(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::FieldCellMode AsFFI() const;
    inline static nucleation::FieldCellMode FromFFI(nucleation::capi::FieldCellMode c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_FieldCellMode_D_HPP
