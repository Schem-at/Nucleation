#ifndef MultiMeshResult_D_HPP
#define MultiMeshResult_D_HPP

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
namespace diplomat::capi { struct MeshResult; }
class MeshResult;
namespace diplomat::capi { struct ResourcePack; }
class ResourcePack;
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct MultiMeshResult;
} // namespace capi
} // namespace

/**
 * Per-region mesh results. Wraps {@link crate::meshing::MultiMeshResult}.
 */
class MultiMeshResult {
public:

  /**
   * Mesh each region separately (old ABI: `schematic_mesh_by_region`).
   */
  inline static diplomat::result<std::unique_ptr<MultiMeshResult>, NucleationError> create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config);

  /**
   * Region names, as a JSON array string.
   */
  inline std::string region_names_json() const;
  template<typename W>
  inline void region_names_json_write(W& writeable_output) const;

  /**
   * The mesh for one region (cloned).
   */
  inline diplomat::result<std::unique_ptr<MeshResult>, NucleationError> get_mesh(std::string_view region_name) const;

  /**
   * Total vertex count across all region meshes.
   */
  inline uint32_t total_vertex_count() const;

  /**
   * Total triangle count across all region meshes.
   */
  inline uint32_t total_triangle_count() const;

  /**
   * Number of region meshes.
   */
  inline uint32_t mesh_count() const;

    inline const diplomat::capi::MultiMeshResult* AsFFI() const;
    inline diplomat::capi::MultiMeshResult* AsFFI();
    inline static const MultiMeshResult* FromFFI(const diplomat::capi::MultiMeshResult* ptr);
    inline static MultiMeshResult* FromFFI(diplomat::capi::MultiMeshResult* ptr);
    inline static void operator delete(void* ptr);
private:
    MultiMeshResult() = delete;
    MultiMeshResult(const MultiMeshResult&) = delete;
    MultiMeshResult(MultiMeshResult&&) noexcept = delete;
    MultiMeshResult operator=(const MultiMeshResult&) = delete;
    MultiMeshResult operator=(MultiMeshResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // MultiMeshResult_D_HPP
