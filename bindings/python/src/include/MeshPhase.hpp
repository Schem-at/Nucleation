#ifndef NUCLEATION_MeshPhase_HPP
#define NUCLEATION_MeshPhase_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::MeshPhase nucleation::MeshPhase::AsFFI() const {
    return static_cast<nucleation::capi::MeshPhase>(value);
}

inline nucleation::MeshPhase nucleation::MeshPhase::FromFFI(nucleation::capi::MeshPhase c_enum) {
    switch (c_enum) {
        case nucleation::capi::MeshPhase_BuildingAtlas:
        case nucleation::capi::MeshPhase_MeshingChunks:
        case nucleation::capi::MeshPhase_Complete:
        case nucleation::capi::MeshPhase_Failed:
            return static_cast<nucleation::MeshPhase::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_MeshPhase_HPP
