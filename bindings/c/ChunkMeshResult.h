#ifndef ChunkMeshResult_H
#define ChunkMeshResult_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "BlockPos.d.h"
#include "MeshConfig.d.h"
#include "MeshResult.d.h"
#include "NucleationError.d.h"
#include "ResourcePack.d.h"
#include "Schematic.d.h"
#include "TextureAtlas.d.h"

#include "ChunkMeshResult.d.h"






typedef struct ChunkMeshResult_create_result {union {ChunkMeshResult* ok; NucleationError err;}; bool is_ok;} ChunkMeshResult_create_result;
ChunkMeshResult_create_result ChunkMeshResult_create(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config);

typedef struct ChunkMeshResult_create_with_size_result {union {ChunkMeshResult* ok; NucleationError err;}; bool is_ok;} ChunkMeshResult_create_with_size_result;
ChunkMeshResult_create_with_size_result ChunkMeshResult_create_with_size(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config, int32_t chunk_size);

typedef struct ChunkMeshResult_create_with_atlas_result {union {ChunkMeshResult* ok; NucleationError err;}; bool is_ok;} ChunkMeshResult_create_with_atlas_result;
ChunkMeshResult_create_with_atlas_result ChunkMeshResult_create_with_atlas(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config, int32_t chunk_size, const TextureAtlas* atlas);

uint32_t ChunkMeshResult_chunk_count(const ChunkMeshResult* self);

typedef struct ChunkMeshResult_chunk_coordinate_at_result {union {BlockPos ok; NucleationError err;}; bool is_ok;} ChunkMeshResult_chunk_coordinate_at_result;
ChunkMeshResult_chunk_coordinate_at_result ChunkMeshResult_chunk_coordinate_at(const ChunkMeshResult* self, uint32_t index);

typedef struct ChunkMeshResult_get_mesh_result {union {MeshResult* ok; NucleationError err;}; bool is_ok;} ChunkMeshResult_get_mesh_result;
ChunkMeshResult_get_mesh_result ChunkMeshResult_get_mesh(const ChunkMeshResult* self, int32_t cx, int32_t cy, int32_t cz);

uint32_t ChunkMeshResult_total_vertex_count(const ChunkMeshResult* self);

uint32_t ChunkMeshResult_total_triangle_count(const ChunkMeshResult* self);

void ChunkMeshResult_nucm_data_b64(const ChunkMeshResult* self, DiplomatWrite* write);

void ChunkMeshResult_nucm_data_with_atlas_b64(const ChunkMeshResult* self, const TextureAtlas* atlas, DiplomatWrite* write);

void ChunkMeshResult_destroy(ChunkMeshResult* self);





#endif // ChunkMeshResult_H
