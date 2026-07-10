#ifndef NUCLEATION_TextureInfo_D_HPP
#define NUCLEATION_TextureInfo_D_HPP

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
    struct TextureInfo {
      uint32_t width;
      uint32_t height;
      bool animated;
      uint32_t frame_count;
    };

    typedef struct TextureInfo_option {union { TextureInfo ok; }; bool is_ok; } TextureInfo_option;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * Texture size/animation metadata for one texture in a resource pack.
 */
struct TextureInfo {
    uint32_t width;
    uint32_t height;
    bool animated;
    uint32_t frame_count;

    inline nucleation::capi::TextureInfo AsFFI() const;
    inline static nucleation::TextureInfo FromFFI(nucleation::capi::TextureInfo c_struct);
};

} // namespace
#endif // NUCLEATION_TextureInfo_D_HPP
