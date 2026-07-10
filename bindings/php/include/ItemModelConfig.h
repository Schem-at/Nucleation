#ifndef ItemModelConfig_H
#define ItemModelConfig_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "ItemModelConfig.d.h"






typedef struct ItemModelConfig_create_result {union {ItemModelConfig* ok; NucleationError err;}; bool is_ok;} ItemModelConfig_create_result;
ItemModelConfig_create_result ItemModelConfig_create(DiplomatStringView model_name);

typedef struct ItemModelConfig_set_namespace_result {union { NucleationError err;}; bool is_ok;} ItemModelConfig_set_namespace_result;
ItemModelConfig_set_namespace_result ItemModelConfig_set_namespace(ItemModelConfig* self, DiplomatStringView namespace);

void ItemModelConfig_set_center(ItemModelConfig* self, bool center);

void ItemModelConfig_set_texture_resolution(ItemModelConfig* self, uint32_t resolution);

typedef struct ItemModelConfig_set_item_result {union { NucleationError err;}; bool is_ok;} ItemModelConfig_set_item_result;
ItemModelConfig_set_item_result ItemModelConfig_set_item(ItemModelConfig* self, DiplomatStringView item);

typedef struct ItemModelConfig_set_custom_model_data_result {union { NucleationError err;}; bool is_ok;} ItemModelConfig_set_custom_model_data_result;
ItemModelConfig_set_custom_model_data_result ItemModelConfig_set_custom_model_data(ItemModelConfig* self, DiplomatStringView cmd);

void ItemModelConfig_set_scale(ItemModelConfig* self, float scale);

void ItemModelConfig_set_scale_xyz(ItemModelConfig* self, float sx, float sy, float sz);

void ItemModelConfig_set_scale_auto(ItemModelConfig* self);

void ItemModelConfig_destroy(ItemModelConfig* self);





#endif // ItemModelConfig_H
