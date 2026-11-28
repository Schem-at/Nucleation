#ifndef ScatterFace_HPP
#define ScatterFace_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace

inline diplomat::capi::ScatterFace ScatterFace::AsFFI() const {
    return static_cast<diplomat::capi::ScatterFace>(value);
}

inline ScatterFace ScatterFace::FromFFI(diplomat::capi::ScatterFace c_enum) {
    switch (c_enum) {
        case diplomat::capi::ScatterFace_Top:
        case diplomat::capi::ScatterFace_Bottom:
        case diplomat::capi::ScatterFace_North:
        case diplomat::capi::ScatterFace_South:
        case diplomat::capi::ScatterFace_East:
        case diplomat::capi::ScatterFace_West:
            return static_cast<ScatterFace::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // ScatterFace_HPP
