#ifndef NUCLEATION_ItemScale_HPP
#define NUCLEATION_ItemScale_HPP

#include "ItemScale.d.hpp"

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


inline nucleation::capi::ItemScale nucleation::ItemScale::AsFFI() const {
    return nucleation::capi::ItemScale {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline nucleation::ItemScale nucleation::ItemScale::FromFFI(nucleation::capi::ItemScale c_struct) {
    return nucleation::ItemScale {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // NUCLEATION_ItemScale_HPP
