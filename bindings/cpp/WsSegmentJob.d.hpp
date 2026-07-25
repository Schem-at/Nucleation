#ifndef WsSegmentJob_D_HPP
#define WsSegmentJob_D_HPP

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
    struct WsSegmentJob;
} // namespace capi
} // namespace

/**
 * One segmentation run's parameters (the primitive knobs of
 * {@link SegmentJob}(crate::world_segment::runner::SegmentJob), plus a
 * `hard_cut` flag selecting {@link PartitionPolicy}). Built once, passed by
 * reference into {@link WsRunResult::run_dir}.
 */
class WsSegmentJob {
public:

  /**
   * `algorithm_version` is pinned to
   * {@link SegConfig::default}(crate::world_segment::segment::SegConfig)'s
   * value; `score_config` uses `ScoreConfig::default()`. Neither is
   * exposed as a knob here — construct a `SegmentJob` directly in Rust
   * if you need to override them.
   */
  inline static diplomat::result<std::unique_ptr<WsSegmentJob>, NucleationError> create(uint32_t cell_size, uint32_t closing_radius, uint64_t min_cluster_blocks, std::string_view source_id, std::string_view snapshot_id, int32_t min_y, int32_t max_y, int64_t extracted_at, float match_iou, bool hard_cut);

    inline const diplomat::capi::WsSegmentJob* AsFFI() const;
    inline diplomat::capi::WsSegmentJob* AsFFI();
    inline static const WsSegmentJob* FromFFI(const diplomat::capi::WsSegmentJob* ptr);
    inline static WsSegmentJob* FromFFI(diplomat::capi::WsSegmentJob* ptr);
    inline static void operator delete(void* ptr);
private:
    WsSegmentJob() = delete;
    WsSegmentJob(const WsSegmentJob&) = delete;
    WsSegmentJob(WsSegmentJob&&) noexcept = delete;
    WsSegmentJob operator=(const WsSegmentJob&) = delete;
    WsSegmentJob operator=(WsSegmentJob&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WsSegmentJob_D_HPP
