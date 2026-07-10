#ifndef NUCLEATION_MeshBounds_D_HPP
#define NUCLEATION_MeshBounds_D_HPP

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


namespace nucleation {
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

    inline nucleation::capi::MeshBounds AsFFI() const;
    inline static nucleation::MeshBounds FromFFI(nucleation::capi::MeshBounds c_struct);
};

} // namespace
#endif // NUCLEATION_MeshBounds_D_HPP
