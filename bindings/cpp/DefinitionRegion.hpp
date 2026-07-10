#ifndef DefinitionRegion_HPP
#define DefinitionRegion_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::DefinitionRegion* DefinitionRegion_create(void);

    diplomat::capi::DefinitionRegion* DefinitionRegion_from_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct DefinitionRegion_from_positions_result {union {diplomat::capi::DefinitionRegion* ok; diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_from_positions_result;
    DefinitionRegion_from_positions_result DefinitionRegion_from_positions(diplomat::capi::DiplomatI32View positions);

    typedef struct DefinitionRegion_from_bounding_boxes_result {union {diplomat::capi::DefinitionRegion* ok; diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_from_bounding_boxes_result;
    DefinitionRegion_from_bounding_boxes_result DefinitionRegion_from_bounding_boxes(diplomat::capi::DiplomatI32View boxes);

    void DefinitionRegion_add_bounds(diplomat::capi::DefinitionRegion* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    void DefinitionRegion_add_point(diplomat::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    typedef struct DefinitionRegion_set_metadata_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_set_metadata_result;
    DefinitionRegion_set_metadata_result DefinitionRegion_set_metadata(diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatStringView value);

    typedef struct DefinitionRegion_get_metadata_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_get_metadata_result;
    DefinitionRegion_get_metadata_result DefinitionRegion_get_metadata(const diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_all_metadata_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_all_metadata_json_result;
    DefinitionRegion_all_metadata_json_result DefinitionRegion_all_metadata_json(const diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_metadata_keys_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_metadata_keys_json_result;
    DefinitionRegion_metadata_keys_json_result DefinitionRegion_metadata_keys_json(const diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_add_filter_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_add_filter_result;
    DefinitionRegion_add_filter_result DefinitionRegion_add_filter(diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatStringView filter);

    bool DefinitionRegion_is_empty(const diplomat::capi::DefinitionRegion* self);

    uint64_t DefinitionRegion_volume(const diplomat::capi::DefinitionRegion* self);

    bool DefinitionRegion_contains(const diplomat::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    void DefinitionRegion_shift(diplomat::capi::DefinitionRegion* self, int32_t dx, int32_t dy, int32_t dz);

    void DefinitionRegion_expand(diplomat::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    void DefinitionRegion_contract(diplomat::capi::DefinitionRegion* self, int32_t amount);

    diplomat::capi::DefinitionRegion* DefinitionRegion_intersected(const diplomat::capi::DefinitionRegion* self, const diplomat::capi::DefinitionRegion* other);

    diplomat::capi::DefinitionRegion* DefinitionRegion_union_with(const diplomat::capi::DefinitionRegion* self, const diplomat::capi::DefinitionRegion* other);

    diplomat::capi::DefinitionRegion* DefinitionRegion_subtracted(const diplomat::capi::DefinitionRegion* self, const diplomat::capi::DefinitionRegion* other);

    void DefinitionRegion_merge(diplomat::capi::DefinitionRegion* self, const diplomat::capi::DefinitionRegion* other);

    void DefinitionRegion_union_into(diplomat::capi::DefinitionRegion* self, const diplomat::capi::DefinitionRegion* other);

    typedef struct DefinitionRegion_bounds_result {union {diplomat::capi::RegionBounds ok; diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_bounds_result;
    DefinitionRegion_bounds_result DefinitionRegion_bounds(const diplomat::capi::DefinitionRegion* self);

    diplomat::capi::Dimensions DefinitionRegion_dimensions(const diplomat::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_center_result {union {diplomat::capi::BlockPos ok; diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_center_result;
    DefinitionRegion_center_result DefinitionRegion_center(const diplomat::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_center_f32_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_center_f32_json_result;
    DefinitionRegion_center_f32_json_result DefinitionRegion_center_f32_json(const diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_positions_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_positions_json_result;
    DefinitionRegion_positions_json_result DefinitionRegion_positions_json(const diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_positions_sorted_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_positions_sorted_json_result;
    DefinitionRegion_positions_sorted_json_result DefinitionRegion_positions_sorted_json(const diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatWrite* write);

    uint32_t DefinitionRegion_box_count(const diplomat::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_get_box_result {union {diplomat::capi::RegionBounds ok; diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_get_box_result;
    DefinitionRegion_get_box_result DefinitionRegion_get_box(const diplomat::capi::DefinitionRegion* self, uint32_t index);

    typedef struct DefinitionRegion_boxes_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_boxes_json_result;
    DefinitionRegion_boxes_json_result DefinitionRegion_boxes_json(const diplomat::capi::DefinitionRegion* self, diplomat::capi::DiplomatWrite* write);

    bool DefinitionRegion_is_contiguous(const diplomat::capi::DefinitionRegion* self);

    uint32_t DefinitionRegion_connected_components(const diplomat::capi::DefinitionRegion* self);

    void DefinitionRegion_simplify(diplomat::capi::DefinitionRegion* self);

    typedef struct DefinitionRegion_filter_by_block_result {union {diplomat::capi::DefinitionRegion* ok; diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_filter_by_block_result;
    DefinitionRegion_filter_by_block_result DefinitionRegion_filter_by_block(const diplomat::capi::DefinitionRegion* self, const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView block_name);

    typedef struct DefinitionRegion_filter_by_properties_result {union {diplomat::capi::DefinitionRegion* ok; diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_filter_by_properties_result;
    DefinitionRegion_filter_by_properties_result DefinitionRegion_filter_by_properties(const diplomat::capi::DefinitionRegion* self, const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView properties_json);

    typedef struct DefinitionRegion_exclude_block_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_exclude_block_result;
    DefinitionRegion_exclude_block_result DefinitionRegion_exclude_block(diplomat::capi::DefinitionRegion* self, const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView block_name);

    bool DefinitionRegion_intersects_bounds(const diplomat::capi::DefinitionRegion* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    diplomat::capi::DefinitionRegion* DefinitionRegion_shifted(const diplomat::capi::DefinitionRegion* self, int32_t dx, int32_t dy, int32_t dz);

    diplomat::capi::DefinitionRegion* DefinitionRegion_expanded(const diplomat::capi::DefinitionRegion* self, int32_t x, int32_t y, int32_t z);

    diplomat::capi::DefinitionRegion* DefinitionRegion_contracted(const diplomat::capi::DefinitionRegion* self, int32_t amount);

    diplomat::capi::DefinitionRegion* DefinitionRegion_copy(const diplomat::capi::DefinitionRegion* self);

    void DefinitionRegion_set_color(diplomat::capi::DefinitionRegion* self, uint32_t color);

    typedef struct DefinitionRegion_blocks_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_blocks_json_result;
    DefinitionRegion_blocks_json_result DefinitionRegion_blocks_json(const diplomat::capi::DefinitionRegion* self, const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatWrite* write);

    typedef struct DefinitionRegion_sync_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} DefinitionRegion_sync_result;
    DefinitionRegion_sync_result DefinitionRegion_sync(const diplomat::capi::DefinitionRegion* self, diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name);

    void DefinitionRegion_destroy(DefinitionRegion* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::create() {
    auto result = diplomat::capi::DefinitionRegion_create();
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::from_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::DefinitionRegion_from_bounds(min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> DefinitionRegion::from_positions(diplomat::span<const int32_t> positions) {
    auto result = diplomat::capi::DefinitionRegion_from_positions({positions.data(), positions.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Ok<std::unique_ptr<DefinitionRegion>>(std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> DefinitionRegion::from_bounding_boxes(diplomat::span<const int32_t> boxes) {
    auto result = diplomat::capi::DefinitionRegion_from_bounding_boxes({boxes.data(), boxes.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Ok<std::unique_ptr<DefinitionRegion>>(std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void DefinitionRegion::add_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    diplomat::capi::DefinitionRegion_add_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
}

inline void DefinitionRegion::add_point(int32_t x, int32_t y, int32_t z) {
    diplomat::capi::DefinitionRegion_add_point(this->AsFFI(),
        x,
        y,
        z);
}

inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::set_metadata(std::string_view key, std::string_view value) {
    auto result = diplomat::capi::DefinitionRegion_set_metadata(this->AsFFI(),
        {key.data(), key.size()},
        {value.data(), value.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::get_metadata(std::string_view key) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_get_metadata(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::get_metadata_write(std::string_view key, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_get_metadata(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::all_metadata_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_all_metadata_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::all_metadata_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_all_metadata_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::metadata_keys_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_metadata_keys_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::metadata_keys_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_metadata_keys_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::add_filter(std::string_view filter) {
    auto result = diplomat::capi::DefinitionRegion_add_filter(this->AsFFI(),
        {filter.data(), filter.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline bool DefinitionRegion::is_empty() const {
    auto result = diplomat::capi::DefinitionRegion_is_empty(this->AsFFI());
    return result;
}

inline uint64_t DefinitionRegion::volume() const {
    auto result = diplomat::capi::DefinitionRegion_volume(this->AsFFI());
    return result;
}

inline bool DefinitionRegion::contains(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::DefinitionRegion_contains(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void DefinitionRegion::shift(int32_t dx, int32_t dy, int32_t dz) {
    diplomat::capi::DefinitionRegion_shift(this->AsFFI(),
        dx,
        dy,
        dz);
}

inline void DefinitionRegion::expand(int32_t x, int32_t y, int32_t z) {
    diplomat::capi::DefinitionRegion_expand(this->AsFFI(),
        x,
        y,
        z);
}

inline void DefinitionRegion::contract(int32_t amount) {
    diplomat::capi::DefinitionRegion_contract(this->AsFFI(),
        amount);
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::intersected(const DefinitionRegion& other) const {
    auto result = diplomat::capi::DefinitionRegion_intersected(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::union_with(const DefinitionRegion& other) const {
    auto result = diplomat::capi::DefinitionRegion_union_with(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::subtracted(const DefinitionRegion& other) const {
    auto result = diplomat::capi::DefinitionRegion_subtracted(this->AsFFI(),
        other.AsFFI());
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline void DefinitionRegion::merge(const DefinitionRegion& other) {
    diplomat::capi::DefinitionRegion_merge(this->AsFFI(),
        other.AsFFI());
}

inline void DefinitionRegion::union_into(const DefinitionRegion& other) {
    diplomat::capi::DefinitionRegion_union_into(this->AsFFI(),
        other.AsFFI());
}

inline diplomat::result<RegionBounds, NucleationError> DefinitionRegion::bounds() const {
    auto result = diplomat::capi::DefinitionRegion_bounds(this->AsFFI());
    return result.is_ok ? diplomat::result<RegionBounds, NucleationError>(diplomat::Ok<RegionBounds>(RegionBounds::FromFFI(result.ok))) : diplomat::result<RegionBounds, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline Dimensions DefinitionRegion::dimensions() const {
    auto result = diplomat::capi::DefinitionRegion_dimensions(this->AsFFI());
    return Dimensions::FromFFI(result);
}

inline diplomat::result<BlockPos, NucleationError> DefinitionRegion::center() const {
    auto result = diplomat::capi::DefinitionRegion_center(this->AsFFI());
    return result.is_ok ? diplomat::result<BlockPos, NucleationError>(diplomat::Ok<BlockPos>(BlockPos::FromFFI(result.ok))) : diplomat::result<BlockPos, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::center_f32_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_center_f32_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::center_f32_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_center_f32_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::positions_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_positions_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::positions_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_positions_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::positions_sorted_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_positions_sorted_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::positions_sorted_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_positions_sorted_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t DefinitionRegion::box_count() const {
    auto result = diplomat::capi::DefinitionRegion_box_count(this->AsFFI());
    return result;
}

inline diplomat::result<RegionBounds, NucleationError> DefinitionRegion::get_box(uint32_t index) const {
    auto result = diplomat::capi::DefinitionRegion_get_box(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<RegionBounds, NucleationError>(diplomat::Ok<RegionBounds>(RegionBounds::FromFFI(result.ok))) : diplomat::result<RegionBounds, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::boxes_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_boxes_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::boxes_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_boxes_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline bool DefinitionRegion::is_contiguous() const {
    auto result = diplomat::capi::DefinitionRegion_is_contiguous(this->AsFFI());
    return result;
}

inline uint32_t DefinitionRegion::connected_components() const {
    auto result = diplomat::capi::DefinitionRegion_connected_components(this->AsFFI());
    return result;
}

inline void DefinitionRegion::simplify() {
    diplomat::capi::DefinitionRegion_simplify(this->AsFFI());
}

inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> DefinitionRegion::filter_by_block(const Schematic& schematic, std::string_view block_name) const {
    auto result = diplomat::capi::DefinitionRegion_filter_by_block(this->AsFFI(),
        schematic.AsFFI(),
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Ok<std::unique_ptr<DefinitionRegion>>(std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> DefinitionRegion::filter_by_properties(const Schematic& schematic, std::string_view properties_json) const {
    auto result = diplomat::capi::DefinitionRegion_filter_by_properties(this->AsFFI(),
        schematic.AsFFI(),
        {properties_json.data(), properties_json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Ok<std::unique_ptr<DefinitionRegion>>(std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::exclude_block(const Schematic& schematic, std::string_view block_name) {
    auto result = diplomat::capi::DefinitionRegion_exclude_block(this->AsFFI(),
        schematic.AsFFI(),
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline bool DefinitionRegion::intersects_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = diplomat::capi::DefinitionRegion_intersects_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result;
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::shifted(int32_t dx, int32_t dy, int32_t dz) const {
    auto result = diplomat::capi::DefinitionRegion_shifted(this->AsFFI(),
        dx,
        dy,
        dz);
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::expanded(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::DefinitionRegion_expanded(this->AsFFI(),
        x,
        y,
        z);
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::contracted(int32_t amount) const {
    auto result = diplomat::capi::DefinitionRegion_contracted(this->AsFFI(),
        amount);
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline std::unique_ptr<DefinitionRegion> DefinitionRegion::copy() const {
    auto result = diplomat::capi::DefinitionRegion_copy(this->AsFFI());
    return std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result));
}

inline void DefinitionRegion::set_color(uint32_t color) {
    diplomat::capi::DefinitionRegion_set_color(this->AsFFI(),
        color);
}

inline diplomat::result<std::string, NucleationError> DefinitionRegion::blocks_json(const Schematic& schematic) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::DefinitionRegion_blocks_json(this->AsFFI(),
        schematic.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::blocks_json_write(const Schematic& schematic, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::DefinitionRegion_blocks_json(this->AsFFI(),
        schematic.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> DefinitionRegion::sync(Schematic& schematic, std::string_view name) const {
    auto result = diplomat::capi::DefinitionRegion_sync(this->AsFFI(),
        schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::DefinitionRegion* DefinitionRegion::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::DefinitionRegion*>(this);
}

inline diplomat::capi::DefinitionRegion* DefinitionRegion::AsFFI() {
    return reinterpret_cast<diplomat::capi::DefinitionRegion*>(this);
}

inline const DefinitionRegion* DefinitionRegion::FromFFI(const diplomat::capi::DefinitionRegion* ptr) {
    return reinterpret_cast<const DefinitionRegion*>(ptr);
}

inline DefinitionRegion* DefinitionRegion::FromFFI(diplomat::capi::DefinitionRegion* ptr) {
    return reinterpret_cast<DefinitionRegion*>(ptr);
}

inline void DefinitionRegion::operator delete(void* ptr) {
    diplomat::capi::DefinitionRegion_destroy(reinterpret_cast<diplomat::capi::DefinitionRegion*>(ptr));
}


#endif // DefinitionRegion_HPP
