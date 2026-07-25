#ifndef FieldRange_D_HPP
#define FieldRange_D_HPP

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
    struct FieldRange {
      float min;
      float max;
    };

    typedef struct FieldRange_option {union { FieldRange ok; }; bool is_ok; } FieldRange_option;
} // namespace capi
} // namespace


/**
 * The closed interval a field's values are analytically proven to lie in.
 */
struct FieldRange {
    float min;
    float max;

    inline diplomat::capi::FieldRange AsFFI() const;
    inline static FieldRange FromFFI(diplomat::capi::FieldRange c_struct);
};


#endif // FieldRange_D_HPP
