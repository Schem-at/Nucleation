#ifndef NUCLEATION_SdfNormal_D_HPP
#define NUCLEATION_SdfNormal_D_HPP

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
    struct SdfNormal {
      float x;
      float y;
      float z;
    };

    typedef struct SdfNormal_option {union { SdfNormal ok; }; bool is_ok; } SdfNormal_option;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * Unit surface normal estimated from the SDF gradient.
 */
struct SdfNormal {
    float x;
    float y;
    float z;

    inline nucleation::capi::SdfNormal AsFFI() const;
    inline static nucleation::SdfNormal FromFFI(nucleation::capi::SdfNormal c_struct);
};

} // namespace
#endif // NUCLEATION_SdfNormal_D_HPP
