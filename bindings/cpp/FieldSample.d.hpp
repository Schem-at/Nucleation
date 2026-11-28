#ifndef FieldSample_D_HPP
#define FieldSample_D_HPP

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
    struct FieldSample {
      float distance;
      uint16_t material_id;
      uint16_t blend_material_id;
      float blend_weight;
    };

    typedef struct FieldSample_option {union { FieldSample ok; }; bool is_ok; } FieldSample_option;
} // namespace capi
} // namespace


/**
 * Distance plus propagated CSG material tag and smooth seam blend weight.
 */
struct FieldSample {
    float distance;
    uint16_t material_id;
    uint16_t blend_material_id;
    float blend_weight;

    inline diplomat::capi::FieldSample AsFFI() const;
    inline static FieldSample FromFFI(diplomat::capi::FieldSample c_struct);
};


#endif // FieldSample_D_HPP
