#ifndef MeshBytes_H
#define MeshBytes_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"


#include "MeshBytes.d.h"






uint32_t MeshBytes_len(const MeshBytes* self);

uint32_t MeshBytes_copy_into(const MeshBytes* self, uint32_t offset, DiplomatU8ViewMut out);

void MeshBytes_destroy(MeshBytes* self);





#endif // MeshBytes_H
