#ifndef WsPartitionHints_H
#define WsPartitionHints_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "WsPartitionHints.d.h"






WsPartitionHints* WsPartitionHints_create(void);

typedef struct WsPartitionHints_add_result {union { NucleationError err;}; bool is_ok;} WsPartitionHints_add_result;
WsPartitionHints_add_result WsPartitionHints_add(WsPartitionHints* self, DiplomatStringView id, int32_t x0, int32_t x1, int32_t z0, int32_t z1);

uint32_t WsPartitionHints_len(const WsPartitionHints* self);

void WsPartitionHints_destroy(WsPartitionHints* self);





#endif // WsPartitionHints_H
