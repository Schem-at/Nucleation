#ifndef Blocks_H
#define Blocks_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "Blocks.d.h"






typedef struct Blocks_get_json_result {union { NucleationError err;}; bool is_ok;} Blocks_get_json_result;
Blocks_get_json_result Blocks_get_json(DiplomatStringView id, DiplomatWrite* write);

void Blocks_ids_json(DiplomatWrite* write);

typedef struct Blocks_by_tag_json_result {union { NucleationError err;}; bool is_ok;} Blocks_by_tag_json_result;
Blocks_by_tag_json_result Blocks_by_tag_json(DiplomatStringView tag, DiplomatWrite* write);

typedef struct Blocks_by_kind_json_result {union { NucleationError err;}; bool is_ok;} Blocks_by_kind_json_result;
Blocks_by_kind_json_result Blocks_by_kind_json(DiplomatStringView kind, DiplomatWrite* write);

typedef struct Blocks_variants_of_json_result {union { NucleationError err;}; bool is_ok;} Blocks_variants_of_json_result;
Blocks_variants_of_json_result Blocks_variants_of_json(DiplomatStringView base_id, DiplomatWrite* write);

void Blocks_tags_json(DiplomatWrite* write);

typedef struct Blocks_states_json_result {union { NucleationError err;}; bool is_ok;} Blocks_states_json_result;
Blocks_states_json_result Blocks_states_json(DiplomatStringView id, DiplomatWrite* write);

size_t Blocks_count(void);

void Blocks_destroy(Blocks* self);





#endif // Blocks_H
