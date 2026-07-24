#ifndef MeshResult_H
#define MeshResult_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "MeshBounds.d.h"
#include "MeshConfig.d.h"
#include "NucleationError.d.h"
#include "ResourcePack.d.h"
#include "Schematic.d.h"

#include "MeshResult.d.h"






typedef struct MeshResult_create_result {union {MeshResult* ok; NucleationError err;}; bool is_ok;} MeshResult_create_result;
MeshResult_create_result MeshResult_create(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config);

typedef struct MeshResult_create_usdz_result {union {MeshResult* ok; NucleationError err;}; bool is_ok;} MeshResult_create_usdz_result;
MeshResult_create_usdz_result MeshResult_create_usdz(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config);

typedef struct MeshResult_glb_data_b64_result {union { NucleationError err;}; bool is_ok;} MeshResult_glb_data_b64_result;
MeshResult_glb_data_b64_result MeshResult_glb_data_b64(const MeshResult* self, DiplomatWrite* write);

typedef struct MeshResult_usdz_data_b64_result {union { NucleationError err;}; bool is_ok;} MeshResult_usdz_data_b64_result;
MeshResult_usdz_data_b64_result MeshResult_usdz_data_b64(const MeshResult* self, DiplomatWrite* write);

void MeshResult_nucm_data_b64(const MeshResult* self, DiplomatWrite* write);

uint32_t MeshResult_vertex_count(const MeshResult* self);

uint32_t MeshResult_triangle_count(const MeshResult* self);

bool MeshResult_has_transparency(const MeshResult* self);

MeshBounds MeshResult_bounds(const MeshResult* self);

void MeshResult_destroy(MeshResult* self);





#endif // MeshResult_H
