#ifndef NUCLEATION_SdfNormal_HPP
#define NUCLEATION_SdfNormal_HPP

#include "SdfNormal.d.hpp"

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


inline nucleation::capi::SdfNormal nucleation::SdfNormal::AsFFI() const {
    return nucleation::capi::SdfNormal {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline nucleation::SdfNormal nucleation::SdfNormal::FromFFI(nucleation::capi::SdfNormal c_struct) {
    return nucleation::SdfNormal {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // NUCLEATION_SdfNormal_HPP
