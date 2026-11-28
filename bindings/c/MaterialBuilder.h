#ifndef MaterialBuilder_H
#define MaterialBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Material.d.h"
#include "MaterialCondition.d.h"
#include "NucleationError.d.h"

#include "MaterialBuilder.d.h"






MaterialBuilder* MaterialBuilder_create_layered(void);

MaterialBuilder* MaterialBuilder_create_tag_matcher(bool blend_seams, int32_t seed);

typedef struct MaterialBuilder_add_layer_result {union { NucleationError err;}; bool is_ok;} MaterialBuilder_add_layer_result;
MaterialBuilder_add_layer_result MaterialBuilder_add_layer(MaterialBuilder* self, const MaterialCondition* condition, const Material* material);

typedef struct MaterialBuilder_add_tag_case_result {union { NucleationError err;}; bool is_ok;} MaterialBuilder_add_tag_case_result;
MaterialBuilder_add_tag_case_result MaterialBuilder_add_tag_case(MaterialBuilder* self, uint16_t material_id, const Material* material);

void MaterialBuilder_set_default(MaterialBuilder* self, const Material* material);

Material* MaterialBuilder_build(const MaterialBuilder* self);

void MaterialBuilder_destroy(MaterialBuilder* self);





#endif // MaterialBuilder_H
