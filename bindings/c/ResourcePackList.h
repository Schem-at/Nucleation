#ifndef ResourcePackList_H
#define ResourcePackList_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"


#include "ResourcePackList.d.h"






ResourcePackList* ResourcePackList_create(void);

void ResourcePackList_add(ResourcePackList* self, DiplomatU8View data);

uint32_t ResourcePackList_len(const ResourcePackList* self);

void ResourcePackList_destroy(ResourcePackList* self);





#endif // ResourcePackList_H
