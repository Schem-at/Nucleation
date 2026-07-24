#ifndef NUCLEATION_WsSegmentJob_HPP
#define NUCLEATION_WsSegmentJob_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct WsSegmentJob_create_result {union {nucleation::capi::WsSegmentJob* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WsSegmentJob_create_result;
    WsSegmentJob_create_result WsSegmentJob_create(uint32_t cell_size, uint32_t closing_radius, uint64_t min_cluster_blocks, nucleation::diplomat::capi::DiplomatStringView source_id, nucleation::diplomat::capi::DiplomatStringView snapshot_id, int32_t min_y, int32_t max_y, int64_t extracted_at, float match_iou, bool hard_cut);

    void WsSegmentJob_destroy(WsSegmentJob* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WsSegmentJob>, nucleation::NucleationError> nucleation::WsSegmentJob::create(uint32_t cell_size, uint32_t closing_radius, uint64_t min_cluster_blocks, std::string_view source_id, std::string_view snapshot_id, int32_t min_y, int32_t max_y, int64_t extracted_at, float match_iou, bool hard_cut) {
    auto result = nucleation::capi::WsSegmentJob_create(cell_size,
        closing_radius,
        min_cluster_blocks,
        {source_id.data(), source_id.size()},
        {snapshot_id.data(), snapshot_id.size()},
        min_y,
        max_y,
        extracted_at,
        match_iou,
        hard_cut);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WsSegmentJob>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WsSegmentJob>>(std::unique_ptr<nucleation::WsSegmentJob>(nucleation::WsSegmentJob::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WsSegmentJob>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::WsSegmentJob* nucleation::WsSegmentJob::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WsSegmentJob*>(this);
}

inline nucleation::capi::WsSegmentJob* nucleation::WsSegmentJob::AsFFI() {
    return reinterpret_cast<nucleation::capi::WsSegmentJob*>(this);
}

inline const nucleation::WsSegmentJob* nucleation::WsSegmentJob::FromFFI(const nucleation::capi::WsSegmentJob* ptr) {
    return reinterpret_cast<const nucleation::WsSegmentJob*>(ptr);
}

inline nucleation::WsSegmentJob* nucleation::WsSegmentJob::FromFFI(nucleation::capi::WsSegmentJob* ptr) {
    return reinterpret_cast<nucleation::WsSegmentJob*>(ptr);
}

inline void nucleation::WsSegmentJob::operator delete(void* ptr) {
    nucleation::capi::WsSegmentJob_destroy(reinterpret_cast<nucleation::capi::WsSegmentJob*>(ptr));
}


#endif // NUCLEATION_WsSegmentJob_HPP
