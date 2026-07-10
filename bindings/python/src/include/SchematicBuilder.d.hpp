#ifndef NUCLEATION_SchematicBuilder_D_HPP
#define NUCLEATION_SchematicBuilder_D_HPP

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
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct SchematicBuilder; }
class SchematicBuilder;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct SchematicBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Fluent builder for schematics from character-mapped text layers.
 *
 * `build` is consuming (PORTING rule 11): the inner builder is held in an
 * `Option` and taken on `build`; every method afterwards returns
 * `AlreadyConsumed`.
 */
class SchematicBuilder {
public:

  inline static std::unique_ptr<nucleation::SchematicBuilder> create();

  /**
   * Parse a builder from the canonical template text format.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::SchematicBuilder>, nucleation::NucleationError> from_template(std::string_view template_);

  /**
   * Set the schematic name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> name(std::string_view name);

  /**
   * Map a palette character to a block string. `ch` must contain exactly
   * one character (its first char is used).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> map(std::string_view ch, std::string_view block);

  /**
   * Append layers. `layers_json` is a JSON array of arrays of row strings,
   * e.g. `[["ab","cd"],["ef","gh"]]`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> layers(std::string_view layers_json);

  /**
   * Append a single layer of rows. `rows_json` is a JSON array of strings,
   * e.g. `["abc", "def"]`. Equivalent to a one-element layers array.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> layer(std::string_view rows_json);

  /**
   * Bulk-register palette characters. `pairs_json` is a JSON array of
   * `[char, block]` two-element arrays, e.g.
   * `[["c", "minecraft:gray_concrete"], [" ", "minecraft:air"]]`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> palette(std::string_view pairs_json);

  /**
   * Set the build offset applied to every placed block.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> offset(int32_t x, int32_t y, int32_t z);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> use_standard_palette();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> use_minimal_palette();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> use_compact_palette();

  /**
   * Run pre-build validation without consuming the builder.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> validate() const;

  /**
   * Serialize the builder back into the canonical template format.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_template() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_template_write(W& writeable_output) const;

  /**
   * Build the schematic. Consuming: the builder cannot be reused afterwards
   * (subsequent calls return `AlreadyConsumed`), including after a failed
   * build.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> build();

    inline const nucleation::capi::SchematicBuilder* AsFFI() const;
    inline nucleation::capi::SchematicBuilder* AsFFI();
    inline static const nucleation::SchematicBuilder* FromFFI(const nucleation::capi::SchematicBuilder* ptr);
    inline static nucleation::SchematicBuilder* FromFFI(nucleation::capi::SchematicBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    SchematicBuilder() = delete;
    SchematicBuilder(const nucleation::SchematicBuilder&) = delete;
    SchematicBuilder(nucleation::SchematicBuilder&&) noexcept = delete;
    SchematicBuilder operator=(const nucleation::SchematicBuilder&) = delete;
    SchematicBuilder operator=(nucleation::SchematicBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_SchematicBuilder_D_HPP
