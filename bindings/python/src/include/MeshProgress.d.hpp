#ifndef NUCLEATION_MeshProgress_D_HPP
#define NUCLEATION_MeshProgress_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "MeshPhase.d.hpp"
#include "diplomat_runtime.hpp"
namespace nucleation {
class MeshPhase;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct MeshProgress {
      nucleation::capi::MeshPhase phase;
      uint32_t current;
      uint32_t total;
    };

    typedef struct MeshProgress_option {union { MeshProgress ok; }; bool is_ok; } MeshProgress_option;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * Snapshot of a {@link MeshJob}'s progress.
 */
struct MeshProgress {
    nucleation::MeshPhase phase;
    uint32_t current;
    uint32_t total;

    inline nucleation::capi::MeshProgress AsFFI() const;
    inline static nucleation::MeshProgress FromFFI(nucleation::capi::MeshProgress c_struct);
};

} // namespace
#endif // NUCLEATION_MeshProgress_D_HPP
