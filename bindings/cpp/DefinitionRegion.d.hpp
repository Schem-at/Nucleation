#ifndef DefinitionRegion_D_HPP
#define DefinitionRegion_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
struct BlockPos;
struct Dimensions;
struct RegionBounds;
class NucleationError;




namespace diplomat {
namespace capi {
    struct DefinitionRegion;
} // namespace capi
} // namespace

/**
 * A named sub-volume of a schematic: a union of boxes plus a metadata map.
 */
class DefinitionRegion {
public:

  inline static std::unique_ptr<DefinitionRegion> create();

  inline static std::unique_ptr<DefinitionRegion> from_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Build a region from single-block positions crossing as flat `[i32]`
   * chunked in threes (PORTING rule 7). Errors with `InvalidArgument` if
   * the length is not a multiple of 3.
   */
  inline static diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> from_positions(diplomat::span<const int32_t> positions);

  /**
   * Build a region from bounding boxes crossing as flat `[i32]` chunked in
   * sixes (`min_x, min_y, min_z, max_x, max_y, max_z` per box). Errors
   * with `InvalidArgument` if the length is not a multiple of 6.
   */
  inline static diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> from_bounding_boxes(diplomat::span<const int32_t> boxes);

  inline void add_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  inline void add_point(int32_t x, int32_t y, int32_t z);

  inline diplomat::result<std::monostate, NucleationError> set_metadata(std::string_view key, std::string_view value);

  /**
   * Errors with `NotFound` when the key is absent.
   */
  inline diplomat::result<std::string, NucleationError> get_metadata(std::string_view key) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> get_metadata_write(std::string_view key, W& writeable_output) const;

  /**
   * The full metadata map, written as a JSON object string (the old ABI
   * returned an array of `"key=value"` strings).
   */
  inline diplomat::result<std::string, NucleationError> all_metadata_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> all_metadata_json_write(W& writeable_output) const;

  /**
   * The metadata keys, written as a JSON array string.
   */
  inline diplomat::result<std::string, NucleationError> metadata_keys_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> metadata_keys_json_write(W& writeable_output) const;

  /**
   * Store a filter expression in the region's metadata under the `filter`
   * key.
   */
  inline diplomat::result<std::monostate, NucleationError> add_filter(std::string_view filter);

  inline bool is_empty() const;

  inline uint64_t volume() const;

  inline bool contains(int32_t x, int32_t y, int32_t z) const;

  inline void shift(int32_t dx, int32_t dy, int32_t dz);

  inline void expand(int32_t x, int32_t y, int32_t z);

  inline void contract(int32_t amount);

  /**
   * A new region: the intersection of `self` and `other`.
   */
  inline std::unique_ptr<DefinitionRegion> intersected(const DefinitionRegion& other) const;

  /**
   * A new region: the union of `self` and `other`.
   */
  inline std::unique_ptr<DefinitionRegion> union_with(const DefinitionRegion& other) const;

  /**
   * A new region: `self` minus `other`.
   */
  inline std::unique_ptr<DefinitionRegion> subtracted(const DefinitionRegion& other) const;

  /**
   * Merge `other`'s boxes and metadata into `self`.
   */
  inline void merge(const DefinitionRegion& other);

  /**
   * Union `other`'s boxes into `self` in place.
   */
  inline void union_into(const DefinitionRegion& other);

  /**
   * The overall bounding box. Errors with `NotFound` when the region is
   * empty.
   */
  inline diplomat::result<RegionBounds, NucleationError> bounds() const;

  inline Dimensions dimensions() const;

  /**
   * The center block position. Errors with `NotFound` when the region is
   * empty.
   */
  inline diplomat::result<BlockPos, NucleationError> center() const;

  /**
   * The exact (fractional) center, written as a JSON `[x, y, z]` array of
   * floats. Errors with `NotFound` when the region is empty.
   */
  inline diplomat::result<std::string, NucleationError> center_f32_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> center_f32_json_write(W& writeable_output) const;

  /**
   * Every contained position, written as a flat JSON array of ints
   * (`[x0, y0, z0, x1, y1, z1, …]`), deduplicated, in box order.
   */
  inline diplomat::result<std::string, NucleationError> positions_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> positions_json_write(W& writeable_output) const;

  /**
   * Every contained position in sorted (y, z, x) order, written as a flat
   * JSON array of ints.
   */
  inline diplomat::result<std::string, NucleationError> positions_sorted_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> positions_sorted_json_write(W& writeable_output) const;

  /**
   * The number of boxes making up this region.
   */
  inline uint32_t box_count() const;

  /**
   * The box at `index`. Errors with `NotFound` when out of range.
   */
  inline diplomat::result<RegionBounds, NucleationError> get_box(uint32_t index) const;

  /**
   * Every box, written as a flat JSON array of ints (six ints per box:
   * `min_x, min_y, min_z, max_x, max_y, max_z`).
   */
  inline diplomat::result<std::string, NucleationError> boxes_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> boxes_json_write(W& writeable_output) const;

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
  inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> filter_by_block(const Schematic& schematic, std::string_view block_name) const;

  /**
   * A new region containing only the positions where the block in
   * `schematic` matches every property in `properties_json` (a JSON
   * object of property name → value strings).
   */
  inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> filter_by_properties(const Schematic& schematic, std::string_view properties_json) const;

  /**
   * Remove every position where `schematic` has a block named
   * `block_name` (in place).
   */
  inline diplomat::result<std::monostate, NucleationError> exclude_block(const Schematic& schematic, std::string_view block_name);

  inline bool intersects_bounds(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  /**
   * A new region shifted by (`dx`, `dy`, `dz`).
   */
  inline std::unique_ptr<DefinitionRegion> shifted(int32_t dx, int32_t dy, int32_t dz) const;

  /**
   * A new region expanded by (`x`, `y`, `z`) on each axis.
   */
  inline std::unique_ptr<DefinitionRegion> expanded(int32_t x, int32_t y, int32_t z) const;

  /**
   * A new region contracted by `amount` on every axis.
   */
  inline std::unique_ptr<DefinitionRegion> contracted(int32_t amount) const;

  /**
   * A deep copy of this region.
   */
  inline std::unique_ptr<DefinitionRegion> copy() const;

  /**
   * Store a display color (`0xRRGGBB`) in the region's metadata.
   */
  inline void set_color(uint32_t color);

  /**
   * The blocks of `schematic` inside this region, written as a JSON array
   * of `{"x", "y", "z", "name", "properties"}` objects (the old ABI
   * returned a `CBlockArray`).
   */
  inline diplomat::result<std::string, NucleationError> blocks_json(const Schematic& schematic) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> blocks_json_write(const Schematic& schematic, W& writeable_output) const;

  /**
   * Write this region into `schematic`'s definition-region map under
   * `name` (insert or overwrite).
   */
  inline diplomat::result<std::monostate, NucleationError> sync(Schematic& schematic, std::string_view name) const;

    inline const diplomat::capi::DefinitionRegion* AsFFI() const;
    inline diplomat::capi::DefinitionRegion* AsFFI();
    inline static const DefinitionRegion* FromFFI(const diplomat::capi::DefinitionRegion* ptr);
    inline static DefinitionRegion* FromFFI(diplomat::capi::DefinitionRegion* ptr);
    inline static void operator delete(void* ptr);
private:
    DefinitionRegion() = delete;
    DefinitionRegion(const DefinitionRegion&) = delete;
    DefinitionRegion(DefinitionRegion&&) noexcept = delete;
    DefinitionRegion operator=(const DefinitionRegion&) = delete;
    DefinitionRegion operator=(DefinitionRegion&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // DefinitionRegion_D_HPP
