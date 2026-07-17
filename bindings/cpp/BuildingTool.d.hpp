#ifndef BuildingTool_D_HPP
#define BuildingTool_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Brush; }
class Brush;
namespace diplomat::capi { struct Schematic; }
class Schematic;
namespace diplomat::capi { struct Shape; }
class Shape;
class NucleationError;




namespace diplomat {
namespace capi {
    struct BuildingTool;
} // namespace capi
} // namespace

/**
 * Namespace for the fill operations that combine a schematic, a shape and a
 * brush (the old `buildingtool_*` free functions).
 */
class BuildingTool {
public:

  /**
   * Fill `shape` into `schematic` using `brush`.
   */
  inline static void fill(Schematic& schematic, const Shape& shape, const Brush& brush);

  /**
   * Fill `count` copies of `shape`, each offset by `offset * i`.
   */
  inline static void rstack(Schematic& schematic, const Shape& shape, const Brush& brush, size_t count, int32_t offset_x, int32_t offset_y, int32_t offset_z);

  /**
   * Masked fill that preserves everything already placed: `brush` is
   * only written where `schematic` currently has air (or nothing at
   * all), so existing structures inside `shape` survive untouched.
   */
  inline static void fill_only_air(Schematic& schematic, const Shape& shape, const Brush& brush);

  /**
   * Masked fill that only overwrites the listed blocks: `targets_json`
   * is a JSON array of block ids (e.g. `["minecraft:stone"]`, state
   * properties ignored) and every cell of `shape` whose current block
   * id is in the list is replaced by `brush` — everything else,
   * including air, is left alone.
   */
  inline static diplomat::result<std::monostate, NucleationError> fill_replacing(Schematic& schematic, const Shape& shape, const Brush& brush, std::string_view targets_json);

    inline const diplomat::capi::BuildingTool* AsFFI() const;
    inline diplomat::capi::BuildingTool* AsFFI();
    inline static const BuildingTool* FromFFI(const diplomat::capi::BuildingTool* ptr);
    inline static BuildingTool* FromFFI(diplomat::capi::BuildingTool* ptr);
    inline static void operator delete(void* ptr);
private:
    BuildingTool() = delete;
    BuildingTool(const BuildingTool&) = delete;
    BuildingTool(BuildingTool&&) noexcept = delete;
    BuildingTool operator=(const BuildingTool&) = delete;
    BuildingTool operator=(BuildingTool&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // BuildingTool_D_HPP
