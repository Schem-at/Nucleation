#ifndef NUCLEATION_BuildAnimation_D_HPP
#define NUCLEATION_BuildAnimation_D_HPP

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
namespace capi { struct AnimationEffect; }
class AnimationEffect;
namespace capi { struct Brush; }
class Brush;
namespace capi { struct BuildAnimation; }
class BuildAnimation;
namespace capi { struct RenderConfig; }
class RenderConfig;
namespace capi { struct Shape; }
class Shape;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct BuildAnimation;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A schematic wrapper that records construction calls as animation targets.
 */
class BuildAnimation {
public:

  inline static std::unique_ptr<nucleation::BuildAnimation> create(std::string_view name);

  inline void set_default_effect(const nucleation::AnimationEffect& effect);

  /**
   * Apply an effect to exactly the next recorded operation or explicit group.
   * The returned borrowed builder enables fluent calls in every generated binding.
   */
  inline nucleation::BuildAnimation& with_effect(const nucleation::AnimationEffect& effect);

  inline void set_step_ms(float step_ms);

  inline void set_stagger_total_ms(float total_ms);

  inline void clear_stagger();

  /**
   * Shift every construction group's start time. Negative offsets let a
   * repeating staggered effect cross the beginning of a loop capture.
   */
  inline void set_stagger_offset_ms(float offset_ms);

  /**
   * Capture exactly one loop period, excluding the duplicate endpoint.
   * The rounded frame count evenly partitions the complete period.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_loop_period_ms(float period_ms);

  inline void clear_loop_period();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> begin_group();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> begin_keyed_group(float key);

  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> end_group();

  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> set_block(int32_t x, int32_t y, int32_t z, std::string_view block);

  /**
   * Fill a parametric shape and record its voxels as ordered groups in
   * the same transactional construction operation.
   */
  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> fill_along_parameter(const nucleation::Shape& shape, const nucleation::Brush& brush, uint32_t group_count);

  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material);

  inline void animate_camera(const nucleation::AnimationEffect& effect, float offset_ms);

  inline uint32_t frame_count(double fps, float hold_ms) const;

  /**
   * Render directly to a looping GIF. The renderer, meshes, timeline and
   * GIF encoder all live in the Rust core; no ffmpeg subprocess is needed.
   */
  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> render_gif(nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, std::string_view path, double fps, float hold_ms) const;

  /**
   * Render numbered PNG frames (`prefix0000.png`, ...) for an external
   * compositor while using the exact same public timeline API.
   */
  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> render_frames(nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, std::string_view prefix, double fps, float hold_ms) const;

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save_to_file(std::string_view path) const;

  inline uint32_t group_count() const;

  inline float duration_ms() const;

    inline const nucleation::capi::BuildAnimation* AsFFI() const;
    inline nucleation::capi::BuildAnimation* AsFFI();
    inline static const nucleation::BuildAnimation* FromFFI(const nucleation::capi::BuildAnimation* ptr);
    inline static nucleation::BuildAnimation* FromFFI(nucleation::capi::BuildAnimation* ptr);
    inline static void operator delete(void* ptr);
private:
    BuildAnimation() = delete;
    BuildAnimation(const nucleation::BuildAnimation&) = delete;
    BuildAnimation(nucleation::BuildAnimation&&) noexcept = delete;
    BuildAnimation operator=(const nucleation::BuildAnimation&) = delete;
    BuildAnimation operator=(nucleation::BuildAnimation&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_BuildAnimation_D_HPP
