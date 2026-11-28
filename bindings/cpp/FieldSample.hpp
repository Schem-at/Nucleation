#ifndef FieldSample_HPP
#define FieldSample_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace


inline diplomat::capi::FieldSample FieldSample::AsFFI() const {
    return diplomat::capi::FieldSample {
        /* .distance = */ distance,
        /* .material_id = */ material_id,
        /* .blend_material_id = */ blend_material_id,
        /* .blend_weight = */ blend_weight,
    };
}

inline FieldSample FieldSample::FromFFI(diplomat::capi::FieldSample c_struct) {
    return FieldSample {
        /* .distance = */ c_struct.distance,
        /* .material_id = */ c_struct.material_id,
        /* .blend_material_id = */ c_struct.blend_material_id,
        /* .blend_weight = */ c_struct.blend_weight,
    };
}


#endif // FieldSample_HPP
