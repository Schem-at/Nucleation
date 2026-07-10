#ifndef RawMeshExport_H
#define RawMeshExport_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "MeshConfig.d.h"
#include "NucleationError.d.h"
#include "ResourcePack.d.h"
#include "Schematic.d.h"

#include "RawMeshExport.d.h"






typedef struct RawMeshExport_create_result {union {RawMeshExport* ok; NucleationError err;}; bool is_ok;} RawMeshExport_create_result;
RawMeshExport_create_result RawMeshExport_create(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config);

uint32_t RawMeshExport_vertex_count(const RawMeshExport* self);

uint32_t RawMeshExport_triangle_count(const RawMeshExport* self);

void RawMeshExport_positions_b64(const RawMeshExport* self, DiplomatWrite* write);

void RawMeshExport_normals_b64(const RawMeshExport* self, DiplomatWrite* write);

void RawMeshExport_uvs_b64(const RawMeshExport* self, DiplomatWrite* write);

void RawMeshExport_colors_b64(const RawMeshExport* self, DiplomatWrite* write);

void RawMeshExport_indices_b64(const RawMeshExport* self, DiplomatWrite* write);

void RawMeshExport_texture_rgba_b64(const RawMeshExport* self, DiplomatWrite* write);

uint32_t RawMeshExport_texture_width(const RawMeshExport* self);

uint32_t RawMeshExport_texture_height(const RawMeshExport* self);

void RawMeshExport_destroy(RawMeshExport* self);





#endif // RawMeshExport_H
