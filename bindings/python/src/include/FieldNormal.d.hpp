#ifndef NUCLEATION_FieldNormal_D_HPP
#define NUCLEATION_FieldNormal_D_HPP

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
    struct FieldNormal {
      float x;
      float y;
      float z;
    };

    typedef struct FieldNormal_option {union { FieldNormal ok; }; bool is_ok; } FieldNormal_option;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * Unit surface normal estimated from the field gradient.
 */
struct FieldNormal {
    float x;
    float y;
    float z;

    inline nucleation::capi::FieldNormal AsFFI() const;
    inline static nucleation::FieldNormal FromFFI(nucleation::capi::FieldNormal c_struct);
};

} // namespace
#endif // NUCLEATION_FieldNormal_D_HPP
