#ifndef WsProfile_D_HPP
#define WsProfile_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct WsProfile;
} // namespace capi
} // namespace

/**
 * A pinned {@link WorldProfile}(crate::world_segment::profile::WorldProfile):
 * the substrate palette + Y band derived (or supplied) once per world and
 * reused across every segmentation run against it.
 */
class WsProfile {
public:

  /**
   * Derive a profile from up to `sample` tiles (regions) of a world
   * directory, in ascending `(x, z)` region order. `coverage` is
   * `ProfileParams::min_slab_coverage`; every other `ProfileParams`
   * field uses its default (`sample_stride: 1`, `y_scan: (-64, 320)`).
   */
  inline static diplomat::result<std::unique_ptr<WsProfile>, NucleationError> derive_from_dir(std::string_view world_dir, int32_t min_y, int32_t max_y, uint32_t sample, float coverage);

  /**
   * The derived substrate Y band's lower bound (inclusive).
   */
  inline int32_t band_min() const;

  /**
   * The derived substrate Y band's upper bound (inclusive).
   */
  inline int32_t band_max() const;

  /**
   * Number of distinct block names in the substrate palette.
   */
  inline uint32_t palette_len() const;

  /**
   * The substrate palette as a JSON array of block-name strings.
   */
  inline diplomat::result<std::string, NucleationError> write_palette_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> write_palette_json_write(W& writeable_output) const;

    inline const diplomat::capi::WsProfile* AsFFI() const;
    inline diplomat::capi::WsProfile* AsFFI();
    inline static const WsProfile* FromFFI(const diplomat::capi::WsProfile* ptr);
    inline static WsProfile* FromFFI(diplomat::capi::WsProfile* ptr);
    inline static void operator delete(void* ptr);
private:
    WsProfile() = delete;
    WsProfile(const WsProfile&) = delete;
    WsProfile(WsProfile&&) noexcept = delete;
    WsProfile operator=(const WsProfile&) = delete;
    WsProfile operator=(WsProfile&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WsProfile_D_HPP
