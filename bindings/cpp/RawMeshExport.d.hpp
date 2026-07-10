#ifndef RawMeshExport_D_HPP
#define RawMeshExport_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct MeshConfig; }
class MeshConfig;
namespace diplomat::capi { struct ResourcePack; }
class ResourcePack;
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct RawMeshExport;
} // namespace capi
} // namespace

/**
 * Raw vertex streams for custom rendering. Wraps {@link crate::meshing::RawMeshExport}.
 */
class RawMeshExport {
public:

  /**
   * Export raw mesh data (old ABI: `schematic_to_raw_mesh`).
   */
  inline static diplomat::result<std::unique_ptr<RawMeshExport>, NucleationError> create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config);

  inline uint32_t vertex_count() const;

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

  inline uint32_t texture_width() const;

  inline uint32_t texture_height() const;

    inline const diplomat::capi::RawMeshExport* AsFFI() const;
    inline diplomat::capi::RawMeshExport* AsFFI();
    inline static const RawMeshExport* FromFFI(const diplomat::capi::RawMeshExport* ptr);
    inline static RawMeshExport* FromFFI(diplomat::capi::RawMeshExport* ptr);
    inline static void operator delete(void* ptr);
private:
    RawMeshExport() = delete;
    RawMeshExport(const RawMeshExport&) = delete;
    RawMeshExport(RawMeshExport&&) noexcept = delete;
    RawMeshExport operator=(const RawMeshExport&) = delete;
    RawMeshExport operator=(RawMeshExport&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // RawMeshExport_D_HPP
