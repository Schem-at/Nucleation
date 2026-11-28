#ifndef NUCLEATION_FieldSample_HPP
#define NUCLEATION_FieldSample_HPP

#include "FieldSample.d.hpp"

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

} // namespace capi
} // namespace


inline nucleation::capi::FieldSample nucleation::FieldSample::AsFFI() const {
    return nucleation::capi::FieldSample {
        /* .distance = */ distance,
        /* .material_id = */ material_id,
        /* .blend_material_id = */ blend_material_id,
        /* .blend_weight = */ blend_weight,
    };
}

inline nucleation::FieldSample nucleation::FieldSample::FromFFI(nucleation::capi::FieldSample c_struct) {
    return nucleation::FieldSample {
        /* .distance = */ c_struct.distance,
        /* .material_id = */ c_struct.material_id,
        /* .blend_material_id = */ c_struct.blend_material_id,
        /* .blend_weight = */ c_struct.blend_weight,
    };
}


#endif // NUCLEATION_FieldSample_HPP
