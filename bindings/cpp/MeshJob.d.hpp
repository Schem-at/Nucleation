#ifndef MeshJob_D_HPP
#define MeshJob_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct ChunkMeshResult; }
class ChunkMeshResult;
namespace diplomat::capi { struct MeshConfig; }
class MeshConfig;
namespace diplomat::capi { struct ResourcePack; }
class ResourcePack;
namespace diplomat::capi { struct Schematic; }
class Schematic;
namespace diplomat::capi { struct TextureAtlas; }
class TextureAtlas;
struct MeshProgress;
class NucleationError;




namespace diplomat {
namespace capi {
    struct MeshJob;
} // namespace capi
} // namespace

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
  inline static std::unique_ptr<MeshJob> start(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config, int32_t chunk_size, const TextureAtlas& atlas);

  /**
   * Cheap, non-blocking progress snapshot. Call from a timer/poll loop.
   */
  inline MeshProgress poll_progress() const;

  /**
   * Block until the job finishes (if it hasn't already) and return the
   * result. Consumes the job: a second call returns `AlreadyConsumed`.
   */
  inline diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> take_result();

    inline const diplomat::capi::MeshJob* AsFFI() const;
    inline diplomat::capi::MeshJob* AsFFI();
    inline static const MeshJob* FromFFI(const diplomat::capi::MeshJob* ptr);
    inline static MeshJob* FromFFI(diplomat::capi::MeshJob* ptr);
    inline static void operator delete(void* ptr);
private:
    MeshJob() = delete;
    MeshJob(const MeshJob&) = delete;
    MeshJob(MeshJob&&) noexcept = delete;
    MeshJob operator=(const MeshJob&) = delete;
    MeshJob operator=(MeshJob&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // MeshJob_D_HPP
