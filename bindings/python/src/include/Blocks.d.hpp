#ifndef NUCLEATION_Blocks_D_HPP
#define NUCLEATION_Blocks_D_HPP

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
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Blocks;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace for read-only queries over the built-in block table
 * (Java 26.2 blocks + official semantics extracted from the vanilla
 * jars: definition kinds, base-block links, block tags, model
 * geometry). All list results are JSON array strings sorted by id;
 * block/tag/kind arguments accept both `minecraft:`-prefixed and
 * short forms (`minecraft:oak_stairs` / `oak_stairs`).
 */
class Blocks {
public:

  /**
   * Full facts for one block as a JSON object:
   * `{id, kind, base_block, tags: [...], full_cube, transparent,
   * color: [r, g, b] | null, properties: {name: [values...]},
   * default_state: {name: value}}`. `kind` is the official
   * definition kind (`minecraft:stair`, plain full blocks are
   * `minecraft:block`); `base_block` is the block this one is a
   * shape variant of (or `null`); `color` is the texture-derived
   * average RGB. Errors with `NotFound` for unknown ids.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> get_json(std::string_view id);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_json_write(std::string_view id, W& writeable_output);

  /**
   * All known block ids as a sorted JSON array string.
   */
  inline static std::string ids_json();
  template<typename W>
  inline static void ids_json_write(W& writeable_output);

  /**
   * Ids of every block carrying the vanilla block tag, as a sorted
   * Blocks whose measured texture color is within `max_distance`
   * (Oklab; ~0.05 = same color family, ~0.15 = generous) of the given
   * RGB, as a JSON array of `{"id", "color": [r,g,b], "distance"}`
   * sorted nearest-first. Blocks without color data never match.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> by_color_json(uint8_t r, uint8_t g, uint8_t b, float max_distance);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> by_color_json_write(uint8_t r, uint8_t g, uint8_t b, float max_distance, W& writeable_output);

  /**
   * JSON array string (`[]` for unknown tags). Accepts
   * `minecraft:wool` and short `wool` forms, including nested paths
   * like `mineable/pickaxe`.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> by_tag_json(std::string_view tag);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> by_tag_json_write(std::string_view tag, W& writeable_output);

  /**
   * Ids of every block of the given official definition kind
   * (`minecraft:stair`, `minecraft:slab`, `minecraft:door`, ...), as
   * a sorted JSON array string (`[]` for unknown kinds).
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> by_kind_json(std::string_view kind);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> by_kind_json_write(std::string_view kind, W& writeable_output);

  /**
   * The base block followed by all its shape variants — blocks whose
   * `base_block` is `base_id` (stairs, slabs, walls, fences of the
   * base) — as a JSON array string. The base itself is always first;
   * variants follow sorted by id. Errors with `NotFound` for unknown
   * base ids.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> variants_of_json(std::string_view base_id);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> variants_of_json_write(std::string_view base_id, W& writeable_output);

  /**
   * All known vanilla block tag names as a sorted JSON array string
   * (`minecraft:`-prefixed, e.g. `minecraft:wool`).
   */
  inline static std::string tags_json();
  template<typename W>
  inline static void tags_json_write(W& writeable_output);

  /**
   * Every property-value combination of the block as a JSON array of
   * `{prop: value}` objects (a single `{}` entry for property-less
   * blocks). Errors with `NotFound` for unknown ids and with
   * `InvalidArgument` if the combination count exceeds 4096 (guard
   * against pathological output; the current data tops out at 1350
   * for `minecraft:note_block`).
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> states_json(std::string_view id);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> states_json_write(std::string_view id, W& writeable_output);

  /**
   * Total number of blocks in the table.
   */
  inline static size_t count();

    inline const nucleation::capi::Blocks* AsFFI() const;
    inline nucleation::capi::Blocks* AsFFI();
    inline static const nucleation::Blocks* FromFFI(const nucleation::capi::Blocks* ptr);
    inline static nucleation::Blocks* FromFFI(nucleation::capi::Blocks* ptr);
    inline static void operator delete(void* ptr);
private:
    Blocks() = delete;
    Blocks(const nucleation::Blocks&) = delete;
    Blocks(nucleation::Blocks&&) noexcept = delete;
    Blocks operator=(const nucleation::Blocks&) = delete;
    Blocks operator=(nucleation::Blocks&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Blocks_D_HPP
