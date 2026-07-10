#ifndef MeshPhase_HPP
#define MeshPhase_HPP

#include "MeshPhase.d.hpp"

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

inline diplomat::capi::MeshPhase MeshPhase::AsFFI() const {
    return static_cast<diplomat::capi::MeshPhase>(value);
}

inline MeshPhase MeshPhase::FromFFI(diplomat::capi::MeshPhase c_enum) {
    switch (c_enum) {
        case diplomat::capi::MeshPhase_BuildingAtlas:
        case diplomat::capi::MeshPhase_MeshingChunks:
        case diplomat::capi::MeshPhase_Complete:
        case diplomat::capi::MeshPhase_Failed:
            return static_cast<MeshPhase::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // MeshPhase_HPP
