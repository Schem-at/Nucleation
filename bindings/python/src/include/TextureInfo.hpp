#ifndef NUCLEATION_TextureInfo_HPP
#define NUCLEATION_TextureInfo_HPP

#include "TextureInfo.d.hpp"

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


inline nucleation::capi::TextureInfo nucleation::TextureInfo::AsFFI() const {
    return nucleation::capi::TextureInfo {
        /* .width = */ width,
        /* .height = */ height,
        /* .animated = */ animated,
        /* .frame_count = */ frame_count,
    };
}

inline nucleation::TextureInfo nucleation::TextureInfo::FromFFI(nucleation::capi::TextureInfo c_struct) {
    return nucleation::TextureInfo {
        /* .width = */ c_struct.width,
        /* .height = */ c_struct.height,
        /* .animated = */ c_struct.animated,
        /* .frame_count = */ c_struct.frame_count,
    };
}


#endif // NUCLEATION_TextureInfo_HPP
