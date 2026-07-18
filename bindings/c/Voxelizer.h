#ifndef Voxelizer_H
#define Voxelizer_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Palette.d.h"
#include "Schematic.d.h"
#include "Shape.d.h"

#include "Voxelizer.d.h"






typedef struct Voxelizer_shape_from_glb_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Voxelizer_shape_from_glb_result;
Voxelizer_shape_from_glb_result Voxelizer_shape_from_glb(DiplomatU8View data, float target_size);

typedef struct Voxelizer_shape_from_obj_result {union {Shape* ok; NucleationError err;}; bool is_ok;} Voxelizer_shape_from_obj_result;
Voxelizer_shape_from_obj_result Voxelizer_shape_from_obj(DiplomatStringView text, float target_size);

typedef struct Voxelizer_schematic_from_glb_textured_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Voxelizer_schematic_from_glb_textured_result;
Voxelizer_schematic_from_glb_textured_result Voxelizer_schematic_from_glb_textured(DiplomatU8View data, float target_size, const Palette* palette, DiplomatStringView name);

void Voxelizer_destroy(Voxelizer* self);





#endif // Voxelizer_H
