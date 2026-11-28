#ifndef FieldNoiseBasis_HPP
#define FieldNoiseBasis_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace

inline diplomat::capi::FieldNoiseBasis FieldNoiseBasis::AsFFI() const {
    return static_cast<diplomat::capi::FieldNoiseBasis>(value);
}

inline FieldNoiseBasis FieldNoiseBasis::FromFFI(diplomat::capi::FieldNoiseBasis c_enum) {
    switch (c_enum) {
        case diplomat::capi::FieldNoiseBasis_ValueNoise:
        case diplomat::capi::FieldNoiseBasis_Perlin:
            return static_cast<FieldNoiseBasis::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // FieldNoiseBasis_HPP
