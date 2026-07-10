#ifndef BlockPos_HPP
#define BlockPos_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace


inline diplomat::capi::BlockPos BlockPos::AsFFI() const {
    return diplomat::capi::BlockPos {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline BlockPos BlockPos::FromFFI(diplomat::capi::BlockPos c_struct) {
    return BlockPos {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // BlockPos_HPP
