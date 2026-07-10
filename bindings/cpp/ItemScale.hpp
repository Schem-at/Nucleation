#ifndef ItemScale_HPP
#define ItemScale_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace


inline diplomat::capi::ItemScale ItemScale::AsFFI() const {
    return diplomat::capi::ItemScale {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline ItemScale ItemScale::FromFFI(diplomat::capi::ItemScale c_struct) {
    return ItemScale {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // ItemScale_HPP
