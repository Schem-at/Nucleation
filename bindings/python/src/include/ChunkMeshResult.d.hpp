#ifndef NUCLEATION_ChunkMeshResult_D_HPP
#define NUCLEATION_ChunkMeshResult_D_HPP

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
namespace capi { struct ChunkMeshResult; }
class ChunkMeshResult;
namespace capi { struct MeshConfig; }
class MeshConfig;
namespace capi { struct MeshResult; }
class MeshResult;
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct TextureAtlas; }
class TextureAtlas;
struct BlockPos;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ChunkMeshResult;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Per-chunk mesh results. Wraps {@link crate::meshing::ChunkMeshResult}.
 */
class ChunkMeshResult {
public:

  /**
   * Mesh with the default chunk size (old ABI: `schematic_mesh_by_chunk`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config);

  /**
   * Mesh with an explicit chunk size (old ABI: `schematic_mesh_by_chunk_size`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> create_with_size(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config, int32_t chunk_size);

  /**
   * Mesh chunks against a pre-built shared atlas, synchronously
   * (old ABI: `schematic_mesh_chunks_with_atlas`). For progress
   * reporting use {@link MeshJob::start} instead.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> create_with_atlas(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config, int32_t chunk_size, const nucleation::TextureAtlas& atlas);

  inline uint32_t chunk_count() const;

  /**
   * Coordinate of the `index`-th chunk (old ABI:
   * `chunkmeshresult_chunk_coordinates` returned all of them flat).
   */
  inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> chunk_coordinate_at(uint32_t index) const;

  /**
   * The mesh for one chunk coordinate (cloned).
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> get_mesh(int32_t cx, int32_t cy, int32_t cz) const;

  inline uint32_t total_vertex_count() const;

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
  inline std::string nucm_data_with_atlas_b64(const nucleation::TextureAtlas& atlas) const;
  template<typename W>
  inline void nucm_data_with_atlas_b64_write(const nucleation::TextureAtlas& atlas, W& writeable_output) const;

    inline const nucleation::capi::ChunkMeshResult* AsFFI() const;
    inline nucleation::capi::ChunkMeshResult* AsFFI();
    inline static const nucleation::ChunkMeshResult* FromFFI(const nucleation::capi::ChunkMeshResult* ptr);
    inline static nucleation::ChunkMeshResult* FromFFI(nucleation::capi::ChunkMeshResult* ptr);
    inline static void operator delete(void* ptr);
private:
    ChunkMeshResult() = delete;
    ChunkMeshResult(const nucleation::ChunkMeshResult&) = delete;
    ChunkMeshResult(nucleation::ChunkMeshResult&&) noexcept = delete;
    ChunkMeshResult operator=(const nucleation::ChunkMeshResult&) = delete;
    ChunkMeshResult operator=(nucleation::ChunkMeshResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ChunkMeshResult_D_HPP
