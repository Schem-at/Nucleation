#ifndef Palette_D_HPP
#define Palette_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct Palette;
} // namespace capi
} // namespace

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
  inline static std::unique_ptr<Palette> all();

  /**
   * Only solid blocks: no transparency, gravity, tile entities, or
   * support requirements.
   */
  inline static std::unique_ptr<Palette> solid();

  /**
   * Conservative structural set (full building blocks).
   */
  inline static std::unique_ptr<Palette> structural();

  /**
   * Decorative set: allows stairs/slabs but no tile entities.
   */
  inline static std::unique_ptr<Palette> decorative();

  /**
   * The 16 concrete colors (excludes concrete powder).
   */
  inline static std::unique_ptr<Palette> concrete();

  /**
   * The 16 wool colors.
   */
  inline static std::unique_ptr<Palette> wool();

  /**
   * Terracotta colors (excludes glazed variants).
   */
  inline static std::unique_ptr<Palette> terracotta();

  /**
   * Grayscale-leaning blocks (stones, basalt, deepslate, ...).
   */
  inline static std::unique_ptr<Palette> grayscale();

  /**
   * Custom palette from a JSON array of block ids, e.g.
   * `["minecraft:stone", "minecraft:oak_planks"]`. Ids blockpedia has
   * no color for are silently skipped — check `len` afterwards.
   */
  inline static diplomat::result<std::unique_ptr<Palette>, NucleationError> from_block_ids(std::string_view ids_json);

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
  inline diplomat::result<std::string, NucleationError> closest_block(uint8_t r, uint8_t g, uint8_t b) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> closest_block_write(uint8_t r, uint8_t g, uint8_t b, W& writeable_output) const;

    inline const diplomat::capi::Palette* AsFFI() const;
    inline diplomat::capi::Palette* AsFFI();
    inline static const Palette* FromFFI(const diplomat::capi::Palette* ptr);
    inline static Palette* FromFFI(diplomat::capi::Palette* ptr);
    inline static void operator delete(void* ptr);
private:
    Palette() = delete;
    Palette(const Palette&) = delete;
    Palette(Palette&&) noexcept = delete;
    Palette operator=(const Palette&) = delete;
    Palette operator=(Palette&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Palette_D_HPP
