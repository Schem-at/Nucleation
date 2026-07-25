#ifndef NUCLEATION_WorldGenerator_D_HPP
#define NUCLEATION_WorldGenerator_D_HPP

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
namespace capi { struct Brush; }
class Brush;
namespace capi { struct CellularSdfConfig; }
class CellularSdfConfig;
namespace capi { struct GeneratedChunk; }
class GeneratedChunk;
namespace capi { struct GeneratedWorldStream; }
class GeneratedWorldStream;
namespace capi { struct Sdf; }
class Sdf;
namespace capi { struct WorldGenerator; }
class WorldGenerator;
class GeneratedChunkOverlayMode;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct WorldGenerator;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> sdf(const nucleation::Sdf& volume, const nucleation::Brush& material, int32_t min_y, int32_t max_y, std::string_view source_id, std::string_view version);

  /**
   * Create a sparse infinite source by placing a bounded SDF motif once per
   * deterministically transformed cell. Reuse `config` across layers to keep
   * terrain, water, vegetation, paths, and structures coordinated.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> cellular_sdf(const nucleation::Sdf& volume, const nucleation::Brush& material, int32_t min_y, int32_t max_y, const nucleation::CellularSdfConfig& config, std::string_view source_id, std::string_view version);

  /**
   * Create a sparse source from projected building footprints, including
   * caller-projected OSM-derived data.
   * `buildings_json` uses the same schema as `Geo.extrude_footprints`:
   * `[{"polygon":[[x,z],...],"height":40,"min_y":1,
   * "block":"minecraft:bricks"}]`. `height` is the absolute top Y, matching
   * `Geo.extrude_footprints`. Fetching and lat/lon projection stay
   * caller-controlled; this source rasterizes only requested chunks.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> projected_footprints(std::string_view buildings_json, std::string_view base_block, std::string_view source_id, std::string_view version);

  /**
   * Create an initially empty ordered source composition.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> composite(std::string_view source_id, std::string_view version);

  /**
   * Append a source to a composite. Later `Replace` layers win at occupied
   * voxels; `KeepExisting` layers only fill air. Errors on non-composites.
   *
   * Streams already created from this generator keep the layer list they
   * were built with; only later `generate`/`stream` calls see the addition.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_layer(const nucleation::WorldGenerator& source, nucleation::GeneratedChunkOverlayMode mode);

  /**
   * Generate one random-access chunk.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError> generate(int32_t cx, int32_t cz) const;

  /**
   * Traverse an inclusive chunk rectangle lazily in canonical region-major
   * order. The stream snapshots the generator's sources at creation, so
   * later `add_layer` calls do not affect a stream already in flight.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedWorldStream>, nucleation::NucleationError> stream(int32_t min_cx, int32_t min_cz, int32_t max_cx, int32_t max_cz) const;

    inline const nucleation::capi::WorldGenerator* AsFFI() const;
    inline nucleation::capi::WorldGenerator* AsFFI();
    inline static const nucleation::WorldGenerator* FromFFI(const nucleation::capi::WorldGenerator* ptr);
    inline static nucleation::WorldGenerator* FromFFI(nucleation::capi::WorldGenerator* ptr);
    inline static void operator delete(void* ptr);
private:
    WorldGenerator() = delete;
    WorldGenerator(const nucleation::WorldGenerator&) = delete;
    WorldGenerator(nucleation::WorldGenerator&&) noexcept = delete;
    WorldGenerator operator=(const nucleation::WorldGenerator&) = delete;
    WorldGenerator operator=(nucleation::WorldGenerator&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_WorldGenerator_D_HPP
