#ifndef WorldChunkView_H
#define WorldChunkView_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "WorldChunkView.d.h"






WorldChunkView* WorldChunkView_create(int32_t cx, int32_t cz);

int32_t WorldChunkView_cx(const WorldChunkView* self);

int32_t WorldChunkView_cz(const WorldChunkView* self);

Schematic* WorldChunkView_to_schematic(const WorldChunkView* self);

WorldChunkView* WorldChunkView_from_schematic(const Schematic* schematic, int32_t cx, int32_t cz);

typedef struct WorldChunkView_set_block_result {union { NucleationError err;}; bool is_ok;} WorldChunkView_set_block_result;
WorldChunkView_set_block_result WorldChunkView_set_block(WorldChunkView* self, int32_t x, int32_t y, int32_t z, DiplomatStringView block_name);

typedef struct WorldChunkView_set_biome_result {union { NucleationError err;}; bool is_ok;} WorldChunkView_set_biome_result;
WorldChunkView_set_biome_result WorldChunkView_set_biome(WorldChunkView* self, DiplomatStringView biome_name);

typedef struct WorldChunkView_biome_palette_json_result {union { NucleationError err;}; bool is_ok;} WorldChunkView_biome_palette_json_result;
WorldChunkView_biome_palette_json_result WorldChunkView_biome_palette_json(const WorldChunkView* self, DiplomatWrite* write);

void WorldChunkView_destroy(WorldChunkView* self);





#endif // WorldChunkView_H
