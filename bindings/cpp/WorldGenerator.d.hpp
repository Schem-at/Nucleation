#ifndef WorldGenerator_D_HPP
#define WorldGenerator_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Brush; }
class Brush;
namespace diplomat::capi { struct CellularSdfConfig; }
class CellularSdfConfig;
namespace diplomat::capi { struct GeneratedChunk; }
class GeneratedChunk;
namespace diplomat::capi { struct GeneratedWorldStream; }
class GeneratedWorldStream;
namespace diplomat::capi { struct Sdf; }
class Sdf;
class GeneratedChunkOverlayMode;
class NucleationError;




namespace diplomat {
namespace capi {
    struct WorldGenerator;
} // namespace capi
} // namespace

/**
 * An immutable native chunk source graph.
 *
 * Generated bindings expose concrete source constructors rather than host
 * callbacks, so SDF evaluation and block placement stay entirely in Rust.
 */
class WorldGenerator {
public:

  /**
   * Create an SDF-backed source evaluated at voxel centers over the inclusive
   * Y range. `source_id` and `version` become chunk provenance/cache metadata.
   */
  inline static diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> sdf(const Sdf& volume, const Brush& material, int32_t min_y, int32_t max_y, std::string_view source_id, std::string_view version);

  /**
   * Create a sparse infinite source by placing a bounded SDF motif once per
   * deterministically transformed cell. Reuse `config` across layers to keep
   * terrain, water, vegetation, paths, and structures coordinated.
   */
  inline static diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> cellular_sdf(const Sdf& volume, const Brush& material, int32_t min_y, int32_t max_y, const CellularSdfConfig& config, std::string_view source_id, std::string_view version);

  /**
   * Create a sparse source from projected building footprints, including
   * caller-projected OSM-derived data.
   * `buildings_json` uses the same schema as `Geo.extrude_footprints`:
   * `[{"polygon":[[x,z],...],"height":40,"min_y":1,
   * "block":"minecraft:bricks"}]`. `height` is the absolute top Y, matching
   * `Geo.extrude_footprints`. Fetching and lat/lon projection stay
   * caller-controlled; this source rasterizes only requested chunks.
   */
  inline static diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> projected_footprints(std::string_view buildings_json, std::string_view base_block, std::string_view source_id, std::string_view version);

  /**
   * Create an initially empty ordered source composition.
   */
  inline static diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> composite(std::string_view source_id, std::string_view version);

  /**
   * Append a source to a composite. Later `Replace` layers win at occupied
   * voxels; `KeepExisting` layers only fill air. Errors on non-composites.
   *
   * Streams already created from this generator keep the layer list they
   * were built with; only later `generate`/`stream` calls see the addition.
   */
  inline diplomat::result<std::monostate, NucleationError> add_layer(const WorldGenerator& source, GeneratedChunkOverlayMode mode);

  /**
   * Generate one random-access chunk.
   */
  inline diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError> generate(int32_t cx, int32_t cz) const;

  /**
   * Traverse an inclusive chunk rectangle lazily in canonical region-major
   * order. The stream snapshots the generator's sources at creation, so
   * later `add_layer` calls do not affect a stream already in flight.
   */
  inline diplomat::result<std::unique_ptr<GeneratedWorldStream>, NucleationError> stream(int32_t min_cx, int32_t min_cz, int32_t max_cx, int32_t max_cz) const;

    inline const diplomat::capi::WorldGenerator* AsFFI() const;
    inline diplomat::capi::WorldGenerator* AsFFI();
    inline static const WorldGenerator* FromFFI(const diplomat::capi::WorldGenerator* ptr);
    inline static WorldGenerator* FromFFI(diplomat::capi::WorldGenerator* ptr);
    inline static void operator delete(void* ptr);
private:
    WorldGenerator() = delete;
    WorldGenerator(const WorldGenerator&) = delete;
    WorldGenerator(WorldGenerator&&) noexcept = delete;
    WorldGenerator operator=(const WorldGenerator&) = delete;
    WorldGenerator operator=(WorldGenerator&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WorldGenerator_D_HPP
