#ifndef Schematic_HPP
#define Schematic_HPP

#include "Schematic.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "BlockPos.hpp"
#include "BlockState.hpp"
#include "Dimensions.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::Schematic* Schematic_create(diplomat::capi::DiplomatStringView name);

    diplomat::capi::Schematic* Schematic_deep_clone(const diplomat::capi::Schematic* self);

    diplomat::capi::Dimensions Schematic_dimensions(const diplomat::capi::Schematic* self);

    typedef struct Schematic_set_block_result {union {bool ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_result;
    Schematic_set_block_result Schematic_set_block(diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_get_block_name_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_name_result;
    Schematic_get_block_name_result Schematic_get_block_name(const diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_save_to_file_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_save_to_file_result;
    Schematic_save_to_file_result Schematic_save_to_file(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_save_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_save_result;
    Schematic_save_result Schematic_save(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_load_from_file_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_load_from_file_result;
    Schematic_load_from_file_result Schematic_load_from_file(diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_open_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_open_result;
    Schematic_open_result Schematic_open(diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_from_data_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_data_result;
    Schematic_from_data_result Schematic_from_data(diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_from_litematic_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_litematic_result;
    Schematic_from_litematic_result Schematic_from_litematic(diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_litematic_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_litematic_b64_result;
    Schematic_to_litematic_b64_result Schematic_to_litematic_b64(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_schematic_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_schematic_result;
    Schematic_from_schematic_result Schematic_from_schematic(diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_schematic_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_schematic_b64_result;
    Schematic_to_schematic_b64_result Schematic_to_schematic_b64(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_snapshot_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_snapshot_result;
    Schematic_from_snapshot_result Schematic_from_snapshot(diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_snapshot_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_snapshot_b64_result;
    Schematic_to_snapshot_b64_result Schematic_to_snapshot_b64(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_mcstructure_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_mcstructure_result;
    Schematic_from_mcstructure_result Schematic_from_mcstructure(diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_mcstructure_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_mcstructure_b64_result;
    Schematic_to_mcstructure_b64_result Schematic_to_mcstructure_b64(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_mca_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_mca_result;
    Schematic_from_mca_result Schematic_from_mca(diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_from_mca_bounded_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_mca_bounded_result;
    Schematic_from_mca_bounded_result Schematic_from_mca_bounded(diplomat::capi::DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Schematic_from_world_zip_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_zip_result;
    Schematic_from_world_zip_result Schematic_from_world_zip(diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_from_world_zip_bounded_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_zip_bounded_result;
    Schematic_from_world_zip_bounded_result Schematic_from_world_zip_bounded(diplomat::capi::DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Schematic_from_world_directory_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_directory_result;
    Schematic_from_world_directory_result Schematic_from_world_directory(diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_from_world_directory_bounded_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_directory_bounded_result;
    Schematic_from_world_directory_bounded_result Schematic_from_world_directory_bounded(diplomat::capi::DiplomatStringView path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Schematic_to_world_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_world_json_result;
    Schematic_to_world_json_result Schematic_to_world_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView options_json, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_save_world_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_save_world_result;
    Schematic_save_world_result Schematic_save_world(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView directory, diplomat::capi::DiplomatStringView options_json);

    typedef struct Schematic_to_world_zip_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_world_zip_b64_result;
    Schematic_to_world_zip_b64_result Schematic_to_world_zip_b64(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView options_json, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_block_with_properties_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_with_properties_result;
    Schematic_set_block_with_properties_result Schematic_set_block_with_properties(diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block_name, diplomat::capi::DiplomatStringView properties_json);

    typedef struct Schematic_set_block_from_string_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_from_string_result;
    Schematic_set_block_from_string_result Schematic_set_block_from_string(diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block_string);

    typedef struct Schematic_prepare_block_result {union {int32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_prepare_block_result;
    Schematic_prepare_block_result Schematic_prepare_block(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_place_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_place_result;
    Schematic_place_result Schematic_place(diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, int32_t palette_index);

    typedef struct Schematic_set_blocks_result {union {int32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_blocks_result;
    Schematic_set_blocks_result Schematic_set_blocks(diplomat::capi::Schematic* self, diplomat::capi::DiplomatI32View positions, diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_get_blocks_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_blocks_json_result;
    Schematic_get_blocks_json_result Schematic_get_blocks_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatI32View positions, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_stamp_box_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_stamp_box_result;
    Schematic_stamp_box_result Schematic_stamp_box(diplomat::capi::Schematic* self, const diplomat::capi::Schematic* source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, diplomat::capi::DiplomatStringView excluded_blocks_json);

    typedef struct Schematic_stamp_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_stamp_region_result;
    Schematic_stamp_region_result Schematic_stamp_region(diplomat::capi::Schematic* self, const diplomat::capi::Schematic* source, diplomat::capi::DiplomatStringView source_region_name, int32_t target_x, int32_t target_y, int32_t target_z, diplomat::capi::DiplomatStringView excluded_blocks_json);

    typedef struct Schematic_copy_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_copy_region_result;
    Schematic_copy_region_result Schematic_copy_region(diplomat::capi::Schematic* self, const diplomat::capi::Schematic* source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, diplomat::capi::DiplomatStringView excluded_blocks_json);

    typedef struct Schematic_get_block_result {union {diplomat::capi::BlockState* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_result;
    Schematic_get_block_result Schematic_get_block(const diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z);

    typedef struct Schematic_get_block_with_properties_result {union {diplomat::capi::BlockState* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_with_properties_result;
    Schematic_get_block_with_properties_result Schematic_get_block_with_properties(const diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z);

    typedef struct Schematic_get_block_in_region_result {union {diplomat::capi::BlockState* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_in_region_result;
    Schematic_get_block_in_region_result Schematic_get_block_in_region(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t x, int32_t y, int32_t z);

    typedef struct Schematic_get_block_string_in_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_string_in_region_result;
    Schematic_get_block_string_in_region_result Schematic_get_block_string_in_region(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_get_block_string_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_string_result;
    Schematic_get_block_string_result Schematic_get_block_string(const diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_get_block_entity_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_entity_json_result;
    Schematic_get_block_entity_json_result Schematic_get_block_entity_json(const diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_get_block_entity_json_in_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_entity_json_in_region_result;
    Schematic_get_block_entity_json_in_region_result Schematic_get_block_entity_json_in_region(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    void Schematic_get_all_block_entities_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    uint32_t Schematic_entity_count(const diplomat::capi::Schematic* self);

    void Schematic_get_entities_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_add_entity_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_add_entity_result;
    Schematic_add_entity_result Schematic_add_entity(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView id, double x, double y, double z, diplomat::capi::DiplomatStringView nbt_json);

    typedef struct Schematic_add_armor_stand_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_add_armor_stand_result;
    Schematic_add_armor_stand_result Schematic_add_armor_stand(diplomat::capi::Schematic* self, double x, double y, double z, float yaw, diplomat::capi::DiplomatStringView armor_material);

    typedef struct Schematic_remove_entity_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_remove_entity_result;
    Schematic_remove_entity_result Schematic_remove_entity(diplomat::capi::Schematic* self, uint32_t index);

    int32_t Schematic_canonical_data_version(void);

    void Schematic_convert_to_data_version(diplomat::capi::Schematic* self, int32_t target_data_version, int32_t source_data_version, diplomat::capi::DiplomatWrite* write);

    void Schematic_convert_to_version(diplomat::capi::Schematic* self, int32_t target_data_version, diplomat::capi::DiplomatWrite* write);

    int32_t Schematic_source_data_version(const diplomat::capi::Schematic* self);

    void Schematic_set_source_data_version(diplomat::capi::Schematic* self, int32_t version);

    typedef struct Schematic_to_litematic_for_version_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_litematic_for_version_json_result;
    Schematic_to_litematic_for_version_json_result Schematic_to_litematic_for_version_json(const diplomat::capi::Schematic* self, int32_t target_data_version, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_get_block_entity_snbt_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_entity_snbt_result;
    Schematic_get_block_entity_snbt_result Schematic_get_block_entity_snbt(const diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_block_entity_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_entity_result;
    Schematic_set_block_entity_result Schematic_set_block_entity(diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView id, diplomat::capi::DiplomatStringView snbt);

    typedef struct Schematic_remove_block_entity_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_remove_block_entity_result;
    Schematic_remove_block_entity_result Schematic_remove_block_entity(diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z);

    void Schematic_get_all_block_entities_snbt_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_get_entities_snbt_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_add_entity_from_snbt_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_add_entity_from_snbt_result;
    Schematic_add_entity_from_snbt_result Schematic_add_entity_from_snbt(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView snbt);

    void Schematic_get_all_blocks_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_get_chunk_blocks_json(const diplomat::capi::Schematic* self, int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length, diplomat::capi::DiplomatWrite* write);

    void Schematic_get_chunks_json(const diplomat::capi::Schematic* self, int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, diplomat::capi::DiplomatWrite* write);

    void Schematic_get_chunks_with_strategy_json(const diplomat::capi::Schematic* self, int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, diplomat::capi::DiplomatStringView strategy, float camera_x, float camera_y, float camera_z, diplomat::capi::DiplomatWrite* write);

    int32_t Schematic_block_count(const diplomat::capi::Schematic* self);

    int32_t Schematic_volume(const diplomat::capi::Schematic* self);

    void Schematic_region_names_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_debug_info(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_print_string(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_print_schematic_string(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_debug_string(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_debug_json_string(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_name_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_name_result;
    Schematic_name_result Schematic_name(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_name_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_name_result;
    Schematic_set_name_result Schematic_set_name(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView name);

    typedef struct Schematic_author_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_author_result;
    Schematic_author_result Schematic_author(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_author_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_author_result;
    Schematic_set_author_result Schematic_set_author(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView author);

    typedef struct Schematic_description_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_description_result;
    Schematic_description_result Schematic_description(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_description_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_description_result;
    Schematic_set_description_result Schematic_set_description(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView description);

    int64_t Schematic_created(const diplomat::capi::Schematic* self);

    void Schematic_set_created(diplomat::capi::Schematic* self, uint64_t created);

    int64_t Schematic_modified(const diplomat::capi::Schematic* self);

    void Schematic_set_modified(diplomat::capi::Schematic* self, uint64_t modified);

    int32_t Schematic_lm_version(const diplomat::capi::Schematic* self);

    void Schematic_set_lm_version(diplomat::capi::Schematic* self, int32_t version);

    int32_t Schematic_mc_version(const diplomat::capi::Schematic* self);

    void Schematic_set_mc_version(diplomat::capi::Schematic* self, int32_t version);

    int32_t Schematic_we_version(const diplomat::capi::Schematic* self);

    void Schematic_set_we_version(diplomat::capi::Schematic* self, int32_t version);

    void Schematic_flip_x(diplomat::capi::Schematic* self);

    void Schematic_flip_y(diplomat::capi::Schematic* self);

    void Schematic_flip_z(diplomat::capi::Schematic* self);

    typedef struct Schematic_rotate_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_x_result;
    Schematic_rotate_x_result Schematic_rotate_x(diplomat::capi::Schematic* self, int32_t degrees);

    typedef struct Schematic_rotate_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_y_result;
    Schematic_rotate_y_result Schematic_rotate_y(diplomat::capi::Schematic* self, int32_t degrees);

    typedef struct Schematic_rotate_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_z_result;
    Schematic_rotate_z_result Schematic_rotate_z(diplomat::capi::Schematic* self, int32_t degrees);

    typedef struct Schematic_translate_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_translate_result;
    Schematic_translate_result Schematic_translate(diplomat::capi::Schematic* self, int32_t dx, int32_t dy, int32_t dz);

    typedef struct Schematic_flip_region_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_flip_region_x_result;
    Schematic_flip_region_x_result Schematic_flip_region_x(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_flip_region_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_flip_region_y_result;
    Schematic_flip_region_y_result Schematic_flip_region_y(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_flip_region_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_flip_region_z_result;
    Schematic_flip_region_z_result Schematic_flip_region_z(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_rotate_region_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_region_x_result;
    Schematic_rotate_region_x_result Schematic_rotate_region_x(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t degrees);

    typedef struct Schematic_rotate_region_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_region_y_result;
    Schematic_rotate_region_y_result Schematic_rotate_region_y(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t degrees);

    typedef struct Schematic_rotate_region_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_region_z_result;
    Schematic_rotate_region_z_result Schematic_rotate_region_z(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t degrees);

    typedef struct Schematic_translate_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_translate_region_result;
    Schematic_translate_region_result Schematic_translate_region(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t dx, int32_t dy, int32_t dz);

    typedef struct Schematic_rotate_schematic_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_schematic_x_result;
    Schematic_rotate_schematic_x_result Schematic_rotate_schematic_x(diplomat::capi::Schematic* self, int32_t degrees);

    typedef struct Schematic_rotate_schematic_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_schematic_y_result;
    Schematic_rotate_schematic_y_result Schematic_rotate_schematic_y(diplomat::capi::Schematic* self, int32_t degrees);

    typedef struct Schematic_rotate_schematic_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_schematic_z_result;
    Schematic_rotate_schematic_z_result Schematic_rotate_schematic_z(diplomat::capi::Schematic* self, int32_t degrees);

    typedef struct Schematic_flip_schematic_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_flip_schematic_x_result;
    Schematic_flip_schematic_x_result Schematic_flip_schematic_x(diplomat::capi::Schematic* self);

    typedef struct Schematic_flip_schematic_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_flip_schematic_y_result;
    Schematic_flip_schematic_y_result Schematic_flip_schematic_y(diplomat::capi::Schematic* self);

    typedef struct Schematic_flip_schematic_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_flip_schematic_z_result;
    Schematic_flip_schematic_z_result Schematic_flip_schematic_z(diplomat::capi::Schematic* self);

    typedef struct Schematic_translate_schematic_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_translate_schematic_result;
    Schematic_translate_schematic_result Schematic_translate_schematic(diplomat::capi::Schematic* self, int32_t dx, int32_t dy, int32_t dz);

    typedef struct Schematic_fill_cuboid_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_fill_cuboid_result;
    Schematic_fill_cuboid_result Schematic_fill_cuboid(diplomat::capi::Schematic* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_fill_sphere_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_fill_sphere_result;
    Schematic_fill_sphere_result Schematic_fill_sphere(diplomat::capi::Schematic* self, float cx, float cy, float cz, float radius, diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_save_as_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_save_as_b64_result;
    Schematic_save_as_b64_result Schematic_save_as_b64(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView format, diplomat::capi::DiplomatStringView version, diplomat::capi::DiplomatStringView settings, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_save_to_file_with_format_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_save_to_file_with_format_result;
    Schematic_save_to_file_with_format_result Schematic_save_to_file_with_format(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView path, diplomat::capi::DiplomatStringView format, diplomat::capi::DiplomatStringView version);

    typedef struct Schematic_to_schematic_version_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_to_schematic_version_b64_result;
    Schematic_to_schematic_version_b64_result Schematic_to_schematic_version_b64(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView version, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_available_schematic_versions_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_available_schematic_versions_json_result;
    Schematic_available_schematic_versions_json_result Schematic_available_schematic_versions_json(diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_block_with_nbt_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_with_nbt_result;
    Schematic_set_block_with_nbt_result Schematic_set_block_with_nbt(diplomat::capi::Schematic* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block_name, diplomat::capi::DiplomatStringView nbt_json);

    typedef struct Schematic_set_block_in_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_in_region_result;
    Schematic_set_block_in_region_result Schematic_set_block_in_region(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_has_region_result {union {bool ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_has_region_result;
    Schematic_has_region_result Schematic_has_region(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_create_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_create_region_result;
    Schematic_create_region_result Schematic_create_region(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_remove_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_remove_region_result;
    Schematic_remove_region_result Schematic_remove_region(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_rename_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_rename_region_result;
    Schematic_rename_region_result Schematic_rename_region(diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView old_name, diplomat::capi::DiplomatStringView new_name);

    void Schematic_bounding_box_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_region_bounding_box_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_region_bounding_box_json_result;
    Schematic_region_bounding_box_json_result Schematic_region_bounding_box_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, diplomat::capi::DiplomatWrite* write);

    void Schematic_palette_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    diplomat::capi::Dimensions Schematic_tight_dimensions(const diplomat::capi::Schematic* self);

    diplomat::capi::Dimensions Schematic_allocated_dimensions(const diplomat::capi::Schematic* self);

    void Schematic_extract_signs_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_compile_insign_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_compile_insign_json_result;
    Schematic_compile_insign_json_result Schematic_compile_insign_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_all_palettes_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    void Schematic_default_region_palette_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_region_palette_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_region_palette_json_result;
    Schematic_region_palette_json_result Schematic_region_palette_json(const diplomat::capi::Schematic* self, diplomat::capi::DiplomatStringView region_name, diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_tight_bounds_min_result {union {diplomat::capi::BlockPos ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_tight_bounds_min_result;
    Schematic_tight_bounds_min_result Schematic_tight_bounds_min(const diplomat::capi::Schematic* self);

    typedef struct Schematic_tight_bounds_max_result {union {diplomat::capi::BlockPos ok; diplomat::capi::NucleationError err;}; bool is_ok;} Schematic_tight_bounds_max_result;
    Schematic_tight_bounds_max_result Schematic_tight_bounds_max(const diplomat::capi::Schematic* self);

    void Schematic_destroy(Schematic* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<Schematic> Schematic::create(std::string_view name) {
    auto result = diplomat::capi::Schematic_create({name.data(), name.size()});
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline std::unique_ptr<Schematic> Schematic::deep_clone() const {
    auto result = diplomat::capi::Schematic_deep_clone(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline Dimensions Schematic::dimensions() const {
    auto result = diplomat::capi::Schematic_dimensions(this->AsFFI());
    return Dimensions::FromFFI(result);
}

inline diplomat::result<bool, NucleationError> Schematic::set_block(int32_t x, int32_t y, int32_t z, std::string_view block_name) {
    auto result = diplomat::capi::Schematic_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<bool, NucleationError>(diplomat::Ok<bool>(result.ok)) : diplomat::result<bool, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::get_block_name(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_get_block_name(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::get_block_name_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_get_block_name(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::save_to_file(std::string_view path) const {
    auto result = diplomat::capi::Schematic_save_to_file(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::save(std::string_view path) const {
    auto result = diplomat::capi::Schematic_save(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::load_from_file(std::string_view path) {
    auto result = diplomat::capi::Schematic_load_from_file({path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::open(std::string_view path) {
    auto result = diplomat::capi::Schematic_open({path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_data(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::Schematic_from_data({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_litematic(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::Schematic_from_litematic({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::to_litematic_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_litematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_litematic_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_litematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_schematic(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::Schematic_from_schematic({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::to_schematic_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_schematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_schematic_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_schematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_snapshot(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::Schematic_from_snapshot({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::to_snapshot_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_snapshot_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_snapshot_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_snapshot_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_mcstructure(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::Schematic_from_mcstructure({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::to_mcstructure_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_mcstructure_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_mcstructure_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_mcstructure_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_mca(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::Schematic_from_mca({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_mca_bounded(diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::Schematic_from_mca_bounded({data.data(), data.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_world_zip(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::Schematic_from_world_zip({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_world_zip_bounded(diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::Schematic_from_world_zip_bounded({data.data(), data.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_world_directory(std::string_view path) {
    auto result = diplomat::capi::Schematic_from_world_directory({path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Schematic::from_world_directory_bounded(std::string_view path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::Schematic_from_world_directory_bounded({path.data(), path.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::to_world_json(std::string_view options_json) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_world_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_world_json_write(std::string_view options_json, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_world_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::save_world(std::string_view directory, std::string_view options_json) const {
    auto result = diplomat::capi::Schematic_save_world(this->AsFFI(),
        {directory.data(), directory.size()},
        {options_json.data(), options_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::to_world_zip_b64(std::string_view options_json) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_world_zip_b64(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_world_zip_b64_write(std::string_view options_json, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_world_zip_b64(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_block_with_properties(int32_t x, int32_t y, int32_t z, std::string_view block_name, std::string_view properties_json) {
    auto result = diplomat::capi::Schematic_set_block_with_properties(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()},
        {properties_json.data(), properties_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_block_from_string(int32_t x, int32_t y, int32_t z, std::string_view block_string) {
    auto result = diplomat::capi::Schematic_set_block_from_string(this->AsFFI(),
        x,
        y,
        z,
        {block_string.data(), block_string.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<int32_t, NucleationError> Schematic::prepare_block(std::string_view block_name) {
    auto result = diplomat::capi::Schematic_prepare_block(this->AsFFI(),
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<int32_t, NucleationError>(diplomat::Ok<int32_t>(result.ok)) : diplomat::result<int32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::place(int32_t x, int32_t y, int32_t z, int32_t palette_index) {
    auto result = diplomat::capi::Schematic_place(this->AsFFI(),
        x,
        y,
        z,
        palette_index);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<int32_t, NucleationError> Schematic::set_blocks(diplomat::span<const int32_t> positions, std::string_view block_name) {
    auto result = diplomat::capi::Schematic_set_blocks(this->AsFFI(),
        {positions.data(), positions.size()},
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<int32_t, NucleationError>(diplomat::Ok<int32_t>(result.ok)) : diplomat::result<int32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::get_blocks_json(diplomat::span<const int32_t> positions) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_get_blocks_json(this->AsFFI(),
        {positions.data(), positions.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::get_blocks_json_write(diplomat::span<const int32_t> positions, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_get_blocks_json(this->AsFFI(),
        {positions.data(), positions.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::stamp_box(const Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, std::string_view excluded_blocks_json) {
    auto result = diplomat::capi::Schematic_stamp_box(this->AsFFI(),
        source.AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        target_x,
        target_y,
        target_z,
        {excluded_blocks_json.data(), excluded_blocks_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::stamp_region(const Schematic& source, std::string_view source_region_name, int32_t target_x, int32_t target_y, int32_t target_z, std::string_view excluded_blocks_json) {
    auto result = diplomat::capi::Schematic_stamp_region(this->AsFFI(),
        source.AsFFI(),
        {source_region_name.data(), source_region_name.size()},
        target_x,
        target_y,
        target_z,
        {excluded_blocks_json.data(), excluded_blocks_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::copy_region(const Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, std::string_view excluded_blocks_json) {
    auto result = diplomat::capi::Schematic_copy_region(this->AsFFI(),
        source.AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        target_x,
        target_y,
        target_z,
        {excluded_blocks_json.data(), excluded_blocks_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<BlockState>, NucleationError> Schematic::get_block(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::Schematic_get_block(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Ok<std::unique_ptr<BlockState>>(std::unique_ptr<BlockState>(BlockState::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<BlockState>, NucleationError> Schematic::get_block_with_properties(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::Schematic_get_block_with_properties(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Ok<std::unique_ptr<BlockState>>(std::unique_ptr<BlockState>(BlockState::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<BlockState>, NucleationError> Schematic::get_block_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::Schematic_get_block_in_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Ok<std::unique_ptr<BlockState>>(std::unique_ptr<BlockState>(BlockState::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::get_block_string_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_get_block_string_in_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::get_block_string_in_region_write(std::string_view region_name, int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_get_block_string_in_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::get_block_string(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_get_block_string(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::get_block_string_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_get_block_string(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::get_block_entity_json(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_get_block_entity_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::get_block_entity_json_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_get_block_entity_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::get_block_entity_json_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_get_block_entity_json_in_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::get_block_entity_json_in_region_write(std::string_view region_name, int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_get_block_entity_json_in_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Schematic::get_all_block_entities_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_all_block_entities_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_all_block_entities_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_all_block_entities_json(this->AsFFI(),
        &write);
}

inline uint32_t Schematic::entity_count() const {
    auto result = diplomat::capi::Schematic_entity_count(this->AsFFI());
    return result;
}

inline std::string Schematic::get_entities_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_entities_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_entities_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_entities_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::monostate, NucleationError> Schematic::add_entity(std::string_view id, double x, double y, double z, std::string_view nbt_json) {
    auto result = diplomat::capi::Schematic_add_entity(this->AsFFI(),
        {id.data(), id.size()},
        x,
        y,
        z,
        {nbt_json.data(), nbt_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material) {
    auto result = diplomat::capi::Schematic_add_armor_stand(this->AsFFI(),
        x,
        y,
        z,
        yaw,
        {armor_material.data(), armor_material.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::remove_entity(uint32_t index) {
    auto result = diplomat::capi::Schematic_remove_entity(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline int32_t Schematic::canonical_data_version() {
    auto result = diplomat::capi::Schematic_canonical_data_version();
    return result;
}

inline std::string Schematic::convert_to_data_version(int32_t target_data_version, int32_t source_data_version) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_convert_to_data_version(this->AsFFI(),
        target_data_version,
        source_data_version,
        &write);
    return output;
}
template<typename W>
inline void Schematic::convert_to_data_version_write(int32_t target_data_version, int32_t source_data_version, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_convert_to_data_version(this->AsFFI(),
        target_data_version,
        source_data_version,
        &write);
}

inline std::string Schematic::convert_to_version(int32_t target_data_version) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_convert_to_version(this->AsFFI(),
        target_data_version,
        &write);
    return output;
}
template<typename W>
inline void Schematic::convert_to_version_write(int32_t target_data_version, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_convert_to_version(this->AsFFI(),
        target_data_version,
        &write);
}

inline int32_t Schematic::source_data_version() const {
    auto result = diplomat::capi::Schematic_source_data_version(this->AsFFI());
    return result;
}

inline void Schematic::set_source_data_version(int32_t version) {
    diplomat::capi::Schematic_set_source_data_version(this->AsFFI(),
        version);
}

inline diplomat::result<std::string, NucleationError> Schematic::to_litematic_for_version_json(int32_t target_data_version) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_litematic_for_version_json(this->AsFFI(),
        target_data_version,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_litematic_for_version_json_write(int32_t target_data_version, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_litematic_for_version_json(this->AsFFI(),
        target_data_version,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::get_block_entity_snbt(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_get_block_entity_snbt(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::get_block_entity_snbt_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_get_block_entity_snbt(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_block_entity(int32_t x, int32_t y, int32_t z, std::string_view id, std::string_view snbt) {
    auto result = diplomat::capi::Schematic_set_block_entity(this->AsFFI(),
        x,
        y,
        z,
        {id.data(), id.size()},
        {snbt.data(), snbt.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::remove_block_entity(int32_t x, int32_t y, int32_t z) {
    auto result = diplomat::capi::Schematic_remove_block_entity(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Schematic::get_all_block_entities_snbt_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_all_block_entities_snbt_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_all_block_entities_snbt_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_all_block_entities_snbt_json(this->AsFFI(),
        &write);
}

inline std::string Schematic::get_entities_snbt_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_entities_snbt_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_entities_snbt_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_entities_snbt_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::monostate, NucleationError> Schematic::add_entity_from_snbt(std::string_view snbt) {
    auto result = diplomat::capi::Schematic_add_entity_from_snbt(this->AsFFI(),
        {snbt.data(), snbt.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Schematic::get_all_blocks_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_all_blocks_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_all_blocks_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_all_blocks_json(this->AsFFI(),
        &write);
}

inline std::string Schematic::get_chunk_blocks_json(int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_chunk_blocks_json(this->AsFFI(),
        offset_x,
        offset_y,
        offset_z,
        width,
        height,
        length,
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_chunk_blocks_json_write(int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_chunk_blocks_json(this->AsFFI(),
        offset_x,
        offset_y,
        offset_z,
        width,
        height,
        length,
        &write);
}

inline std::string Schematic::get_chunks_json(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_chunks_json(this->AsFFI(),
        chunk_width,
        chunk_height,
        chunk_length,
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_chunks_json_write(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_chunks_json(this->AsFFI(),
        chunk_width,
        chunk_height,
        chunk_length,
        &write);
}

inline std::string Schematic::get_chunks_with_strategy_json(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, std::string_view strategy, float camera_x, float camera_y, float camera_z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_get_chunks_with_strategy_json(this->AsFFI(),
        chunk_width,
        chunk_height,
        chunk_length,
        {strategy.data(), strategy.size()},
        camera_x,
        camera_y,
        camera_z,
        &write);
    return output;
}
template<typename W>
inline void Schematic::get_chunks_with_strategy_json_write(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, std::string_view strategy, float camera_x, float camera_y, float camera_z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_get_chunks_with_strategy_json(this->AsFFI(),
        chunk_width,
        chunk_height,
        chunk_length,
        {strategy.data(), strategy.size()},
        camera_x,
        camera_y,
        camera_z,
        &write);
}

inline int32_t Schematic::block_count() const {
    auto result = diplomat::capi::Schematic_block_count(this->AsFFI());
    return result;
}

inline int32_t Schematic::volume() const {
    auto result = diplomat::capi::Schematic_volume(this->AsFFI());
    return result;
}

inline std::string Schematic::region_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_region_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::region_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_region_names_json(this->AsFFI(),
        &write);
}

inline std::string Schematic::debug_info() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_debug_info(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::debug_info_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_debug_info(this->AsFFI(),
        &write);
}

inline std::string Schematic::print_string() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_print_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::print_string_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_print_string(this->AsFFI(),
        &write);
}

inline std::string Schematic::print_schematic_string() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_print_schematic_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::print_schematic_string_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_print_schematic_string(this->AsFFI(),
        &write);
}

inline std::string Schematic::debug_string() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_debug_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::debug_string_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_debug_string(this->AsFFI(),
        &write);
}

inline std::string Schematic::debug_json_string() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_debug_json_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::debug_json_string_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_debug_json_string(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> Schematic::name() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_name(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::name_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_name(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_name(std::string_view name) {
    auto result = diplomat::capi::Schematic_set_name(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::author() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_author(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::author_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_author(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_author(std::string_view author) {
    auto result = diplomat::capi::Schematic_set_author(this->AsFFI(),
        {author.data(), author.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::description() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_description(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::description_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_description(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_description(std::string_view description) {
    auto result = diplomat::capi::Schematic_set_description(this->AsFFI(),
        {description.data(), description.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline int64_t Schematic::created() const {
    auto result = diplomat::capi::Schematic_created(this->AsFFI());
    return result;
}

inline void Schematic::set_created(uint64_t created) {
    diplomat::capi::Schematic_set_created(this->AsFFI(),
        created);
}

inline int64_t Schematic::modified() const {
    auto result = diplomat::capi::Schematic_modified(this->AsFFI());
    return result;
}

inline void Schematic::set_modified(uint64_t modified) {
    diplomat::capi::Schematic_set_modified(this->AsFFI(),
        modified);
}

inline int32_t Schematic::lm_version() const {
    auto result = diplomat::capi::Schematic_lm_version(this->AsFFI());
    return result;
}

inline void Schematic::set_lm_version(int32_t version) {
    diplomat::capi::Schematic_set_lm_version(this->AsFFI(),
        version);
}

inline int32_t Schematic::mc_version() const {
    auto result = diplomat::capi::Schematic_mc_version(this->AsFFI());
    return result;
}

inline void Schematic::set_mc_version(int32_t version) {
    diplomat::capi::Schematic_set_mc_version(this->AsFFI(),
        version);
}

inline int32_t Schematic::we_version() const {
    auto result = diplomat::capi::Schematic_we_version(this->AsFFI());
    return result;
}

inline void Schematic::set_we_version(int32_t version) {
    diplomat::capi::Schematic_set_we_version(this->AsFFI(),
        version);
}

inline void Schematic::flip_x() {
    diplomat::capi::Schematic_flip_x(this->AsFFI());
}

inline void Schematic::flip_y() {
    diplomat::capi::Schematic_flip_y(this->AsFFI());
}

inline void Schematic::flip_z() {
    diplomat::capi::Schematic_flip_z(this->AsFFI());
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_x(int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_x(this->AsFFI(),
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_y(int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_y(this->AsFFI(),
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_z(int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_z(this->AsFFI(),
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::translate(int32_t dx, int32_t dy, int32_t dz) {
    auto result = diplomat::capi::Schematic_translate(this->AsFFI(),
        dx,
        dy,
        dz);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::flip_region_x(std::string_view region_name) {
    auto result = diplomat::capi::Schematic_flip_region_x(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::flip_region_y(std::string_view region_name) {
    auto result = diplomat::capi::Schematic_flip_region_y(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::flip_region_z(std::string_view region_name) {
    auto result = diplomat::capi::Schematic_flip_region_z(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_region_x(std::string_view region_name, int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_region_x(this->AsFFI(),
        {region_name.data(), region_name.size()},
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_region_y(std::string_view region_name, int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_region_y(this->AsFFI(),
        {region_name.data(), region_name.size()},
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_region_z(std::string_view region_name, int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_region_z(this->AsFFI(),
        {region_name.data(), region_name.size()},
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::translate_region(std::string_view region_name, int32_t dx, int32_t dy, int32_t dz) {
    auto result = diplomat::capi::Schematic_translate_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        dx,
        dy,
        dz);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_schematic_x(int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_schematic_x(this->AsFFI(),
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_schematic_y(int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_schematic_y(this->AsFFI(),
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rotate_schematic_z(int32_t degrees) {
    auto result = diplomat::capi::Schematic_rotate_schematic_z(this->AsFFI(),
        degrees);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::flip_schematic_x() {
    auto result = diplomat::capi::Schematic_flip_schematic_x(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::flip_schematic_y() {
    auto result = diplomat::capi::Schematic_flip_schematic_y(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::flip_schematic_z() {
    auto result = diplomat::capi::Schematic_flip_schematic_z(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::translate_schematic(int32_t dx, int32_t dy, int32_t dz) {
    auto result = diplomat::capi::Schematic_translate_schematic(this->AsFFI(),
        dx,
        dy,
        dz);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::fill_cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, std::string_view block_name) {
    auto result = diplomat::capi::Schematic_fill_cuboid(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::fill_sphere(float cx, float cy, float cz, float radius, std::string_view block_name) {
    auto result = diplomat::capi::Schematic_fill_sphere(this->AsFFI(),
        cx,
        cy,
        cz,
        radius,
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::save_as_b64(std::string_view format, std::string_view version, std::string_view settings) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_save_as_b64(this->AsFFI(),
        {format.data(), format.size()},
        {version.data(), version.size()},
        {settings.data(), settings.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::save_as_b64_write(std::string_view format, std::string_view version, std::string_view settings, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_save_as_b64(this->AsFFI(),
        {format.data(), format.size()},
        {version.data(), version.size()},
        {settings.data(), settings.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::save_to_file_with_format(std::string_view path, std::string_view format, std::string_view version) const {
    auto result = diplomat::capi::Schematic_save_to_file_with_format(this->AsFFI(),
        {path.data(), path.size()},
        {format.data(), format.size()},
        {version.data(), version.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::to_schematic_version_b64(std::string_view version) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_to_schematic_version_b64(this->AsFFI(),
        {version.data(), version.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::to_schematic_version_b64_write(std::string_view version, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_to_schematic_version_b64(this->AsFFI(),
        {version.data(), version.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Schematic::available_schematic_versions_json() {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_available_schematic_versions_json(&write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::available_schematic_versions_json_write(W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_available_schematic_versions_json(&write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_block_with_nbt(int32_t x, int32_t y, int32_t z, std::string_view block_name, std::string_view nbt_json) {
    auto result = diplomat::capi::Schematic_set_block_with_nbt(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()},
        {nbt_json.data(), nbt_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::set_block_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z, std::string_view block_name) {
    auto result = diplomat::capi::Schematic_set_block_in_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        x,
        y,
        z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<bool, NucleationError> Schematic::has_region(std::string_view region_name) const {
    auto result = diplomat::capi::Schematic_has_region(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? diplomat::result<bool, NucleationError>(diplomat::Ok<bool>(result.ok)) : diplomat::result<bool, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::create_region(std::string_view region_name) {
    auto result = diplomat::capi::Schematic_create_region(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::remove_region(std::string_view region_name) {
    auto result = diplomat::capi::Schematic_remove_region(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Schematic::rename_region(std::string_view old_name, std::string_view new_name) {
    auto result = diplomat::capi::Schematic_rename_region(this->AsFFI(),
        {old_name.data(), old_name.size()},
        {new_name.data(), new_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Schematic::bounding_box_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_bounding_box_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::bounding_box_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_bounding_box_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> Schematic::region_bounding_box_json(std::string_view region_name) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_region_bounding_box_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::region_bounding_box_json_write(std::string_view region_name, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_region_bounding_box_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Schematic::palette_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_palette_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::palette_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_palette_json(this->AsFFI(),
        &write);
}

inline Dimensions Schematic::tight_dimensions() const {
    auto result = diplomat::capi::Schematic_tight_dimensions(this->AsFFI());
    return Dimensions::FromFFI(result);
}

inline Dimensions Schematic::allocated_dimensions() const {
    auto result = diplomat::capi::Schematic_allocated_dimensions(this->AsFFI());
    return Dimensions::FromFFI(result);
}

inline std::string Schematic::extract_signs_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_extract_signs_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::extract_signs_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_extract_signs_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> Schematic::compile_insign_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_compile_insign_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::compile_insign_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_compile_insign_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Schematic::all_palettes_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_all_palettes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::all_palettes_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_all_palettes_json(this->AsFFI(),
        &write);
}

inline std::string Schematic::default_region_palette_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Schematic_default_region_palette_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Schematic::default_region_palette_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Schematic_default_region_palette_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> Schematic::region_palette_json(std::string_view region_name) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Schematic_region_palette_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Schematic::region_palette_json_write(std::string_view region_name, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Schematic_region_palette_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<BlockPos, NucleationError> Schematic::tight_bounds_min() const {
    auto result = diplomat::capi::Schematic_tight_bounds_min(this->AsFFI());
    return result.is_ok ? diplomat::result<BlockPos, NucleationError>(diplomat::Ok<BlockPos>(BlockPos::FromFFI(result.ok))) : diplomat::result<BlockPos, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<BlockPos, NucleationError> Schematic::tight_bounds_max() const {
    auto result = diplomat::capi::Schematic_tight_bounds_max(this->AsFFI());
    return result.is_ok ? diplomat::result<BlockPos, NucleationError>(diplomat::Ok<BlockPos>(BlockPos::FromFFI(result.ok))) : diplomat::result<BlockPos, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Schematic* Schematic::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Schematic*>(this);
}

inline diplomat::capi::Schematic* Schematic::AsFFI() {
    return reinterpret_cast<diplomat::capi::Schematic*>(this);
}

inline const Schematic* Schematic::FromFFI(const diplomat::capi::Schematic* ptr) {
    return reinterpret_cast<const Schematic*>(ptr);
}

inline Schematic* Schematic::FromFFI(diplomat::capi::Schematic* ptr) {
    return reinterpret_cast<Schematic*>(ptr);
}

inline void Schematic::operator delete(void* ptr) {
    diplomat::capi::Schematic_destroy(reinterpret_cast<diplomat::capi::Schematic*>(ptr));
}


#endif // Schematic_HPP
