#ifndef ItemModelPackBuilder_H
#define ItemModelPackBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "ItemModelPackBuilder.d.h"






ItemModelPackBuilder* ItemModelPackBuilder_create(void);

uint32_t ItemModelPackBuilder_len(const ItemModelPackBuilder* self);

typedef struct ItemModelPackBuilder_build_zip_b64_result {union { NucleationError err;}; bool is_ok;} ItemModelPackBuilder_build_zip_b64_result;
ItemModelPackBuilder_build_zip_b64_result ItemModelPackBuilder_build_zip_b64(const ItemModelPackBuilder* self, DiplomatWrite* write);

void ItemModelPackBuilder_destroy(ItemModelPackBuilder* self);





#endif // ItemModelPackBuilder_H
