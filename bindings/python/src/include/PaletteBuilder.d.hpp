#ifndef NUCLEATION_PaletteBuilder_D_HPP
#define NUCLEATION_PaletteBuilder_D_HPP

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
namespace capi { struct Palette; }
class Palette;
namespace capi { struct PaletteBuilder; }
class PaletteBuilder;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct PaletteBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Filter-driven palette construction (wraps
 * {@link crate::building::PaletteBuilder}, which fronts blockpedia's
 * `BlockFilter`). Call flag methods, then `build` — the builder is
 * consumed; further calls error with `AlreadyConsumed`.
 */
class PaletteBuilder {
public:

  /**
   * A builder matching every colored block (no filters yet).
   */
  inline static std::unique_ptr<nucleation::PaletteBuilder> create();

  /**
   * Exclude gravity-affected blocks (sand, gravel, ...).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_falling();

  /**
   * Exclude blocks with block entities (chests, furnaces, ...).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_tile_entities();

  /**
   * Keep only full cube blocks (no stairs, slabs, fences, ...).
   * Metadata-driven: uses the official model geometry extracted from
   * the vanilla jars, not block-name guessing.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> full_blocks_only();

  /**
   * Exclude blocks that need supporting blocks (torches, rails, ...).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_needs_support();

  /**
   * Exclude transparent/translucent blocks (glass, leaves, ...).
   * Metadata-driven: uses the per-block transparency flag from the
   * block-data pipeline, not block-name guessing.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_transparent();

  /**
   * Exclude light-emitting blocks (glowstone, lanterns, ...).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_light_sources();

  /**
   * Keep only blocks obtainable in survival.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> survival_only();

  /**
   * Exclude blocks whose id contains `keyword`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_keyword(std::string_view keyword);

  /**
   * Keep only blocks whose id contains `keyword` (repeatable; matches
   * any of the included keywords).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> include_keyword(std::string_view keyword);

  /**
   * Require the vanilla block tag `t` (`minecraft:wool` or short
   * `wool`, nested paths like `mineable/pickaxe` too). Repeatable —
   * a block must carry ALL required tags (AND semantics).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> tag(std::string_view t);

  /**
   * Exclude blocks carrying the vanilla block tag `t` (any listed
   * tag disqualifies). Repeatable.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> exclude_tag(std::string_view t);

  /**
   * Keep only blocks of the official definition kind `k`
   * (`minecraft:stair` or short `stair`; plain full blocks are
   * `minecraft:block`). Repeatable — a block matching ANY listed
   * kind passes (OR semantics).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> kind(std::string_view k);

  /**
   * Build the palette; consumes the builder.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError> build();

    inline const nucleation::capi::PaletteBuilder* AsFFI() const;
    inline nucleation::capi::PaletteBuilder* AsFFI();
    inline static const nucleation::PaletteBuilder* FromFFI(const nucleation::capi::PaletteBuilder* ptr);
    inline static nucleation::PaletteBuilder* FromFFI(nucleation::capi::PaletteBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    PaletteBuilder() = delete;
    PaletteBuilder(const nucleation::PaletteBuilder&) = delete;
    PaletteBuilder(nucleation::PaletteBuilder&&) noexcept = delete;
    PaletteBuilder operator=(const nucleation::PaletteBuilder&) = delete;
    PaletteBuilder operator=(nucleation::PaletteBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_PaletteBuilder_D_HPP
