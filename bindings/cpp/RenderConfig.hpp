#ifndef RenderConfig_HPP
#define RenderConfig_HPP

#include "RenderConfig.d.hpp"

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
    extern "C" {

    diplomat::capi::RenderConfig* RenderConfig_create(uint32_t width, uint32_t height);

    void RenderConfig_set_yaw(diplomat::capi::RenderConfig* self, float yaw);

    void RenderConfig_set_pitch(diplomat::capi::RenderConfig* self, float pitch);

    void RenderConfig_set_zoom(diplomat::capi::RenderConfig* self, float zoom);

    void RenderConfig_set_sphere_fit(diplomat::capi::RenderConfig* self, bool sphere_fit);

    void RenderConfig_set_fov(diplomat::capi::RenderConfig* self, float fov);

    void RenderConfig_set_background(diplomat::capi::RenderConfig* self, float r, float g, float b, float a);

    void RenderConfig_clear_background(diplomat::capi::RenderConfig* self);

    void RenderConfig_set_orthographic(diplomat::capi::RenderConfig* self, bool orthographic);

    void RenderConfig_set_isometric(diplomat::capi::RenderConfig* self);

    void RenderConfig_destroy(RenderConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<RenderConfig> RenderConfig::create(uint32_t width, uint32_t height) {
    auto result = diplomat::capi::RenderConfig_create(width,
        height);
    return std::unique_ptr<RenderConfig>(RenderConfig::FromFFI(result));
}

inline void RenderConfig::set_yaw(float yaw) {
    diplomat::capi::RenderConfig_set_yaw(this->AsFFI(),
        yaw);
}

inline void RenderConfig::set_pitch(float pitch) {
    diplomat::capi::RenderConfig_set_pitch(this->AsFFI(),
        pitch);
}

inline void RenderConfig::set_zoom(float zoom) {
    diplomat::capi::RenderConfig_set_zoom(this->AsFFI(),
        zoom);
}

inline void RenderConfig::set_sphere_fit(bool sphere_fit) {
    diplomat::capi::RenderConfig_set_sphere_fit(this->AsFFI(),
        sphere_fit);
}

inline void RenderConfig::set_fov(float fov) {
    diplomat::capi::RenderConfig_set_fov(this->AsFFI(),
        fov);
}

inline void RenderConfig::set_background(float r, float g, float b, float a) {
    diplomat::capi::RenderConfig_set_background(this->AsFFI(),
        r,
        g,
        b,
        a);
}

inline void RenderConfig::clear_background() {
    diplomat::capi::RenderConfig_clear_background(this->AsFFI());
}

inline void RenderConfig::set_orthographic(bool orthographic) {
    diplomat::capi::RenderConfig_set_orthographic(this->AsFFI(),
        orthographic);
}

inline void RenderConfig::set_isometric() {
    diplomat::capi::RenderConfig_set_isometric(this->AsFFI());
}

inline const diplomat::capi::RenderConfig* RenderConfig::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::RenderConfig*>(this);
}

inline diplomat::capi::RenderConfig* RenderConfig::AsFFI() {
    return reinterpret_cast<diplomat::capi::RenderConfig*>(this);
}

inline const RenderConfig* RenderConfig::FromFFI(const diplomat::capi::RenderConfig* ptr) {
    return reinterpret_cast<const RenderConfig*>(ptr);
}

inline RenderConfig* RenderConfig::FromFFI(diplomat::capi::RenderConfig* ptr) {
    return reinterpret_cast<RenderConfig*>(ptr);
}

inline void RenderConfig::operator delete(void* ptr) {
    diplomat::capi::RenderConfig_destroy(reinterpret_cast<diplomat::capi::RenderConfig*>(ptr));
}


#endif // RenderConfig_HPP
