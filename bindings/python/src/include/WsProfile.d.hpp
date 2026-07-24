#ifndef NUCLEATION_WsProfile_D_HPP
#define NUCLEATION_WsProfile_D_HPP

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
namespace capi { struct WsProfile; }
class WsProfile;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct WsProfile;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WsProfile>, nucleation::NucleationError> derive_from_dir(std::string_view world_dir, int32_t min_y, int32_t max_y, uint32_t sample, float coverage);

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
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> write_palette_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> write_palette_json_write(W& writeable_output) const;

    inline const nucleation::capi::WsProfile* AsFFI() const;
    inline nucleation::capi::WsProfile* AsFFI();
    inline static const nucleation::WsProfile* FromFFI(const nucleation::capi::WsProfile* ptr);
    inline static nucleation::WsProfile* FromFFI(nucleation::capi::WsProfile* ptr);
    inline static void operator delete(void* ptr);
private:
    WsProfile() = delete;
    WsProfile(const nucleation::WsProfile&) = delete;
    WsProfile(nucleation::WsProfile&&) noexcept = delete;
    WsProfile operator=(const nucleation::WsProfile&) = delete;
    WsProfile operator=(nucleation::WsProfile&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_WsProfile_D_HPP
