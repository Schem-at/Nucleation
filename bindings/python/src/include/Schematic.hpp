#ifndef NUCLEATION_Schematic_HPP
#define NUCLEATION_Schematic_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::Schematic* Schematic_create(nucleation::diplomat::capi::DiplomatStringView name);

    nucleation::capi::Dimensions Schematic_dimensions(const nucleation::capi::Schematic* self);

    typedef struct Schematic_set_block_result {union {bool ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_result;
    Schematic_set_block_result Schematic_set_block(nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_get_block_name_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_name_result;
    Schematic_get_block_name_result Schematic_get_block_name(const nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_save_to_file_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_save_to_file_result;
    Schematic_save_to_file_result Schematic_save_to_file(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_load_from_file_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_load_from_file_result;
    Schematic_load_from_file_result Schematic_load_from_file(nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_from_data_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_data_result;
    Schematic_from_data_result Schematic_from_data(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_from_litematic_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_litematic_result;
    Schematic_from_litematic_result Schematic_from_litematic(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_litematic_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_litematic_b64_result;
    Schematic_to_litematic_b64_result Schematic_to_litematic_b64(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_schematic_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_schematic_result;
    Schematic_from_schematic_result Schematic_from_schematic(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_schematic_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_schematic_b64_result;
    Schematic_to_schematic_b64_result Schematic_to_schematic_b64(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_snapshot_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_snapshot_result;
    Schematic_from_snapshot_result Schematic_from_snapshot(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_snapshot_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_snapshot_b64_result;
    Schematic_to_snapshot_b64_result Schematic_to_snapshot_b64(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_mcstructure_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_mcstructure_result;
    Schematic_from_mcstructure_result Schematic_from_mcstructure(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_to_mcstructure_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_mcstructure_b64_result;
    Schematic_to_mcstructure_b64_result Schematic_to_mcstructure_b64(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_from_mca_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_mca_result;
    Schematic_from_mca_result Schematic_from_mca(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_from_mca_bounded_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_mca_bounded_result;
    Schematic_from_mca_bounded_result Schematic_from_mca_bounded(nucleation::diplomat::capi::DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Schematic_from_world_zip_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_zip_result;
    Schematic_from_world_zip_result Schematic_from_world_zip(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Schematic_from_world_zip_bounded_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_zip_bounded_result;
    Schematic_from_world_zip_bounded_result Schematic_from_world_zip_bounded(nucleation::diplomat::capi::DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Schematic_from_world_directory_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_directory_result;
    Schematic_from_world_directory_result Schematic_from_world_directory(nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Schematic_from_world_directory_bounded_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_from_world_directory_bounded_result;
    Schematic_from_world_directory_bounded_result Schematic_from_world_directory_bounded(nucleation::diplomat::capi::DiplomatStringView path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Schematic_to_world_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_world_json_result;
    Schematic_to_world_json_result Schematic_to_world_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView options_json, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_save_world_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_save_world_result;
    Schematic_save_world_result Schematic_save_world(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView directory, nucleation::diplomat::capi::DiplomatStringView options_json);

    typedef struct Schematic_to_world_zip_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_world_zip_b64_result;
    Schematic_to_world_zip_b64_result Schematic_to_world_zip_b64(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView options_json, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_block_with_properties_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_with_properties_result;
    Schematic_set_block_with_properties_result Schematic_set_block_with_properties(nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block_name, nucleation::diplomat::capi::DiplomatStringView properties_json);

    typedef struct Schematic_set_block_from_string_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_from_string_result;
    Schematic_set_block_from_string_result Schematic_set_block_from_string(nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block_string);

    typedef struct Schematic_prepare_block_result {union {int32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_prepare_block_result;
    Schematic_prepare_block_result Schematic_prepare_block(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_place_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_place_result;
    Schematic_place_result Schematic_place(nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, int32_t palette_index);

    typedef struct Schematic_set_blocks_result {union {int32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_blocks_result;
    Schematic_set_blocks_result Schematic_set_blocks(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatI32View positions, nucleation::diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_get_blocks_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_get_blocks_json_result;
    Schematic_get_blocks_json_result Schematic_get_blocks_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatI32View positions, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_copy_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_copy_region_result;
    Schematic_copy_region_result Schematic_copy_region(nucleation::capi::Schematic* self, const nucleation::capi::Schematic* source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, nucleation::diplomat::capi::DiplomatStringView excluded_blocks_json);

    typedef struct Schematic_get_block_result {union {nucleation::capi::BlockState* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_result;
    Schematic_get_block_result Schematic_get_block(const nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z);

    typedef struct Schematic_get_block_with_properties_result {union {nucleation::capi::BlockState* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_with_properties_result;
    Schematic_get_block_with_properties_result Schematic_get_block_with_properties(const nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z);

    typedef struct Schematic_get_block_string_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_string_result;
    Schematic_get_block_string_result Schematic_get_block_string(const nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_get_block_entity_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_entity_json_result;
    Schematic_get_block_entity_json_result Schematic_get_block_entity_json(const nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_get_all_block_entities_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t Schematic_entity_count(const nucleation::capi::Schematic* self);

    void Schematic_get_entities_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_add_entity_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_add_entity_result;
    Schematic_add_entity_result Schematic_add_entity(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView id, double x, double y, double z, nucleation::diplomat::capi::DiplomatStringView nbt_json);

    typedef struct Schematic_add_armor_stand_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_add_armor_stand_result;
    Schematic_add_armor_stand_result Schematic_add_armor_stand(nucleation::capi::Schematic* self, double x, double y, double z, float yaw, nucleation::diplomat::capi::DiplomatStringView armor_material);

    typedef struct Schematic_remove_entity_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_remove_entity_result;
    Schematic_remove_entity_result Schematic_remove_entity(nucleation::capi::Schematic* self, uint32_t index);

    int32_t Schematic_canonical_data_version(void);

    void Schematic_convert_to_data_version(nucleation::capi::Schematic* self, int32_t target_data_version, int32_t source_data_version, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_convert_to_version(nucleation::capi::Schematic* self, int32_t target_data_version, nucleation::diplomat::capi::DiplomatWrite* write);

    int32_t Schematic_source_data_version(const nucleation::capi::Schematic* self);

    void Schematic_set_source_data_version(nucleation::capi::Schematic* self, int32_t version);

    typedef struct Schematic_to_litematic_for_version_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_litematic_for_version_json_result;
    Schematic_to_litematic_for_version_json_result Schematic_to_litematic_for_version_json(const nucleation::capi::Schematic* self, int32_t target_data_version, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_get_block_entity_snbt_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_get_block_entity_snbt_result;
    Schematic_get_block_entity_snbt_result Schematic_get_block_entity_snbt(const nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_block_entity_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_entity_result;
    Schematic_set_block_entity_result Schematic_set_block_entity(nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView id, nucleation::diplomat::capi::DiplomatStringView snbt);

    typedef struct Schematic_remove_block_entity_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_remove_block_entity_result;
    Schematic_remove_block_entity_result Schematic_remove_block_entity(nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z);

    void Schematic_get_all_block_entities_snbt_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_get_entities_snbt_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_add_entity_from_snbt_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_add_entity_from_snbt_result;
    Schematic_add_entity_from_snbt_result Schematic_add_entity_from_snbt(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView snbt);

    void Schematic_get_all_blocks_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_get_chunk_blocks_json(const nucleation::capi::Schematic* self, int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_get_chunks_json(const nucleation::capi::Schematic* self, int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_get_chunks_with_strategy_json(const nucleation::capi::Schematic* self, int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, nucleation::diplomat::capi::DiplomatStringView strategy, float camera_x, float camera_y, float camera_z, nucleation::diplomat::capi::DiplomatWrite* write);

    int32_t Schematic_block_count(const nucleation::capi::Schematic* self);

    int32_t Schematic_volume(const nucleation::capi::Schematic* self);

    void Schematic_region_names_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_debug_info(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_print_string(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_print_schematic_string(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_debug_string(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_debug_json_string(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_name_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_name_result;
    Schematic_name_result Schematic_name(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_name_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_name_result;
    Schematic_set_name_result Schematic_set_name(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct Schematic_author_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_author_result;
    Schematic_author_result Schematic_author(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_author_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_author_result;
    Schematic_set_author_result Schematic_set_author(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView author);

    typedef struct Schematic_description_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_description_result;
    Schematic_description_result Schematic_description(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_description_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_description_result;
    Schematic_set_description_result Schematic_set_description(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView description);

    int64_t Schematic_created(const nucleation::capi::Schematic* self);

    void Schematic_set_created(nucleation::capi::Schematic* self, uint64_t created);

    int64_t Schematic_modified(const nucleation::capi::Schematic* self);

    void Schematic_set_modified(nucleation::capi::Schematic* self, uint64_t modified);

    int32_t Schematic_lm_version(const nucleation::capi::Schematic* self);

    void Schematic_set_lm_version(nucleation::capi::Schematic* self, int32_t version);

    int32_t Schematic_mc_version(const nucleation::capi::Schematic* self);

    void Schematic_set_mc_version(nucleation::capi::Schematic* self, int32_t version);

    int32_t Schematic_we_version(const nucleation::capi::Schematic* self);

    void Schematic_set_we_version(nucleation::capi::Schematic* self, int32_t version);

    void Schematic_flip_x(nucleation::capi::Schematic* self);

    void Schematic_flip_y(nucleation::capi::Schematic* self);

    void Schematic_flip_z(nucleation::capi::Schematic* self);

    void Schematic_rotate_x(nucleation::capi::Schematic* self, int32_t degrees);

    void Schematic_rotate_y(nucleation::capi::Schematic* self, int32_t degrees);

    void Schematic_rotate_z(nucleation::capi::Schematic* self, int32_t degrees);

    typedef struct Schematic_flip_region_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_flip_region_x_result;
    Schematic_flip_region_x_result Schematic_flip_region_x(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_flip_region_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_flip_region_y_result;
    Schematic_flip_region_y_result Schematic_flip_region_y(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_flip_region_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_flip_region_z_result;
    Schematic_flip_region_z_result Schematic_flip_region_z(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name);

    typedef struct Schematic_rotate_region_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_region_x_result;
    Schematic_rotate_region_x_result Schematic_rotate_region_x(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name, int32_t degrees);

    typedef struct Schematic_rotate_region_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_region_y_result;
    Schematic_rotate_region_y_result Schematic_rotate_region_y(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name, int32_t degrees);

    typedef struct Schematic_rotate_region_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_rotate_region_z_result;
    Schematic_rotate_region_z_result Schematic_rotate_region_z(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name, int32_t degrees);

    typedef struct Schematic_fill_cuboid_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_fill_cuboid_result;
    Schematic_fill_cuboid_result Schematic_fill_cuboid(nucleation::capi::Schematic* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, nucleation::diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_fill_sphere_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_fill_sphere_result;
    Schematic_fill_sphere_result Schematic_fill_sphere(nucleation::capi::Schematic* self, float cx, float cy, float cz, float radius, nucleation::diplomat::capi::DiplomatStringView block_name);

    typedef struct Schematic_save_as_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_save_as_b64_result;
    Schematic_save_as_b64_result Schematic_save_as_b64(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView format, nucleation::diplomat::capi::DiplomatStringView version, nucleation::diplomat::capi::DiplomatStringView settings, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_save_to_file_with_format_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_save_to_file_with_format_result;
    Schematic_save_to_file_with_format_result Schematic_save_to_file_with_format(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView path, nucleation::diplomat::capi::DiplomatStringView format, nucleation::diplomat::capi::DiplomatStringView version);

    typedef struct Schematic_to_schematic_version_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_to_schematic_version_b64_result;
    Schematic_to_schematic_version_b64_result Schematic_to_schematic_version_b64(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView version, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_available_schematic_versions_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_available_schematic_versions_json_result;
    Schematic_available_schematic_versions_json_result Schematic_available_schematic_versions_json(nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_set_block_with_nbt_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_with_nbt_result;
    Schematic_set_block_with_nbt_result Schematic_set_block_with_nbt(nucleation::capi::Schematic* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block_name, nucleation::diplomat::capi::DiplomatStringView nbt_json);

    typedef struct Schematic_set_block_in_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_set_block_in_region_result;
    Schematic_set_block_in_region_result Schematic_set_block_in_region(nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block_name);

    void Schematic_bounding_box_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_region_bounding_box_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_region_bounding_box_json_result;
    Schematic_region_bounding_box_json_result Schematic_region_bounding_box_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_palette_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    nucleation::capi::Dimensions Schematic_tight_dimensions(const nucleation::capi::Schematic* self);

    nucleation::capi::Dimensions Schematic_allocated_dimensions(const nucleation::capi::Schematic* self);

    void Schematic_extract_signs_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_compile_insign_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_compile_insign_json_result;
    Schematic_compile_insign_json_result Schematic_compile_insign_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_all_palettes_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Schematic_default_region_palette_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_region_palette_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_region_palette_json_result;
    Schematic_region_palette_json_result Schematic_region_palette_json(const nucleation::capi::Schematic* self, nucleation::diplomat::capi::DiplomatStringView region_name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Schematic_tight_bounds_min_result {union {nucleation::capi::BlockPos ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_tight_bounds_min_result;
    Schematic_tight_bounds_min_result Schematic_tight_bounds_min(const nucleation::capi::Schematic* self);

    typedef struct Schematic_tight_bounds_max_result {union {nucleation::capi::BlockPos ok; nucleation::capi::NucleationError err;}; bool is_ok;} Schematic_tight_bounds_max_result;
    Schematic_tight_bounds_max_result Schematic_tight_bounds_max(const nucleation::capi::Schematic* self);

    void Schematic_destroy(Schematic* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::Schematic> nucleation::Schematic::create(std::string_view name) {
    auto result = nucleation::capi::Schematic_create({name.data(), name.size()});
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline nucleation::Dimensions nucleation::Schematic::dimensions() const {
    auto result = nucleation::capi::Schematic_dimensions(this->AsFFI());
    return nucleation::Dimensions::FromFFI(result);
}

inline nucleation::diplomat::result<bool, nucleation::NucleationError> nucleation::Schematic::set_block(int32_t x, int32_t y, int32_t z, std::string_view block_name) {
    auto result = nucleation::capi::Schematic_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Ok<bool>(result.ok)) : nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::get_block_name(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_get_block_name(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::get_block_name_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_get_block_name(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::save_to_file(std::string_view path) const {
    auto result = nucleation::capi::Schematic_save_to_file(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::load_from_file(std::string_view path) {
    auto result = nucleation::capi::Schematic_load_from_file({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_data(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Schematic_from_data({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_litematic(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Schematic_from_litematic({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_litematic_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_litematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_litematic_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_litematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_schematic(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Schematic_from_schematic({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_schematic_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_schematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_schematic_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_schematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_snapshot(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Schematic_from_snapshot({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_snapshot_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_snapshot_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_snapshot_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_snapshot_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_mcstructure(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Schematic_from_mcstructure({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_mcstructure_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_mcstructure_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_mcstructure_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_mcstructure_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_mca(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Schematic_from_mca({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_mca_bounded(nucleation::diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::Schematic_from_mca_bounded({data.data(), data.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_world_zip(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Schematic_from_world_zip({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_world_zip_bounded(nucleation::diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::Schematic_from_world_zip_bounded({data.data(), data.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_world_directory(std::string_view path) {
    auto result = nucleation::capi::Schematic_from_world_directory({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Schematic::from_world_directory_bounded(std::string_view path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::Schematic_from_world_directory_bounded({path.data(), path.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_world_json(std::string_view options_json) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_world_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_world_json_write(std::string_view options_json, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_world_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::save_world(std::string_view directory, std::string_view options_json) const {
    auto result = nucleation::capi::Schematic_save_world(this->AsFFI(),
        {directory.data(), directory.size()},
        {options_json.data(), options_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_world_zip_b64(std::string_view options_json) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_world_zip_b64(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_world_zip_b64_write(std::string_view options_json, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_world_zip_b64(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_block_with_properties(int32_t x, int32_t y, int32_t z, std::string_view block_name, std::string_view properties_json) {
    auto result = nucleation::capi::Schematic_set_block_with_properties(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()},
        {properties_json.data(), properties_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_block_from_string(int32_t x, int32_t y, int32_t z, std::string_view block_string) {
    auto result = nucleation::capi::Schematic_set_block_from_string(this->AsFFI(),
        x,
        y,
        z,
        {block_string.data(), block_string.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> nucleation::Schematic::prepare_block(std::string_view block_name) {
    auto result = nucleation::capi::Schematic_prepare_block(this->AsFFI(),
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<int32_t>(result.ok)) : nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::place(int32_t x, int32_t y, int32_t z, int32_t palette_index) {
    auto result = nucleation::capi::Schematic_place(this->AsFFI(),
        x,
        y,
        z,
        palette_index);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> nucleation::Schematic::set_blocks(nucleation::diplomat::span<const int32_t> positions, std::string_view block_name) {
    auto result = nucleation::capi::Schematic_set_blocks(this->AsFFI(),
        {positions.data(), positions.size()},
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<int32_t>(result.ok)) : nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::get_blocks_json(nucleation::diplomat::span<const int32_t> positions) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_get_blocks_json(this->AsFFI(),
        {positions.data(), positions.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::get_blocks_json_write(nucleation::diplomat::span<const int32_t> positions, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_get_blocks_json(this->AsFFI(),
        {positions.data(), positions.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::copy_region(const nucleation::Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, std::string_view excluded_blocks_json) {
    auto result = nucleation::capi::Schematic_copy_region(this->AsFFI(),
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
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError> nucleation::Schematic::get_block(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::Schematic_get_block(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::BlockState>>(std::unique_ptr<nucleation::BlockState>(nucleation::BlockState::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError> nucleation::Schematic::get_block_with_properties(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::Schematic_get_block_with_properties(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::BlockState>>(std::unique_ptr<nucleation::BlockState>(nucleation::BlockState::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::get_block_string(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_get_block_string(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::get_block_string_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_get_block_string(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::get_block_entity_json(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_get_block_entity_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::get_block_entity_json_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_get_block_entity_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Schematic::get_all_block_entities_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_all_block_entities_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::get_all_block_entities_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_all_block_entities_json(this->AsFFI(),
        &write);
}

inline uint32_t nucleation::Schematic::entity_count() const {
    auto result = nucleation::capi::Schematic_entity_count(this->AsFFI());
    return result;
}

inline std::string nucleation::Schematic::get_entities_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_entities_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::get_entities_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_entities_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::add_entity(std::string_view id, double x, double y, double z, std::string_view nbt_json) {
    auto result = nucleation::capi::Schematic_add_entity(this->AsFFI(),
        {id.data(), id.size()},
        x,
        y,
        z,
        {nbt_json.data(), nbt_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material) {
    auto result = nucleation::capi::Schematic_add_armor_stand(this->AsFFI(),
        x,
        y,
        z,
        yaw,
        {armor_material.data(), armor_material.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::remove_entity(uint32_t index) {
    auto result = nucleation::capi::Schematic_remove_entity(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline int32_t nucleation::Schematic::canonical_data_version() {
    auto result = nucleation::capi::Schematic_canonical_data_version();
    return result;
}

inline std::string nucleation::Schematic::convert_to_data_version(int32_t target_data_version, int32_t source_data_version) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_convert_to_data_version(this->AsFFI(),
        target_data_version,
        source_data_version,
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::convert_to_data_version_write(int32_t target_data_version, int32_t source_data_version, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_convert_to_data_version(this->AsFFI(),
        target_data_version,
        source_data_version,
        &write);
}

inline std::string nucleation::Schematic::convert_to_version(int32_t target_data_version) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_convert_to_version(this->AsFFI(),
        target_data_version,
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::convert_to_version_write(int32_t target_data_version, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_convert_to_version(this->AsFFI(),
        target_data_version,
        &write);
}

inline int32_t nucleation::Schematic::source_data_version() const {
    auto result = nucleation::capi::Schematic_source_data_version(this->AsFFI());
    return result;
}

inline void nucleation::Schematic::set_source_data_version(int32_t version) {
    nucleation::capi::Schematic_set_source_data_version(this->AsFFI(),
        version);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_litematic_for_version_json(int32_t target_data_version) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_litematic_for_version_json(this->AsFFI(),
        target_data_version,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_litematic_for_version_json_write(int32_t target_data_version, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_litematic_for_version_json(this->AsFFI(),
        target_data_version,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::get_block_entity_snbt(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_get_block_entity_snbt(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::get_block_entity_snbt_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_get_block_entity_snbt(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_block_entity(int32_t x, int32_t y, int32_t z, std::string_view id, std::string_view snbt) {
    auto result = nucleation::capi::Schematic_set_block_entity(this->AsFFI(),
        x,
        y,
        z,
        {id.data(), id.size()},
        {snbt.data(), snbt.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::remove_block_entity(int32_t x, int32_t y, int32_t z) {
    auto result = nucleation::capi::Schematic_remove_block_entity(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Schematic::get_all_block_entities_snbt_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_all_block_entities_snbt_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::get_all_block_entities_snbt_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_all_block_entities_snbt_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::get_entities_snbt_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_entities_snbt_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::get_entities_snbt_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_entities_snbt_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::add_entity_from_snbt(std::string_view snbt) {
    auto result = nucleation::capi::Schematic_add_entity_from_snbt(this->AsFFI(),
        {snbt.data(), snbt.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Schematic::get_all_blocks_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_all_blocks_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::get_all_blocks_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_all_blocks_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::get_chunk_blocks_json(int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_chunk_blocks_json(this->AsFFI(),
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
inline void nucleation::Schematic::get_chunk_blocks_json_write(int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_chunk_blocks_json(this->AsFFI(),
        offset_x,
        offset_y,
        offset_z,
        width,
        height,
        length,
        &write);
}

inline std::string nucleation::Schematic::get_chunks_json(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_chunks_json(this->AsFFI(),
        chunk_width,
        chunk_height,
        chunk_length,
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::get_chunks_json_write(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_chunks_json(this->AsFFI(),
        chunk_width,
        chunk_height,
        chunk_length,
        &write);
}

inline std::string nucleation::Schematic::get_chunks_with_strategy_json(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, std::string_view strategy, float camera_x, float camera_y, float camera_z) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_get_chunks_with_strategy_json(this->AsFFI(),
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
inline void nucleation::Schematic::get_chunks_with_strategy_json_write(int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, std::string_view strategy, float camera_x, float camera_y, float camera_z, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_get_chunks_with_strategy_json(this->AsFFI(),
        chunk_width,
        chunk_height,
        chunk_length,
        {strategy.data(), strategy.size()},
        camera_x,
        camera_y,
        camera_z,
        &write);
}

inline int32_t nucleation::Schematic::block_count() const {
    auto result = nucleation::capi::Schematic_block_count(this->AsFFI());
    return result;
}

inline int32_t nucleation::Schematic::volume() const {
    auto result = nucleation::capi::Schematic_volume(this->AsFFI());
    return result;
}

inline std::string nucleation::Schematic::region_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_region_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::region_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_region_names_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::debug_info() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_debug_info(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::debug_info_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_debug_info(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::print_string() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_print_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::print_string_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_print_string(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::print_schematic_string() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_print_schematic_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::print_schematic_string_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_print_schematic_string(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::debug_string() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_debug_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::debug_string_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_debug_string(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::debug_json_string() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_debug_json_string(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::debug_json_string_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_debug_json_string(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::name() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_name(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::name_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_name(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_name(std::string_view name) {
    auto result = nucleation::capi::Schematic_set_name(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::author() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_author(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::author_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_author(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_author(std::string_view author) {
    auto result = nucleation::capi::Schematic_set_author(this->AsFFI(),
        {author.data(), author.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::description() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_description(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::description_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_description(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_description(std::string_view description) {
    auto result = nucleation::capi::Schematic_set_description(this->AsFFI(),
        {description.data(), description.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline int64_t nucleation::Schematic::created() const {
    auto result = nucleation::capi::Schematic_created(this->AsFFI());
    return result;
}

inline void nucleation::Schematic::set_created(uint64_t created) {
    nucleation::capi::Schematic_set_created(this->AsFFI(),
        created);
}

inline int64_t nucleation::Schematic::modified() const {
    auto result = nucleation::capi::Schematic_modified(this->AsFFI());
    return result;
}

inline void nucleation::Schematic::set_modified(uint64_t modified) {
    nucleation::capi::Schematic_set_modified(this->AsFFI(),
        modified);
}

inline int32_t nucleation::Schematic::lm_version() const {
    auto result = nucleation::capi::Schematic_lm_version(this->AsFFI());
    return result;
}

inline void nucleation::Schematic::set_lm_version(int32_t version) {
    nucleation::capi::Schematic_set_lm_version(this->AsFFI(),
        version);
}

inline int32_t nucleation::Schematic::mc_version() const {
    auto result = nucleation::capi::Schematic_mc_version(this->AsFFI());
    return result;
}

inline void nucleation::Schematic::set_mc_version(int32_t version) {
    nucleation::capi::Schematic_set_mc_version(this->AsFFI(),
        version);
}

inline int32_t nucleation::Schematic::we_version() const {
    auto result = nucleation::capi::Schematic_we_version(this->AsFFI());
    return result;
}

inline void nucleation::Schematic::set_we_version(int32_t version) {
    nucleation::capi::Schematic_set_we_version(this->AsFFI(),
        version);
}

inline void nucleation::Schematic::flip_x() {
    nucleation::capi::Schematic_flip_x(this->AsFFI());
}

inline void nucleation::Schematic::flip_y() {
    nucleation::capi::Schematic_flip_y(this->AsFFI());
}

inline void nucleation::Schematic::flip_z() {
    nucleation::capi::Schematic_flip_z(this->AsFFI());
}

inline void nucleation::Schematic::rotate_x(int32_t degrees) {
    nucleation::capi::Schematic_rotate_x(this->AsFFI(),
        degrees);
}

inline void nucleation::Schematic::rotate_y(int32_t degrees) {
    nucleation::capi::Schematic_rotate_y(this->AsFFI(),
        degrees);
}

inline void nucleation::Schematic::rotate_z(int32_t degrees) {
    nucleation::capi::Schematic_rotate_z(this->AsFFI(),
        degrees);
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::flip_region_x(std::string_view region_name) {
    auto result = nucleation::capi::Schematic_flip_region_x(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::flip_region_y(std::string_view region_name) {
    auto result = nucleation::capi::Schematic_flip_region_y(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::flip_region_z(std::string_view region_name) {
    auto result = nucleation::capi::Schematic_flip_region_z(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::rotate_region_x(std::string_view region_name, int32_t degrees) {
    auto result = nucleation::capi::Schematic_rotate_region_x(this->AsFFI(),
        {region_name.data(), region_name.size()},
        degrees);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::rotate_region_y(std::string_view region_name, int32_t degrees) {
    auto result = nucleation::capi::Schematic_rotate_region_y(this->AsFFI(),
        {region_name.data(), region_name.size()},
        degrees);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::rotate_region_z(std::string_view region_name, int32_t degrees) {
    auto result = nucleation::capi::Schematic_rotate_region_z(this->AsFFI(),
        {region_name.data(), region_name.size()},
        degrees);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::fill_cuboid(int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, std::string_view block_name) {
    auto result = nucleation::capi::Schematic_fill_cuboid(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::fill_sphere(float cx, float cy, float cz, float radius, std::string_view block_name) {
    auto result = nucleation::capi::Schematic_fill_sphere(this->AsFFI(),
        cx,
        cy,
        cz,
        radius,
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::save_as_b64(std::string_view format, std::string_view version, std::string_view settings) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_save_as_b64(this->AsFFI(),
        {format.data(), format.size()},
        {version.data(), version.size()},
        {settings.data(), settings.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::save_as_b64_write(std::string_view format, std::string_view version, std::string_view settings, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_save_as_b64(this->AsFFI(),
        {format.data(), format.size()},
        {version.data(), version.size()},
        {settings.data(), settings.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::save_to_file_with_format(std::string_view path, std::string_view format, std::string_view version) const {
    auto result = nucleation::capi::Schematic_save_to_file_with_format(this->AsFFI(),
        {path.data(), path.size()},
        {format.data(), format.size()},
        {version.data(), version.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::to_schematic_version_b64(std::string_view version) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_to_schematic_version_b64(this->AsFFI(),
        {version.data(), version.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::to_schematic_version_b64_write(std::string_view version, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_to_schematic_version_b64(this->AsFFI(),
        {version.data(), version.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::available_schematic_versions_json() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_available_schematic_versions_json(&write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::available_schematic_versions_json_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_available_schematic_versions_json(&write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_block_with_nbt(int32_t x, int32_t y, int32_t z, std::string_view block_name, std::string_view nbt_json) {
    auto result = nucleation::capi::Schematic_set_block_with_nbt(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()},
        {nbt_json.data(), nbt_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::set_block_in_region(std::string_view region_name, int32_t x, int32_t y, int32_t z, std::string_view block_name) {
    auto result = nucleation::capi::Schematic_set_block_in_region(this->AsFFI(),
        {region_name.data(), region_name.size()},
        x,
        y,
        z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Schematic::bounding_box_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_bounding_box_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::bounding_box_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_bounding_box_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::region_bounding_box_json(std::string_view region_name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_region_bounding_box_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::region_bounding_box_json_write(std::string_view region_name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_region_bounding_box_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Schematic::palette_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_palette_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::palette_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_palette_json(this->AsFFI(),
        &write);
}

inline nucleation::Dimensions nucleation::Schematic::tight_dimensions() const {
    auto result = nucleation::capi::Schematic_tight_dimensions(this->AsFFI());
    return nucleation::Dimensions::FromFFI(result);
}

inline nucleation::Dimensions nucleation::Schematic::allocated_dimensions() const {
    auto result = nucleation::capi::Schematic_allocated_dimensions(this->AsFFI());
    return nucleation::Dimensions::FromFFI(result);
}

inline std::string nucleation::Schematic::extract_signs_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_extract_signs_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::extract_signs_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_extract_signs_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::compile_insign_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_compile_insign_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::compile_insign_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_compile_insign_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Schematic::all_palettes_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_all_palettes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::all_palettes_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_all_palettes_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::Schematic::default_region_palette_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Schematic_default_region_palette_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Schematic::default_region_palette_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Schematic_default_region_palette_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Schematic::region_palette_json(std::string_view region_name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Schematic_region_palette_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Schematic::region_palette_json_write(std::string_view region_name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Schematic_region_palette_json(this->AsFFI(),
        {region_name.data(), region_name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> nucleation::Schematic::tight_bounds_min() const {
    auto result = nucleation::capi::Schematic_tight_bounds_min(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::BlockPos>(nucleation::BlockPos::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> nucleation::Schematic::tight_bounds_max() const {
    auto result = nucleation::capi::Schematic_tight_bounds_max(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::BlockPos>(nucleation::BlockPos::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Schematic* nucleation::Schematic::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Schematic*>(this);
}

inline nucleation::capi::Schematic* nucleation::Schematic::AsFFI() {
    return reinterpret_cast<nucleation::capi::Schematic*>(this);
}

inline const nucleation::Schematic* nucleation::Schematic::FromFFI(const nucleation::capi::Schematic* ptr) {
    return reinterpret_cast<const nucleation::Schematic*>(ptr);
}

inline nucleation::Schematic* nucleation::Schematic::FromFFI(nucleation::capi::Schematic* ptr) {
    return reinterpret_cast<nucleation::Schematic*>(ptr);
}

inline void nucleation::Schematic::operator delete(void* ptr) {
    nucleation::capi::Schematic_destroy(reinterpret_cast<nucleation::capi::Schematic*>(ptr));
}


#endif // NUCLEATION_Schematic_HPP
