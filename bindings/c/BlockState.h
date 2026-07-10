#ifndef BlockState_H
#define BlockState_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "BlockState.d.h"






BlockState* BlockState_create(DiplomatStringView name);

typedef struct BlockState_with_property_result {union {BlockState* ok; NucleationError err;}; bool is_ok;} BlockState_with_property_result;
BlockState_with_property_result BlockState_with_property(const BlockState* self, DiplomatStringView key, DiplomatStringView value);

void BlockState_name(const BlockState* self, DiplomatWrite* write);

void BlockState_properties_json(const BlockState* self, DiplomatWrite* write);

void BlockState_destroy(BlockState* self);





#endif // BlockState_H
