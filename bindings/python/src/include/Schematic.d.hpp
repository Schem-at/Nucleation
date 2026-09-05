#ifndef NUCLEATION_Schematic_D_HPP
#define NUCLEATION_Schematic_D_HPP

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
namespace capi { struct BlockState; }
class BlockState;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct SchematicSplitResult; }
class SchematicSplitResult;
struct BlockPos;
struct Dimensions;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Schematic;
} // namespace capi
} // namespace

namespace nucleation {
class Schematic {
public:

  /**
   * Create a new, empty schematic with the given name.
   */
  inline static std::unique_ptr<nucleation::Schematic> create(std::string_view name);

  /**
   * Return an independent deep copy. Subsequent block, region, entity,
   * metadata, or transform changes do not affect the original.
   */
  inline std::unique_ptr<nucleation::Schematic> deep_clone() const;

  /**
   * Inspect a versioned transform-plan JSON document without modifying
   * this schematic. Writes a deterministic audit-report JSON document.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> inspect_transform_plan_json(std::string_view plan_json) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> inspect_transform_plan_json_write(std::string_view plan_json, W& writeable_output) const;

  /**
   * Atomically apply a versioned transform-plan JSON document. Policy
   * rejection is represented by `report.rejected == true` and leaves the
   * schematic unchanged; malformed plans raise `InvalidArgument`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> apply_transform_plan_json(std::string_view plan_json);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> apply_transform_plan_json_write(std::string_view plan_json, W& writeable_output);

  /**
   * Apply the bundled deterministic, lossless canonicalization preset.
   */
  inline std::string canonicalize_json();
  template<typename W>
  inline void canonicalize_json_write(W& writeable_output);

  /**
   * Inspect the bundled public-registry policy without modifying this
   * schematic. Applications should review `rejected` and `quarantined`
   * before choosing whether to call `apply_transform_plan_json`.
   */
  inline std::string inspect_registry_safe_json() const;
  template<typename W>
  inline void inspect_registry_safe_json_write(W& writeable_output) const;

  /**
   * Split spatially independent machines while keeping nearby tiny
   * detached parts with their machine. Components at least
   * `min_standalone_blocks` large always remain independent; smaller
   * components attach only directly to a core within `max_air_gap`.
   * Attachment is non-transitive and the operation is lossless.
   */
  inline std::unique_ptr<nucleation::SchematicSplitResult> split_connected_attach_nearby(uint32_t min_standalone_blocks, uint32_t max_air_gap) const;

  /**
   * The allocated dimensions (width, height, length) of the schematic's
   * bounding box.
   */
  inline nucleation::Dimensions dimensions() const;

  /**
   * Returns `true` if a block was placed (out-of-range coordinates extend the
   * schematic rather than erroring, matching `UniversalSchematic::set_block`).
   */
  inline nucleation::diplomat::result<bool, nucleation::NucleationError> set_block(int32_t x, int32_t y, int32_t z, std::string_view block_name);

  /**
   * The name of the block at a position. `NotFound` if the position is
   * outside every region.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_block_name(int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_block_name_write(int32_t x, int32_t y, int32_t z, W& writeable_output) const;

  /**
   * Save the schematic to a file, picking the format from the file
   * extension (`.litematic`, `.schem`, `.schematic`, `.mcstructure`,
   * `.nbt`, `.nusn`; unknown extensions write Litematic). For an
   * explicit format or version, use `save_to_file_with_format`.
   * Not available in JS: the WASM build has no filesystem — use
   * `save_as_b64` there.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save_to_file(std::string_view path) const;

  /**
   * Convenience alias for `save_to_file`, matching the established
   * Python API (`schematic.save("build.schem")`).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save(std::string_view path) const;

  /**
   * Load a schematic from a file, auto-detecting the format from the
   * contents (any supported format, whatever the extension says).
   * Not available in JS: the WASM build has no filesystem — read the
   * bytes yourself and use `from_data`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> load_from_file(std::string_view path);

  /**
   * Convenience alias for `load_from_file`, matching the established
   * Python API (`Schematic.open("build.schem")`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> open(std::string_view path);

  /**
   * Build a schematic from raw byte data, auto-detecting the format.
   * Supports Litematic, Sponge Schematic, and McStructure (Bedrock) formats.
   * `Parse` if a format was detected but failed to parse, `InvalidArgument` if
   * no format was recognized.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_data(nucleation::diplomat::span<const uint8_t> data);

  /**
   * Decode untrusted bytes using a serialized `DecodeLimits` object.
   * Empty JSON selects the conservative library defaults. Limits are
   * enforced while decompressing/parsing and again before region
   * allocations are accepted.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_data_bounded(nucleation::diplomat::span<const uint8_t> data, std::string_view limits_json);

  /**
   * Build a schematic from Litematic data.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_litematic(nucleation::diplomat::span<const uint8_t> data);

  /**
   * The schematic as Litematic bytes, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_litematic_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_litematic_b64_write(W& writeable_output) const;

  /**
   * Build a schematic from classic `.schematic` data.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_schematic(nucleation::diplomat::span<const uint8_t> data);

  /**
   * The schematic as classic `.schematic` bytes, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_schematic_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_schematic_b64_write(W& writeable_output) const;

  /**
   * Build a schematic from snapshot (fast binary) data.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_snapshot(nucleation::diplomat::span<const uint8_t> data);

  /**
   * The schematic as snapshot (fast binary) bytes, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_snapshot_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_snapshot_b64_write(W& writeable_output) const;

  /**
   * Build a schematic from McStructure (Bedrock) data.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_mcstructure(nucleation::diplomat::span<const uint8_t> data);

  /**
   * The schematic as McStructure (Bedrock) bytes, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_mcstructure_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_mcstructure_b64_write(W& writeable_output) const;

  /**
   * Import from a single MCA region file.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_mca(nucleation::diplomat::span<const uint8_t> data);

  /**
   * Import from MCA with coordinate bounds.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_mca_bounded(nucleation::diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Import from a zipped world folder.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_world_zip(nucleation::diplomat::span<const uint8_t> data);

  /**
   * Import from zipped world with coordinate bounds.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_world_zip_bounded(nucleation::diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Import from a Minecraft world directory path.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_world_directory(std::string_view path);

  /**
   * Import from world directory with coordinate bounds.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_world_directory_bounded(std::string_view path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Export the schematic as a Minecraft world: a JSON array of
   * `{"path": <relative file path>, "data_b64": <base64 bytes>}` entries
   * (the old `CFileMap`). `options_json` may be empty for defaults.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_world_json(std::string_view options_json) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_world_json_write(std::string_view options_json, W& writeable_output) const;

  /**
   * Export and write world files to a directory. `options_json` may be empty.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save_world(std::string_view directory, std::string_view options_json) const;

  /**
   * Export the schematic as a zipped Minecraft world, base64-encoded.
   * `options_json` may be empty for defaults.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_world_zip_b64(std::string_view options_json) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_world_zip_b64_write(std::string_view options_json, W& writeable_output) const;

  /**
   * Set a block with properties given as a JSON object of string→string
   * (the old `CProperty` array).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_block_with_properties(int32_t x, int32_t y, int32_t z, std::string_view block_name, std::string_view properties_json);

  /**
   * Set a block from a full block string, e.g.
   * `minecraft:chest[facing=north]{Items:[...]}`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_block_from_string(int32_t x, int32_t y, int32_t z, std::string_view block_string);

  /**
   * Pre-resolve a plain block name to a palette index for use with `place`.
   * Pair them in hot loops with many unique block names to skip the per-call
   * name → palette lookup.
   */
  inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> prepare_block(std::string_view block_name);

  /**
   * Place a block by pre-resolved palette index (from `prepare_block`).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> place(int32_t x, int32_t y, int32_t z, int32_t palette_index);

  /**
   * Batch-set blocks at multiple positions to the same block (name, block
   * string with properties, or block string with NBT). `positions` is flat
   * `[x0,y0,z0, x1,y1,z1, ...]` (length must be a multiple of 3).
   * Returns the number of blocks set.
   */
  inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> set_blocks(nucleation::diplomat::span<const int32_t> positions, std::string_view block_name);

  /**
   * Sequentially hand-place the same block at many positions in one
   * local simulated component. `positions` is flat
   * `[x0,y0,z0, x1,y1,z1, ...]`; placements run in that order and each
   * settles before the next. Returns the number of final cells written
   * back, including neighbours changed by redstone or pistons.
   *
   * Nearby passive blocks are loaded as environmental context, but the
   * write-back is confined to the active component's effect window. Its
   * runtime is therefore independent of unrelated schematic volume.
   * Use `set_blocks_simulated_full_world` to opt into global updates.
   *
   * This is the efficient bulk form of repeated `{simulate=true}`:
   * structure conversion and simulator wiring happen once for the
   * complete sequence. Propagation is not constant-time—a placement can
   * affect an arbitrarily large circuit—but fixed setup is amortized.
   */
  inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> set_blocks_simulated(nucleation::diplomat::span<const int32_t> positions, std::string_view block_name);

  /**
   * Explicit full-world counterpart to `set_blocks_simulated`.
   * Unrelated schematic volume participates in setup and any resulting
   * changes anywhere in the loaded world are written back.
   */
  inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> set_blocks_simulated_full_world(nucleation::diplomat::span<const int32_t> positions, std::string_view block_name);

  /**
   * Batch-get block names at multiple positions. `positions` is flat
   * `[x0,y0,z0, ...]` (length must be a multiple of 3). Writes a JSON array,
   * one entry per position: the block name string, or `null` for
   * empty/out-of-bounds positions.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_blocks_json(nucleation::diplomat::span<const int32_t> positions) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_blocks_json_write(nucleation::diplomat::span<const int32_t> positions, W& writeable_output) const;

  /**
   * Stamp a merged source box into the default region. Excluded blocks
   * are skipped, preserving destination content. Empty string or `[]`
   * means no exclusions.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> stamp_box(const nucleation::Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, std::string_view excluded_blocks_json);

  /**
   * Stamp one explicitly named source region into the default region.
   * The region's minimum corner is mapped to the target position.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> stamp_region(const nucleation::Schematic& source, std::string_view source_region_name, int32_t target_x, int32_t target_y, int32_t target_z, std::string_view excluded_blocks_json);

  /**
   * Compatibility alias for `stamp_box`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> copy_region(const nucleation::Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, std::string_view excluded_blocks_json);

  /**
   * The full block state at a position. `NotFound` if the position is
   * outside every region.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError> get_block(int32_t x, int32_t y, int32_t z) const;

  /**
   * The block at a position with its properties, as a `BlockState`.
   * Kept as an explicit alias for callers migrating from the older API.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError> get_block_with_properties(int32_t x, int32_t y, int32_t z) const;

  /**
   * The full block state at a position in one specific region. This
   * avoids composite lookup ambiguity when regions overlap.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError> get_block_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z) const;

  /**
   * The block string at a position in one specific region.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_block_string_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_block_string_in_region_write(std::string_view region_name, int32_t x, int32_t y, int32_t z, W& writeable_output) const;

  /**
   * The full block string (name, properties, NBT) at a position.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_block_string(int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_block_string_write(int32_t x, int32_t y, int32_t z, W& writeable_output) const;

  /**
   * The block entity at a position as JSON
   * `{"id": ..., "position": [x,y,z], "nbt": {...}}` (the old `CBlockEntity`).
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_block_entity_json(int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_block_entity_json_write(int32_t x, int32_t y, int32_t z, W& writeable_output) const;

  /**
   * The block entity at a position in one specific region as JSON.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_block_entity_json_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_block_entity_json_in_region_write(std::string_view region_name, int32_t x, int32_t y, int32_t z, W& writeable_output) const;

  /**
   * Every block entity as a JSON array of
   * `{"id": ..., "position": [x,y,z], "nbt": {...}}`.
   */
  inline std::string get_all_block_entities_json() const;
  template<typename W>
  inline void get_all_block_entities_json_write(W& writeable_output) const;

  /**
   * The number of mobile entities (not block entities).
   */
  inline uint32_t entity_count() const;

  /**
   * Every mobile entity as a JSON array of
   * `{"id": ..., "position": [x,y,z], "nbt": {...}}` (the old `CEntityArray`).
   */
  inline std::string get_entities_json() const;
  template<typename W>
  inline void get_entities_json_write(W& writeable_output) const;

  /**
   * Add a mobile entity. `nbt_json` is a JSON object (may be empty).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_entity(std::string_view id, double x, double y, double z, std::string_view nbt_json);

  /**
   * Add an armor stand without hand-authoring entity NBT.
   *
   * `armor_material` accepts `diamond`, `netherite`, `iron`, etc.; an
   * empty string creates an unarmored stand. `yaw` uses Minecraft degrees.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material);

  /**
   * Remove a mobile entity by index.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> remove_entity(uint32_t index);

  /**
   * The canonical in-memory data version (the forward-conversion target).
   */
  inline static int32_t canonical_data_version();

  /**
   * Convert block/item/entity data between Minecraft data versions. Forward
   * (`target >= source`) is lossless; reverse is lossy. Writes a JSON loss
   * report (`[]` when lossless).
   */
  inline std::string convert_to_data_version(int32_t target_data_version, int32_t source_data_version);
  template<typename W>
  inline void convert_to_data_version_write(int32_t target_data_version, int32_t source_data_version, W& writeable_output);

  /**
   * Convert to `target_data_version` using the schematic's captured source
   * version (else `mc_version`, else canonical) as origin, updating metadata
   * to the target. Writes a JSON loss report (`[]` when lossless).
   */
  inline std::string convert_to_version(int32_t target_data_version);
  template<typename W>
  inline void convert_to_version_write(int32_t target_data_version, W& writeable_output);

  /**
   * The Minecraft data version of the file this schematic was loaded from, or
   * `-1` if none was captured (versionless / freshly built).
   */
  inline int32_t source_data_version() const;

  /**
   * Override the source data version for formats that carry no Java data
   * version, so the converter knows what to convert *from*.
   */
  inline void set_source_data_version(int32_t version);

  /**
   * Serialize a `.litematic` targeting a specific Minecraft data version. A
   * COPY is converted and the matching Version header written; the schematic
   * is left unchanged. Writes JSON
   * `{"data_b64": <base64 .litematic>, "loss": <loss report>}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_litematic_for_version_json(int32_t target_data_version) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_litematic_for_version_json_write(int32_t target_data_version, W& writeable_output) const;

  /**
   * The block entity's NBT as a typed SNBT string. Round-trips losslessly.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_block_entity_snbt(int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_block_entity_snbt_write(int32_t x, int32_t y, int32_t z, W& writeable_output) const;

  /**
   * Set (or replace) a block entity at a position from a typed SNBT string.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_block_entity(int32_t x, int32_t y, int32_t z, std::string_view id, std::string_view snbt);

  /**
   * Remove the block entity at a position. `NotFound` if none was there.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> remove_block_entity(int32_t x, int32_t y, int32_t z);

  /**
   * Every block entity as a JSON array of `{id, position: [x,y,z], snbt}`.
   * The `snbt` is the inner data only (no `Id`/`Pos`).
   */
  inline std::string get_all_block_entities_snbt_json() const;
  template<typename W>
  inline void get_all_block_entities_snbt_json_write(W& writeable_output) const;

  /**
   * Every mobile entity as a JSON array of typed SNBT strings (full compound
   * incl. `id`/`Pos`).
   */
  inline std::string get_entities_snbt_json() const;
  template<typename W>
  inline void get_entities_snbt_json_write(W& writeable_output) const;

  /**
   * Add a mobile entity from a full SNBT entity compound (must contain `id`
   * and `Pos`).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_entity_from_snbt(std::string_view snbt);

  /**
   * Every IN-BOUNDS cell as a JSON array of
   * `{"x", "y", "z", "name", "properties"}` (the old `CBlockArray`).
   * Air cells are materialized too — on a large sparse build this
   * dump is `volume()`-sized and can exhaust wasm memory; renderers
   * and analyzers want `get_non_air_blocks_json`.
   *
   * Prefer `get_non_air_blocks_json` for a block list,
   * `count_blocks_json` for a material tally and
   * `non_air_blocks_packed_b64` for bulk transfer. This method is
   * kept for compatibility and is the wrong tool at any real size.
   */
  inline std::string get_all_blocks_json() const;
  template<typename W>
  inline void get_all_blocks_json_write(W& writeable_output) const;

  /**
   * Every non-air block of ONE named region (a flattened design names
   * one per layer: `inst:{name}`, `bus:{name}`), same JSON shape as
   * `get_all_blocks_json`. Unknown region names error.
   */
  inline nucleation::diplomat::result<nucleation::diplomat::result<std::string, nucleation::NucleationError>, nucleation::diplomat::Utf8Error> get_region_non_air_blocks_json(std::string_view region_name) const;
  template<typename W>
  inline nucleation::diplomat::result<nucleation::diplomat::result<std::monostate, nucleation::NucleationError>, nucleation::diplomat::Utf8Error> get_region_non_air_blocks_json_write(std::string_view region_name, W& writeable_output) const;

  /**
   * Every non-air block, same JSON shape as `get_all_blocks_json`.
   * `block_count()`-sized regardless of the bounding volume.
   */
  inline std::string get_non_air_blocks_json() const;
  template<typename W>
  inline void get_non_air_blocks_json_write(W& writeable_output) const;

  /**
   * Non-air blocks tallied by id: `{"minecraft:stone": 123, ...}`.
   * One pass, no per block allocation, so a caller that only wants a
   * material list never has to pull `get_non_air_blocks_json`. "Air"
   * covers `minecraft:air`, `cave_air` and `void_air` alike.
   */
  inline std::string count_blocks_json() const;
  template<typename W>
  inline void count_blocks_json_write(W& writeable_output) const;

  /**
   * Apply a `{"from id": "to id"}` map in place and return how many
   * blocks changed. Keys match on block id only, ignoring block
   * states; values may carry states (`minecraft:oak_stairs[facing=north]`),
   * but not NBT: `parse_block_string` only returns a `BlockState`, so
   * any `{...}` payload on a `to` value is silently dropped rather
   * than copied onto the replaced block.
   * A block whose id is not a key is left alone, and so is one that
   * already equals its target: the count is the number of blocks
   * actually changed, so a map that rewrites stone to stone returns 0.
   * Errors with `Parse` on malformed JSON or an unparseable target id.
   */
  inline nucleation::diplomat::result<uint64_t, nucleation::NucleationError> replace_blocks_json(std::string_view map_json);

  /**
   * Every non-air block as a compact binary blob, base64 encoded
   * (`DiplomatWrite` is UTF-8 only, see `to_litematic_b64`). Little
   * endian throughout:
   *
   * ```text
   * u32 count
   * count * { i32 x, i32 y, i32 z, u16 palette_index }
   * u32 palette_json_len
   * u8[palette_json_len]   ["minecraft:stone", ...]
   * ```
   *
   * Palette indices are assigned in first-seen order, so the same
   * schematic always packs identically. About seven times smaller
   * than `get_non_air_blocks_json` and free of per block JSON
   * parsing on the far side.
   *
   * Palette indices are `u16`, so at most 65,535 distinct non-air
   * block states can be addressed. A schematic with more than that
   * writes **an empty string**, not a truncated palette: callers must
   * treat an empty result as "too many distinct states, fall back to
   * `get_non_air_blocks_json`". No real build has that many.
   */
  inline std::string non_air_blocks_packed_b64() const;
  template<typename W>
  inline void non_air_blocks_packed_b64_write(W& writeable_output) const;

  /**
   * All blocks within a sub-region (chunk) of the schematic, as the same
   * JSON array shape as `get_all_blocks_json`.
   */
  inline std::string get_chunk_blocks_json(int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length) const;
  template<typename W>
  inline void get_chunk_blocks_json_write(int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length, W& writeable_output) const;

  /**
   * Split the schematic into chunks (default bottom-up strategy). Writes a
   * JSON array of `{"chunk_x", "chunk_y", "chunk_z", "blocks": [...]}` where
   * blocks have the `get_all_blocks_json` shape (the old `CChunkArray`).
   */
  inline std::string get_chunks_json(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length) const;
  template<typename W>
  inline void get_chunks_json_write(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, W& writeable_output) const;

  /**
   * Split the schematic into chunks with a loading strategy: one of
   * `distance_to_camera`, `top_down`, `bottom_up`, `center_outward`,
   * `random` (anything else falls back to `bottom_up`). Camera coordinates
   * are only used by `distance_to_camera`. Same JSON shape as
   * `get_chunks_json`.
   */
  inline std::string get_chunks_with_strategy_json(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, std::string_view strategy, float camera_x, float camera_y, float camera_z) const;
  template<typename W>
  inline void get_chunks_with_strategy_json_write(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, std::string_view strategy, float camera_x, float camera_y, float camera_z, W& writeable_output) const;

  /**
   * The total number of non-air blocks in the schematic.
   */
  inline int32_t block_count() const;

  /**
   * The total volume of the schematic's bounding box.
   */
  inline int32_t volume() const;

  /**
   * The names of all regions, as a JSON array of strings.
   */
  inline std::string region_names_json() const;
  template<typename W>
  inline void region_names_json_write(W& writeable_output) const;

  /**
   * Basic debug info about the schematic (name + region count).
   */
  inline std::string debug_info() const;
  template<typename W>
  inline void debug_info_write(W& writeable_output) const;

  /**
   * A formatted schematic layout string (old `schematic_print`).
   */
  inline std::string print_string() const;
  template<typename W>
  inline void print_string_write(W& writeable_output) const;

  /**
   * A formatted schematic layout string (old `schematic_print_schematic`;
   * same output as `print_string`).
   */
  inline std::string print_schematic_string() const;
  template<typename W>
  inline void print_schematic_string_write(W& writeable_output) const;

  /**
   * A detailed debug string, including a visual layout (old `debug_schematic`).
   */
  inline std::string debug_string() const;
  template<typename W>
  inline void debug_string_write(W& writeable_output) const;

  /**
   * A detailed debug string with a JSON layout (old `debug_json_schematic`).
   */
  inline std::string debug_json_string() const;
  template<typename W>
  inline void debug_json_string_write(W& writeable_output) const;

  /**
   * The schematic name, or the empty string if not set.
   *
   * Total, like every other metadata accessor: absence is a blank
   * field, not an error — a file that simply doesn't carry the field
   * (Sponge without attribution, a fresh schematic) reads as `""`,
   * the same value a litematic round-trip of an unset field yields.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> name() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> name_write(W& writeable_output) const;

  /**
   * Set the schematic name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_name(std::string_view name);

  /**
   * The schematic author, or the empty string if not set.
   *
   * Total, like every other metadata accessor: absence is a blank
   * field, not an error — a file that simply doesn't carry the field
   * (Sponge without attribution, a fresh schematic) reads as `""`,
   * the same value a litematic round-trip of an unset field yields.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> author() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> author_write(W& writeable_output) const;

  /**
   * Set the schematic author.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_author(std::string_view author);

  /**
   * The schematic description, or the empty string if not set.
   *
   * Total, like every other metadata accessor: absence is a blank
   * field, not an error — a file that simply doesn't carry the field
   * (Sponge without attribution, a fresh schematic) reads as `""`,
   * the same value a litematic round-trip of an unset field yields.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> description() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> description_write(W& writeable_output) const;

  /**
   * Set the schematic description.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_description(std::string_view description);

  /**
   * The creation timestamp (milliseconds since epoch), or `-1` if not set.
   */
  inline int64_t created() const;

  /**
   * Set the creation timestamp (milliseconds since epoch).
   */
  inline void set_created(uint64_t created);

  /**
   * The modification timestamp (milliseconds since epoch), or `-1` if not set.
   */
  inline int64_t modified() const;

  /**
   * Set the modification timestamp (milliseconds since epoch).
   */
  inline void set_modified(uint64_t modified);

  /**
   * The Litematic format version, or `-1` if not set.
   */
  inline int32_t lm_version() const;

  /**
   * Set the Litematic format version.
   */
  inline void set_lm_version(int32_t version);

  /**
   * The Minecraft data version, or `-1` if not set.
   */
  inline int32_t mc_version() const;

  /**
   * Set the Minecraft data version.
   */
  inline void set_mc_version(int32_t version);

  /**
   * The WorldEdit version, or `-1` if not set.
   */
  inline int32_t we_version() const;

  /**
   * Set the WorldEdit version.
   */
  inline void set_we_version(int32_t version);

  /**
   * Standard embedded source provenance as canonical JSON. Returns an
   * empty string when none is present.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> provenance_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> provenance_json_write(W& writeable_output) const;

  /**
   * Validate and set standard embedded source provenance from JSON.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_provenance_json(std::string_view json);

  /**
   * Remove embedded source provenance.
   */
  inline void clear_provenance();

  /**
   * Content-addressed processing history as a JSON array. This audit
   * trail is deliberately separate from immutable source provenance.
   */
  inline std::string transformation_history_json() const;
  template<typename W>
  inline void transformation_history_json_write(W& writeable_output) const;

  /**
   * Clear processing history without changing source provenance or
   * schematic content. Intended for callers constructing a new artifact
   * lineage, not for hiding registry audit records.
   */
  inline void clear_transformation_history();

  /**
   * Mirror the default region along the X axis (in place). Block
   * orientations, block entities, and entities are mirrored too.
   */
  inline void flip_x();

  /**
   * Mirror the default region along the Y axis (in place).
   */
  inline void flip_y();

  /**
   * Mirror the default region along the Z axis (in place).
   */
  inline void flip_z();

  /**
   * Rotate the default region about the X axis. +90° maps south (+Z)
   * to down (-Y). Only multiples of 90 are accepted; invalid angles
   * return `InvalidArgument` without changing the schematic. Negative
   * values wrap.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_x(int32_t degrees);

  /**
   * Rotate the default region clockwise about the Y axis when viewed
   * from above. +90° maps east (+X) to south (+Z).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_y(int32_t degrees);

  /**
   * Rotate the default region about the Z axis. +90° maps up (+Y) to
   * west (-X).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_z(int32_t degrees);

  /**
   * Move the default region and all attached block entities/entities.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> translate(int32_t dx, int32_t dy, int32_t dz);

  /**
   * Mirror a named region along the X axis.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_region_x(std::string_view region_name);

  /**
   * Mirror a named region along the Y axis.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_region_y(std::string_view region_name);

  /**
   * Mirror a named region along the Z axis.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_region_z(std::string_view region_name);

  /**
   * Rotate a named region about the X axis by a multiple of 90 degrees.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_region_x(std::string_view region_name, int32_t degrees);

  /**
   * Rotate a named region clockwise about the Y axis by a multiple of
   * 90 degrees.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_region_y(std::string_view region_name, int32_t degrees);

  /**
   * Rotate a named region about the Z axis by a multiple of 90 degrees.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_region_z(std::string_view region_name, int32_t degrees);

  /**
   * Move one named region without affecting its siblings.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> translate_region(std::string_view region_name, int32_t dx, int32_t dy, int32_t dz);

  /**
   * Rotate every region as one rigid schematic around the shared bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_schematic_x(int32_t degrees);

  /**
   * Rotate every region as one rigid schematic around the shared bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_schematic_y(int32_t degrees);

  /**
   * Rotate every region as one rigid schematic around the shared bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_schematic_z(int32_t degrees);

  /**
   * Mirror every region across the shared schematic X bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_schematic_x();

  /**
   * Mirror every region across the shared schematic Y bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_schematic_y();

  /**
   * Mirror every region across the shared schematic Z bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_schematic_z();

  /**
   * Move every region by the same delta, preserving their relative layout.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> translate_schematic(int32_t dx, int32_t dy, int32_t dz);

  /**
   * Fill a cuboid with a block.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> fill_cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, std::string_view block_name);

  /**
   * Fill a sphere with a block.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> fill_sphere(float cx, float cy, float cz, float radius, std::string_view block_name);

  /**
   * Serialize to a named format, base64-encoded. `version` and `settings`
   * may be empty strings for defaults.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> save_as_b64(std::string_view format, std::string_view version, std::string_view settings) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save_as_b64_write(std::string_view format, std::string_view version, std::string_view settings, W& writeable_output) const;

  /**
   * Save to a file. If `format` is empty, the format is auto-detected from
   * the file extension; `version` may be empty for the default.
   * Not available in JS (no filesystem in WASM) — use `save_as_b64`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save_to_file_with_format(std::string_view path, std::string_view format, std::string_view version) const;

  /**
   * Serialize as a Sponge schematic targeting a specific format version,
   * base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_schematic_version_b64(std::string_view version) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_schematic_version_b64_write(std::string_view version, W& writeable_output) const;

  /**
   * The available Sponge schematic exporter versions, as a JSON array of
   * strings.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> available_schematic_versions_json();
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> available_schematic_versions_json_write(W& writeable_output);

  /**
   * Set a block with NBT data given as a JSON object of string→string
   * (may be empty).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_block_with_nbt(int32_t x, int32_t y, int32_t z, std::string_view block_name, std::string_view nbt_json);

  /**
   * Set a block (by name) in a named region.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_block_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z, std::string_view block_name);

  /**
   * Whether a default or named schematic region exists.
   */
  inline nucleation::diplomat::result<bool, nucleation::NucleationError> has_region(std::string_view region_name) const;

  /**
   * Create an empty named region. Its first block anchors its bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> create_region(std::string_view region_name);

  /**
   * Remove a named region. The default region cannot be removed.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> remove_region(std::string_view region_name);

  /**
   * Rename a named region. The default region cannot be renamed.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rename_region(std::string_view old_name, std::string_view new_name);

  /**
   * The schematic bounding box as a JSON array
   * `[min_x, min_y, min_z, max_x, max_y, max_z]`.
   */
  inline std::string bounding_box_json() const;
  template<typename W>
  inline void bounding_box_json_write(W& writeable_output) const;

  /**
   * A named region's bounding box as a JSON array
   * `[min_x, min_y, min_z, max_x, max_y, max_z]`. `"default"`/`"Default"`
   * address the default region.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> region_bounding_box_json(std::string_view region_name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> region_bounding_box_json_write(std::string_view region_name, W& writeable_output) const;

  /**
   * The merged-region palette block names, as a JSON array of strings.
   */
  inline std::string palette_json() const;
  template<typename W>
  inline void palette_json_write(W& writeable_output) const;

  /**
   * The tight (content) dimensions.
   */
  inline nucleation::Dimensions tight_dimensions() const;

  /**
   * The allocated dimensions (same as `dimensions`; named for parity with
   * the old `schematic_get_allocated_dimensions`).
   */
  inline nucleation::Dimensions allocated_dimensions() const;

  /**
   * Every sign in the schematic, as a JSON array of
   * `{"pos": [x,y,z], "text": [...]}`.
   */
  inline std::string extract_signs_json() const;
  template<typename W>
  inline void extract_signs_json_write(W& writeable_output) const;

  /**
   * Compile the schematic's insign annotations to JSON.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> compile_insign_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> compile_insign_json_write(W& writeable_output) const;

  /**
   * Embed a `CellContract` (JSON) in the schematic's metadata,
   * validating it parses first. The contract is carried through
   * `.schem` save/open and autodetected on open — schematic +
   * contract = one self-describing typed cell.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_cell_contract_json(std::string_view json);

  /**
   * The contract embedded in the schematic's metadata, as JSON.
   * Errors with `NotFound` when none is embedded, `Parse` when an
   * embedded string exists but is corrupt (loud, never silent).
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> cell_contract_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> cell_contract_json_write(W& writeable_output) const;

  /**
   * Resolve the schematic's cell contract from its sources in
   * strict precedence — embedded metadata over Insign signs — with
   * loud conflict warnings. Writes `{"contract": ..., "warnings":
   * [...]}`; errors with `NotFound` when no source defines one.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> resolve_cell_contract_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> resolve_cell_contract_json_write(W& writeable_output) const;

  /**
   * Parse the schematic's IO-contract insign annotations (`#cell`
   * header, `bus.*` port annotations, `#route_zone` zones) to JSON:
   * `{"cell": ..., "buses": [...], "route_zones": {...}}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> compile_io_contracts_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> compile_io_contracts_json_write(W& writeable_output) const;

  /**
   * Every region's palette, as a JSON object mapping region name → array of
   * block names (the default region under `"default"`).
   */
  inline std::string all_palettes_json() const;
  template<typename W>
  inline void all_palettes_json_write(W& writeable_output) const;

  /**
   * The default region's palette block names, as a JSON array of strings.
   */
  inline std::string default_region_palette_json() const;
  template<typename W>
  inline void default_region_palette_json_write(W& writeable_output) const;

  /**
   * A named region's palette block names, as a JSON array of strings.
   * `"default"`/`"Default"` address the default region.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> region_palette_json(std::string_view region_name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> region_palette_json_write(std::string_view region_name, W& writeable_output) const;

  /**
   * The minimum corner of the tight (content) bounds. `NotFound` when the
   * schematic has no content.
   */
  inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> tight_bounds_min() const;

  /**
   * The maximum corner of the tight (content) bounds. `NotFound` when the
   * schematic has no content.
   */
  inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> tight_bounds_max() const;

    inline const nucleation::capi::Schematic* AsFFI() const;
    inline nucleation::capi::Schematic* AsFFI();
    inline static const nucleation::Schematic* FromFFI(const nucleation::capi::Schematic* ptr);
    inline static nucleation::Schematic* FromFFI(nucleation::capi::Schematic* ptr);
    inline static void operator delete(void* ptr);
private:
    Schematic() = delete;
    Schematic(const nucleation::Schematic&) = delete;
    Schematic(nucleation::Schematic&&) noexcept = delete;
    Schematic operator=(const nucleation::Schematic&) = delete;
    Schematic operator=(nucleation::Schematic&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Schematic_D_HPP
