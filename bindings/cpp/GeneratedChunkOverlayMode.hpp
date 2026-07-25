#ifndef GeneratedChunkOverlayMode_HPP
#define GeneratedChunkOverlayMode_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace

inline diplomat::capi::GeneratedChunkOverlayMode GeneratedChunkOverlayMode::AsFFI() const {
    return static_cast<diplomat::capi::GeneratedChunkOverlayMode>(value);
}

inline GeneratedChunkOverlayMode GeneratedChunkOverlayMode::FromFFI(diplomat::capi::GeneratedChunkOverlayMode c_enum) {
    switch (c_enum) {
        case diplomat::capi::GeneratedChunkOverlayMode_Replace:
        case diplomat::capi::GeneratedChunkOverlayMode_KeepExisting:
            return static_cast<GeneratedChunkOverlayMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // GeneratedChunkOverlayMode_HPP
