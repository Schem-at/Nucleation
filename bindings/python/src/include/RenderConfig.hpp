#ifndef NUCLEATION_RenderConfig_HPP
#define NUCLEATION_RenderConfig_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::RenderConfig* RenderConfig_create(uint32_t width, uint32_t height);

    void RenderConfig_set_yaw(nucleation::capi::RenderConfig* self, float yaw);

    void RenderConfig_set_pitch(nucleation::capi::RenderConfig* self, float pitch);

    void RenderConfig_set_zoom(nucleation::capi::RenderConfig* self, float zoom);

    void RenderConfig_set_sphere_fit(nucleation::capi::RenderConfig* self, bool sphere_fit);

    void RenderConfig_set_fov(nucleation::capi::RenderConfig* self, float fov);

    void RenderConfig_set_background(nucleation::capi::RenderConfig* self, float r, float g, float b, float a);

    void RenderConfig_clear_background(nucleation::capi::RenderConfig* self);

    void RenderConfig_set_orthographic(nucleation::capi::RenderConfig* self, bool orthographic);

    void RenderConfig_set_isometric(nucleation::capi::RenderConfig* self);

    void RenderConfig_destroy(RenderConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::RenderConfig> nucleation::RenderConfig::create(uint32_t width, uint32_t height) {
    auto result = nucleation::capi::RenderConfig_create(width,
        height);
    return std::unique_ptr<nucleation::RenderConfig>(nucleation::RenderConfig::FromFFI(result));
}

inline void nucleation::RenderConfig::set_yaw(float yaw) {
    nucleation::capi::RenderConfig_set_yaw(this->AsFFI(),
        yaw);
}

inline void nucleation::RenderConfig::set_pitch(float pitch) {
    nucleation::capi::RenderConfig_set_pitch(this->AsFFI(),
        pitch);
}

inline void nucleation::RenderConfig::set_zoom(float zoom) {
    nucleation::capi::RenderConfig_set_zoom(this->AsFFI(),
        zoom);
}

inline void nucleation::RenderConfig::set_sphere_fit(bool sphere_fit) {
    nucleation::capi::RenderConfig_set_sphere_fit(this->AsFFI(),
        sphere_fit);
}

inline void nucleation::RenderConfig::set_fov(float fov) {
    nucleation::capi::RenderConfig_set_fov(this->AsFFI(),
        fov);
}

inline void nucleation::RenderConfig::set_background(float r, float g, float b, float a) {
    nucleation::capi::RenderConfig_set_background(this->AsFFI(),
        r,
        g,
        b,
        a);
}

inline void nucleation::RenderConfig::clear_background() {
    nucleation::capi::RenderConfig_clear_background(this->AsFFI());
}

inline void nucleation::RenderConfig::set_orthographic(bool orthographic) {
    nucleation::capi::RenderConfig_set_orthographic(this->AsFFI(),
        orthographic);
}

inline void nucleation::RenderConfig::set_isometric() {
    nucleation::capi::RenderConfig_set_isometric(this->AsFFI());
}

inline const nucleation::capi::RenderConfig* nucleation::RenderConfig::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::RenderConfig*>(this);
}

inline nucleation::capi::RenderConfig* nucleation::RenderConfig::AsFFI() {
    return reinterpret_cast<nucleation::capi::RenderConfig*>(this);
}

inline const nucleation::RenderConfig* nucleation::RenderConfig::FromFFI(const nucleation::capi::RenderConfig* ptr) {
    return reinterpret_cast<const nucleation::RenderConfig*>(ptr);
}

inline nucleation::RenderConfig* nucleation::RenderConfig::FromFFI(nucleation::capi::RenderConfig* ptr) {
    return reinterpret_cast<nucleation::RenderConfig*>(ptr);
}

inline void nucleation::RenderConfig::operator delete(void* ptr) {
    nucleation::capi::RenderConfig_destroy(reinterpret_cast<nucleation::capi::RenderConfig*>(ptr));
}


#endif // NUCLEATION_RenderConfig_HPP
