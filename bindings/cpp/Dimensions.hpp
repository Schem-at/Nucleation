#ifndef Dimensions_HPP
#define Dimensions_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace


inline diplomat::capi::Dimensions Dimensions::AsFFI() const {
    return diplomat::capi::Dimensions {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline Dimensions Dimensions::FromFFI(diplomat::capi::Dimensions c_struct) {
    return Dimensions {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // Dimensions_HPP
