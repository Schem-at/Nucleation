#ifndef PaletteBuilder_D_HPP
#define PaletteBuilder_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Palette; }
class Palette;
class NucleationError;




namespace diplomat {
namespace capi {
    struct PaletteBuilder;
} // namespace capi
} // namespace

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
  inline static std::unique_ptr<PaletteBuilder> create();

  /**
   * Exclude gravity-affected blocks (sand, gravel, ...).
   */
  inline diplomat::result<std::monostate, NucleationError> exclude_falling();

  /**
   * Exclude blocks with block entities (chests, furnaces, ...).
   */
  inline diplomat::result<std::monostate, NucleationError> exclude_tile_entities();

  /**
   * Keep only full cube blocks (no stairs, slabs, fences, ...).
   */
  inline diplomat::result<std::monostate, NucleationError> full_blocks_only();

  /**
   * Exclude blocks that need supporting blocks (torches, rails, ...).
   */
  inline diplomat::result<std::monostate, NucleationError> exclude_needs_support();

  /**
   * Exclude transparent/translucent blocks (glass, leaves, ...).
   */
  inline diplomat::result<std::monostate, NucleationError> exclude_transparent();

  /**
   * Exclude light-emitting blocks (glowstone, lanterns, ...).
   */
  inline diplomat::result<std::monostate, NucleationError> exclude_light_sources();

  /**
   * Keep only blocks obtainable in survival.
   */
  inline diplomat::result<std::monostate, NucleationError> survival_only();

  /**
   * Exclude blocks whose id contains `keyword`.
   */
  inline diplomat::result<std::monostate, NucleationError> exclude_keyword(std::string_view keyword);

  /**
   * Keep only blocks whose id contains `keyword` (repeatable; matches
   * any of the included keywords).
   */
  inline diplomat::result<std::monostate, NucleationError> include_keyword(std::string_view keyword);

  /**
   * Build the palette; consumes the builder.
   */
  inline diplomat::result<std::unique_ptr<Palette>, NucleationError> build();

    inline const diplomat::capi::PaletteBuilder* AsFFI() const;
    inline diplomat::capi::PaletteBuilder* AsFFI();
    inline static const PaletteBuilder* FromFFI(const diplomat::capi::PaletteBuilder* ptr);
    inline static PaletteBuilder* FromFFI(diplomat::capi::PaletteBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    PaletteBuilder() = delete;
    PaletteBuilder(const PaletteBuilder&) = delete;
    PaletteBuilder(PaletteBuilder&&) noexcept = delete;
    PaletteBuilder operator=(const PaletteBuilder&) = delete;
    PaletteBuilder operator=(PaletteBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // PaletteBuilder_D_HPP
