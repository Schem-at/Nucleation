#ifndef WsSegmentJob_HPP
#define WsSegmentJob_HPP

#include "WsSegmentJob.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct WsSegmentJob_create_result {union {diplomat::capi::WsSegmentJob* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WsSegmentJob_create_result;
    WsSegmentJob_create_result WsSegmentJob_create(uint32_t cell_size, uint32_t closing_radius, uint64_t min_cluster_blocks, diplomat::capi::DiplomatStringView source_id, diplomat::capi::DiplomatStringView snapshot_id, int32_t min_y, int32_t max_y, int64_t extracted_at, float match_iou, bool hard_cut);

    void WsSegmentJob_destroy(WsSegmentJob* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<WsSegmentJob>, NucleationError> WsSegmentJob::create(uint32_t cell_size, uint32_t closing_radius, uint64_t min_cluster_blocks, std::string_view source_id, std::string_view snapshot_id, int32_t min_y, int32_t max_y, int64_t extracted_at, float match_iou, bool hard_cut) {
    auto result = diplomat::capi::WsSegmentJob_create(cell_size,
        closing_radius,
        min_cluster_blocks,
        {source_id.data(), source_id.size()},
        {snapshot_id.data(), snapshot_id.size()},
        min_y,
        max_y,
        extracted_at,
        match_iou,
        hard_cut);
    return result.is_ok ? diplomat::result<std::unique_ptr<WsSegmentJob>, NucleationError>(diplomat::Ok<std::unique_ptr<WsSegmentJob>>(std::unique_ptr<WsSegmentJob>(WsSegmentJob::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WsSegmentJob>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::WsSegmentJob* WsSegmentJob::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WsSegmentJob*>(this);
}

inline diplomat::capi::WsSegmentJob* WsSegmentJob::AsFFI() {
    return reinterpret_cast<diplomat::capi::WsSegmentJob*>(this);
}

inline const WsSegmentJob* WsSegmentJob::FromFFI(const diplomat::capi::WsSegmentJob* ptr) {
    return reinterpret_cast<const WsSegmentJob*>(ptr);
}

inline WsSegmentJob* WsSegmentJob::FromFFI(diplomat::capi::WsSegmentJob* ptr) {
    return reinterpret_cast<WsSegmentJob*>(ptr);
}

inline void WsSegmentJob::operator delete(void* ptr) {
    diplomat::capi::WsSegmentJob_destroy(reinterpret_cast<diplomat::capi::WsSegmentJob*>(ptr));
}


#endif // WsSegmentJob_HPP
