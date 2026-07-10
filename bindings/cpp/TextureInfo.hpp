#ifndef TextureInfo_HPP
#define TextureInfo_HPP

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


namespace diplomat {
namespace capi {

} // namespace capi
} // namespace


inline diplomat::capi::TextureInfo TextureInfo::AsFFI() const {
    return diplomat::capi::TextureInfo {
        /* .width = */ width,
        /* .height = */ height,
        /* .animated = */ animated,
        /* .frame_count = */ frame_count,
    };
}

inline TextureInfo TextureInfo::FromFFI(diplomat::capi::TextureInfo c_struct) {
    return TextureInfo {
        /* .width = */ c_struct.width,
        /* .height = */ c_struct.height,
        /* .animated = */ c_struct.animated,
        /* .frame_count = */ c_struct.frame_count,
    };
}


#endif // TextureInfo_HPP
