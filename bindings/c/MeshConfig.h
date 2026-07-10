#ifndef MeshConfig_H
#define MeshConfig_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "MeshConfig.d.h"






MeshConfig* MeshConfig_create(void);

void MeshConfig_set_cull_hidden_faces(MeshConfig* self, bool val);

bool MeshConfig_cull_hidden_faces(const MeshConfig* self);

void MeshConfig_set_ambient_occlusion(MeshConfig* self, bool val);

bool MeshConfig_ambient_occlusion(const MeshConfig* self);

void MeshConfig_set_ao_intensity(MeshConfig* self, float val);

float MeshConfig_ao_intensity(const MeshConfig* self);

typedef struct MeshConfig_set_biome_result {union { NucleationError err;}; bool is_ok;} MeshConfig_set_biome_result;
MeshConfig_set_biome_result MeshConfig_set_biome(MeshConfig* self, DiplomatStringView biome);

void MeshConfig_clear_biome(MeshConfig* self);

typedef struct MeshConfig_biome_result {union { NucleationError err;}; bool is_ok;} MeshConfig_biome_result;
MeshConfig_biome_result MeshConfig_biome(const MeshConfig* self, DiplomatWrite* write);

void MeshConfig_set_atlas_max_size(MeshConfig* self, uint32_t size);

uint32_t MeshConfig_atlas_max_size(const MeshConfig* self);

void MeshConfig_set_cull_occluded_blocks(MeshConfig* self, bool val);

bool MeshConfig_cull_occluded_blocks(const MeshConfig* self);

void MeshConfig_set_greedy_meshing(MeshConfig* self, bool val);

bool MeshConfig_greedy_meshing(const MeshConfig* self);

void MeshConfig_destroy(MeshConfig* self);





#endif // MeshConfig_H
