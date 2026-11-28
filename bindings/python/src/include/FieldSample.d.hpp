#ifndef NUCLEATION_FieldSample_D_HPP
#define NUCLEATION_FieldSample_D_HPP

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
    struct FieldSample {
      float distance;
      uint16_t material_id;
      uint16_t blend_material_id;
      float blend_weight;
    };

    typedef struct FieldSample_option {union { FieldSample ok; }; bool is_ok; } FieldSample_option;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * Distance plus propagated CSG material tag and smooth seam blend weight.
 */
struct FieldSample {
    float distance;
    uint16_t material_id;
    uint16_t blend_material_id;
    float blend_weight;

    inline nucleation::capi::FieldSample AsFFI() const;
    inline static nucleation::FieldSample FromFFI(nucleation::capi::FieldSample c_struct);
};

} // namespace
#endif // NUCLEATION_FieldSample_D_HPP
