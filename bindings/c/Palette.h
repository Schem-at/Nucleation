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

typedef struct Palette_from_block_ids_result {union {Palette* ok; NucleationError err;}; bool is_ok;} Palette_from_block_ids_result;
Palette_from_block_ids_result Palette_from_block_ids(DiplomatStringView ids_json);

size_t Palette_len(const Palette* self);

void Palette_block_ids_json(const Palette* self, DiplomatWrite* write);

typedef struct Palette_closest_block_result {union { NucleationError err;}; bool is_ok;} Palette_closest_block_result;
Palette_closest_block_result Palette_closest_block(const Palette* self, uint8_t r, uint8_t g, uint8_t b, DiplomatWrite* write);

void Palette_destroy(Palette* self);





#endif // Palette_H
