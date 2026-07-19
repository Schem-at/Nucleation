#ifndef WorldChunkView_D_HPP
#define WorldChunkView_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct WorldChunkView;
} // namespace capi
} // namespace

/**
 * A single decoded chunk (or a from-scratch chunk under construction).
 */
class WorldChunkView {
public:

  /**
   * Create an empty chunk view at the given chunk coordinates — the
   * starting point for generating worlds from scratch. Sections are
   * created on demand by `set_block`. Serialized with
   * `status = "minecraft:full"` (Minecraft will not regenerate over it)
   * and the default data version.
   */
  inline static std::unique_ptr<WorldChunkView> create(int32_t cx, int32_t cz);

  /**
   * The chunk X coordinate (in chunk units).
   */
  inline int32_t cx() const;

  /**
   * The chunk Z coordinate (in chunk units).
   */
  inline int32_t cz() const;

  /**
   * Convert the chunk view to a standalone schematic.
   */
  inline std::unique_ptr<Schematic> to_schematic() const;

  /**
   * Build a chunk view at (`cx`, `cz`) from a schematic — every non-air
   * block whose world (x, z) falls in this chunk is copied in, the rest
   * ignored. The write-side twin of `to_schematic`: this is how the
   * schematic building tools become a *world generator*. Fill a schematic
   * with any shape, SDF, brush, or footprint (intersect it with the
   * chunk's cuboid to keep memory flat), then hand it here per chunk and
   * `WorldSink.write_chunk` it. Also the transform step of a world filter:
   * `to_schematic` a streamed chunk, edit it, rebuild with this.
   */
  inline static std::unique_ptr<WorldChunkView> from_schematic(const Schematic& schematic, int32_t cx, int32_t cz);

  /**
   * Set a block at absolute world coordinates inside this chunk view.
   * `block_name` must be a valid Minecraft block identifier (e.g.
   * `minecraft:stone`). Errors with `InvalidArgument` if (x, z) is outside
   * this chunk's column.
   */
  inline diplomat::result<std::monostate, NucleationError> set_block(int32_t x, int32_t y, int32_t z, std::string_view block_name);

  /**
   * Overwrite the biome of every currently-present section of the chunk
   * view with `biome_name` (e.g. `minecraft:desert`). Sections are created
   * lazily by `set_block`, so call this AFTER placing blocks.
   */
  inline diplomat::result<std::monostate, NucleationError> set_biome(std::string_view biome_name);

  /**
   * Deduped union of all sections' biome palette entries, in order of
   * first appearance, written as a JSON array string (`[]` if no section
   * carries biome data).
   */
  inline diplomat::result<std::string, NucleationError> biome_palette_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> biome_palette_json_write(W& writeable_output) const;

    inline const diplomat::capi::WorldChunkView* AsFFI() const;
    inline diplomat::capi::WorldChunkView* AsFFI();
    inline static const WorldChunkView* FromFFI(const diplomat::capi::WorldChunkView* ptr);
    inline static WorldChunkView* FromFFI(diplomat::capi::WorldChunkView* ptr);
    inline static void operator delete(void* ptr);
private:
    WorldChunkView() = delete;
    WorldChunkView(const WorldChunkView&) = delete;
    WorldChunkView(WorldChunkView&&) noexcept = delete;
    WorldChunkView operator=(const WorldChunkView&) = delete;
    WorldChunkView operator=(WorldChunkView&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WorldChunkView_D_HPP
