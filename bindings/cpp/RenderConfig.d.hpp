#ifndef RenderConfig_D_HPP
#define RenderConfig_D_HPP

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
    struct RenderConfig;
} // namespace capi
} // namespace

/**
 * Camera / output configuration for rendering.
 */
class RenderConfig {
public:

  inline static std::unique_ptr<RenderConfig> create(uint32_t width, uint32_t height);

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

    inline const diplomat::capi::RenderConfig* AsFFI() const;
    inline diplomat::capi::RenderConfig* AsFFI();
    inline static const RenderConfig* FromFFI(const diplomat::capi::RenderConfig* ptr);
    inline static RenderConfig* FromFFI(diplomat::capi::RenderConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    RenderConfig() = delete;
    RenderConfig(const RenderConfig&) = delete;
    RenderConfig(RenderConfig&&) noexcept = delete;
    RenderConfig operator=(const RenderConfig&) = delete;
    RenderConfig operator=(RenderConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // RenderConfig_D_HPP
