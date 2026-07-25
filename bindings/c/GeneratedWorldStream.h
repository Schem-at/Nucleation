#ifndef GeneratedWorldStream_H
#define GeneratedWorldStream_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "GeneratedChunk.d.h"
#include "NucleationError.d.h"

#include "GeneratedWorldStream.d.h"






uint64_t GeneratedWorldStream_remaining(const GeneratedWorldStream* self);

typedef struct GeneratedWorldStream_next_result {union {GeneratedChunk* ok; NucleationError err;}; bool is_ok;} GeneratedWorldStream_next_result;
GeneratedWorldStream_next_result GeneratedWorldStream_next(GeneratedWorldStream* self);

void GeneratedWorldStream_destroy(GeneratedWorldStream* self);





#endif // GeneratedWorldStream_H
