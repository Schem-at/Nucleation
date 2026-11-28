#ifndef Material_H
#define Material_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Brush.d.h"
#include "Field.d.h"
#include "FieldChannel.d.h"
#include "InterpolationSpace.d.h"
#include "NucleationError.d.h"
#include "Palette.d.h"

#include "Material.d.h"






typedef struct Material_solid_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_solid_result;
Material_solid_result Material_solid(DiplomatStringView block);

Material* Material_skip(void);

Material* Material_color(uint8_t r, uint8_t g, uint8_t b, const Palette* palette);

Material* Material_from_brush(const Brush* brush);

typedef struct Material_channel_gradient_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_channel_gradient_result;
Material_channel_gradient_result Material_channel_gradient(FieldChannel channel, DiplomatF32View stops, DiplomatU8View colors, float lo, float hi, const Palette* palette, InterpolationSpace space);

typedef struct Material_field_gradient_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_field_gradient_result;
Material_field_gradient_result Material_field_gradient(const Field* field, DiplomatF32View stops, DiplomatU8View colors, float lo, float hi, const Palette* palette, InterpolationSpace space);

typedef struct Material_channel_ramp_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_channel_ramp_result;
Material_channel_ramp_result Material_channel_ramp(FieldChannel channel, float lo, float hi, DiplomatStringView blocks_json, bool dither);

typedef struct Material_field_ramp_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_field_ramp_result;
Material_field_ramp_result Material_field_ramp(const Field* field, float lo, float hi, DiplomatStringView blocks_json, bool dither);

typedef struct Material_weighted_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_weighted_result;
Material_weighted_result Material_weighted(DiplomatStringView blocks_json, DiplomatF32View weights, int32_t seed);

typedef struct Material_shaded_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_shaded_result;
Material_shaded_result Material_shaded(uint8_t r, uint8_t g, uint8_t b, float lx, float ly, float lz, float ambient, const Palette* palette);

typedef struct Material_spotlight_result {union {Material* ok; NucleationError err;}; bool is_ok;} Material_spotlight_result;
Material_spotlight_result Material_spotlight(float px, float py, float pz, float dx, float dy, float dz, float cone_angle_deg, uint8_t r, uint8_t g, uint8_t b, float ambient, const Palette* palette);

void Material_destroy(Material* self);





#endif // Material_H
