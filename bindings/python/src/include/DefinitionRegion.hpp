#ifndef NUCLEATION_DefinitionRegion_HPP
#define NUCLEATION_DefinitionRegion_HPP

#include "DefinitionRegion.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "BlockPos.hpp"
#include "Dimensions.hpp"
#include "NucleationError.hpp"
#include "RegionBounds.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::DefinitionRegion* DefinitionRegion_create(void);

    nucleation::capi::DefinitionRegion* DefinitionRegion_from_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct DefinitionRegion_from_positions_result {union {nucleation::capi::DefinitionRegion* ok; nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_from_positions_result;
    DefinitionRegion_from_positions_result DefinitionRegion_from_positions(nucleation::diplomat::capi::DiplomatI32View positions);

    typedef struct DefinitionRegion_from_bounding_boxes_result {union {nucleation::capi::DefinitionRegion* ok; nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_from_bounding_boxes_result;
    DefinitionRegion_from_bounding_boxes_result DefinitionRegion_from_bounding_boxes(nucleation::diplomat::capi::DiplomatI32View boxes);

    void DefinitionRegion_add_bounds(nucleation::capi::DefinitionRegion* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    void DefinitionRegion_add_point(nucleation::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    typedef struct DefinitionRegion_set_metadata_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_set_metadata_result;
    DefinitionRegion_set_metadata_result DefinitionRegion_set_metadata(nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatStringView value);

    typedef struct DefinitionRegion_get_metadata_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_get_metadata_result;
    DefinitionRegion_get_metadata_result DefinitionRegion_get_metadata(const nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_all_metadata_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_all_metadata_json_result;
    DefinitionRegion_all_metadata_json_result DefinitionRegion_all_metadata_json(const nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_metadata_keys_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_metadata_keys_json_result;
    DefinitionRegion_metadata_keys_json_result DefinitionRegion_metadata_keys_json(const nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_add_filter_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_add_filter_result;
    DefinitionRegion_add_filter_result DefinitionRegion_add_filter(nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatStringView filter);

    bool DefinitionRegion_is_empty(const nucleation::capi::DefinitionRegion* self);

    uint64_t DefinitionRegion_volume(const nucleation::capi::DefinitionRegion* self);

    bool DefinitionRegion_contains(const nucleation::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    void DefinitionRegion_shift(nucleation::capi::DefinitionRegion* self, int32_t dx, int32_t dy, int32_t dz);

    void DefinitionRegion_expand(nucleation::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    void DefinitionRegion_contract(nucleation::capi::DefinitionRegion* self, int32_t amount);

    nucleation::capi::DefinitionRegion* DefinitionRegion_intersected(const nucleation::capi::DefinitionRegion* self, const nucleation::capi::DefinitionRegion* other);

    nucleation::capi::DefinitionRegion* DefinitionRegion_union_with(const nucleation::capi::DefinitionRegion* self, const nucleation::capi::DefinitionRegion* other);

    nucleation::capi::DefinitionRegion* DefinitionRegion_subtracted(const nucleation::capi::DefinitionRegion* self, const nucleation::capi::DefinitionRegion* other);

    void DefinitionRegion_merge(nucleation::capi::DefinitionRegion* self, const nucleation::capi::DefinitionRegion* other);

    void DefinitionRegion_union_into(nucleation::capi::DefinitionRegion* self, const nucleation::capi::DefinitionRegion* other);

    typedef struct DefinitionRegion_bounds_result {union {nucleation::capi::RegionBounds ok; nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_bounds_result;
    DefinitionRegion_bounds_result DefinitionRegion_bounds(const nucleation::capi::DefinitionRegion* self);

    nucleation::capi::Dimensions DefinitionRegion_dimensions(const nucleation::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_center_result {union {nucleation::capi::BlockPos ok; nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_center_result;
    DefinitionRegion_center_result DefinitionRegion_center(const nucleation::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_center_f32_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_center_f32_json_result;
    DefinitionRegion_center_f32_json_result DefinitionRegion_center_f32_json(const nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_positions_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_positions_json_result;
    DefinitionRegion_positions_json_result DefinitionRegion_positions_json(const nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_positions_sorted_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_positions_sorted_json_result;
    DefinitionRegion_positions_sorted_json_result DefinitionRegion_positions_sorted_json(const nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t DefinitionRegion_box_count(const nucleation::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_get_box_result {union {nucleation::capi::RegionBounds ok; nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_get_box_result;
    DefinitionRegion_get_box_result DefinitionRegion_get_box(const nucleation::capi::DefinitionRegion* self, uint32_t index);

    typedef struct DefinitionRegion_boxes_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_boxes_json_result;
    DefinitionRegion_boxes_json_result DefinitionRegion_boxes_json(const nucleation::capi::DefinitionRegion* self, nucleation::diplomat::capi::DiplomatWrite* write);

    bool DefinitionRegion_is_contiguous(const nucleation::capi::DefinitionRegion* self);

    uint32_t DefinitionRegion_connected_components(const nucleation::capi::DefinitionRegion* self);

    void DefinitionRegion_simplify(nucleation::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_filter_by_block_result {union {nucleation::capi::DefinitionRegion* ok; nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_filter_by_block_result;
    DefinitionRegion_filter_by_block_result DefinitionRegion_filter_by_block(const nucleation::capi::DefinitionRegion* self, const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView block_name);

    typedef struct DefinitionRegion_filter_by_properties_result {union {nucleation::capi::DefinitionRegion* ok; nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_filter_by_properties_result;
    DefinitionRegion_filter_by_properties_result DefinitionRegion_filter_by_properties(const nucleation::capi::DefinitionRegion* self, const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView properties_json);

    typedef struct DefinitionRegion_exclude_block_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_exclude_block_result;
    DefinitionRegion_exclude_block_result DefinitionRegion_exclude_block(nucleation::capi::DefinitionRegion* self, const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView block_name);

    bool DefinitionRegion_intersects_bounds(const nucleation::capi::DefinitionRegion* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    nucleation::capi::DefinitionRegion* DefinitionRegion_shifted(const nucleation::capi::DefinitionRegion* self, int32_t dx, int32_t dy, int32_t dz);

    nucleation::capi::DefinitionRegion* DefinitionRegion_expanded(const nucleation::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    nucleation::capi::DefinitionRegion* DefinitionRegion_contracted(const nucleation::capi::DefinitionRegion* self, int32_t amount);

    nucleation::capi::DefinitionRegion* DefinitionRegion_copy(const nucleation::capi::DefinitionRegion* self);

    void DefinitionRegion_set_color(nucleation::capi::DefinitionRegion* self, uint32_t color);

    typedef struct DefinitionRegion_blocks_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_blocks_json_result;
    DefinitionRegion_blocks_json_result DefinitionRegion_blocks_json(const nucleation::capi::DefinitionRegion* self, const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_sync_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_sync_result;
    DefinitionRegion_sync_result DefinitionRegion_sync(const nucleation::capi::DefinitionRegion* self, nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name);

    void DefinitionRegion_destroy(DefinitionRegion* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::create() {
    auto result = nucleation::capi::DefinitionRegion_create();
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::from_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::DefinitionRegion_from_bounds(min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> nucleation::DefinitionRegion::from_positions(nucleation::diplomat::span<const int32_t> positions) {
    auto result = nucleation::capi::DefinitionRegion_from_positions({positions.data(), positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::DefinitionRegion>>(std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> nucleation::DefinitionRegion::from_bounding_boxes(nucleation::diplomat::span<const int32_t> boxes) {
    auto result = nucleation::capi::DefinitionRegion_from_bounding_boxes({boxes.data(), boxes.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::DefinitionRegion>>(std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::DefinitionRegion::add_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    nucleation::capi::DefinitionRegion_add_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
}

inline void nucleation::DefinitionRegion::add_point(int32_t x, int32_t y, int32_t z) {
    nucleation::capi::DefinitionRegion_add_point(this->AsFFI(),
        x,
        y,
        z);
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::set_metadata(std::string_view key, std::string_view value) {
    auto result = nucleation::capi::DefinitionRegion_set_metadata(this->AsFFI(),
        {key.data(), key.size()},
        {value.data(), value.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::get_metadata(std::string_view key) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_get_metadata(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::get_metadata_write(std::string_view key, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_get_metadata(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::all_metadata_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_all_metadata_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::all_metadata_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_all_metadata_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::metadata_keys_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_metadata_keys_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::metadata_keys_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_metadata_keys_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::add_filter(std::string_view filter) {
    auto result = nucleation::capi::DefinitionRegion_add_filter(this->AsFFI(),
        {filter.data(), filter.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline bool nucleation::DefinitionRegion::is_empty() const {
    auto result = nucleation::capi::DefinitionRegion_is_empty(this->AsFFI());
    return result;
}

inline uint64_t nucleation::DefinitionRegion::volume() const {
    auto result = nucleation::capi::DefinitionRegion_volume(this->AsFFI());
    return result;
}

inline bool nucleation::DefinitionRegion::contains(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::DefinitionRegion_contains(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void nucleation::DefinitionRegion::shift(int32_t dx, int32_t dy, int32_t dz) {
    nucleation::capi::DefinitionRegion_shift(this->AsFFI(),
        dx,
        dy,
        dz);
}

inline void nucleation::DefinitionRegion::expand(int32_t x, int32_t y, int32_t z) {
    nucleation::capi::DefinitionRegion_expand(this->AsFFI(),
        x,
        y,
        z);
}

inline void nucleation::DefinitionRegion::contract(int32_t amount) {
    nucleation::capi::DefinitionRegion_contract(this->AsFFI(),
        amount);
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::intersected(const nucleation::DefinitionRegion& other) const {
    auto result = nucleation::capi::DefinitionRegion_intersected(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::union_with(const nucleation::DefinitionRegion& other) const {
    auto result = nucleation::capi::DefinitionRegion_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::subtracted(const nucleation::DefinitionRegion& other) const {
    auto result = nucleation::capi::DefinitionRegion_subtracted(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline void nucleation::DefinitionRegion::merge(const nucleation::DefinitionRegion& other) {
    nucleation::capi::DefinitionRegion_merge(this->AsFFI(),
        other.AsFFI());
}

inline void nucleation::DefinitionRegion::union_into(const nucleation::DefinitionRegion& other) {
    nucleation::capi::DefinitionRegion_union_into(this->AsFFI(),
        other.AsFFI());
}

inline nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError> nucleation::DefinitionRegion::bounds() const {
    auto result = nucleation::capi::DefinitionRegion_bounds(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::RegionBounds>(nucleation::RegionBounds::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::Dimensions nucleation::DefinitionRegion::dimensions() const {
    auto result = nucleation::capi::DefinitionRegion_dimensions(this->AsFFI());
    return nucleation::Dimensions::FromFFI(result);
}

inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> nucleation::DefinitionRegion::center() const {
    auto result = nucleation::capi::DefinitionRegion_center(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::BlockPos>(nucleation::BlockPos::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::center_f32_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_center_f32_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::center_f32_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_center_f32_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::positions_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_positions_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::positions_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_positions_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::positions_sorted_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_positions_sorted_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::positions_sorted_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_positions_sorted_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::DefinitionRegion::box_count() const {
    auto result = nucleation::capi::DefinitionRegion_box_count(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError> nucleation::DefinitionRegion::get_box(uint32_t index) const {
    auto result = nucleation::capi::DefinitionRegion_get_box(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::RegionBounds>(nucleation::RegionBounds::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::boxes_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_boxes_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::boxes_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_boxes_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline bool nucleation::DefinitionRegion::is_contiguous() const {
    auto result = nucleation::capi::DefinitionRegion_is_contiguous(this->AsFFI());
    return result;
}

inline uint32_t nucleation::DefinitionRegion::connected_components() const {
    auto result = nucleation::capi::DefinitionRegion_connected_components(this->AsFFI());
    return result;
}

inline void nucleation::DefinitionRegion::simplify() {
    nucleation::capi::DefinitionRegion_simplify(this->AsFFI());
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> nucleation::DefinitionRegion::filter_by_block(const nucleation::Schematic& schematic, std::string_view block_name) const {
    auto result = nucleation::capi::DefinitionRegion_filter_by_block(this->AsFFI(),
        schematic.AsFFI(),
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::DefinitionRegion>>(std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> nucleation::DefinitionRegion::filter_by_properties(const nucleation::Schematic& schematic, std::string_view properties_json) const {
    auto result = nucleation::capi::DefinitionRegion_filter_by_properties(this->AsFFI(),
        schematic.AsFFI(),
        {properties_json.data(), properties_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::DefinitionRegion>>(std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::exclude_block(const nucleation::Schematic& schematic, std::string_view block_name) {
    auto result = nucleation::capi::DefinitionRegion_exclude_block(this->AsFFI(),
        schematic.AsFFI(),
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline bool nucleation::DefinitionRegion::intersects_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = nucleation::capi::DefinitionRegion_intersects_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result;
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::shifted(int32_t dx, int32_t dy, int32_t dz) const {
    auto result = nucleation::capi::DefinitionRegion_shifted(this->AsFFI(),
        dx,
        dy,
        dz);
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::expanded(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::DefinitionRegion_expanded(this->AsFFI(),
        x,
        y,
        z);
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::contracted(int32_t amount) const {
    auto result = nucleation::capi::DefinitionRegion_contracted(this->AsFFI(),
        amount);
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<nucleation::DefinitionRegion> nucleation::DefinitionRegion::copy() const {
    auto result = nucleation::capi::DefinitionRegion_copy(this->AsFFI());
    return std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result));
}

inline void nucleation::DefinitionRegion::set_color(uint32_t color) {
    nucleation::capi::DefinitionRegion_set_color(this->AsFFI(),
        color);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::DefinitionRegion::blocks_json(const nucleation::Schematic& schematic) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::DefinitionRegion_blocks_json(this->AsFFI(),
        schematic.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::blocks_json_write(const nucleation::Schematic& schematic, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::DefinitionRegion_blocks_json(this->AsFFI(),
        schematic.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::DefinitionRegion::sync(nucleation::Schematic& schematic, std::string_view name) const {
    auto result = nucleation::capi::DefinitionRegion_sync(this->AsFFI(),
        schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::DefinitionRegion* nucleation::DefinitionRegion::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::DefinitionRegion*>(this);
}

inline nucleation::capi::DefinitionRegion* nucleation::DefinitionRegion::AsFFI() {
    return reinterpret_cast<nucleation::capi::DefinitionRegion*>(this);
}

inline const nucleation::DefinitionRegion* nucleation::DefinitionRegion::FromFFI(const nucleation::capi::DefinitionRegion* ptr) {
    return reinterpret_cast<const nucleation::DefinitionRegion*>(ptr);
}

inline nucleation::DefinitionRegion* nucleation::DefinitionRegion::FromFFI(nucleation::capi::DefinitionRegion* ptr) {
    return reinterpret_cast<nucleation::DefinitionRegion*>(ptr);
}

inline void nucleation::DefinitionRegion::operator delete(void* ptr) {
    nucleation::capi::DefinitionRegion_destroy(reinterpret_cast<nucleation::capi::DefinitionRegion*>(ptr));
}


#endif // NUCLEATION_DefinitionRegion_HPP
