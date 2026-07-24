#ifndef SdfBounds_D_HPP
#define SdfBounds_D_HPP

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
    struct SdfBounds {
      float min_x;
      float min_y;
      float min_z;
      float max_x;
      float max_y;
      float max_z;
    };

    typedef struct SdfBounds_option {union { SdfBounds ok; }; bool is_ok; } SdfBounds_option;
} // namespace capi
} // namespace


/**
 * Continuous bounds of a bounded SDF graph.
 */
struct SdfBounds {
    float min_x;
    float min_y;
    float min_z;
    float max_x;
    float max_y;
    float max_z;

    inline diplomat::capi::SdfBounds AsFFI() const;
    inline static SdfBounds FromFFI(diplomat::capi::SdfBounds c_struct);
};


#endif // SdfBounds_D_HPP
