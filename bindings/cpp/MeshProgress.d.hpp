#ifndef MeshProgress_D_HPP
#define MeshProgress_D_HPP

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

class MeshPhase;




namespace diplomat {
namespace capi {
    struct MeshProgress {
      diplomat::capi::MeshPhase phase;
      uint32_t current;
      uint32_t total;
    };

    typedef struct MeshProgress_option {union { MeshProgress ok; }; bool is_ok; } MeshProgress_option;
} // namespace capi
} // namespace


/**
 * Snapshot of a {@link MeshJob}'s progress.
 */
struct MeshProgress {
    MeshPhase phase;
    uint32_t current;
    uint32_t total;

    inline diplomat::capi::MeshProgress AsFFI() const;
    inline static MeshProgress FromFFI(diplomat::capi::MeshProgress c_struct);
};


#endif // MeshProgress_D_HPP
