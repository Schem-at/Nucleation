#ifndef ResourcePack_H
#define ResourcePack_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "ResourcePackList.d.h"
#include "TextureInfo.d.h"

#include "ResourcePack.d.h"






typedef struct ResourcePack_from_bytes_result {union {ResourcePack* ok; NucleationError err;}; bool is_ok;} ResourcePack_from_bytes_result;
ResourcePack_from_bytes_result ResourcePack_from_bytes(DiplomatU8View data);

typedef struct ResourcePack_from_list_result {union {ResourcePack* ok; NucleationError err;}; bool is_ok;} ResourcePack_from_list_result;
ResourcePack_from_list_result ResourcePack_from_list(const ResourcePackList* list);

uint32_t ResourcePack_blockstate_count(const ResourcePack* self);

uint32_t ResourcePack_model_count(const ResourcePack* self);

uint32_t ResourcePack_texture_count(const ResourcePack* self);

void ResourcePack_namespaces_json(const ResourcePack* self, DiplomatWrite* write);

void ResourcePack_list_blockstates_json(const ResourcePack* self, DiplomatWrite* write);

void ResourcePack_list_models_json(const ResourcePack* self, DiplomatWrite* write);

void ResourcePack_list_textures_json(const ResourcePack* self, DiplomatWrite* write);

typedef struct ResourcePack_get_blockstate_json_result {union { NucleationError err;}; bool is_ok;} ResourcePack_get_blockstate_json_result;
ResourcePack_get_blockstate_json_result ResourcePack_get_blockstate_json(const ResourcePack* self, DiplomatStringView name, DiplomatWrite* write);

typedef struct ResourcePack_get_model_json_result {union { NucleationError err;}; bool is_ok;} ResourcePack_get_model_json_result;
ResourcePack_get_model_json_result ResourcePack_get_model_json(const ResourcePack* self, DiplomatStringView name, DiplomatWrite* write);

typedef struct ResourcePack_get_texture_info_result {union {TextureInfo ok; NucleationError err;}; bool is_ok;} ResourcePack_get_texture_info_result;
ResourcePack_get_texture_info_result ResourcePack_get_texture_info(const ResourcePack* self, DiplomatStringView name);

typedef struct ResourcePack_get_texture_pixels_b64_result {union { NucleationError err;}; bool is_ok;} ResourcePack_get_texture_pixels_b64_result;
ResourcePack_get_texture_pixels_b64_result ResourcePack_get_texture_pixels_b64(const ResourcePack* self, DiplomatStringView name, DiplomatWrite* write);

typedef struct ResourcePack_add_blockstate_json_result {union { NucleationError err;}; bool is_ok;} ResourcePack_add_blockstate_json_result;
ResourcePack_add_blockstate_json_result ResourcePack_add_blockstate_json(ResourcePack* self, DiplomatStringView name, DiplomatStringView json);

typedef struct ResourcePack_add_model_json_result {union { NucleationError err;}; bool is_ok;} ResourcePack_add_model_json_result;
ResourcePack_add_model_json_result ResourcePack_add_model_json(ResourcePack* self, DiplomatStringView name, DiplomatStringView json);

typedef struct ResourcePack_add_texture_result {union { NucleationError err;}; bool is_ok;} ResourcePack_add_texture_result;
ResourcePack_add_texture_result ResourcePack_add_texture(ResourcePack* self, DiplomatStringView name, uint32_t width, uint32_t height, DiplomatU8View pixels);

typedef struct ResourcePack_register_mesh_exporter_result {union { NucleationError err;}; bool is_ok;} ResourcePack_register_mesh_exporter_result;
ResourcePack_register_mesh_exporter_result ResourcePack_register_mesh_exporter(const ResourcePack* self);

void ResourcePack_destroy(ResourcePack* self);





#endif // ResourcePack_H
