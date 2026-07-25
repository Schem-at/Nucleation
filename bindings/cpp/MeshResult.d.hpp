#ifndef MeshResult_D_HPP
#define MeshResult_D_HPP

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
struct MeshBounds;
class NucleationError;




namespace diplomat {
namespace capi {
    struct MeshResult;
} // namespace capi
} // namespace

/**
 * A single mesh output. Wraps {@link crate::meshing::MeshResult}.
 */
class MeshResult {
public:

  /**
   * Mesh an entire schematic in one pass (old ABI: `schematic_to_mesh`).
   */
  inline static diplomat::result<std::unique_ptr<MeshResult>, NucleationError> create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config);

  /**
   * Mesh a schematic with USDZ-compatible output (old ABI: `schematic_to_usdz`).
   */
  inline static diplomat::result<std::unique_ptr<MeshResult>, NucleationError> create_usdz(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config);

  /**
   * The mesh as a binary GLB, base64-encoded.
   */
  inline diplomat::result<std::string, NucleationError> glb_data_b64() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> glb_data_b64_write(W& writeable_output) const;

  /**
   * The mesh as a USDZ archive, base64-encoded.
   */
  inline diplomat::result<std::string, NucleationError> usdz_data_b64() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> usdz_data_b64_write(W& writeable_output) const;

  /**
   * The mesh serialized in the NUCM cache format, base64-encoded.
   */
  inline std::string nucm_data_b64() const;
  template<typename W>
  inline void nucm_data_b64_write(W& writeable_output) const;

  /**
   * Total number of vertices in the mesh.
   */
  inline uint32_t vertex_count() const;

  /**
   * Total number of triangles in the mesh.
   */
  inline uint32_t triangle_count() const;

  /**
   * Whether the mesh contains any transparent or translucent geometry.
   */
  inline bool has_transparency() const;

  /**
   * Axis-aligned bounding box of the mesh, in world units.
   */
  inline MeshBounds bounds() const;

    inline const diplomat::capi::MeshResult* AsFFI() const;
    inline diplomat::capi::MeshResult* AsFFI();
    inline static const MeshResult* FromFFI(const diplomat::capi::MeshResult* ptr);
    inline static MeshResult* FromFFI(diplomat::capi::MeshResult* ptr);
    inline static void operator delete(void* ptr);
private:
    MeshResult() = delete;
    MeshResult(const MeshResult&) = delete;
    MeshResult(MeshResult&&) noexcept = delete;
    MeshResult operator=(const MeshResult&) = delete;
    MeshResult operator=(MeshResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // MeshResult_D_HPP
