#ifndef NUCLEATION_FieldRange_HPP
#define NUCLEATION_FieldRange_HPP

#include "FieldRange.d.hpp"

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


inline nucleation::capi::FieldRange nucleation::FieldRange::AsFFI() const {
    return nucleation::capi::FieldRange {
        /* .min = */ min,
        /* .max = */ max,
    };
}

inline nucleation::FieldRange nucleation::FieldRange::FromFFI(nucleation::capi::FieldRange c_struct) {
    return nucleation::FieldRange {
        /* .min = */ c_struct.min,
        /* .max = */ c_struct.max,
    };
}


#endif // NUCLEATION_FieldRange_HPP
