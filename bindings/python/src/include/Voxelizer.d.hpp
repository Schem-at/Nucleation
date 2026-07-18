#ifndef NUCLEATION_Voxelizer_D_HPP
#define NUCLEATION_Voxelizer_D_HPP

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
namespace capi { struct Shape; }
class Shape;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Voxelizer;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace for the mesh-voxelization entry points (GLB/OBJ → shapes
 * and textured schematics).
 */
class Voxelizer {
public:

  /**
   * Load a binary glTF (`.glb`, embedded buffers/images) and voxelize
   * it into a fillable Shape: the model is uniformly scaled so its
   * largest dimension equals `target_size` voxels, centered on x/z
   * with its base resting at y = 0. Solidity is a parity test at each
   * voxel center (robust on closed meshes; open meshes are
   * best-effort). Errors with `Parse` on malformed/triangle-less GLB
   * and `InvalidArgument` on a non-positive `target_size`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> shape_from_glb(nucleation::diplomat::span<const uint8_t> data, float target_size);

  /**
   * Load a Wavefront OBJ (`v`/`vt`/`f` lines; polygon faces are
   * fan-triangulated, negative indices supported, materials ignored)
   * and voxelize it into a fillable Shape, fitted exactly like
   * `shape_from_glb`. Errors with `Parse` on malformed/triangle-less
   * OBJ and `InvalidArgument` on invalid UTF-8 or a non-positive
   * `target_size`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> shape_from_obj(std::string_view text, float target_size);

  /**
   * Load a binary glTF and voxelize it directly into a textured
   * schematic named `name`: every solid voxel becomes the `palette`
   * block closest to its nearest-surface texture color (interior
   * voxels inherit the nearest surface color; voxels without texture
   * info snap to mid-gray). Errors with `Parse` on malformed GLB and
   * `InvalidArgument` on invalid UTF-8 or a non-positive
   * `target_size`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> schematic_from_glb_textured(nucleation::diplomat::span<const uint8_t> data, float target_size, const nucleation::Palette& palette, std::string_view name);

    inline const nucleation::capi::Voxelizer* AsFFI() const;
    inline nucleation::capi::Voxelizer* AsFFI();
    inline static const nucleation::Voxelizer* FromFFI(const nucleation::capi::Voxelizer* ptr);
    inline static nucleation::Voxelizer* FromFFI(nucleation::capi::Voxelizer* ptr);
    inline static void operator delete(void* ptr);
private:
    Voxelizer() = delete;
    Voxelizer(const nucleation::Voxelizer&) = delete;
    Voxelizer(nucleation::Voxelizer&&) noexcept = delete;
    Voxelizer operator=(const nucleation::Voxelizer&) = delete;
    Voxelizer operator=(nucleation::Voxelizer&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Voxelizer_D_HPP
