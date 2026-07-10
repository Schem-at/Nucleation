#ifndef ItemScale_D_HPP
#define ItemScale_D_HPP

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
    struct ItemScale {
      float x;
      float y;
      float z;
    };

    typedef struct ItemScale_option {union { ItemScale ok; }; bool is_ok; } ItemScale_option;
} // namespace capi
} // namespace


/**
 * Non-uniform model scale factors.
 */
struct ItemScale {
    float x;
    float y;
    float z;

    inline diplomat::capi::ItemScale AsFFI() const;
    inline static ItemScale FromFFI(diplomat::capi::ItemScale c_struct);
};


#endif // ItemScale_D_HPP
