#ifndef ChunkMeshResult_D_HPP
#define ChunkMeshResult_D_HPP

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
namespace diplomat::capi { struct TextureAtlas; }
class TextureAtlas;
struct BlockPos;
class NucleationError;




namespace diplomat {
namespace capi {
    struct ChunkMeshResult;
} // namespace capi
} // namespace

/**
 * Per-chunk mesh results. Wraps {@link crate::meshing::ChunkMeshResult}.
 */
class ChunkMeshResult {
public:

  /**
   * Mesh with the default chunk size (old ABI: `schematic_mesh_by_chunk`).
   */
  inline static diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config);

  /**
   * Mesh with an explicit chunk size (old ABI: `schematic_mesh_by_chunk_size`).
   */
  inline static diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> create_with_size(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config, int32_t chunk_size);

  /**
   * Mesh chunks against a pre-built shared atlas, synchronously
   * (old ABI: `schematic_mesh_chunks_with_atlas`). For progress
   * reporting use {@link MeshJob::start} instead.
   */
  inline static diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> create_with_atlas(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config, int32_t chunk_size, const TextureAtlas& atlas);

  /**
   * Number of chunk meshes.
   */
  inline uint32_t chunk_count() const;

  /**
   * Coordinate of the `index`-th chunk (old ABI:
   * `chunkmeshresult_chunk_coordinates` returned all of them flat).
   */
  inline diplomat::result<BlockPos, NucleationError> chunk_coordinate_at(uint32_t index) const;

  /**
   * The mesh for one chunk coordinate (cloned).
   */
  inline diplomat::result<std::unique_ptr<MeshResult>, NucleationError> get_mesh(int32_t cx, int32_t cy, int32_t cz) const;

  /**
   * Total vertex count across all chunk meshes.
   */
  inline uint32_t total_vertex_count() const;

  /**
   * Total triangle count across all chunk meshes.
   */
  inline uint32_t total_triangle_count() const;

  /**
   * All chunk meshes serialized in the NUCM cache format, base64-encoded.
   */
  inline std::string nucm_data_b64() const;
  template<typename W>
  inline void nucm_data_b64_write(W& writeable_output) const;

  /**
   * NUCM v2 with a shared atlas, base64-encoded.
   */
  inline std::string nucm_data_with_atlas_b64(const TextureAtlas& atlas) const;
  template<typename W>
  inline void nucm_data_with_atlas_b64_write(const TextureAtlas& atlas, W& writeable_output) const;

    inline const diplomat::capi::ChunkMeshResult* AsFFI() const;
    inline diplomat::capi::ChunkMeshResult* AsFFI();
    inline static const ChunkMeshResult* FromFFI(const diplomat::capi::ChunkMeshResult* ptr);
    inline static ChunkMeshResult* FromFFI(diplomat::capi::ChunkMeshResult* ptr);
    inline static void operator delete(void* ptr);
private:
    ChunkMeshResult() = delete;
    ChunkMeshResult(const ChunkMeshResult&) = delete;
    ChunkMeshResult(ChunkMeshResult&&) noexcept = delete;
    ChunkMeshResult operator=(const ChunkMeshResult&) = delete;
    ChunkMeshResult operator=(ChunkMeshResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ChunkMeshResult_D_HPP
