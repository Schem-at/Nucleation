#ifndef MeshProgress_HPP
#define MeshProgress_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace


inline diplomat::capi::MeshProgress MeshProgress::AsFFI() const {
    return diplomat::capi::MeshProgress {
        /* .phase = */ phase.AsFFI(),
        /* .current = */ current,
        /* .total = */ total,
    };
}

inline MeshProgress MeshProgress::FromFFI(diplomat::capi::MeshProgress c_struct) {
    return MeshProgress {
        /* .phase = */ MeshPhase::FromFFI(c_struct.phase),
        /* .current = */ c_struct.current,
        /* .total = */ c_struct.total,
    };
}


#endif // MeshProgress_HPP
