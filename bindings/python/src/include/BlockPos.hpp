#ifndef NUCLEATION_BlockPos_HPP
#define NUCLEATION_BlockPos_HPP

#include "BlockPos.d.hpp"

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


inline nucleation::capi::BlockPos nucleation::BlockPos::AsFFI() const {
    return nucleation::capi::BlockPos {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline nucleation::BlockPos nucleation::BlockPos::FromFFI(nucleation::capi::BlockPos c_struct) {
    return nucleation::BlockPos {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // NUCLEATION_BlockPos_HPP
