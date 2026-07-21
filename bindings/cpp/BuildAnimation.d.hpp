#ifndef BuildAnimation_D_HPP
#define BuildAnimation_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct AnimationEffect; }
class AnimationEffect;
namespace diplomat::capi { struct RenderConfig; }
class RenderConfig;
class NucleationError;




namespace diplomat {
namespace capi {
    struct BuildAnimation;
} // namespace capi
} // namespace

/**
 * A schematic wrapper that records construction calls as animation targets.
 */
class BuildAnimation {
public:

  inline static std::unique_ptr<BuildAnimation> create(std::string_view name);

  inline void set_default_effect(const AnimationEffect& effect);

  /**
   * Apply an effect to exactly the next recorded operation or explicit group.
   * The returned borrowed builder enables fluent calls in every generated binding.
   */
  inline BuildAnimation& with_effect(const AnimationEffect& effect);

  inline void set_step_ms(float step_ms);

  inline void set_stagger_total_ms(float total_ms);

  inline void clear_stagger();

  inline diplomat::result<std::monostate, NucleationError> begin_group();

  inline diplomat::result<std::monostate, NucleationError> begin_keyed_group(float key);

  inline diplomat::result<uint32_t, NucleationError> end_group();

  inline diplomat::result<uint32_t, NucleationError> set_block(int32_t x, int32_t y, int32_t z, std::string_view block);

  inline diplomat::result<uint32_t, NucleationError> add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material);

  inline void animate_camera(const AnimationEffect& effect, float offset_ms);

  inline uint32_t frame_count(double fps, float hold_ms) const;

  /**
   * Render directly to a looping GIF. The renderer, meshes, timeline and
   * GIF encoder all live in the Rust core; no ffmpeg subprocess is needed.
   */
  inline diplomat::result<uint32_t, NucleationError> render_gif(diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, std::string_view path, double fps, float hold_ms) const;

  /**
   * Render numbered PNG frames (`prefix0000.png`, ...) for an external
   * compositor while using the exact same public timeline API.
   */
  inline diplomat::result<uint32_t, NucleationError> render_frames(diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, std::string_view prefix, double fps, float hold_ms) const;

  inline diplomat::result<std::monostate, NucleationError> save_to_file(std::string_view path) const;

  inline uint32_t group_count() const;

  inline float duration_ms() const;

    inline const diplomat::capi::BuildAnimation* AsFFI() const;
    inline diplomat::capi::BuildAnimation* AsFFI();
    inline static const BuildAnimation* FromFFI(const diplomat::capi::BuildAnimation* ptr);
    inline static BuildAnimation* FromFFI(diplomat::capi::BuildAnimation* ptr);
    inline static void operator delete(void* ptr);
private:
    BuildAnimation() = delete;
    BuildAnimation(const BuildAnimation&) = delete;
    BuildAnimation(BuildAnimation&&) noexcept = delete;
    BuildAnimation operator=(const BuildAnimation&) = delete;
    BuildAnimation operator=(BuildAnimation&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // BuildAnimation_D_HPP
