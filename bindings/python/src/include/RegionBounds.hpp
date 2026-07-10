#ifndef NUCLEATION_RegionBounds_HPP
#define NUCLEATION_RegionBounds_HPP

#include "RegionBounds.d.hpp"

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


inline nucleation::capi::RegionBounds nucleation::RegionBounds::AsFFI() const {
    return nucleation::capi::RegionBounds {
        /* .min_x = */ min_x,
        /* .min_y = */ min_y,
        /* .min_z = */ min_z,
        /* .max_x = */ max_x,
        /* .max_y = */ max_y,
        /* .max_z = */ max_z,
    };
}

inline nucleation::RegionBounds nucleation::RegionBounds::FromFFI(nucleation::capi::RegionBounds c_struct) {
    return nucleation::RegionBounds {
        /* .min_x = */ c_struct.min_x,
        /* .min_y = */ c_struct.min_y,
        /* .min_z = */ c_struct.min_z,
        /* .max_x = */ c_struct.max_x,
        /* .max_y = */ c_struct.max_y,
        /* .max_z = */ c_struct.max_z,
    };
}


#endif // NUCLEATION_RegionBounds_HPP
