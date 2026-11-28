#ifndef NUCLEATION_FieldBounds_HPP
#define NUCLEATION_FieldBounds_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace


inline nucleation::capi::FieldBounds nucleation::FieldBounds::AsFFI() const {
    return nucleation::capi::FieldBounds {
        /* .min_x = */ min_x,
        /* .min_y = */ min_y,
        /* .min_z = */ min_z,
        /* .max_x = */ max_x,
        /* .max_y = */ max_y,
        /* .max_z = */ max_z,
    };
}

inline nucleation::FieldBounds nucleation::FieldBounds::FromFFI(nucleation::capi::FieldBounds c_struct) {
    return nucleation::FieldBounds {
        /* .min_x = */ c_struct.min_x,
        /* .min_y = */ c_struct.min_y,
        /* .min_z = */ c_struct.min_z,
        /* .max_x = */ c_struct.max_x,
        /* .max_y = */ c_struct.max_y,
        /* .max_z = */ c_struct.max_z,
    };
}


#endif // NUCLEATION_FieldBounds_HPP
