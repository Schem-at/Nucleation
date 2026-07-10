#ifndef NUCLEATION_SchematicRegions_D_HPP
#define NUCLEATION_SchematicRegions_D_HPP

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
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct SchematicRegions;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace type for the schematic-attached definition-region operations
 * (PORTING rule 12; the `Schematic` opaque lives in another module, so
 * these are statics taking it explicitly, like `Autostack`).
 */
class SchematicRegions {
public:

  /**
   * Insert (or overwrite) `region` under `name`.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add(nucleation::Schematic& schematic, std::string_view name, const nucleation::DefinitionRegion& region);

  /**
   * Overwrite the region stored under `name` (identical to `add` in the
   * old ABI too; kept as a separate method for 1:1 coverage of
   * `schematic_update_region`).
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> update(nucleation::Schematic& schematic, std::string_view name, const nucleation::DefinitionRegion& region);

  /**
   * A copy of the region stored under `name`. Errors with `NotFound` when
   * absent.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> get(const nucleation::Schematic& schematic, std::string_view name);

  /**
   * Remove the region stored under `name`. Errors with `NotFound` when
   * absent.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> remove(nucleation::Schematic& schematic, std::string_view name);

  /**
   * The names of every definition region, written as a JSON array string.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> names_json(const nucleation::Schematic& schematic);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> names_json_write(const nucleation::Schematic& schematic, W& writeable_output);

  /**
   * Create an empty region under `name`.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> create(nucleation::Schematic& schematic, std::string_view name);

  /**
   * Create a single-point region under `name`.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> create_from_point(nucleation::Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z);

  /**
   * Create a single-box region under `name`.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> create_from_bounds(nucleation::Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Create a single-box region under `name` and return a copy of it.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> create_region(nucleation::Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Add a box to the region stored under `name`. Errors with `NotFound`
   * when absent.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_bounds_to(nucleation::Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Add a point to the region stored under `name`. Errors with `NotFound`
   * when absent.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_point_to(nucleation::Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z);

  /**
   * Set a metadata entry on the region stored under `name`. Errors with
   * `NotFound` when absent.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_metadata_on(nucleation::Schematic& schematic, std::string_view name, std::string_view key, std::string_view value);

  /**
   * Shift the region stored under `name` by (`dx`, `dy`, `dz`). Errors
   * with `NotFound` when absent.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> shift_region(nucleation::Schematic& schematic, std::string_view name, int32_t dx, int32_t dy, int32_t dz);

    inline const nucleation::capi::SchematicRegions* AsFFI() const;
    inline nucleation::capi::SchematicRegions* AsFFI();
    inline static const nucleation::SchematicRegions* FromFFI(const nucleation::capi::SchematicRegions* ptr);
    inline static nucleation::SchematicRegions* FromFFI(nucleation::capi::SchematicRegions* ptr);
    inline static void operator delete(void* ptr);
private:
    SchematicRegions() = delete;
    SchematicRegions(const nucleation::SchematicRegions&) = delete;
    SchematicRegions(nucleation::SchematicRegions&&) noexcept = delete;
    SchematicRegions operator=(const nucleation::SchematicRegions&) = delete;
    SchematicRegions operator=(nucleation::SchematicRegions&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_SchematicRegions_D_HPP
