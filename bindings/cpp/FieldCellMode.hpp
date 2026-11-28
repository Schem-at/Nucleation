#ifndef FieldCellMode_HPP
#define FieldCellMode_HPP

#include "FieldCellMode.d.hpp"

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

inline diplomat::capi::FieldCellMode FieldCellMode::AsFFI() const {
    return static_cast<diplomat::capi::FieldCellMode>(value);
}

inline FieldCellMode FieldCellMode::FromFFI(diplomat::capi::FieldCellMode c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldCellMode_F1:
        case diplomat::capi::FieldCellMode_F2:
        case diplomat::capi::FieldCellMode_F2MinusF1:
        case diplomat::capi::FieldCellMode_CellValue:
            return static_cast<FieldCellMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldCellMode_HPP
