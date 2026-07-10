#ifndef NucleationError_HPP
#define NucleationError_HPP

#include "NucleationError.d.hpp"

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

inline diplomat::capi::NucleationError NucleationError::AsFFI() const {
    return static_cast<diplomat::capi::NucleationError>(value);
}

inline NucleationError NucleationError::FromFFI(diplomat::capi::NucleationError c_enum) {
    switch (c_enum) {
        case diplomat::capi::NucleationError_NullArgument:
        case diplomat::capi::NucleationError_InvalidArgument:
        case diplomat::capi::NucleationError_Parse:
        case diplomat::capi::NucleationError_Serialize:
        case diplomat::capi::NucleationError_Io:
        case diplomat::capi::NucleationError_Lock:
        case diplomat::capi::NucleationError_Store:
        case diplomat::capi::NucleationError_Mesh:
        case diplomat::capi::NucleationError_Render:
        case diplomat::capi::NucleationError_Simulation:
        case diplomat::capi::NucleationError_AlreadyConsumed:
        case diplomat::capi::NucleationError_NotFound:
            return static_cast<NucleationError::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NucleationError_HPP
