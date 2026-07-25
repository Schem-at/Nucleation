#ifndef FieldRange_HPP
#define FieldRange_HPP

#include "FieldRange.d.hpp"

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


inline diplomat::capi::FieldRange FieldRange::AsFFI() const {
    return diplomat::capi::FieldRange {
        /* .min = */ min,
        /* .max = */ max,
    };
}

inline FieldRange FieldRange::FromFFI(diplomat::capi::FieldRange c_struct) {
    return FieldRange {
        /* .min = */ c_struct.min,
        /* .max = */ c_struct.max,
    };
}


#endif // FieldRange_HPP
