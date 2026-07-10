#ifndef NUCLEATION_Dimensions_HPP
#define NUCLEATION_Dimensions_HPP

#include "Dimensions.d.hpp"

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


inline nucleation::capi::Dimensions nucleation::Dimensions::AsFFI() const {
    return nucleation::capi::Dimensions {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline nucleation::Dimensions nucleation::Dimensions::FromFFI(nucleation::capi::Dimensions c_struct) {
    return nucleation::Dimensions {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // NUCLEATION_Dimensions_HPP
