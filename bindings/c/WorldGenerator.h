#ifndef WorldGenerator_H
#define WorldGenerator_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Brush.d.h"
#include "CellularSdfConfig.d.h"
#include "GeneratedChunk.d.h"
#include "GeneratedChunkOverlayMode.d.h"
#include "GeneratedWorldStream.d.h"
#include "NucleationError.d.h"
#include "Sdf.d.h"

#include "WorldGenerator.d.h"






typedef struct WorldGenerator_sdf_result {union {WorldGenerator* ok; NucleationError err;}; bool is_ok;} WorldGenerator_sdf_result;
WorldGenerator_sdf_result WorldGenerator_sdf(const Sdf* volume, const Brush* material, int32_t min_y, int32_t max_y, DiplomatStringView source_id, DiplomatStringView version);

typedef struct WorldGenerator_cellular_sdf_result {union {WorldGenerator* ok; NucleationError err;}; bool is_ok;} WorldGenerator_cellular_sdf_result;
WorldGenerator_cellular_sdf_result WorldGenerator_cellular_sdf(const Sdf* volume, const Brush* material, int32_t min_y, int32_t max_y, const CellularSdfConfig* config, DiplomatStringView source_id, DiplomatStringView version);

typedef struct WorldGenerator_projected_footprints_result {union {WorldGenerator* ok; NucleationError err;}; bool is_ok;} WorldGenerator_projected_footprints_result;
WorldGenerator_projected_footprints_result WorldGenerator_projected_footprints(DiplomatStringView buildings_json, DiplomatStringView base_block, DiplomatStringView source_id, DiplomatStringView version);

typedef struct WorldGenerator_composite_result {union {WorldGenerator* ok; NucleationError err;}; bool is_ok;} WorldGenerator_composite_result;
WorldGenerator_composite_result WorldGenerator_composite(DiplomatStringView source_id, DiplomatStringView version);

typedef struct WorldGenerator_add_layer_result {union { NucleationError err;}; bool is_ok;} WorldGenerator_add_layer_result;
WorldGenerator_add_layer_result WorldGenerator_add_layer(WorldGenerator* self, const WorldGenerator* source, GeneratedChunkOverlayMode mode);

typedef struct WorldGenerator_generate_result {union {GeneratedChunk* ok; NucleationError err;}; bool is_ok;} WorldGenerator_generate_result;
WorldGenerator_generate_result WorldGenerator_generate(const WorldGenerator* self, int32_t cx, int32_t cz);

typedef struct WorldGenerator_stream_result {union {GeneratedWorldStream* ok; NucleationError err;}; bool is_ok;} WorldGenerator_stream_result;
WorldGenerator_stream_result WorldGenerator_stream(const WorldGenerator* self, int32_t min_cx, int32_t min_cz, int32_t max_cx, int32_t max_cz);

void WorldGenerator_destroy(WorldGenerator* self);





#endif // WorldGenerator_H
