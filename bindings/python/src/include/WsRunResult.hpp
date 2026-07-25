#ifndef NUCLEATION_WsRunResult_HPP
#define NUCLEATION_WsRunResult_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct WsRunResult_run_dir_result {union {nucleation::capi::WsRunResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_run_dir_result;
    WsRunResult_run_dir_result WsRunResult_run_dir(const nucleation::capi::WsSegmentJob* job, const nucleation::capi::WsPartitionHints* hints, const nucleation::capi::WsProfile* profile, nucleation::diplomat::capi::DiplomatStringView world_dir);

    uint64_t WsRunResult_builds(const nucleation::capi::WsRunResult* self);

    uint64_t WsRunResult_tier_confident(const nucleation::capi::WsRunResult* self);

    uint64_t WsRunResult_tier_probable(const nucleation::capi::WsRunResult* self);

    uint64_t WsRunResult_tier_debris(const nucleation::capi::WsRunResult* self);

    uint64_t WsRunResult_cross_tile(const nucleation::capi::WsRunResult* self);

    uint64_t WsRunResult_largest_block_count(const nucleation::capi::WsRunResult* self);

    uint32_t WsRunResult_build_count(const nucleation::capi::WsRunResult* self);

    typedef struct WsRunResult_stable_id_hex_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_stable_id_hex_result;
    WsRunResult_stable_id_hex_result WsRunResult_stable_id_hex(const nucleation::capi::WsRunResult* self, uint32_t index, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct WsRunResult_fingerprint_hex_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_fingerprint_hex_result;
    WsRunResult_fingerprint_hex_result WsRunResult_fingerprint_hex(const nucleation::capi::WsRunResult* self, uint32_t index, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct WsRunResult_tier_of_result {union {uint8_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_tier_of_result;
    WsRunResult_tier_of_result WsRunResult_tier_of(const nucleation::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_block_count_of_result {union {uint64_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_block_count_of_result;
    WsRunResult_block_count_of_result WsRunResult_block_count_of(const nucleation::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_bbox_min_of_result {union {nucleation::capi::BlockPos ok; nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_bbox_min_of_result;
    WsRunResult_bbox_min_of_result WsRunResult_bbox_min_of(const nucleation::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_bbox_max_of_result {union {nucleation::capi::BlockPos ok; nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_bbox_max_of_result;
    WsRunResult_bbox_max_of_result WsRunResult_bbox_max_of(const nucleation::capi::WsRunResult* self, uint32_t index);

    typedef struct WsRunResult_write_schem_to_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WsRunResult_write_schem_to_result;
    WsRunResult_write_schem_to_result WsRunResult_write_schem_to(const nucleation::capi::WsRunResult* self, uint32_t index, nucleation::diplomat::capi::DiplomatStringView path);

    void WsRunResult_destroy(WsRunResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WsRunResult>, nucleation::NucleationError> nucleation::WsRunResult::run_dir(const nucleation::WsSegmentJob& job, const nucleation::WsPartitionHints& hints, const nucleation::WsProfile& profile, std::string_view world_dir) {
    auto result = nucleation::capi::WsRunResult_run_dir(job.AsFFI(),
        hints.AsFFI(),
        profile.AsFFI(),
        {world_dir.data(), world_dir.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WsRunResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WsRunResult>>(std::unique_ptr<nucleation::WsRunResult>(nucleation::WsRunResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WsRunResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint64_t nucleation::WsRunResult::builds() const {
    auto result = nucleation::capi::WsRunResult_builds(this->AsFFI());
    return result;
}

inline uint64_t nucleation::WsRunResult::tier_confident() const {
    auto result = nucleation::capi::WsRunResult_tier_confident(this->AsFFI());
    return result;
}

inline uint64_t nucleation::WsRunResult::tier_probable() const {
    auto result = nucleation::capi::WsRunResult_tier_probable(this->AsFFI());
    return result;
}

inline uint64_t nucleation::WsRunResult::tier_debris() const {
    auto result = nucleation::capi::WsRunResult_tier_debris(this->AsFFI());
    return result;
}

inline uint64_t nucleation::WsRunResult::cross_tile() const {
    auto result = nucleation::capi::WsRunResult_cross_tile(this->AsFFI());
    return result;
}

inline uint64_t nucleation::WsRunResult::largest_block_count() const {
    auto result = nucleation::capi::WsRunResult_largest_block_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::WsRunResult::build_count() const {
    auto result = nucleation::capi::WsRunResult_build_count(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::WsRunResult::stable_id_hex(uint32_t index) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::WsRunResult_stable_id_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WsRunResult::stable_id_hex_write(uint32_t index, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::WsRunResult_stable_id_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::WsRunResult::fingerprint_hex(uint32_t index) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::WsRunResult_fingerprint_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WsRunResult::fingerprint_hex_write(uint32_t index, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::WsRunResult_fingerprint_hex(this->AsFFI(),
        index,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint8_t, nucleation::NucleationError> nucleation::WsRunResult::tier_of(uint32_t index) const {
    auto result = nucleation::capi::WsRunResult_tier_of(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<uint8_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint8_t>(result.ok)) : nucleation::diplomat::result<uint8_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint64_t, nucleation::NucleationError> nucleation::WsRunResult::block_count_of(uint32_t index) const {
    auto result = nucleation::capi::WsRunResult_block_count_of(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<uint64_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint64_t>(result.ok)) : nucleation::diplomat::result<uint64_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> nucleation::WsRunResult::bbox_min_of(uint32_t index) const {
    auto result = nucleation::capi::WsRunResult_bbox_min_of(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::BlockPos>(nucleation::BlockPos::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> nucleation::WsRunResult::bbox_max_of(uint32_t index) const {
    auto result = nucleation::capi::WsRunResult_bbox_max_of(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::BlockPos>(nucleation::BlockPos::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WsRunResult::write_schem_to(uint32_t index, std::string_view path) const {
    auto result = nucleation::capi::WsRunResult_write_schem_to(this->AsFFI(),
        index,
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::WsRunResult* nucleation::WsRunResult::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WsRunResult*>(this);
}

inline nucleation::capi::WsRunResult* nucleation::WsRunResult::AsFFI() {
    return reinterpret_cast<nucleation::capi::WsRunResult*>(this);
}

inline const nucleation::WsRunResult* nucleation::WsRunResult::FromFFI(const nucleation::capi::WsRunResult* ptr) {
    return reinterpret_cast<const nucleation::WsRunResult*>(ptr);
}

inline nucleation::WsRunResult* nucleation::WsRunResult::FromFFI(nucleation::capi::WsRunResult* ptr) {
    return reinterpret_cast<nucleation::WsRunResult*>(ptr);
}

inline void nucleation::WsRunResult::operator delete(void* ptr) {
    nucleation::capi::WsRunResult_destroy(reinterpret_cast<nucleation::capi::WsRunResult*>(ptr));
}


#endif // NUCLEATION_WsRunResult_HPP
