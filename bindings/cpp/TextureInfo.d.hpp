#ifndef TextureInfo_D_HPP
#define TextureInfo_D_HPP

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
    struct TextureInfo {
      uint32_t width;
      uint32_t height;
      bool animated;
      uint32_t frame_count;
    };

    typedef struct TextureInfo_option {union { TextureInfo ok; }; bool is_ok; } TextureInfo_option;
} // namespace capi
} // namespace


/**
 * Texture size/animation metadata for one texture in a resource pack.
 */
struct TextureInfo {
    uint32_t width;
    uint32_t height;
    bool animated;
    uint32_t frame_count;

    inline diplomat::capi::TextureInfo AsFFI() const;
    inline static TextureInfo FromFFI(diplomat::capi::TextureInfo c_struct);
};


#endif // TextureInfo_D_HPP
