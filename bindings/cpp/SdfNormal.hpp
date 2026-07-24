#ifndef SdfNormal_HPP
#define SdfNormal_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace


inline diplomat::capi::SdfNormal SdfNormal::AsFFI() const {
    return diplomat::capi::SdfNormal {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline SdfNormal SdfNormal::FromFFI(diplomat::capi::SdfNormal c_struct) {
    return SdfNormal {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // SdfNormal_HPP
