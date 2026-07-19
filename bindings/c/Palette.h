#ifndef Palette_H
#define Palette_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "Palette.d.h"






Palette* Palette_all(void);

Palette* Palette_solid(void);

Palette* Palette_structural(void);

Palette* Palette_decorative(void);

Palette* Palette_concrete(void);

Palette* Palette_wool(void);

Palette* Palette_terracotta(void);

Palette* Palette_grayscale(void);

Palette* Palette_wood(void);

Palette* Palette_dithered(const Palette* self);

Palette* Palette_sorted_by_lightness(const Palette* self);

typedef struct Palette_ramp_ids_json_result {union { NucleationError err;}; bool is_ok;} Palette_ramp_ids_json_result;
Palette_ramp_ids_json_result Palette_ramp_ids_json(const Palette* self, uint8_t r1, uint8_t g1, uint8_t b1, uint8_t r2, uint8_t g2, uint8_t b2, uint32_t steps, DiplomatWrite* write);

typedef struct Palette_gradient_ids_json_result {union { NucleationError err;}; bool is_ok;} Palette_gradient_ids_json_result;
Palette_gradient_ids_json_result Palette_gradient_ids_json(const Palette* self, uint8_t r1, uint8_t g1, uint8_t b1, uint8_t r2, uint8_t g2, uint8_t b2, uint32_t steps, DiplomatWrite* write);

typedef struct Palette_from_block_ids_result {union {Palette* ok; NucleationError err;}; bool is_ok;} Palette_from_block_ids_result;
Palette_from_block_ids_result Palette_from_block_ids(DiplomatStringView ids_json);

size_t Palette_len(const Palette* self);

void Palette_block_ids_json(const Palette* self, DiplomatWrite* write);

typedef struct Palette_closest_block_dithered_result {union { NucleationError err;}; bool is_ok;} Palette_closest_block_dithered_result;
Palette_closest_block_dithered_result Palette_closest_block_dithered(const Palette* self, uint8_t r, uint8_t g, uint8_t b, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct Palette_closest_block_result {union { NucleationError err;}; bool is_ok;} Palette_closest_block_result;
Palette_closest_block_result Palette_closest_block(const Palette* self, uint8_t r, uint8_t g, uint8_t b, DiplomatWrite* write);

void Palette_destroy(Palette* self);





#endif // Palette_H
