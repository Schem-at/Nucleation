#ifndef Brush_H
#define Brush_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "InterpolationSpace.d.h"
#include "NucleationError.d.h"
#include "Palette.d.h"

#include "Brush.d.h"






typedef struct Brush_solid_result {union {Brush* ok; NucleationError err;}; bool is_ok;} Brush_solid_result;
Brush_solid_result Brush_solid(DiplomatStringView block_name);

Brush* Brush_color(uint8_t r, uint8_t g, uint8_t b);

Brush* Brush_linear_gradient(int32_t x1, int32_t y1, int32_t z1, uint8_t r1, uint8_t g1, uint8_t b1, int32_t x2, int32_t y2, int32_t z2, uint8_t r2, uint8_t g2, uint8_t b2, InterpolationSpace space);

Brush* Brush_shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz);

Brush* Brush_bilinear_gradient(int32_t ox, int32_t oy, int32_t oz, int32_t ux, int32_t uy, int32_t uz, int32_t vx, int32_t vy, int32_t vz, uint8_t r00, uint8_t g00, uint8_t b00, uint8_t r10, uint8_t g10, uint8_t b10, uint8_t r01, uint8_t g01, uint8_t b01, uint8_t r11, uint8_t g11, uint8_t b11, InterpolationSpace space);

typedef struct Brush_point_gradient_result {union {Brush* ok; NucleationError err;}; bool is_ok;} Brush_point_gradient_result;
Brush_point_gradient_result Brush_point_gradient(DiplomatI32View positions, DiplomatU8View colors, float falloff, InterpolationSpace space);

Brush* Brush_spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b);

void Brush_set_palette(Brush* self, const Palette* palette);

typedef struct Brush_curve_gradient_result {union {Brush* ok; NucleationError err;}; bool is_ok;} Brush_curve_gradient_result;
Brush_curve_gradient_result Brush_curve_gradient(DiplomatF32View stops, DiplomatU8View colors, InterpolationSpace space);

typedef struct Brush_field_result {union {Brush* ok; NucleationError err;}; bool is_ok;} Brush_field_result;
Brush_field_result Brush_field(DiplomatStringView field_json, DiplomatF32View stops, DiplomatU8View colors, float lo, float hi, InterpolationSpace space);

void Brush_destroy(Brush* self);





#endif // Brush_H
