#ifndef Geo_D_HPP
#define Geo_D_HPP

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
    struct Geo;
} // namespace capi
} // namespace

/**
 * Namespace for the geodata entry points (no network — data goes in,
 * blocks come out).
 */
class Geo {
public:

  /**
   * Extrude building footprints into a massed schematic. `buildings_json`
   * is a JSON array of objects:
   * `{"polygon": [[x, z], ...], "height": <blocks>, "block": "minecraft:...",
   * "min_y": <optional base, default 1>}`. Footprints are stamped
   * tallest-last, so overlaps keep the tallest occupant per column.
   * `base_block` (empty string = none) lays a ground slab at y=0 under the
   * whole extent. Errors `Parse` on bad JSON, `InvalidArgument` on non-UTF-8.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> extrude_footprints(std::string_view buildings_json, std::string_view base_block, std::string_view name);

  /**
   * Raise terrain from a heightmap. `heights_json` is a flat row-major
   * JSON array of per-column heights (blocks); `width` is columns per row.
   * `surface_blocks_json` is a JSON array of block names — one entry (the
   * same surface everywhere) or one per column, row-major and the same
   * length as `heights`, for elevation/slope banding. Each column's top
   * `surface_depth` blocks are its surface block, the rest are
   * `subsurface_block`. Errors `Parse` on bad JSON, `InvalidArgument` on a
   * non-positive width, empty surface list, or non-UTF-8.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> heightmap_terrain(std::string_view heights_json, int32_t width, std::string_view surface_blocks_json, std::string_view subsurface_block, int32_t surface_depth, std::string_view name);

    inline const diplomat::capi::Geo* AsFFI() const;
    inline diplomat::capi::Geo* AsFFI();
    inline static const Geo* FromFFI(const diplomat::capi::Geo* ptr);
    inline static Geo* FromFFI(diplomat::capi::Geo* ptr);
    inline static void operator delete(void* ptr);
private:
    Geo() = delete;
    Geo(const Geo&) = delete;
    Geo(Geo&&) noexcept = delete;
    Geo operator=(const Geo&) = delete;
    Geo operator=(Geo&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Geo_D_HPP
