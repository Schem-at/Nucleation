#ifndef SchematicBuilder_D_HPP
#define SchematicBuilder_D_HPP

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
class NucleationError;




namespace diplomat {
namespace capi {
    struct SchematicBuilder;
} // namespace capi
} // namespace

/**
 * Fluent builder for schematics from character-mapped text layers.
 *
 * `build` is consuming (PORTING rule 11): the inner builder is held in an
 * `Option` and taken on `build`; every method afterwards returns
 * `AlreadyConsumed`.
 */
class SchematicBuilder {
public:

  inline static std::unique_ptr<SchematicBuilder> create();

  /**
   * Parse a builder from the canonical template text format.
   */
  inline static diplomat::result<std::unique_ptr<SchematicBuilder>, NucleationError> from_template(std::string_view template_);

  /**
   * Set the schematic name.
   */
  inline diplomat::result<std::monostate, NucleationError> name(std::string_view name);

  /**
   * Map a palette character to a block string. `ch` must contain exactly
   * one character (its first char is used).
   */
  inline diplomat::result<std::monostate, NucleationError> map(std::string_view ch, std::string_view block);

  /**
   * Append layers. `layers_json` is a JSON array of arrays of row strings,
   * e.g. `[["ab","cd"],["ef","gh"]]`.
   */
  inline diplomat::result<std::monostate, NucleationError> layers(std::string_view layers_json);

  /**
   * Append a single layer of rows. `rows_json` is a JSON array of strings,
   * e.g. `["abc", "def"]`. Equivalent to a one-element layers array.
   */
  inline diplomat::result<std::monostate, NucleationError> layer(std::string_view rows_json);

  /**
   * Bulk-register palette characters. `pairs_json` is a JSON array of
   * `[char, block]` two-element arrays, e.g.
   * `[["c", "minecraft:gray_concrete"], [" ", "minecraft:air"]]`.
   */
  inline diplomat::result<std::monostate, NucleationError> palette(std::string_view pairs_json);

  /**
   * Set the build offset applied to every placed block.
   */
  inline diplomat::result<std::monostate, NucleationError> offset(int32_t x, int32_t y, int32_t z);

  inline diplomat::result<std::monostate, NucleationError> use_standard_palette();

  inline diplomat::result<std::monostate, NucleationError> use_minimal_palette();

  inline diplomat::result<std::monostate, NucleationError> use_compact_palette();

  /**
   * Run pre-build validation without consuming the builder.
   */
  inline diplomat::result<std::monostate, NucleationError> validate() const;

  /**
   * Serialize the builder back into the canonical template format.
   */
  inline diplomat::result<std::string, NucleationError> to_template() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_template_write(W& writeable_output) const;

  /**
   * Build the schematic. Consuming: the builder cannot be reused afterwards
   * (subsequent calls return `AlreadyConsumed`), including after a failed
   * build.
   */
  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> build();

    inline const diplomat::capi::SchematicBuilder* AsFFI() const;
    inline diplomat::capi::SchematicBuilder* AsFFI();
    inline static const SchematicBuilder* FromFFI(const diplomat::capi::SchematicBuilder* ptr);
    inline static SchematicBuilder* FromFFI(diplomat::capi::SchematicBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    SchematicBuilder() = delete;
    SchematicBuilder(const SchematicBuilder&) = delete;
    SchematicBuilder(SchematicBuilder&&) noexcept = delete;
    SchematicBuilder operator=(const SchematicBuilder&) = delete;
    SchematicBuilder operator=(SchematicBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // SchematicBuilder_D_HPP
