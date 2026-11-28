#ifndef ScatterRule_H
#define ScatterRule_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "MaterialCondition.d.h"
#include "NucleationError.d.h"
#include "ScatterFace.d.h"

#include "ScatterRule.d.h"






typedef struct ScatterRule_create_result {union {ScatterRule* ok; NucleationError err;}; bool is_ok;} ScatterRule_create_result;
ScatterRule_create_result ScatterRule_create(ScatterFace face, float density, DiplomatStringView blocks_json, DiplomatF32View weights, int32_t seed);

void ScatterRule_set_condition(ScatterRule* self, const MaterialCondition* condition);

typedef struct ScatterRule_set_on_block_result {union { NucleationError err;}; bool is_ok;} ScatterRule_set_on_block_result;
ScatterRule_set_on_block_result ScatterRule_set_on_block(ScatterRule* self, DiplomatStringView prefix);

typedef struct ScatterRule_set_height_range_result {union { NucleationError err;}; bool is_ok;} ScatterRule_set_height_range_result;
ScatterRule_set_height_range_result ScatterRule_set_height_range(ScatterRule* self, uint32_t min, uint32_t max);

void ScatterRule_destroy(ScatterRule* self);





#endif // ScatterRule_H
