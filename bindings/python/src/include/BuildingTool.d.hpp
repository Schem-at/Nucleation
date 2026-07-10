#ifndef NUCLEATION_BuildingTool_D_HPP
#define NUCLEATION_BuildingTool_D_HPP

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
namespace capi { struct Brush; }
class Brush;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct Shape; }
class Shape;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct BuildingTool;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace for the fill operations that combine a schematic, a shape and a
 * brush (the old `buildingtool_*` free functions).
 */
class BuildingTool {
public:

  /**
   * Fill `shape` into `schematic` using `brush`.
   */
  inline static void fill(nucleation::Schematic& schematic, const nucleation::Shape& shape, const nucleation::Brush& brush);

  /**
   * Fill `count` copies of `shape`, each offset by `offset * i`.
   */
  inline static void rstack(nucleation::Schematic& schematic, const nucleation::Shape& shape, const nucleation::Brush& brush, size_t count, int32_t offset_x, int32_t offset_y, int32_t offset_z);

    inline const nucleation::capi::BuildingTool* AsFFI() const;
    inline nucleation::capi::BuildingTool* AsFFI();
    inline static const nucleation::BuildingTool* FromFFI(const nucleation::capi::BuildingTool* ptr);
    inline static nucleation::BuildingTool* FromFFI(nucleation::capi::BuildingTool* ptr);
    inline static void operator delete(void* ptr);
private:
    BuildingTool() = delete;
    BuildingTool(const nucleation::BuildingTool&) = delete;
    BuildingTool(nucleation::BuildingTool&&) noexcept = delete;
    BuildingTool operator=(const nucleation::BuildingTool&) = delete;
    BuildingTool operator=(nucleation::BuildingTool&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_BuildingTool_D_HPP
