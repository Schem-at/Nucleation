#ifndef NUCLEATION_RenderConfig_D_HPP
#define NUCLEATION_RenderConfig_D_HPP

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
namespace capi { struct RenderConfig; }
class RenderConfig;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct RenderConfig;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Camera / output configuration for rendering.
 */
class RenderConfig {
public:

  inline static std::unique_ptr<nucleation::RenderConfig> create(uint32_t width, uint32_t height);

  inline void set_yaw(float yaw);

  inline void set_pitch(float pitch);

  inline void set_zoom(float zoom);

  inline void set_fov(float fov);

  /**
   * Set a solid RGBA clear color (linear 0.0–1.0). Alpha < 1.0 yields a
   * transparent PNG. Ignored when HDRI is enabled.
   */
  inline void set_background(float r, float g, float b, float a);

  /**
   * Clear the custom background — revert to default sky / HDRI.
   */
  inline void clear_background();

  /**
   * Enable (`true`) or disable orthographic projection.
   */
  inline void set_orthographic(bool orthographic);

  /**
   * Configure a true isometric view: orthographic at yaw 45° /
   * pitch ≈35.264° (preserves the current width/height).
   */
  inline void set_isometric();

    inline const nucleation::capi::RenderConfig* AsFFI() const;
    inline nucleation::capi::RenderConfig* AsFFI();
    inline static const nucleation::RenderConfig* FromFFI(const nucleation::capi::RenderConfig* ptr);
    inline static nucleation::RenderConfig* FromFFI(nucleation::capi::RenderConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    RenderConfig() = delete;
    RenderConfig(const nucleation::RenderConfig&) = delete;
    RenderConfig(nucleation::RenderConfig&&) noexcept = delete;
    RenderConfig operator=(const nucleation::RenderConfig&) = delete;
    RenderConfig operator=(nucleation::RenderConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_RenderConfig_D_HPP
