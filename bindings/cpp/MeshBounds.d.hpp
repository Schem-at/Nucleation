#ifndef MeshBounds_D_HPP
#define MeshBounds_D_HPP

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
    struct MeshBounds {
      float min_x;
      float min_y;
      float min_z;
      float max_x;
      float max_y;
      float max_z;
    };

    typedef struct MeshBounds_option {union { MeshBounds ok; }; bool is_ok; } MeshBounds_option;
} // namespace capi
} // namespace


/**
 * Axis-aligned bounding box of a mesh result.
 */
struct MeshBounds {
    float min_x;
    float min_y;
    float min_z;
    float max_x;
    float max_y;
    float max_z;

    inline diplomat::capi::MeshBounds AsFFI() const;
    inline static MeshBounds FromFFI(diplomat::capi::MeshBounds c_struct);
};


#endif // MeshBounds_D_HPP
