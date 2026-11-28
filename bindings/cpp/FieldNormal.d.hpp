#ifndef FieldNormal_D_HPP
#define FieldNormal_D_HPP

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
    struct FieldNormal {
      float x;
      float y;
      float z;
    };

    typedef struct FieldNormal_option {union { FieldNormal ok; }; bool is_ok; } FieldNormal_option;
} // namespace capi
} // namespace


/**
 * Unit surface normal estimated from the field gradient.
 */
struct FieldNormal {
    float x;
    float y;
    float z;

    inline diplomat::capi::FieldNormal AsFFI() const;
    inline static FieldNormal FromFFI(diplomat::capi::FieldNormal c_struct);
};


#endif // FieldNormal_D_HPP
