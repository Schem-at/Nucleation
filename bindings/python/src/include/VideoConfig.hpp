#ifndef NUCLEATION_VideoConfig_HPP
#define NUCLEATION_VideoConfig_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct VideoConfig_prores_4444_result {union {nucleation::capi::VideoConfig* ok; nucleation::capi::NucleationError err;}; bool is_ok;} VideoConfig_prores_4444_result;
    VideoConfig_prores_4444_result VideoConfig_prores_4444(double fps);

    typedef struct VideoConfig_h264_result {union {nucleation::capi::VideoConfig* ok; nucleation::capi::NucleationError err;}; bool is_ok;} VideoConfig_h264_result;
    VideoConfig_h264_result VideoConfig_h264(double fps);

    typedef struct VideoConfig_set_ffmpeg_path_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} VideoConfig_set_ffmpeg_path_result;
    VideoConfig_set_ffmpeg_path_result VideoConfig_set_ffmpeg_path(nucleation::capi::VideoConfig* self, nucleation::diplomat::capi::DiplomatStringView path);

    void VideoConfig_destroy(VideoConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError> nucleation::VideoConfig::prores_4444(double fps) {
    auto result = nucleation::capi::VideoConfig_prores_4444(fps);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::VideoConfig>>(std::unique_ptr<nucleation::VideoConfig>(nucleation::VideoConfig::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError> nucleation::VideoConfig::h264(double fps) {
    auto result = nucleation::capi::VideoConfig_h264(fps);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::VideoConfig>>(std::unique_ptr<nucleation::VideoConfig>(nucleation::VideoConfig::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::VideoConfig>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::VideoConfig::set_ffmpeg_path(std::string_view path) {
    auto result = nucleation::capi::VideoConfig_set_ffmpeg_path(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::VideoConfig* nucleation::VideoConfig::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::VideoConfig*>(this);
}

inline nucleation::capi::VideoConfig* nucleation::VideoConfig::AsFFI() {
    return reinterpret_cast<nucleation::capi::VideoConfig*>(this);
}

inline const nucleation::VideoConfig* nucleation::VideoConfig::FromFFI(const nucleation::capi::VideoConfig* ptr) {
    return reinterpret_cast<const nucleation::VideoConfig*>(ptr);
}

inline nucleation::VideoConfig* nucleation::VideoConfig::FromFFI(nucleation::capi::VideoConfig* ptr) {
    return reinterpret_cast<nucleation::VideoConfig*>(ptr);
}

inline void nucleation::VideoConfig::operator delete(void* ptr) {
    nucleation::capi::VideoConfig_destroy(reinterpret_cast<nucleation::capi::VideoConfig*>(ptr));
}


#endif // NUCLEATION_VideoConfig_HPP
