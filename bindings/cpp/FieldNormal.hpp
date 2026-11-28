#ifndef FieldNormal_HPP
#define FieldNormal_HPP

#include "FieldNormal.d.hpp"

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


inline diplomat::capi::FieldNormal FieldNormal::AsFFI() const {
    return diplomat::capi::FieldNormal {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline FieldNormal FieldNormal::FromFFI(diplomat::capi::FieldNormal c_struct) {
    return FieldNormal {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // FieldNormal_HPP
