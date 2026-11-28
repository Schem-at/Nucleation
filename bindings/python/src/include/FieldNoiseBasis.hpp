#ifndef NUCLEATION_FieldNoiseBasis_HPP
#define NUCLEATION_FieldNoiseBasis_HPP

#include "FieldNoiseBasis.d.hpp"

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

inline nucleation::capi::FieldNoiseBasis nucleation::FieldNoiseBasis::AsFFI() const {
    return static_cast<nucleation::capi::FieldNoiseBasis>(value);
}

inline nucleation::FieldNoiseBasis nucleation::FieldNoiseBasis::FromFFI(nucleation::capi::FieldNoiseBasis c_enum) {
    switch (c_enum) {
        case nucleation::capi::FieldNoiseBasis_ValueNoise:
        case nucleation::capi::FieldNoiseBasis_Perlin:
            return static_cast<nucleation::FieldNoiseBasis::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_FieldNoiseBasis_HPP
