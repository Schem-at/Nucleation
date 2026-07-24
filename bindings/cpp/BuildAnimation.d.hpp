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
namespace diplomat::capi { struct Brush; }
class Brush;
namespace diplomat::capi { struct RenderConfig; }
class RenderConfig;
namespace diplomat::capi { struct ResourcePack; }
class ResourcePack;
namespace diplomat::capi { struct Schematic; }
class Schematic;
namespace diplomat::capi { struct Shape; }
class Shape;
namespace diplomat::capi { struct VideoConfig; }
class VideoConfig;
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

  /**
   * Clone an existing schematic into one animation group. Entity-only
   * schematics are supported; overlapping block/entity integer positions
   * are rejected rather than silently dropping a model.
   */
  inline static diplomat::result<std::unique_ptr<BuildAnimation>, NucleationError> from_schematic(const Schematic& schematic);

  /**
   * Apply one effect to every existing animation group.
   */
  inline void animate_all(const AnimationEffect& effect);

  inline void set_default_effect(const AnimationEffect& effect);

  /**
   * Apply an effect to exactly the next recorded operation or explicit group.
   * The returned borrowed builder enables fluent calls in every generated binding.
   */
  inline BuildAnimation& with_effect(const AnimationEffect& effect);

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
  inline diplomat::result<std::monostate, NucleationError> set_loop_period_ms(float period_ms);

  inline void clear_loop_period();

  inline diplomat::result<std::monostate, NucleationError> begin_group();

  inline diplomat::result<std::monostate, NucleationError> begin_keyed_group(float key);

  inline diplomat::result<uint32_t, NucleationError> end_group();

  inline diplomat::result<uint32_t, NucleationError> set_block(int32_t x, int32_t y, int32_t z, std::string_view block);

  inline diplomat::result<std::monostate, NucleationError> create_region(std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  inline diplomat::result<uint32_t, NucleationError> set_block_in_region(std::string_view region, int32_t x, int32_t y, int32_t z, std::string_view block);

  inline diplomat::result<std::monostate, NucleationError> translate(int32_t x, int32_t y, int32_t z, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> translate_region(std::string_view region, int32_t x, int32_t y, int32_t z, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> translate_all(int32_t x, int32_t y, int32_t z, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_x(int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_y(int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_z(int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_region_x(std::string_view region, int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_region_y(std::string_view region, int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_region_z(std::string_view region, int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_all_x(int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_all_y(int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> rotate_all_z(int32_t degrees, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_x(float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_y(float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_z(float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_region_x(std::string_view region, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_region_y(std::string_view region, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_region_z(std::string_view region, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_all_x(float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_all_y(float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> flip_all_z(float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> stamp_region(const Schematic& source, std::string_view region, int32_t x, int32_t y, int32_t z, std::string_view exclusions, float duration_ms);

  inline diplomat::result<std::monostate, NucleationError> stamp_box(const Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t x, int32_t y, int32_t z, std::string_view exclusions, float duration_ms);

  inline void set_operation_gizmos(bool enabled);

  inline diplomat::result<std::string, NucleationError> operations_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> operations_json_write(W& writeable_output) const;

  inline diplomat::result<std::string, NucleationError> frame_json(float time_ms) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> frame_json_write(float time_ms, W& writeable_output) const;

  /**
   * Fill a parametric shape and record its voxels as ordered groups in
   * the same transactional construction operation.
   */
  inline diplomat::result<uint32_t, NucleationError> fill_along_parameter(const Shape& shape, const Brush& brush, uint32_t group_count);

  inline diplomat::result<uint32_t, NucleationError> add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material);

  inline void animate_camera(const AnimationEffect& effect, float offset_ms);

  inline uint32_t frame_count(double fps, float hold_ms) const;

  /**
   * Render directly to a looping GIF. The renderer, meshes, timeline and
   * GIF encoder all live in the Rust core; no ffmpeg subprocess is needed.
   */
  inline diplomat::result<uint32_t, NucleationError> render_gif(diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, std::string_view path, double fps, float hold_ms) const;

  /**
   * Render and stream with an already parsed resource pack.
   */
  inline diplomat::result<uint32_t, NucleationError> render_video_with_pack(const ResourcePack& pack, const RenderConfig& config, const VideoConfig& video, std::string_view path, float hold_ms) const;

  /**
   * Render and stream directly to video. The GPU renderer and meshes are
   * reused for the complete animation and no frame sequence is retained.
   */
  inline diplomat::result<uint32_t, NucleationError> render_video(diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, const VideoConfig& video, std::string_view path, float hold_ms) const;

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
