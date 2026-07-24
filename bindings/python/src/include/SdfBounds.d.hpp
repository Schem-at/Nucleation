#ifndef NUCLEATION_SdfBounds_D_HPP
#define NUCLEATION_SdfBounds_D_HPP

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


namespace nucleation {
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

    inline nucleation::capi::SdfBounds AsFFI() const;
    inline static nucleation::SdfBounds FromFFI(nucleation::capi::SdfBounds c_struct);
};

} // namespace
#endif // NUCLEATION_SdfBounds_D_HPP
