#ifndef FieldBounds_HPP
#define FieldBounds_HPP

#include "FieldBounds.d.hpp"

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


inline diplomat::capi::FieldBounds FieldBounds::AsFFI() const {
    return diplomat::capi::FieldBounds {
        /* .min_x = */ min_x,
        /* .min_y = */ min_y,
        /* .min_z = */ min_z,
        /* .max_x = */ max_x,
        /* .max_y = */ max_y,
        /* .max_z = */ max_z,
    };
}

inline FieldBounds FieldBounds::FromFFI(diplomat::capi::FieldBounds c_struct) {
    return FieldBounds {
        /* .min_x = */ c_struct.min_x,
        /* .min_y = */ c_struct.min_y,
        /* .min_z = */ c_struct.min_z,
        /* .max_x = */ c_struct.max_x,
        /* .max_y = */ c_struct.max_y,
        /* .max_z = */ c_struct.max_z,
    };
}


#endif // FieldBounds_HPP
