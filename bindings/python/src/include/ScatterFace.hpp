#ifndef NUCLEATION_ScatterFace_HPP
#define NUCLEATION_ScatterFace_HPP

#include "ScatterFace.d.hpp"

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

inline nucleation::capi::ScatterFace nucleation::ScatterFace::AsFFI() const {
    return static_cast<nucleation::capi::ScatterFace>(value);
}

inline nucleation::ScatterFace nucleation::ScatterFace::FromFFI(nucleation::capi::ScatterFace c_enum) {
    switch (c_enum) {
        case nucleation::capi::ScatterFace_Top:
        case nucleation::capi::ScatterFace_Bottom:
        case nucleation::capi::ScatterFace_North:
        case nucleation::capi::ScatterFace_South:
        case nucleation::capi::ScatterFace_East:
        case nucleation::capi::ScatterFace_West:
            return static_cast<nucleation::ScatterFace::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_ScatterFace_HPP
