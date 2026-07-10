#ifndef NUCLEATION_MeshJob_D_HPP
#define NUCLEATION_MeshJob_D_HPP

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
namespace capi { struct MeshJob; }
class MeshJob;
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct TextureAtlas; }
class TextureAtlas;
struct MeshProgress;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct MeshJob;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A chunk-meshing job running on a background thread. Replaces the old
 * `schematic_mesh_chunks_with_atlas_progress` C callback: poll it from a
 * timer loop with {@link MeshJob::poll_progress}, then call
 * {@link MeshJob::take_result} once (it blocks until the job finishes and
 * consumes the job — a second call returns `AlreadyConsumed`).
 */
class MeshJob {
public:

  /**
   * Kick off chunk meshing with a shared atlas on a background thread
   * and return immediately. Takes the same parameters as
   * {@link ChunkMeshResult::create_with_atlas}.
   */
  inline static std::unique_ptr<nucleation::MeshJob> start(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config, int32_t chunk_size, const nucleation::TextureAtlas& atlas);

  /**
   * Cheap, non-blocking progress snapshot. Call from a timer/poll loop.
   */
  inline nucleation::MeshProgress poll_progress() const;

  /**
   * Block until the job finishes (if it hasn't already) and return the
   * result. Consumes the job: a second call returns `AlreadyConsumed`.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> take_result();

    inline const nucleation::capi::MeshJob* AsFFI() const;
    inline nucleation::capi::MeshJob* AsFFI();
    inline static const nucleation::MeshJob* FromFFI(const nucleation::capi::MeshJob* ptr);
    inline static nucleation::MeshJob* FromFFI(nucleation::capi::MeshJob* ptr);
    inline static void operator delete(void* ptr);
private:
    MeshJob() = delete;
    MeshJob(const nucleation::MeshJob&) = delete;
    MeshJob(nucleation::MeshJob&&) noexcept = delete;
    MeshJob operator=(const nucleation::MeshJob&) = delete;
    MeshJob operator=(nucleation::MeshJob&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_MeshJob_D_HPP
