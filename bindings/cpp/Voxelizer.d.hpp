#ifndef Voxelizer_D_HPP
#define Voxelizer_D_HPP

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
namespace diplomat::capi { struct Shape; }
class Shape;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Voxelizer;
} // namespace capi
} // namespace

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
   * voxel center (robust on closed meshes), plus — when `shell` > 0 —
   * every voxel whose center is within `shell` blocks of the surface,
   * which rescues thin walls and hollow vessels (0.7–1.0 closes
   * single-voxel shells; 0 = pure parity). Errors with `Parse` on
   * malformed/triangle-less GLB and `InvalidArgument` on a
   * non-positive `target_size`.
   */
  inline static diplomat::result<std::unique_ptr<Shape>, NucleationError> shape_from_glb(diplomat::span<const uint8_t> data, float target_size, float shell);

  /**
   * Load a Wavefront OBJ (`v`/`vt`/`f` lines; polygon faces are
   * fan-triangulated, negative indices supported, materials ignored)
   * and voxelize it into a fillable Shape, fitted and shelled exactly
   * like `shape_from_glb`. Errors with `Parse` on malformed/triangle-less
   * OBJ and `InvalidArgument` on invalid UTF-8 or a non-positive
   * `target_size`.
   */
  inline static diplomat::result<std::unique_ptr<Shape>, NucleationError> shape_from_obj(std::string_view text, float target_size, float shell);

  /**
   * Load a binary glTF and voxelize it directly into a textured
   * schematic named `name`: every solid voxel becomes the `palette`
   * block closest to its nearest-surface texture color (interior
   * voxels inherit the nearest surface color; voxels without texture
   * info snap to mid-gray). `shell` behaves as in `shape_from_glb` —
   * use ~0.7 for thin-walled models. Errors with `Parse` on malformed GLB and
   * `InvalidArgument` on invalid UTF-8 or a non-positive
   * `target_size`.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> schematic_from_glb_textured(diplomat::span<const uint8_t> data, float target_size, float shell, const Palette& palette, std::string_view name);

    inline const diplomat::capi::Voxelizer* AsFFI() const;
    inline diplomat::capi::Voxelizer* AsFFI();
    inline static const Voxelizer* FromFFI(const diplomat::capi::Voxelizer* ptr);
    inline static Voxelizer* FromFFI(diplomat::capi::Voxelizer* ptr);
    inline static void operator delete(void* ptr);
private:
    Voxelizer() = delete;
    Voxelizer(const Voxelizer&) = delete;
    Voxelizer(Voxelizer&&) noexcept = delete;
    Voxelizer operator=(const Voxelizer&) = delete;
    Voxelizer operator=(Voxelizer&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Voxelizer_D_HPP
