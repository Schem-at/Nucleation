#ifndef SdfCellMode_HPP
#define SdfCellMode_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace

inline diplomat::capi::SdfCellMode SdfCellMode::AsFFI() const {
    return static_cast<diplomat::capi::SdfCellMode>(value);
}

inline SdfCellMode SdfCellMode::FromFFI(diplomat::capi::SdfCellMode c_enum) {
    switch (c_enum) {
        case diplomat::capi::SdfCellMode_F1:
        case diplomat::capi::SdfCellMode_F2:
        case diplomat::capi::SdfCellMode_F2MinusF1:
        case diplomat::capi::SdfCellMode_CellValue:
            return static_cast<SdfCellMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // SdfCellMode_HPP
