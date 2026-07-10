#ifndef WorldSink_H
#define WorldSink_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "WorldChunkView.d.h"

#include "WorldSink.d.h"






typedef struct WorldSink_create_result {union {WorldSink* ok; NucleationError err;}; bool is_ok;} WorldSink_create_result;
WorldSink_create_result WorldSink_create(DiplomatStringView dir, DiplomatStringView options_json);

typedef struct WorldSink_open_existing_result {union {WorldSink* ok; NucleationError err;}; bool is_ok;} WorldSink_open_existing_result;
WorldSink_open_existing_result WorldSink_open_existing(DiplomatStringView dir);

typedef struct WorldSink_write_chunk_result {union { NucleationError err;}; bool is_ok;} WorldSink_write_chunk_result;
WorldSink_write_chunk_result WorldSink_write_chunk(WorldSink* self, const WorldChunkView* view);

typedef struct WorldSink_put_chunk_result {union { NucleationError err;}; bool is_ok;} WorldSink_put_chunk_result;
WorldSink_put_chunk_result WorldSink_put_chunk(WorldSink* self, const WorldChunkView* view);

typedef struct WorldSink_finish_result {union { NucleationError err;}; bool is_ok;} WorldSink_finish_result;
WorldSink_finish_result WorldSink_finish(WorldSink* self);

void WorldSink_destroy(WorldSink* self);





#endif // WorldSink_H
