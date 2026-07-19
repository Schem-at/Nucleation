#ifndef Geo_H
#define Geo_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Geo.d.h"






typedef struct Geo_extrude_footprints_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Geo_extrude_footprints_result;
Geo_extrude_footprints_result Geo_extrude_footprints(DiplomatStringView buildings_json, DiplomatStringView base_block, DiplomatStringView name);

typedef struct Geo_heightmap_terrain_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Geo_heightmap_terrain_result;
Geo_heightmap_terrain_result Geo_heightmap_terrain(DiplomatStringView heights_json, int32_t width, DiplomatStringView surface_block, DiplomatStringView subsurface_block, int32_t surface_depth, DiplomatStringView name);

void Geo_destroy(Geo* self);





#endif // Geo_H
