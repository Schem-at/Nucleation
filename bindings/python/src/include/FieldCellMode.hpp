#ifndef NUCLEATION_FieldCellMode_HPP
#define NUCLEATION_FieldCellMode_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::FieldCellMode nucleation::FieldCellMode::AsFFI() const {
    return static_cast<nucleation::capi::FieldCellMode>(value);
}

inline nucleation::FieldCellMode nucleation::FieldCellMode::FromFFI(nucleation::capi::FieldCellMode c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldCellMode_F1:
        case nucleation::capi::FieldCellMode_F2:
        case nucleation::capi::FieldCellMode_F2MinusF1:
        case nucleation::capi::FieldCellMode_CellValue:
            return static_cast<nucleation::FieldCellMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldCellMode_HPP
