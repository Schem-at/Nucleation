#ifndef NUCLEATION_FieldBounds_D_HPP
#define NUCLEATION_FieldBounds_D_HPP

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


namespace nucleation {
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

    inline nucleation::capi::FieldBounds AsFFI() const;
    inline static nucleation::FieldBounds FromFFI(nucleation::capi::FieldBounds c_struct);
};

} // namespace
#endif // NUCLEATION_FieldBounds_D_HPP
