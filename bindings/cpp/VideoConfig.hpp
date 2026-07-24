#ifndef VideoConfig_HPP
#define VideoConfig_HPP

#include "VideoConfig.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct VideoConfig_prores_4444_result {union {diplomat::capi::VideoConfig* ok; diplomat::capi::NucleationError err;}; bool is_ok;} VideoConfig_prores_4444_result;
    VideoConfig_prores_4444_result VideoConfig_prores_4444(double fps);

    typedef struct VideoConfig_h264_result {union {diplomat::capi::VideoConfig* ok; diplomat::capi::NucleationError err;}; bool is_ok;} VideoConfig_h264_result;
    VideoConfig_h264_result VideoConfig_h264(double fps);

    typedef struct VideoConfig_set_ffmpeg_path_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} VideoConfig_set_ffmpeg_path_result;
    VideoConfig_set_ffmpeg_path_result VideoConfig_set_ffmpeg_path(diplomat::capi::VideoConfig* self, diplomat::capi::DiplomatStringView path);

    void VideoConfig_destroy(VideoConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<VideoConfig>, NucleationError> VideoConfig::prores_4444(double fps) {
    auto result = diplomat::capi::VideoConfig_prores_4444(fps);
    return result.is_ok ? diplomat::result<std::unique_ptr<VideoConfig>, NucleationError>(diplomat::Ok<std::unique_ptr<VideoConfig>>(std::unique_ptr<VideoConfig>(VideoConfig::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<VideoConfig>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<VideoConfig>, NucleationError> VideoConfig::h264(double fps) {
    auto result = diplomat::capi::VideoConfig_h264(fps);
    return result.is_ok ? diplomat::result<std::unique_ptr<VideoConfig>, NucleationError>(diplomat::Ok<std::unique_ptr<VideoConfig>>(std::unique_ptr<VideoConfig>(VideoConfig::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<VideoConfig>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> VideoConfig::set_ffmpeg_path(std::string_view path) {
    auto result = diplomat::capi::VideoConfig_set_ffmpeg_path(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::VideoConfig* VideoConfig::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::VideoConfig*>(this);
}

inline diplomat::capi::VideoConfig* VideoConfig::AsFFI() {
    return reinterpret_cast<diplomat::capi::VideoConfig*>(this);
}

inline const VideoConfig* VideoConfig::FromFFI(const diplomat::capi::VideoConfig* ptr) {
    return reinterpret_cast<const VideoConfig*>(ptr);
}

inline VideoConfig* VideoConfig::FromFFI(diplomat::capi::VideoConfig* ptr) {
    return reinterpret_cast<VideoConfig*>(ptr);
}

inline void VideoConfig::operator delete(void* ptr) {
    diplomat::capi::VideoConfig_destroy(reinterpret_cast<diplomat::capi::VideoConfig*>(ptr));
}


#endif // VideoConfig_HPP
