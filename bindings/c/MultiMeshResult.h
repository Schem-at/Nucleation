#ifndef MultiMeshResult_H
#define MultiMeshResult_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "MeshConfig.d.h"
#include "MeshResult.d.h"
#include "NucleationError.d.h"
#include "ResourcePack.d.h"
#include "Schematic.d.h"

#include "MultiMeshResult.d.h"






typedef struct MultiMeshResult_create_result {union {MultiMeshResult* ok; NucleationError err;}; bool is_ok;} MultiMeshResult_create_result;
MultiMeshResult_create_result MultiMeshResult_create(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config);

void MultiMeshResult_region_names_json(const MultiMeshResult* self, DiplomatWrite* write);

typedef struct MultiMeshResult_get_mesh_result {union {MeshResult* ok; NucleationError err;}; bool is_ok;} MultiMeshResult_get_mesh_result;
MultiMeshResult_get_mesh_result MultiMeshResult_get_mesh(const MultiMeshResult* self, DiplomatStringView region_name);

uint32_t MultiMeshResult_total_vertex_count(const MultiMeshResult* self);

uint32_t MultiMeshResult_total_triangle_count(const MultiMeshResult* self);

uint32_t MultiMeshResult_mesh_count(const MultiMeshResult* self);

void MultiMeshResult_destroy(MultiMeshResult* self);





#endif // MultiMeshResult_H
