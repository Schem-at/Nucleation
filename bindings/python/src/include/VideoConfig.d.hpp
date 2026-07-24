#ifndef NUCLEATION_VideoConfig_D_HPP
#define NUCLEATION_VideoConfig_D_HPP

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
namespace capi { struct VideoConfig; }
class VideoConfig;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct VideoConfig;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Native FFmpeg video output preset.
 */
class VideoConfig {
public:

  /**
   * Alpha-preserving ProRes 4444 in a MOV container.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError> prores_4444(double fps);

  /**
   * H.264 in an MP4 or MOV container. H.264 does not preserve alpha.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError> h264(double fps);

  /**
   * Override the FFmpeg executable. The default resolves `ffmpeg` on PATH.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_ffmpeg_path(std::string_view path);

    inline const nucleation::capi::VideoConfig* AsFFI() const;
    inline nucleation::capi::VideoConfig* AsFFI();
    inline static const nucleation::VideoConfig* FromFFI(const nucleation::capi::VideoConfig* ptr);
    inline static nucleation::VideoConfig* FromFFI(nucleation::capi::VideoConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    VideoConfig() = delete;
    VideoConfig(const nucleation::VideoConfig&) = delete;
    VideoConfig(nucleation::VideoConfig&&) noexcept = delete;
    VideoConfig operator=(const nucleation::VideoConfig&) = delete;
    VideoConfig operator=(nucleation::VideoConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_VideoConfig_D_HPP
