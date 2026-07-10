#ifndef SchematicRegions_D_HPP
#define SchematicRegions_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct DefinitionRegion; }
class DefinitionRegion;
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct SchematicRegions;
} // namespace capi
} // namespace

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
  inline static diplomat::result<std::monostate, NucleationError> add(Schematic& schematic, std::string_view name, const DefinitionRegion& region);

  /**
   * Overwrite the region stored under `name` (identical to `add` in the
   * old ABI too; kept as a separate method for 1:1 coverage of
   * `schematic_update_region`).
   */
  inline static diplomat::result<std::monostate, NucleationError> update(Schematic& schematic, std::string_view name, const DefinitionRegion& region);

  /**
   * A copy of the region stored under `name`. Errors with `NotFound` when
   * absent.
   */
  inline static diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> get(const Schematic& schematic, std::string_view name);

  /**
   * Remove the region stored under `name`. Errors with `NotFound` when
   * absent.
   */
  inline static diplomat::result<std::monostate, NucleationError> remove(Schematic& schematic, std::string_view name);

  /**
   * The names of every definition region, written as a JSON array string.
   */
  inline static diplomat::result<std::string, NucleationError> names_json(const Schematic& schematic);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> names_json_write(const Schematic& schematic, W& writeable_output);

  /**
   * Create an empty region under `name`.
   */
  inline static diplomat::result<std::monostate, NucleationError> create(Schematic& schematic, std::string_view name);

  /**
   * Create a single-point region under `name`.
   */
  inline static diplomat::result<std::monostate, NucleationError> create_from_point(Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z);

  /**
   * Create a single-box region under `name`.
   */
  inline static diplomat::result<std::monostate, NucleationError> create_from_bounds(Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Create a single-box region under `name` and return a copy of it.
   */
  inline static diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> create_region(Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Add a box to the region stored under `name`. Errors with `NotFound`
   * when absent.
   */
  inline static diplomat::result<std::monostate, NucleationError> add_bounds_to(Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Add a point to the region stored under `name`. Errors with `NotFound`
   * when absent.
   */
  inline static diplomat::result<std::monostate, NucleationError> add_point_to(Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z);

  /**
   * Set a metadata entry on the region stored under `name`. Errors with
   * `NotFound` when absent.
   */
  inline static diplomat::result<std::monostate, NucleationError> set_metadata_on(Schematic& schematic, std::string_view name, std::string_view key, std::string_view value);

  /**
   * Shift the region stored under `name` by (`dx`, `dy`, `dz`). Errors
   * with `NotFound` when absent.
   */
  inline static diplomat::result<std::monostate, NucleationError> shift_region(Schematic& schematic, std::string_view name, int32_t dx, int32_t dy, int32_t dz);

    inline const diplomat::capi::SchematicRegions* AsFFI() const;
    inline diplomat::capi::SchematicRegions* AsFFI();
    inline static const SchematicRegions* FromFFI(const diplomat::capi::SchematicRegions* ptr);
    inline static SchematicRegions* FromFFI(diplomat::capi::SchematicRegions* ptr);
    inline static void operator delete(void* ptr);
private:
    SchematicRegions() = delete;
    SchematicRegions(const SchematicRegions&) = delete;
    SchematicRegions(SchematicRegions&&) noexcept = delete;
    SchematicRegions operator=(const SchematicRegions&) = delete;
    SchematicRegions operator=(SchematicRegions&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // SchematicRegions_D_HPP
