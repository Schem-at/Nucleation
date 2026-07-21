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
   * Load a schematic from a file, auto-detecting the format from the
   * contents (any supported format, whatever the extension says).
   * Not available in JS: the WASM build has no filesystem — read the
   * bytes yourself and use `from_data`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> load_from_file(std::string_view path);

  /**
   * Build a schematic from raw byte data, auto-detecting the format.
   * Supports Litematic, Sponge Schematic, and McStructure (Bedrock) formats.
   * `Parse` if a format was detected but failed to parse, `InvalidArgument` if
   * no format was recognized.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> from_data(nucleation::diplomat::span<const uint8_t> data);

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
   * Batch-get block names at multiple positions. `positions` is flat
   * `[x0,y0,z0, ...]` (length must be a multiple of 3). Writes a JSON array,
   * one entry per position: the block name string, or `null` for
   * empty/out-of-bounds positions.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_blocks_json(nucleation::diplomat::span<const int32_t> positions) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_blocks_json_write(nucleation::diplomat::span<const int32_t> positions, W& writeable_output) const;

  /**
   * Copy a region from `source` into this schematic. `excluded_blocks_json`
   * is a JSON array of block strings to skip (empty string or `[]` for none).
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
   * Every non-air block as a JSON array of
   * `{"x", "y", "z", "name", "properties"}` (the old `CBlockArray`).
   */
  inline std::string get_all_blocks_json() const;
  template<typename W>
  inline void get_all_blocks_json_write(W& writeable_output) const;

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
   * The schematic name. `NotFound` if not set.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> name() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> name_write(W& writeable_output) const;

  /**
   * Set the schematic name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_name(std::string_view name);

  /**
   * The schematic author. `NotFound` if not set.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> author() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> author_write(W& writeable_output) const;

  /**
   * Set the schematic author.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_author(std::string_view author);

  /**
   * The schematic description. `NotFound` if not set.
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
   * Mirror the default region along the X axis (in place). Block
   * orientations (e.g. `facing` properties), block entities, and
   * entities are mirrored too.
   */
  inline void flip_x();

  /**
   * Mirror the default region along the Y axis (in place). Block
   * orientations, block entities, and entities are mirrored too.
   */
  inline void flip_y();

  /**
   * Mirror the default region along the Z axis (in place). Block
   * orientations, block entities, and entities are mirrored too.
   */
  inline void flip_z();

  /**
   * Rotate the default region about the X axis. `degrees` must be a
   * multiple of 90 (anything else is a no-op; negative values wrap).
   * +90° maps +Z onto +Y (south face rotates up). The region keeps its
   * minimum corner; block orientations and entities are updated.
   */
  inline void rotate_x(int32_t degrees);

  /**
   * Rotate the default region about the Y axis (horizontal plane).
   * `degrees` must be a multiple of 90 (anything else is a no-op;
   * negative values wrap). +90° maps +X onto -Z (east to north, i.e.
   * counterclockwise seen from above). The region keeps its minimum
   * corner; block orientations and entities are updated.
   */
  inline void rotate_y(int32_t degrees);

  /**
   * Rotate the default region about the Z axis. `degrees` must be a
   * multiple of 90 (anything else is a no-op; negative values wrap).
   * +90° maps +Y onto +X (up rotates east). The region keeps its
   * minimum corner; block orientations and entities are updated.
   */
  inline void rotate_z(int32_t degrees);

  /**
   * Mirror a named region along the X axis (like `flip_x`). `NotFound`
   * if no region has that name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_region_x(std::string_view region_name);

  /**
   * Mirror a named region along the Y axis (like `flip_y`). `NotFound`
   * if no region has that name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_region_y(std::string_view region_name);

  /**
   * Mirror a named region along the Z axis (like `flip_z`). `NotFound`
   * if no region has that name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> flip_region_z(std::string_view region_name);

  /**
   * Rotate a named region about the X axis by a multiple of 90 degrees
   * (same semantics as `rotate_x`). `NotFound` if no region has that
   * name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_region_x(std::string_view region_name, int32_t degrees);

  /**
   * Rotate a named region about the Y axis by a multiple of 90 degrees
   * (same semantics as `rotate_y`). `NotFound` if no region has that
   * name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_region_y(std::string_view region_name, int32_t degrees);

  /**
   * Rotate a named region about the Z axis by a multiple of 90 degrees
   * (same semantics as `rotate_z`). `NotFound` if no region has that
   * name.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rotate_region_z(std::string_view region_name, int32_t degrees);

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
