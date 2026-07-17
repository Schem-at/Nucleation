#ifndef NUCLEATION_Palette_D_HPP
#define NUCLEATION_Palette_D_HPP

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
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Palette;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A set of colored blocks that color/gradient brushes snap their computed
 * colors to (nearest neighbor in Oklab space). Wraps an Arc'd
 * {@link crate::building::BlockPalette}; sharing one palette across many
 * brushes is cheap.
 */
class Palette {
public:

  /**
   * Every block blockpedia knows a color for (the default palette
   * brushes use when none is set).
   */
  inline static std::unique_ptr<nucleation::Palette> all();

  /**
   * Only solid blocks: no transparency, gravity, tile entities, or
   * support requirements.
   */
  inline static std::unique_ptr<nucleation::Palette> solid();

  /**
   * Conservative structural set (full building blocks).
   */
  inline static std::unique_ptr<nucleation::Palette> structural();

  /**
   * Decorative set: allows stairs/slabs but no tile entities.
   */
  inline static std::unique_ptr<nucleation::Palette> decorative();

  /**
   * The 16 concrete colors (excludes concrete powder).
   */
  inline static std::unique_ptr<nucleation::Palette> concrete();

  /**
   * The 16 wool colors.
   */
  inline static std::unique_ptr<nucleation::Palette> wool();

  /**
   * Terracotta colors (excludes glazed variants).
   */
  inline static std::unique_ptr<nucleation::Palette> terracotta();

  /**
   * Grayscale-leaning blocks (stones, basalt, deepslate, ...).
   */
  inline static std::unique_ptr<nucleation::Palette> grayscale();

  /**
   * Custom palette from a JSON array of block ids, e.g.
   * `["minecraft:stone", "minecraft:oak_planks"]`. Ids blockpedia has
   * no color for are silently skipped — check `len` afterwards.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError> from_block_ids(std::string_view ids_json);

  /**
   * Number of blocks in the palette.
   */
  inline size_t len() const;

  /**
   * The palette's block ids as a JSON array string.
   */
  inline std::string block_ids_json() const;
  template<typename W>
  inline void block_ids_json_write(W& writeable_output) const;

  /**
   * The palette block whose color is closest (Oklab distance) to the
   * given RGB. Errors with `NotFound` on an empty palette.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> closest_block(uint8_t r, uint8_t g, uint8_t b) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> closest_block_write(uint8_t r, uint8_t g, uint8_t b, W& writeable_output) const;

    inline const nucleation::capi::Palette* AsFFI() const;
    inline nucleation::capi::Palette* AsFFI();
    inline static const nucleation::Palette* FromFFI(const nucleation::capi::Palette* ptr);
    inline static nucleation::Palette* FromFFI(nucleation::capi::Palette* ptr);
    inline static void operator delete(void* ptr);
private:
    Palette() = delete;
    Palette(const nucleation::Palette&) = delete;
    Palette(nucleation::Palette&&) noexcept = delete;
    Palette operator=(const nucleation::Palette&) = delete;
    Palette operator=(nucleation::Palette&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Palette_D_HPP
