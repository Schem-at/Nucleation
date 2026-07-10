#ifndef WorldStream_H
#define WorldStream_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "WorldChunkView.d.h"

#include "WorldStream.d.h"






typedef struct WorldStream_open_dir_result {union {WorldStream* ok; NucleationError err;}; bool is_ok;} WorldStream_open_dir_result;
WorldStream_open_dir_result WorldStream_open_dir(DiplomatStringView path);

typedef struct WorldStream_open_dir_bounded_result {union {WorldStream* ok; NucleationError err;}; bool is_ok;} WorldStream_open_dir_bounded_result;
WorldStream_open_dir_bounded_result WorldStream_open_dir_bounded(DiplomatStringView path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct WorldStream_from_zip_result {union {WorldStream* ok; NucleationError err;}; bool is_ok;} WorldStream_from_zip_result;
WorldStream_from_zip_result WorldStream_from_zip(DiplomatU8View data);

typedef struct WorldStream_from_zip_bounded_result {union {WorldStream* ok; NucleationError err;}; bool is_ok;} WorldStream_from_zip_bounded_result;
WorldStream_from_zip_bounded_result WorldStream_from_zip_bounded(DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

typedef struct WorldStream_next_result {union {WorldChunkView* ok; NucleationError err;}; bool is_ok;} WorldStream_next_result;
WorldStream_next_result WorldStream_next(WorldStream* self);

void WorldStream_destroy(WorldStream* self);





#endif // WorldStream_H
