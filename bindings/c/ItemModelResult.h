#ifndef ItemModelResult_H
#define ItemModelResult_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Dimensions.d.h"
#include "ItemModelConfig.d.h"
#include "ItemModelPackBuilder.d.h"
#include "ItemScale.d.h"
#include "NucleationError.d.h"
#include "ResourcePack.d.h"
#include "Schematic.d.h"

#include "ItemModelResult.d.h"






typedef struct ItemModelResult_create_result {union {ItemModelResult* ok; NucleationError err;}; bool is_ok;} ItemModelResult_create_result;
ItemModelResult_create_result ItemModelResult_create(const Schematic* schematic, const ResourcePack* pack, const ItemModelConfig* config);

typedef struct ItemModelResult_model_json_result {union { NucleationError err;}; bool is_ok;} ItemModelResult_model_json_result;
ItemModelResult_model_json_result ItemModelResult_model_json(const ItemModelResult* self, DiplomatWrite* write);

typedef struct ItemModelResult_element_count_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} ItemModelResult_element_count_result;
ItemModelResult_element_count_result ItemModelResult_element_count(const ItemModelResult* self);

typedef struct ItemModelResult_texture_count_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} ItemModelResult_texture_count_result;
ItemModelResult_texture_count_result ItemModelResult_texture_count(const ItemModelResult* self);

typedef struct ItemModelResult_plane_count_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} ItemModelResult_plane_count_result;
ItemModelResult_plane_count_result ItemModelResult_plane_count(const ItemModelResult* self);

typedef struct ItemModelResult_dimensions_result {union {Dimensions ok; NucleationError err;}; bool is_ok;} ItemModelResult_dimensions_result;
ItemModelResult_dimensions_result ItemModelResult_dimensions(const ItemModelResult* self);

typedef struct ItemModelResult_scale_result {union {ItemScale ok; NucleationError err;}; bool is_ok;} ItemModelResult_scale_result;
ItemModelResult_scale_result ItemModelResult_scale(const ItemModelResult* self);

typedef struct ItemModelResult_to_resource_pack_zip_b64_result {union { NucleationError err;}; bool is_ok;} ItemModelResult_to_resource_pack_zip_b64_result;
ItemModelResult_to_resource_pack_zip_b64_result ItemModelResult_to_resource_pack_zip_b64(const ItemModelResult* self, DiplomatWrite* write);

typedef struct ItemModelResult_add_to_pack_result {union { NucleationError err;}; bool is_ok;} ItemModelResult_add_to_pack_result;
ItemModelResult_add_to_pack_result ItemModelResult_add_to_pack(ItemModelResult* self, const ItemModelPackBuilder* builder);

void ItemModelResult_destroy(ItemModelResult* self);





#endif // ItemModelResult_H
