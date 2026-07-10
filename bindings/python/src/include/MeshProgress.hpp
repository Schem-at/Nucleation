#ifndef NUCLEATION_MeshProgress_HPP
#define NUCLEATION_MeshProgress_HPP

#include "MeshProgress.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "MeshPhase.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace


inline nucleation::capi::MeshProgress nucleation::MeshProgress::AsFFI() const {
    return nucleation::capi::MeshProgress {
        /* .phase = */ phase.AsFFI(),
        /* .current = */ current,
        /* .total = */ total,
    };
}

inline nucleation::MeshProgress nucleation::MeshProgress::FromFFI(nucleation::capi::MeshProgress c_struct) {
    return nucleation::MeshProgress {
        /* .phase = */ nucleation::MeshPhase::FromFFI(c_struct.phase),
        /* .current = */ c_struct.current,
        /* .total = */ c_struct.total,
    };
}


#endif // NUCLEATION_MeshProgress_HPP
