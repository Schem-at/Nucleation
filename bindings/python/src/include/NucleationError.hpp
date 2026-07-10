#ifndef NUCLEATION_NucleationError_HPP
#define NUCLEATION_NucleationError_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::NucleationError nucleation::NucleationError::AsFFI() const {
    return static_cast<nucleation::capi::NucleationError>(value);
}

inline nucleation::NucleationError nucleation::NucleationError::FromFFI(nucleation::capi::NucleationError c_enum) {
    switch (c_enum) {
        case nucleation::capi::NucleationError_NullArgument:
        case nucleation::capi::NucleationError_InvalidArgument:
        case nucleation::capi::NucleationError_Parse:
        case nucleation::capi::NucleationError_Serialize:
        case nucleation::capi::NucleationError_Io:
        case nucleation::capi::NucleationError_Lock:
        case nucleation::capi::NucleationError_Store:
        case nucleation::capi::NucleationError_Mesh:
        case nucleation::capi::NucleationError_Render:
        case nucleation::capi::NucleationError_Simulation:
        case nucleation::capi::NucleationError_AlreadyConsumed:
        case nucleation::capi::NucleationError_NotFound:
            return static_cast<nucleation::NucleationError::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_NucleationError_HPP
