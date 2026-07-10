#ifndef MeshJob_H
#define MeshJob_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "ChunkMeshResult.d.h"
#include "MeshConfig.d.h"
#include "MeshProgress.d.h"
#include "NucleationError.d.h"
#include "ResourcePack.d.h"
#include "Schematic.d.h"
#include "TextureAtlas.d.h"

#include "MeshJob.d.h"






MeshJob* MeshJob_start(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config, int32_t chunk_size, const TextureAtlas* atlas);

MeshProgress MeshJob_poll_progress(const MeshJob* self);

typedef struct MeshJob_take_result_result {union {ChunkMeshResult* ok; NucleationError err;}; bool is_ok;} MeshJob_take_result_result;
MeshJob_take_result_result MeshJob_take_result(MeshJob* self);

void MeshJob_destroy(MeshJob* self);





#endif // MeshJob_H
