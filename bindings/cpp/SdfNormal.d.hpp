#ifndef SdfNormal_D_HPP
#define SdfNormal_D_HPP

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
    struct SdfNormal {
      float x;
      float y;
      float z;
    };

    typedef struct SdfNormal_option {union { SdfNormal ok; }; bool is_ok; } SdfNormal_option;
} // namespace capi
} // namespace


/**
 * Unit surface normal estimated from the SDF gradient.
 */
struct SdfNormal {
    float x;
    float y;
    float z;

    inline diplomat::capi::SdfNormal AsFFI() const;
    inline static SdfNormal FromFFI(diplomat::capi::SdfNormal c_struct);
};


#endif // SdfNormal_D_HPP
