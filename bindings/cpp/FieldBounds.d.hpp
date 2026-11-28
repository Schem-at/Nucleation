#ifndef FieldBounds_D_HPP
#define FieldBounds_D_HPP

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
    struct FieldBounds {
      float min_x;
      float min_y;
      float min_z;
      float max_x;
      float max_y;
      float max_z;
    };

    typedef struct FieldBounds_option {union { FieldBounds ok; }; bool is_ok; } FieldBounds_option;
} // namespace capi
} // namespace


/**
 * Conservative continuous bounds of a bounded {@link Field}.
 */
struct FieldBounds {
    float min_x;
    float min_y;
    float min_z;
    float max_x;
    float max_y;
    float max_z;

    inline diplomat::capi::FieldBounds AsFFI() const;
    inline static FieldBounds FromFFI(diplomat::capi::FieldBounds c_struct);
};


#endif // FieldBounds_D_HPP
