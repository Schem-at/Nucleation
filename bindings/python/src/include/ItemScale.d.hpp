#ifndef NUCLEATION_ItemScale_D_HPP
#define NUCLEATION_ItemScale_D_HPP

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
    struct ItemScale {
      float x;
      float y;
      float z;
    };

    typedef struct ItemScale_option {union { ItemScale ok; }; bool is_ok; } ItemScale_option;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * Non-uniform model scale factors.
 */
struct ItemScale {
    float x;
    float y;
    float z;

    inline nucleation::capi::ItemScale AsFFI() const;
    inline static nucleation::ItemScale FromFFI(nucleation::capi::ItemScale c_struct);
};

} // namespace
#endif // NUCLEATION_ItemScale_D_HPP
