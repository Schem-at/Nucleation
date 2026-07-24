#ifndef Schematic_H
#define Schematic_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "BlockPos.d.h"
#include "BlockState.d.h"
#include "Dimensions.d.h"
#include "NucleationError.d.h"

#include "Schematic.d.h"






Schematic* Schematic_create(DiplomatStringView name);

Schematic* Schematic_deep_clone(const Schematic* self);

Dimensions Schematic_dimensions(const Schematic* self);

typedef struct Schematic_set_block_result {union {bool ok; NucleationError err;}; bool is_ok;} Schematic_set_block_result;
Schematic_set_block_result Schematic_set_block(Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatStringView block_name);

typedef struct Schematic_get_block_name_result {union { NucleationError err;}; bool is_ok;} Schematic_get_block_name_result;
Schematic_get_block_name_result Schematic_get_block_name(const Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct Schematic_save_to_file_result {union { NucleationError err;}; bool is_ok;} Schematic_save_to_file_result;
Schematic_save_to_file_result Schematic_save_to_file(const Schematic* self, DiplomatStringView path);

typedef struct Schematic_save_result {union { NucleationError err;}; bool is_ok;} Schematic_save_result;
Schematic_save_result Schematic_save(const Schematic* self, DiplomatStringView path);

typedef struct Schematic_load_from_file_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_load_from_file_result;
Schematic_load_from_file_result Schematic_load_from_file(DiplomatStringView path);

typedef struct Schematic_open_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_open_result;
Schematic_open_result Schematic_open(DiplomatStringView path);

typedef struct Schematic_from_data_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_data_result;
Schematic_from_data_result Schematic_from_data(DiplomatU8View data);

typedef struct Schematic_from_litematic_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_litematic_result;
Schematic_from_litematic_result Schematic_from_litematic(DiplomatU8View data);

typedef struct Schematic_to_litematic_b64_result {union { NucleationError err;}; bool is_ok;} Schematic_to_litematic_b64_result;
Schematic_to_litematic_b64_result Schematic_to_litematic_b64(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_from_schematic_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_schematic_result;
Schematic_from_schematic_result Schematic_from_schematic(DiplomatU8View data);

typedef struct Schematic_to_schematic_b64_result {union { NucleationError err;}; bool is_ok;} Schematic_to_schematic_b64_result;
Schematic_to_schematic_b64_result Schematic_to_schematic_b64(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_from_snapshot_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_snapshot_result;
Schematic_from_snapshot_result Schematic_from_snapshot(DiplomatU8View data);

typedef struct Schematic_to_snapshot_b64_result {union { NucleationError err;}; bool is_ok;} Schematic_to_snapshot_b64_result;
Schematic_to_snapshot_b64_result Schematic_to_snapshot_b64(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_from_mcstructure_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_mcstructure_result;
Schematic_from_mcstructure_result Schematic_from_mcstructure(DiplomatU8View data);

typedef struct Schematic_to_mcstructure_b64_result {union { NucleationError err;}; bool is_ok;} Schematic_to_mcstructure_b64_result;
Schematic_to_mcstructure_b64_result Schematic_to_mcstructure_b64(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_from_mca_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_mca_result;
Schematic_from_mca_result Schematic_from_mca(DiplomatU8View data);

typedef struct Schematic_from_mca_bounded_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_mca_bounded_result;
Schematic_from_mca_bounded_result Schematic_from_mca_bounded(DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Schematic_from_world_zip_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_world_zip_result;
Schematic_from_world_zip_result Schematic_from_world_zip(DiplomatU8View data);

typedef struct Schematic_from_world_zip_bounded_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_world_zip_bounded_result;
Schematic_from_world_zip_bounded_result Schematic_from_world_zip_bounded(DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Schematic_from_world_directory_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_world_directory_result;
Schematic_from_world_directory_result Schematic_from_world_directory(DiplomatStringView path);

typedef struct Schematic_from_world_directory_bounded_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Schematic_from_world_directory_bounded_result;
Schematic_from_world_directory_bounded_result Schematic_from_world_directory_bounded(DiplomatStringView path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct Schematic_to_world_json_result {union { NucleationError err;}; bool is_ok;} Schematic_to_world_json_result;
Schematic_to_world_json_result Schematic_to_world_json(const Schematic* self, DiplomatStringView options_json, DiplomatWrite* write);

typedef struct Schematic_save_world_result {union { NucleationError err;}; bool is_ok;} Schematic_save_world_result;
Schematic_save_world_result Schematic_save_world(const Schematic* self, DiplomatStringView directory, DiplomatStringView options_json);

typedef struct Schematic_to_world_zip_b64_result {union { NucleationError err;}; bool is_ok;} Schematic_to_world_zip_b64_result;
Schematic_to_world_zip_b64_result Schematic_to_world_zip_b64(const Schematic* self, DiplomatStringView options_json, DiplomatWrite* write);

typedef struct Schematic_set_block_with_properties_result {union { NucleationError err;}; bool is_ok;} Schematic_set_block_with_properties_result;
Schematic_set_block_with_properties_result Schematic_set_block_with_properties(Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatStringView block_name, DiplomatStringView properties_json);

typedef struct Schematic_set_block_from_string_result {union { NucleationError err;}; bool is_ok;} Schematic_set_block_from_string_result;
Schematic_set_block_from_string_result Schematic_set_block_from_string(Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatStringView block_string);

typedef struct Schematic_prepare_block_result {union {int32_t ok; NucleationError err;}; bool is_ok;} Schematic_prepare_block_result;
Schematic_prepare_block_result Schematic_prepare_block(Schematic* self, DiplomatStringView block_name);

typedef struct Schematic_place_result {union { NucleationError err;}; bool is_ok;} Schematic_place_result;
Schematic_place_result Schematic_place(Schematic* self, int32_t x, int32_t y, int32_t z, int32_t palette_index);

typedef struct Schematic_set_blocks_result {union {int32_t ok; NucleationError err;}; bool is_ok;} Schematic_set_blocks_result;
Schematic_set_blocks_result Schematic_set_blocks(Schematic* self, DiplomatI32View positions, DiplomatStringView block_name);

typedef struct Schematic_get_blocks_json_result {union { NucleationError err;}; bool is_ok;} Schematic_get_blocks_json_result;
Schematic_get_blocks_json_result Schematic_get_blocks_json(const Schematic* self, DiplomatI32View positions, DiplomatWrite* write);

typedef struct Schematic_stamp_box_result {union { NucleationError err;}; bool is_ok;} Schematic_stamp_box_result;
Schematic_stamp_box_result Schematic_stamp_box(Schematic* self, const Schematic* source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, DiplomatStringView excluded_blocks_json);

typedef struct Schematic_stamp_region_result {union { NucleationError err;}; bool is_ok;} Schematic_stamp_region_result;
Schematic_stamp_region_result Schematic_stamp_region(Schematic* self, const Schematic* source, DiplomatStringView source_region_name, int32_t target_x, int32_t target_y, int32_t target_z, DiplomatStringView excluded_blocks_json);

typedef struct Schematic_copy_region_result {union { NucleationError err;}; bool is_ok;} Schematic_copy_region_result;
Schematic_copy_region_result Schematic_copy_region(Schematic* self, const Schematic* source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t target_x, int32_t target_y, int32_t target_z, DiplomatStringView excluded_blocks_json);

typedef struct Schematic_get_block_result {union {BlockState* ok; NucleationError err;}; bool is_ok;} Schematic_get_block_result;
Schematic_get_block_result Schematic_get_block(const Schematic* self, int32_t x, int32_t y, int32_t z);

typedef struct Schematic_get_block_with_properties_result {union {BlockState* ok; NucleationError err;}; bool is_ok;} Schematic_get_block_with_properties_result;
Schematic_get_block_with_properties_result Schematic_get_block_with_properties(const Schematic* self, int32_t x, int32_t y, int32_t z);

typedef struct Schematic_get_block_in_region_result {union {BlockState* ok; NucleationError err;}; bool is_ok;} Schematic_get_block_in_region_result;
Schematic_get_block_in_region_result Schematic_get_block_in_region(const Schematic* self, DiplomatStringView region_name, int32_t x, int32_t y, int32_t z);

typedef struct Schematic_get_block_string_in_region_result {union { NucleationError err;}; bool is_ok;} Schematic_get_block_string_in_region_result;
Schematic_get_block_string_in_region_result Schematic_get_block_string_in_region(const Schematic* self, DiplomatStringView region_name, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct Schematic_get_block_string_result {union { NucleationError err;}; bool is_ok;} Schematic_get_block_string_result;
Schematic_get_block_string_result Schematic_get_block_string(const Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct Schematic_get_block_entity_json_result {union { NucleationError err;}; bool is_ok;} Schematic_get_block_entity_json_result;
Schematic_get_block_entity_json_result Schematic_get_block_entity_json(const Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct Schematic_get_block_entity_json_in_region_result {union { NucleationError err;}; bool is_ok;} Schematic_get_block_entity_json_in_region_result;
Schematic_get_block_entity_json_in_region_result Schematic_get_block_entity_json_in_region(const Schematic* self, DiplomatStringView region_name, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

void Schematic_get_all_block_entities_json(const Schematic* self, DiplomatWrite* write);

uint32_t Schematic_entity_count(const Schematic* self);

void Schematic_get_entities_json(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_add_entity_result {union { NucleationError err;}; bool is_ok;} Schematic_add_entity_result;
Schematic_add_entity_result Schematic_add_entity(Schematic* self, DiplomatStringView id, double x, double y, double z, DiplomatStringView nbt_json);

typedef struct Schematic_add_armor_stand_result {union { NucleationError err;}; bool is_ok;} Schematic_add_armor_stand_result;
Schematic_add_armor_stand_result Schematic_add_armor_stand(Schematic* self, double x, double y, double z, float yaw, DiplomatStringView armor_material);

typedef struct Schematic_remove_entity_result {union { NucleationError err;}; bool is_ok;} Schematic_remove_entity_result;
Schematic_remove_entity_result Schematic_remove_entity(Schematic* self, uint32_t index);

int32_t Schematic_canonical_data_version(void);

void Schematic_convert_to_data_version(Schematic* self, int32_t target_data_version, int32_t source_data_version, DiplomatWrite* write);

void Schematic_convert_to_version(Schematic* self, int32_t target_data_version, DiplomatWrite* write);

int32_t Schematic_source_data_version(const Schematic* self);

void Schematic_set_source_data_version(Schematic* self, int32_t version);

typedef struct Schematic_to_litematic_for_version_json_result {union { NucleationError err;}; bool is_ok;} Schematic_to_litematic_for_version_json_result;
Schematic_to_litematic_for_version_json_result Schematic_to_litematic_for_version_json(const Schematic* self, int32_t target_data_version, DiplomatWrite* write);

typedef struct Schematic_get_block_entity_snbt_result {union { NucleationError err;}; bool is_ok;} Schematic_get_block_entity_snbt_result;
Schematic_get_block_entity_snbt_result Schematic_get_block_entity_snbt(const Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct Schematic_set_block_entity_result {union { NucleationError err;}; bool is_ok;} Schematic_set_block_entity_result;
Schematic_set_block_entity_result Schematic_set_block_entity(Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatStringView id, DiplomatStringView snbt);

typedef struct Schematic_remove_block_entity_result {union { NucleationError err;}; bool is_ok;} Schematic_remove_block_entity_result;
Schematic_remove_block_entity_result Schematic_remove_block_entity(Schematic* self, int32_t x, int32_t y, int32_t z);

void Schematic_get_all_block_entities_snbt_json(const Schematic* self, DiplomatWrite* write);

void Schematic_get_entities_snbt_json(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_add_entity_from_snbt_result {union { NucleationError err;}; bool is_ok;} Schematic_add_entity_from_snbt_result;
Schematic_add_entity_from_snbt_result Schematic_add_entity_from_snbt(Schematic* self, DiplomatStringView snbt);

void Schematic_get_all_blocks_json(const Schematic* self, DiplomatWrite* write);

void Schematic_get_chunk_blocks_json(const Schematic* self, int32_t offset_x, int32_t offset_y, int32_t offset_z, int32_t width, int32_t height, int32_t length, DiplomatWrite* write);

void Schematic_get_chunks_json(const Schematic* self, int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, DiplomatWrite* write);

void Schematic_get_chunks_with_strategy_json(const Schematic* self, int32_t chunk_width, int32_t chunk_height, int32_t chunk_length, DiplomatStringView strategy, float camera_x, float camera_y, float camera_z, DiplomatWrite* write);

int32_t Schematic_block_count(const Schematic* self);

int32_t Schematic_volume(const Schematic* self);

void Schematic_region_names_json(const Schematic* self, DiplomatWrite* write);

void Schematic_debug_info(const Schematic* self, DiplomatWrite* write);

void Schematic_print_string(const Schematic* self, DiplomatWrite* write);

void Schematic_print_schematic_string(const Schematic* self, DiplomatWrite* write);

void Schematic_debug_string(const Schematic* self, DiplomatWrite* write);

void Schematic_debug_json_string(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_name_result {union { NucleationError err;}; bool is_ok;} Schematic_name_result;
Schematic_name_result Schematic_name(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_set_name_result {union { NucleationError err;}; bool is_ok;} Schematic_set_name_result;
Schematic_set_name_result Schematic_set_name(Schematic* self, DiplomatStringView name);

typedef struct Schematic_author_result {union { NucleationError err;}; bool is_ok;} Schematic_author_result;
Schematic_author_result Schematic_author(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_set_author_result {union { NucleationError err;}; bool is_ok;} Schematic_set_author_result;
Schematic_set_author_result Schematic_set_author(Schematic* self, DiplomatStringView author);

typedef struct Schematic_description_result {union { NucleationError err;}; bool is_ok;} Schematic_description_result;
Schematic_description_result Schematic_description(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_set_description_result {union { NucleationError err;}; bool is_ok;} Schematic_set_description_result;
Schematic_set_description_result Schematic_set_description(Schematic* self, DiplomatStringView description);

int64_t Schematic_created(const Schematic* self);

void Schematic_set_created(Schematic* self, uint64_t created);

int64_t Schematic_modified(const Schematic* self);

void Schematic_set_modified(Schematic* self, uint64_t modified);

int32_t Schematic_lm_version(const Schematic* self);

void Schematic_set_lm_version(Schematic* self, int32_t version);

int32_t Schematic_mc_version(const Schematic* self);

void Schematic_set_mc_version(Schematic* self, int32_t version);

int32_t Schematic_we_version(const Schematic* self);

void Schematic_set_we_version(Schematic* self, int32_t version);

void Schematic_flip_x(Schematic* self);

void Schematic_flip_y(Schematic* self);

void Schematic_flip_z(Schematic* self);

typedef struct Schematic_rotate_x_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_x_result;
Schematic_rotate_x_result Schematic_rotate_x(Schematic* self, int32_t degrees);

typedef struct Schematic_rotate_y_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_y_result;
Schematic_rotate_y_result Schematic_rotate_y(Schematic* self, int32_t degrees);

typedef struct Schematic_rotate_z_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_z_result;
Schematic_rotate_z_result Schematic_rotate_z(Schematic* self, int32_t degrees);

typedef struct Schematic_translate_result {union { NucleationError err;}; bool is_ok;} Schematic_translate_result;
Schematic_translate_result Schematic_translate(Schematic* self, int32_t dx, int32_t dy, int32_t dz);

typedef struct Schematic_flip_region_x_result {union { NucleationError err;}; bool is_ok;} Schematic_flip_region_x_result;
Schematic_flip_region_x_result Schematic_flip_region_x(Schematic* self, DiplomatStringView region_name);

typedef struct Schematic_flip_region_y_result {union { NucleationError err;}; bool is_ok;} Schematic_flip_region_y_result;
Schematic_flip_region_y_result Schematic_flip_region_y(Schematic* self, DiplomatStringView region_name);

typedef struct Schematic_flip_region_z_result {union { NucleationError err;}; bool is_ok;} Schematic_flip_region_z_result;
Schematic_flip_region_z_result Schematic_flip_region_z(Schematic* self, DiplomatStringView region_name);

typedef struct Schematic_rotate_region_x_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_region_x_result;
Schematic_rotate_region_x_result Schematic_rotate_region_x(Schematic* self, DiplomatStringView region_name, int32_t degrees);

typedef struct Schematic_rotate_region_y_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_region_y_result;
Schematic_rotate_region_y_result Schematic_rotate_region_y(Schematic* self, DiplomatStringView region_name, int32_t degrees);

typedef struct Schematic_rotate_region_z_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_region_z_result;
Schematic_rotate_region_z_result Schematic_rotate_region_z(Schematic* self, DiplomatStringView region_name, int32_t degrees);

typedef struct Schematic_translate_region_result {union { NucleationError err;}; bool is_ok;} Schematic_translate_region_result;
Schematic_translate_region_result Schematic_translate_region(Schematic* self, DiplomatStringView region_name, int32_t dx, int32_t dy, int32_t dz);

typedef struct Schematic_rotate_schematic_x_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_schematic_x_result;
Schematic_rotate_schematic_x_result Schematic_rotate_schematic_x(Schematic* self, int32_t degrees);

typedef struct Schematic_rotate_schematic_y_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_schematic_y_result;
Schematic_rotate_schematic_y_result Schematic_rotate_schematic_y(Schematic* self, int32_t degrees);

typedef struct Schematic_rotate_schematic_z_result {union { NucleationError err;}; bool is_ok;} Schematic_rotate_schematic_z_result;
Schematic_rotate_schematic_z_result Schematic_rotate_schematic_z(Schematic* self, int32_t degrees);

typedef struct Schematic_flip_schematic_x_result {union { NucleationError err;}; bool is_ok;} Schematic_flip_schematic_x_result;
Schematic_flip_schematic_x_result Schematic_flip_schematic_x(Schematic* self);

typedef struct Schematic_flip_schematic_y_result {union { NucleationError err;}; bool is_ok;} Schematic_flip_schematic_y_result;
Schematic_flip_schematic_y_result Schematic_flip_schematic_y(Schematic* self);

typedef struct Schematic_flip_schematic_z_result {union { NucleationError err;}; bool is_ok;} Schematic_flip_schematic_z_result;
Schematic_flip_schematic_z_result Schematic_flip_schematic_z(Schematic* self);

typedef struct Schematic_translate_schematic_result {union { NucleationError err;}; bool is_ok;} Schematic_translate_schematic_result;
Schematic_translate_schematic_result Schematic_translate_schematic(Schematic* self, int32_t dx, int32_t dy, int32_t dz);

typedef struct Schematic_fill_cuboid_result {union { NucleationError err;}; bool is_ok;} Schematic_fill_cuboid_result;
Schematic_fill_cuboid_result Schematic_fill_cuboid(Schematic* self, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, DiplomatStringView block_name);

typedef struct Schematic_fill_sphere_result {union { NucleationError err;}; bool is_ok;} Schematic_fill_sphere_result;
Schematic_fill_sphere_result Schematic_fill_sphere(Schematic* self, float cx, float cy, float cz, float radius, DiplomatStringView block_name);

typedef struct Schematic_save_as_b64_result {union { NucleationError err;}; bool is_ok;} Schematic_save_as_b64_result;
Schematic_save_as_b64_result Schematic_save_as_b64(const Schematic* self, DiplomatStringView format, DiplomatStringView version, DiplomatStringView settings, DiplomatWrite* write);

typedef struct Schematic_save_to_file_with_format_result {union { NucleationError err;}; bool is_ok;} Schematic_save_to_file_with_format_result;
Schematic_save_to_file_with_format_result Schematic_save_to_file_with_format(const Schematic* self, DiplomatStringView path, DiplomatStringView format, DiplomatStringView version);

typedef struct Schematic_to_schematic_version_b64_result {union { NucleationError err;}; bool is_ok;} Schematic_to_schematic_version_b64_result;
Schematic_to_schematic_version_b64_result Schematic_to_schematic_version_b64(const Schematic* self, DiplomatStringView version, DiplomatWrite* write);

typedef struct Schematic_available_schematic_versions_json_result {union { NucleationError err;}; bool is_ok;} Schematic_available_schematic_versions_json_result;
Schematic_available_schematic_versions_json_result Schematic_available_schematic_versions_json(DiplomatWrite* write);

typedef struct Schematic_set_block_with_nbt_result {union { NucleationError err;}; bool is_ok;} Schematic_set_block_with_nbt_result;
Schematic_set_block_with_nbt_result Schematic_set_block_with_nbt(Schematic* self, int32_t x, int32_t y, int32_t z, DiplomatStringView block_name, DiplomatStringView nbt_json);

typedef struct Schematic_set_block_in_region_result {union { NucleationError err;}; bool is_ok;} Schematic_set_block_in_region_result;
Schematic_set_block_in_region_result Schematic_set_block_in_region(Schematic* self, DiplomatStringView region_name, int32_t x, int32_t y, int32_t z, DiplomatStringView block_name);

typedef struct Schematic_has_region_result {union {bool ok; NucleationError err;}; bool is_ok;} Schematic_has_region_result;
Schematic_has_region_result Schematic_has_region(const Schematic* self, DiplomatStringView region_name);

typedef struct Schematic_create_region_result {union { NucleationError err;}; bool is_ok;} Schematic_create_region_result;
Schematic_create_region_result Schematic_create_region(Schematic* self, DiplomatStringView region_name);

typedef struct Schematic_remove_region_result {union { NucleationError err;}; bool is_ok;} Schematic_remove_region_result;
Schematic_remove_region_result Schematic_remove_region(Schematic* self, DiplomatStringView region_name);

typedef struct Schematic_rename_region_result {union { NucleationError err;}; bool is_ok;} Schematic_rename_region_result;
Schematic_rename_region_result Schematic_rename_region(Schematic* self, DiplomatStringView old_name, DiplomatStringView new_name);

void Schematic_bounding_box_json(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_region_bounding_box_json_result {union { NucleationError err;}; bool is_ok;} Schematic_region_bounding_box_json_result;
Schematic_region_bounding_box_json_result Schematic_region_bounding_box_json(const Schematic* self, DiplomatStringView region_name, DiplomatWrite* write);

void Schematic_palette_json(const Schematic* self, DiplomatWrite* write);

Dimensions Schematic_tight_dimensions(const Schematic* self);

Dimensions Schematic_allocated_dimensions(const Schematic* self);

void Schematic_extract_signs_json(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_compile_insign_json_result {union { NucleationError err;}; bool is_ok;} Schematic_compile_insign_json_result;
Schematic_compile_insign_json_result Schematic_compile_insign_json(const Schematic* self, DiplomatWrite* write);

void Schematic_all_palettes_json(const Schematic* self, DiplomatWrite* write);

void Schematic_default_region_palette_json(const Schematic* self, DiplomatWrite* write);

typedef struct Schematic_region_palette_json_result {union { NucleationError err;}; bool is_ok;} Schematic_region_palette_json_result;
Schematic_region_palette_json_result Schematic_region_palette_json(const Schematic* self, DiplomatStringView region_name, DiplomatWrite* write);

typedef struct Schematic_tight_bounds_min_result {union {BlockPos ok; NucleationError err;}; bool is_ok;} Schematic_tight_bounds_min_result;
Schematic_tight_bounds_min_result Schematic_tight_bounds_min(const Schematic* self);

typedef struct Schematic_tight_bounds_max_result {union {BlockPos ok; NucleationError err;}; bool is_ok;} Schematic_tight_bounds_max_result;
Schematic_tight_bounds_max_result Schematic_tight_bounds_max(const Schematic* self);

void Schematic_destroy(Schematic* self);





#endif // Schematic_H
