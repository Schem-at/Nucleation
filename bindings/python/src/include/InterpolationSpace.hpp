#ifndef NUCLEATION_InterpolationSpace_HPP
#define NUCLEATION_InterpolationSpace_HPP

#include "InterpolationSpace.d.hpp"

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

inline nucleation::capi::InterpolationSpace nucleation::InterpolationSpace::AsFFI() const {
    return static_cast<nucleation::capi::InterpolationSpace>(value);
}

inline nucleation::InterpolationSpace nucleation::InterpolationSpace::FromFFI(nucleation::capi::InterpolationSpace c_enum) {
    switch (c_enum) {
        case nucleation::capi::InterpolationSpace_Rgb:
        case nucleation::capi::InterpolationSpace_Oklab:
            return static_cast<nucleation::InterpolationSpace::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_InterpolationSpace_HPP
