#ifndef VoxelModel_D_HPP
#define VoxelModel_D_HPP

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
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct VoxelModel;
} // namespace capi
} // namespace

/**
 * A parsed GLB/OBJ, reusable for size estimates and configured imports.
 */
class VoxelModel {
public:

  /**
   * Return {dimensions:[width,height,depth],volume} or {error:message}.
   * Options: target_size, axis (longest/x/y/z), hollow, optional lighting
   * {direction:[x,y,z],strength:0..1}, optional untextured_block.
   * Estimates preserve proportions and run before voxel-grid allocation.
   */
  inline diplomat::result<std::string, NucleationError> plan_json(std::string_view options_json) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> plan_json_write(std::string_view options_json, W& writeable_output) const;

  /**
   * Import using plan_json's options. Anchored at (0,0,0), with exact
   * axis-based uniform scaling. Hollow uses a sparse surface raster;
   * lighting darkens sampled texture colours before palette matching.
   * Rejects oversized/over-complex output with InvalidArgument.
   */
  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> to_schematic(std::string_view options_json, const Palette& palette, std::string_view name) const;

    inline const diplomat::capi::VoxelModel* AsFFI() const;
    inline diplomat::capi::VoxelModel* AsFFI();
    inline static const VoxelModel* FromFFI(const diplomat::capi::VoxelModel* ptr);
    inline static VoxelModel* FromFFI(diplomat::capi::VoxelModel* ptr);
    inline static void operator delete(void* ptr);
private:
    VoxelModel() = delete;
    VoxelModel(const VoxelModel&) = delete;
    VoxelModel(VoxelModel&&) noexcept = delete;
    VoxelModel operator=(const VoxelModel&) = delete;
    VoxelModel operator=(VoxelModel&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // VoxelModel_D_HPP
