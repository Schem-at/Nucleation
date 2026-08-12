#ifndef SchematicSplitResult_H
#define SchematicSplitResult_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "SchematicSplitResult.d.h"






uint32_t SchematicSplitResult_len(const SchematicSplitResult* self);

typedef struct SchematicSplitResult_piece_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} SchematicSplitResult_piece_result;
SchematicSplitResult_piece_result SchematicSplitResult_piece(const SchematicSplitResult* self, uint32_t index);

void SchematicSplitResult_destroy(SchematicSplitResult* self);





#endif // SchematicSplitResult_H
