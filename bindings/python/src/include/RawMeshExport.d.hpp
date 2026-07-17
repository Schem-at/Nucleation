#ifndef NUCLEATION_RawMeshExport_D_HPP
#define NUCLEATION_RawMeshExport_D_HPP

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
namespace capi { struct MeshConfig; }
class MeshConfig;
namespace capi { struct RawMeshExport; }
class RawMeshExport;
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct RawMeshExport;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Raw vertex streams for custom rendering. Wraps {@link crate::meshing::RawMeshExport}.
 */
class RawMeshExport {
public:

  /**
   * Export raw mesh data (old ABI: `schematic_to_raw_mesh`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::RawMeshExport>, nucleation::NucleationError> create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config);

  /**
   * Number of vertices in the exported mesh.
   */
  inline uint32_t vertex_count() const;

  /**
   * Number of triangles in the exported mesh.
   */
  inline uint32_t triangle_count() const;

  /**
   * Flat `[x,y,z,...]` positions as little-endian `f32` bytes, base64-encoded.
   */
  inline std::string positions_b64() const;
  template<typename W>
  inline void positions_b64_write(W& writeable_output) const;

  /**
   * Flat normals as little-endian `f32` bytes, base64-encoded.
   */
  inline std::string normals_b64() const;
  template<typename W>
  inline void normals_b64_write(W& writeable_output) const;

  /**
   * Flat UVs as little-endian `f32` bytes, base64-encoded.
   */
  inline std::string uvs_b64() const;
  template<typename W>
  inline void uvs_b64_write(W& writeable_output) const;

  /**
   * Flat vertex colors as little-endian `f32` bytes, base64-encoded.
   */
  inline std::string colors_b64() const;
  template<typename W>
  inline void colors_b64_write(W& writeable_output) const;

  /**
   * Triangle indices as little-endian `u32` bytes, base64-encoded.
   */
  inline std::string indices_b64() const;
  template<typename W>
  inline void indices_b64_write(W& writeable_output) const;

  /**
   * Raw RGBA texture pixels, base64-encoded.
   */
  inline std::string texture_rgba_b64() const;
  template<typename W>
  inline void texture_rgba_b64_write(W& writeable_output) const;

  /**
   * Width of the baked texture in pixels.
   */
  inline uint32_t texture_width() const;

  /**
   * Height of the baked texture in pixels.
   */
  inline uint32_t texture_height() const;

    inline const nucleation::capi::RawMeshExport* AsFFI() const;
    inline nucleation::capi::RawMeshExport* AsFFI();
    inline static const nucleation::RawMeshExport* FromFFI(const nucleation::capi::RawMeshExport* ptr);
    inline static nucleation::RawMeshExport* FromFFI(nucleation::capi::RawMeshExport* ptr);
    inline static void operator delete(void* ptr);
private:
    RawMeshExport() = delete;
    RawMeshExport(const nucleation::RawMeshExport&) = delete;
    RawMeshExport(nucleation::RawMeshExport&&) noexcept = delete;
    RawMeshExport operator=(const nucleation::RawMeshExport&) = delete;
    RawMeshExport operator=(nucleation::RawMeshExport&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_RawMeshExport_D_HPP
