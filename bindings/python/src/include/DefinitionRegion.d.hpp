#ifndef NUCLEATION_DefinitionRegion_D_HPP
#define NUCLEATION_DefinitionRegion_D_HPP

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
namespace capi { struct DefinitionRegion; }
class DefinitionRegion;
namespace capi { struct Schematic; }
class Schematic;
struct BlockPos;
struct Dimensions;
struct RegionBounds;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct DefinitionRegion;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A named sub-volume of a schematic: a union of boxes plus a metadata map.
 */
class DefinitionRegion {
public:

  inline static std::unique_ptr<nucleation::DefinitionRegion> create();

  inline static std::unique_ptr<nucleation::DefinitionRegion> from_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Build a region from single-block positions crossing as flat `[i32]`
   * chunked in threes (PORTING rule 7). Errors with `InvalidArgument` if
   * the length is not a multiple of 3.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> from_positions(nucleation::diplomat::span<const int32_t> positions);

  /**
   * Build a region from bounding boxes crossing as flat `[i32]` chunked in
   * sixes (`min_x, min_y, min_z, max_x, max_y, max_z` per box). Errors
   * with `InvalidArgument` if the length is not a multiple of 6.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> from_bounding_boxes(nucleation::diplomat::span<const int32_t> boxes);

  inline void add_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  inline void add_point(int32_t x, int32_t y, int32_t z);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_metadata(std::string_view key, std::string_view value);

  /**
   * Errors with `NotFound` when the key is absent.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_metadata(std::string_view key) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_metadata_write(std::string_view key, W& writeable_output) const;

  /**
   * The full metadata map, written as a JSON object string (the old ABI
   * returned an array of `"key=value"` strings).
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> all_metadata_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> all_metadata_json_write(W& writeable_output) const;

  /**
   * The metadata keys, written as a JSON array string.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> metadata_keys_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> metadata_keys_json_write(W& writeable_output) const;

  /**
   * Store a filter expression in the region's metadata under the `filter`
   * key.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_filter(std::string_view filter);

  inline bool is_empty() const;

  inline uint64_t volume() const;

  inline bool contains(int32_t x, int32_t y, int32_t z) const;

  inline void shift(int32_t dx, int32_t dy, int32_t dz);

  inline void expand(int32_t x, int32_t y, int32_t z);

  inline void contract(int32_t amount);

  /**
   * A new region: the intersection of `self` and `other`.
   */
  inline std::unique_ptr<nucleation::DefinitionRegion> intersected(const nucleation::DefinitionRegion& other) const;

  /**
   * A new region: the union of `self` and `other`.
   */
  inline std::unique_ptr<nucleation::DefinitionRegion> union_with(const nucleation::DefinitionRegion& other) const;

  /**
   * A new region: `self` minus `other`.
   */
  inline std::unique_ptr<nucleation::DefinitionRegion> subtracted(const nucleation::DefinitionRegion& other) const;

  /**
   * Merge `other`'s boxes and metadata into `self`.
   */
  inline void merge(const nucleation::DefinitionRegion& other);

  /**
   * Union `other`'s boxes into `self` in place.
   */
  inline void union_into(const nucleation::DefinitionRegion& other);

  /**
   * The overall bounding box. Errors with `NotFound` when the region is
   * empty.
   */
  inline nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError> bounds() const;

  inline nucleation::Dimensions dimensions() const;

  /**
   * The center block position. Errors with `NotFound` when the region is
   * empty.
   */
  inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> center() const;

  /**
   * The exact (fractional) center, written as a JSON `[x, y, z]` array of
   * floats. Errors with `NotFound` when the region is empty.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> center_f32_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> center_f32_json_write(W& writeable_output) const;

  /**
   * Every contained position, written as a flat JSON array of ints
   * (`[x0, y0, z0, x1, y1, z1, …]`), deduplicated, in box order.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> positions_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> positions_json_write(W& writeable_output) const;

  /**
   * Every contained position in sorted (y, z, x) order, written as a flat
   * JSON array of ints.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> positions_sorted_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> positions_sorted_json_write(W& writeable_output) const;

  /**
   * The number of boxes making up this region.
   */
  inline uint32_t box_count() const;

  /**
   * The box at `index`. Errors with `NotFound` when out of range.
   */
  inline nucleation::diplomat::result<nucleation::RegionBounds, nucleation::NucleationError> get_box(uint32_t index) const;

  /**
   * Every box, written as a flat JSON array of ints (six ints per box:
   * `min_x, min_y, min_z, max_x, max_y, max_z`).
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> boxes_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> boxes_json_write(W& writeable_output) const;

  inline bool is_contiguous() const;

  inline uint32_t connected_components() const;

  /**
   * Merge overlapping/adjacent boxes into a minimal representation.
   */
  inline void simplify();

  /**
   * A new region containing only the positions where `schematic` has a
   * block named `block_name`.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> filter_by_block(const nucleation::Schematic& schematic, std::string_view block_name) const;

  /**
   * A new region containing only the positions where the block in
   * `schematic` matches every property in `properties_json` (a JSON
   * object of property name → value strings).
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> filter_by_properties(const nucleation::Schematic& schematic, std::string_view properties_json) const;

  /**
   * Remove every position where `schematic` has a block named
   * `block_name` (in place).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_block(const nucleation::Schematic& schematic, std::string_view block_name);

  inline bool intersects_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  /**
   * A new region shifted by (`dx`, `dy`, `dz`).
   */
  inline std::unique_ptr<nucleation::DefinitionRegion> shifted(int32_t dx, int32_t dy, int32_t dz) const;

  /**
   * A new region expanded by (`x`, `y`, `z`) on each axis.
   */
  inline std::unique_ptr<nucleation::DefinitionRegion> expanded(int32_t x, int32_t y, int32_t z) const;

  /**
   * A new region contracted by `amount` on every axis.
   */
  inline std::unique_ptr<nucleation::DefinitionRegion> contracted(int32_t amount) const;

  /**
   * A deep copy of this region.
   */
  inline std::unique_ptr<nucleation::DefinitionRegion> copy() const;

  /**
   * Store a display color (`0xRRGGBB`) in the region's metadata.
   */
  inline void set_color(uint32_t color);

  /**
   * The blocks of `schematic` inside this region, written as a JSON array
   * of `{"x", "y", "z", "name", "properties"}` objects (the old ABI
   * returned a `CBlockArray`).
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> blocks_json(const nucleation::Schematic& schematic) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> blocks_json_write(const nucleation::Schematic& schematic, W& writeable_output) const;

  /**
   * Write this region into `schematic`'s definition-region map under
   * `name` (insert or overwrite).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> sync(nucleation::Schematic& schematic, std::string_view name) const;

    inline const nucleation::capi::DefinitionRegion* AsFFI() const;
    inline nucleation::capi::DefinitionRegion* AsFFI();
    inline static const nucleation::DefinitionRegion* FromFFI(const nucleation::capi::DefinitionRegion* ptr);
    inline static nucleation::DefinitionRegion* FromFFI(nucleation::capi::DefinitionRegion* ptr);
    inline static void operator delete(void* ptr);
private:
    DefinitionRegion() = delete;
    DefinitionRegion(const nucleation::DefinitionRegion&) = delete;
    DefinitionRegion(nucleation::DefinitionRegion&&) noexcept = delete;
    DefinitionRegion operator=(const nucleation::DefinitionRegion&) = delete;
    DefinitionRegion operator=(nucleation::DefinitionRegion&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_DefinitionRegion_D_HPP
