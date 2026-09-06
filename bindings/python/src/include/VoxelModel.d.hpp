#ifndef NUCLEATION_VoxelModel_D_HPP
#define NUCLEATION_VoxelModel_D_HPP

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
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct VoxelModel;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> plan_json(std::string_view options_json) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> plan_json_write(std::string_view options_json, W& writeable_output) const;

  /**
   * Import using plan_json's options. Anchored at (0,0,0), with exact
   * axis-based uniform scaling. Hollow uses a sparse surface raster;
   * lighting darkens sampled texture colours before palette matching.
   * Rejects oversized/over-complex output with InvalidArgument.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> to_schematic(std::string_view options_json, const nucleation::Palette& palette, std::string_view name) const;

    inline const nucleation::capi::VoxelModel* AsFFI() const;
    inline nucleation::capi::VoxelModel* AsFFI();
    inline static const nucleation::VoxelModel* FromFFI(const nucleation::capi::VoxelModel* ptr);
    inline static nucleation::VoxelModel* FromFFI(nucleation::capi::VoxelModel* ptr);
    inline static void operator delete(void* ptr);
private:
    VoxelModel() = delete;
    VoxelModel(const nucleation::VoxelModel&) = delete;
    VoxelModel(nucleation::VoxelModel&&) noexcept = delete;
    VoxelModel operator=(const nucleation::VoxelModel&) = delete;
    VoxelModel operator=(nucleation::VoxelModel&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_VoxelModel_D_HPP
