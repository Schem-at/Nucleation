#ifndef VideoConfig_D_HPP
#define VideoConfig_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct VideoConfig;
} // namespace capi
} // namespace

/**
 * Native FFmpeg video output preset.
 */
class VideoConfig {
public:

  /**
   * Alpha-preserving ProRes 4444 in a MOV container.
   */
  inline static diplomat::result<std::unique_ptr<VideoConfig>, NucleationError> prores_4444(double fps);

  /**
   * H.264 in an MP4 or MOV container. H.264 does not preserve alpha.
   */
  inline static diplomat::result<std::unique_ptr<VideoConfig>, NucleationError> h264(double fps);

  /**
   * Override the FFmpeg executable. The default resolves `ffmpeg` on PATH.
   */
  inline diplomat::result<std::monostate, NucleationError> set_ffmpeg_path(std::string_view path);

    inline const diplomat::capi::VideoConfig* AsFFI() const;
    inline diplomat::capi::VideoConfig* AsFFI();
    inline static const VideoConfig* FromFFI(const diplomat::capi::VideoConfig* ptr);
    inline static VideoConfig* FromFFI(diplomat::capi::VideoConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    VideoConfig() = delete;
    VideoConfig(const VideoConfig&) = delete;
    VideoConfig(VideoConfig&&) noexcept = delete;
    VideoConfig operator=(const VideoConfig&) = delete;
    VideoConfig operator=(VideoConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // VideoConfig_D_HPP
