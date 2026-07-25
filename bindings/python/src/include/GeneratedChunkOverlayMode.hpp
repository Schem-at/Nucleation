#ifndef NUCLEATION_GeneratedChunkOverlayMode_HPP
#define NUCLEATION_GeneratedChunkOverlayMode_HPP

#include "GeneratedChunkOverlayMode.d.hpp"

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

inline nucleation::capi::GeneratedChunkOverlayMode nucleation::GeneratedChunkOverlayMode::AsFFI() const {
    return static_cast<nucleation::capi::GeneratedChunkOverlayMode>(value);
}

inline nucleation::GeneratedChunkOverlayMode nucleation::GeneratedChunkOverlayMode::FromFFI(nucleation::capi::GeneratedChunkOverlayMode c_enum) {
    switch (c_enum) {
        case nucleation::capi::GeneratedChunkOverlayMode_Replace:
        case nucleation::capi::GeneratedChunkOverlayMode_KeepExisting:
            return static_cast<nucleation::GeneratedChunkOverlayMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_GeneratedChunkOverlayMode_HPP
