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

  /**
   * Create a config with the given output size in pixels. Camera starts
   * at the defaults: yaw 45°, pitch 30°, zoom 1.0, fov 45°, perspective
   * projection, default sky background.
   */
  inline static std::unique_ptr<RenderConfig> create(uint32_t width, uint32_t height);

  /**
   * Set the camera yaw (horizontal orbit angle) in degrees. Default: 45.
   */
  inline void set_yaw(float yaw);

  /**
   * Set the camera pitch (downward tilt) in degrees. Default: 30.
   */
  inline void set_pitch(float pitch);

  /**
   * Set the zoom factor applied to the auto-fitted framing
   * (1.0 = frame the whole model; 2.0 = twice as close; 0.5 = twice
   * as far). Default: 1.0.
   */
  inline void set_zoom(float zoom);

  /**
   * Fit the camera to the model's bounding sphere instead of its
   * yaw-dependent silhouette. The sphere is rotation invariant, so
   * orbiting cameras (turntables) keep a constant distance instead
   * of pulsing as the silhouette changes. Frames slightly looser
   * than the default fit. Default: false.
   */
  inline void set_sphere_fit(bool sphere_fit);

  /**
   * Set the vertical field of view in degrees (perspective projection
   * only). Default: 45.
   */
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
   * Configure a one-block world grid. Models are centred on integer
   * schematic coordinates, so grid lines are placed on half-integer
   * block boundaries automatically.
   */
  inline void set_grid(int32_t half_extent, int32_t spacing, float plane_y, bool show_axes, float red, float green, float blue, float alpha);

  /**
   * Configure a compact grid fitted to half-integer block boundaries.
   */
  inline void set_fitted_grid(int32_t margin, int32_t spacing, float plane_y, bool show_axes, float red, float green, float blue, float alpha);

  inline void clear_grid();

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
