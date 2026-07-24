#ifndef NUCLEATION_SdfCellMode_HPP
#define NUCLEATION_SdfCellMode_HPP

#include "SdfCellMode.d.hpp"

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

inline nucleation::capi::SdfCellMode nucleation::SdfCellMode::AsFFI() const {
    return static_cast<nucleation::capi::SdfCellMode>(value);
}

inline nucleation::SdfCellMode nucleation::SdfCellMode::FromFFI(nucleation::capi::SdfCellMode c_enum) {
    switch (c_enum) {
        case nucleation::capi::SdfCellMode_F1:
        case nucleation::capi::SdfCellMode_F2:
        case nucleation::capi::SdfCellMode_F2MinusF1:
        case nucleation::capi::SdfCellMode_CellValue:
            return static_cast<nucleation::SdfCellMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_SdfCellMode_HPP
