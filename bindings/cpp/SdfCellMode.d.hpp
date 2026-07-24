#ifndef SdfCellMode_D_HPP
#define SdfCellMode_D_HPP

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
    enum SdfCellMode {
      SdfCellMode_F1 = 0,
      SdfCellMode_F2 = 1,
      SdfCellMode_F2MinusF1 = 2,
      SdfCellMode_CellValue = 3,
    };

    typedef struct SdfCellMode_option {union { SdfCellMode ok; }; bool is_ok; } SdfCellMode_option;
} // namespace capi
} // namespace

/**
 * Cellular/Worley field output.
 */
class SdfCellMode {
public:
    enum Value {
        F1 = 0,
        F2 = 1,
        F2MinusF1 = 2,
        CellValue = 3,
    };

    SdfCellMode(): value(Value::F1) {}

    // Implicit conversions between enum and ::Value
    constexpr SdfCellMode(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::SdfCellMode AsFFI() const;
    inline static SdfCellMode FromFFI(diplomat::capi::SdfCellMode c_enum);
private:
    Value value;
};


#endif // SdfCellMode_D_HPP
