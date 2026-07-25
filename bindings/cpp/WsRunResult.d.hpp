#ifndef WsRunResult_D_HPP
#define WsRunResult_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct WsPartitionHints; }
class WsPartitionHints;
namespace diplomat::capi { struct WsProfile; }
class WsProfile;
namespace diplomat::capi { struct WsSegmentJob; }
class WsSegmentJob;
struct BlockPos;
class NucleationError;




namespace diplomat {
namespace capi {
    struct WsRunResult;
} // namespace capi
} // namespace

/**
 * The materialized output of one segmentation run: every build (in the
 * pipeline's deterministic stable-id order) plus the aggregate
 * {@link RunStats}(crate::world_segment::runner::RunStats).
 */
class WsRunResult {
public:

  /**
   * Run the full pipeline (source -> segment -> stitch -> score ->
   * identity -> materialize) over a world directory. No prior-snapshot
   * builds are supplied, so every build seeds a fresh stable id (see
   * `StableBuildId::seed`).
   *
   * See the module docs for why this catches a panic instead of
   * propagating it.
   */
  inline static diplomat::result<std::unique_ptr<WsRunResult>, NucleationError> run_dir(const WsSegmentJob& job, const WsPartitionHints& hints, const WsProfile& profile, std::string_view world_dir);

  /**
   * Total builds materialized (same as `build_count`, from `RunStats`).
   */
  inline uint64_t builds() const;

  inline uint64_t tier_confident() const;

  inline uint64_t tier_probable() const;

  inline uint64_t tier_debris() const;

  inline uint64_t cross_tile() const;

  inline uint64_t largest_block_count() const;

  /**
   * Number of builds held in this result (indices `0..build_count()`
   * are valid for every per-index accessor below).
   */
  inline uint32_t build_count() const;

  /**
   * The build's stable id (hex), stable across re-runs against the
   * same source under the same config, absent a prior-snapshot match.
   */
  inline diplomat::result<std::string, NucleationError> stable_id_hex(uint32_t index) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> stable_id_hex_write(uint32_t index, W& writeable_output) const;

  /**
   * The build's content fingerprint, as 32 lowercase hex digits (u128,
   * big-endian).
   */
  inline diplomat::result<std::string, NucleationError> fingerprint_hex(uint32_t index) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> fingerprint_hex_write(uint32_t index, W& writeable_output) const;

  /**
   * `0` = Confident, `1` = Probable, `2` = Debris.
   */
  inline diplomat::result<uint8_t, NucleationError> tier_of(uint32_t index) const;

  inline diplomat::result<uint64_t, NucleationError> block_count_of(uint32_t index) const;

  /**
   * The build's world-space bounding box minimum (inclusive).
   */
  inline diplomat::result<BlockPos, NucleationError> bbox_min_of(uint32_t index) const;

  /**
   * The build's world-space bounding box maximum (inclusive).
   */
  inline diplomat::result<BlockPos, NucleationError> bbox_max_of(uint32_t index) const;

  /**
   * Save the build's schematic to a file, picking the format from the
   * file extension — same serializer as
   * {@link Schematic::save_to_file}(super::super::schematic::ffi::Schematic::save_to_file).
   * Not available in JS: the WASM build has no filesystem.
   */
  inline diplomat::result<std::monostate, NucleationError> write_schem_to(uint32_t index, std::string_view path) const;

    inline const diplomat::capi::WsRunResult* AsFFI() const;
    inline diplomat::capi::WsRunResult* AsFFI();
    inline static const WsRunResult* FromFFI(const diplomat::capi::WsRunResult* ptr);
    inline static WsRunResult* FromFFI(diplomat::capi::WsRunResult* ptr);
    inline static void operator delete(void* ptr);
private:
    WsRunResult() = delete;
    WsRunResult(const WsRunResult&) = delete;
    WsRunResult(WsRunResult&&) noexcept = delete;
    WsRunResult operator=(const WsRunResult&) = delete;
    WsRunResult operator=(WsRunResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WsRunResult_D_HPP
