#ifndef WsRunResult_HPP
#define WsRunResult_HPP

#include "WsRunResult.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "BlockPos.hpp"
#include "NucleationError.hpp"
#include "WsPartitionHints.hpp"
#include "WsProfile.hpp"
#include "WsSegmentJob.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct WsRunResult_run_dir_result {union {diplomat::capi::WsRunResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_run_dir_result;
    WsRunResult_run_dir_result WsRunResult_run_dir(const diplomat::capi::WsSegmentJob* job, const diplomat::capi::WsPartitionHints* hints, const diplomat::capi::WsProfile* profile, diplomat::capi::DiplomatStringView world_dir);

    uint64_t WsRunResult_builds(const diplomat::capi::WsRunResult* self);

    uint64_t WsRunResult_tier_confident(const diplomat::capi::WsRunResult* self);

    uint64_t WsRunResult_tier_probable(const diplomat::capi::WsRunResult* self);

    uint64_t WsRunResult_tier_debris(const diplomat::capi::WsRunResult* self);

    uint64_t WsRunResult_cross_tile(const diplomat::capi::WsRunResult* self);

    uint64_t WsRunResult_largest_block_count(const diplomat::capi::WsRunResult* self);

    uint32_t WsRunResult_build_count(const diplomat::capi::WsRunResult* self);

    typedef struct WsRunResult_stable_id_hex_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_stable_id_hex_result;
    WsRunResult_stable_id_hex_result WsRunResult_stable_id_hex(const diplomat::capi::WsRunResult* self, uint32_t index, diplomat::capi::DiplomatWrite* write);

    typedef struct WsRunResult_fingerprint_hex_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_fingerprint_hex_result;
    WsRunResult_fingerprint_hex_result WsRunResult_fingerprint_hex(const diplomat::capi::WsRunResult* self, uint32_t index, diplomat::capi::DiplomatWrite* write);

    typedef struct WsRunResult_tier_of_result {union {uint8_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_tier_of_result;
    WsRunResult_tier_of_result WsRunResult_tier_of(const diplomat::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_block_count_of_result {union {uint64_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_block_count_of_result;
    WsRunResult_block_count_of_result WsRunResult_block_count_of(const diplomat::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_bbox_min_of_result {union {diplomat::capi::BlockPos ok; diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_bbox_min_of_result;
    WsRunResult_bbox_min_of_result WsRunResult_bbox_min_of(const diplomat::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_bbox_max_of_result {union {diplomat::capi::BlockPos ok; diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_bbox_max_of_result;
    WsRunResult_bbox_max_of_result WsRunResult_bbox_max_of(const diplomat::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_write_schem_to_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WsRunResult_write_schem_to_result;
    WsRunResult_write_schem_to_result WsRunResult_write_schem_to(const diplomat::capi::WsRunResult* self, uint32_t index, diplomat::capi::DiplomatStringView path);

    void WsRunResult_destroy(WsRunResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<WsRunResult>, NucleationError> WsRunResult::run_dir(const WsSegmentJob& job, const WsPartitionHints& hints, const WsProfile& profile, std::string_view world_dir) {
    auto result = diplomat::capi::WsRunResult_run_dir(job.AsFFI(),
        hints.AsFFI(),
        profile.AsFFI(),
        {world_dir.data(), world_dir.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WsRunResult>, NucleationError>(diplomat::Ok<std::unique_ptr<WsRunResult>>(std::unique_ptr<WsRunResult>(WsRunResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WsRunResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint64_t WsRunResult::builds() const {
    auto result = diplomat::capi::WsRunResult_builds(this->AsFFI());
    return result;
}

inline uint64_t WsRunResult::tier_confident() const {
    auto result = diplomat::capi::WsRunResult_tier_confident(this->AsFFI());
    return result;
}

inline uint64_t WsRunResult::tier_probable() const {
    auto result = diplomat::capi::WsRunResult_tier_probable(this->AsFFI());
    return result;
}

inline uint64_t WsRunResult::tier_debris() const {
    auto result = diplomat::capi::WsRunResult_tier_debris(this->AsFFI());
    return result;
}

inline uint64_t WsRunResult::cross_tile() const {
    auto result = diplomat::capi::WsRunResult_cross_tile(this->AsFFI());
    return result;
}

inline uint64_t WsRunResult::largest_block_count() const {
    auto result = diplomat::capi::WsRunResult_largest_block_count(this->AsFFI());
    return result;
}

inline uint32_t WsRunResult::build_count() const {
    auto result = diplomat::capi::WsRunResult_build_count(this->AsFFI());
    return result;
}

inline diplomat::result<std::string, NucleationError> WsRunResult::stable_id_hex(uint32_t index) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::WsRunResult_stable_id_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> WsRunResult::stable_id_hex_write(uint32_t index, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::WsRunResult_stable_id_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> WsRunResult::fingerprint_hex(uint32_t index) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::WsRunResult_fingerprint_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> WsRunResult::fingerprint_hex_write(uint32_t index, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::WsRunResult_fingerprint_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint8_t, NucleationError> WsRunResult::tier_of(uint32_t index) const {
    auto result = diplomat::capi::WsRunResult_tier_of(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<uint8_t, NucleationError>(diplomat::Ok<uint8_t>(result.ok)) : diplomat::result<uint8_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint64_t, NucleationError> WsRunResult::block_count_of(uint32_t index) const {
    auto result = diplomat::capi::WsRunResult_block_count_of(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<uint64_t, NucleationError>(diplomat::Ok<uint64_t>(result.ok)) : diplomat::result<uint64_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<BlockPos, NucleationError> WsRunResult::bbox_min_of(uint32_t index) const {
    auto result = diplomat::capi::WsRunResult_bbox_min_of(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<BlockPos, NucleationError>(diplomat::Ok<BlockPos>(BlockPos::FromFFI(result.ok))) : diplomat::result<BlockPos, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<BlockPos, NucleationError> WsRunResult::bbox_max_of(uint32_t index) const {
    auto result = diplomat::capi::WsRunResult_bbox_max_of(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<BlockPos, NucleationError>(diplomat::Ok<BlockPos>(BlockPos::FromFFI(result.ok))) : diplomat::result<BlockPos, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> WsRunResult::write_schem_to(uint32_t index, std::string_view path) const {
    auto result = diplomat::capi::WsRunResult_write_schem_to(this->AsFFI(),
        index,
        {path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::WsRunResult* WsRunResult::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WsRunResult*>(this);
}

inline diplomat::capi::WsRunResult* WsRunResult::AsFFI() {
    return reinterpret_cast<diplomat::capi::WsRunResult*>(this);
}

inline const WsRunResult* WsRunResult::FromFFI(const diplomat::capi::WsRunResult* ptr) {
    return reinterpret_cast<const WsRunResult*>(ptr);
}

inline WsRunResult* WsRunResult::FromFFI(diplomat::capi::WsRunResult* ptr) {
    return reinterpret_cast<WsRunResult*>(ptr);
}

inline void WsRunResult::operator delete(void* ptr) {
    diplomat::capi::WsRunResult_destroy(reinterpret_cast<diplomat::capi::WsRunResult*>(ptr));
}


#endif // WsRunResult_HPP
