#ifndef NUCLEATION_MultiMeshResult_D_HPP
#define NUCLEATION_MultiMeshResult_D_HPP

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
namespace capi { struct MeshResult; }
class MeshResult;
namespace capi { struct MultiMeshResult; }
class MultiMeshResult;
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct MultiMeshResult;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Per-region mesh results. Wraps {@link crate::meshing::MultiMeshResult}.
 */
class MultiMeshResult {
public:

  /**
   * Mesh each region separately (old ABI: `schematic_mesh_by_region`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MultiMeshResult>, nucleation::NucleationError> create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config);

  /**
   * Region names, as a JSON array string.
   */
  inline std::string region_names_json() const;
  template<typename W>
  inline void region_names_json_write(W& writeable_output) const;

  /**
   * The mesh for one region (cloned).
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> get_mesh(std::string_view region_name) const;

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

    inline const nucleation::capi::MultiMeshResult* AsFFI() const;
    inline nucleation::capi::MultiMeshResult* AsFFI();
    inline static const nucleation::MultiMeshResult* FromFFI(const nucleation::capi::MultiMeshResult* ptr);
    inline static nucleation::MultiMeshResult* FromFFI(nucleation::capi::MultiMeshResult* ptr);
    inline static void operator delete(void* ptr);
private:
    MultiMeshResult() = delete;
    MultiMeshResult(const nucleation::MultiMeshResult&) = delete;
    MultiMeshResult(nucleation::MultiMeshResult&&) noexcept = delete;
    MultiMeshResult operator=(const nucleation::MultiMeshResult&) = delete;
    MultiMeshResult operator=(nucleation::MultiMeshResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_MultiMeshResult_D_HPP
