#ifndef NUCLEATION_FieldNormal_HPP
#define NUCLEATION_FieldNormal_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace


inline nucleation::capi::FieldNormal nucleation::FieldNormal::AsFFI() const {
    return nucleation::capi::FieldNormal {
        /* .x = */ x,
        /* .y = */ y,
        /* .z = */ z,
    };
}

inline nucleation::FieldNormal nucleation::FieldNormal::FromFFI(nucleation::capi::FieldNormal c_struct) {
    return nucleation::FieldNormal {
        /* .x = */ c_struct.x,
        /* .y = */ c_struct.y,
        /* .z = */ c_struct.z,
    };
}


#endif // NUCLEATION_FieldNormal_HPP
