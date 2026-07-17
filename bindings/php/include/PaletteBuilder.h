#ifndef PaletteBuilder_H
#define PaletteBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Palette.d.h"

#include "PaletteBuilder.d.h"






PaletteBuilder* PaletteBuilder_create(void);

typedef struct PaletteBuilder_exclude_falling_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_falling_result;
PaletteBuilder_exclude_falling_result PaletteBuilder_exclude_falling(PaletteBuilder* self);

typedef struct PaletteBuilder_exclude_tile_entities_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_tile_entities_result;
PaletteBuilder_exclude_tile_entities_result PaletteBuilder_exclude_tile_entities(PaletteBuilder* self);

typedef struct PaletteBuilder_full_blocks_only_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_full_blocks_only_result;
PaletteBuilder_full_blocks_only_result PaletteBuilder_full_blocks_only(PaletteBuilder* self);

typedef struct PaletteBuilder_exclude_needs_support_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_needs_support_result;
PaletteBuilder_exclude_needs_support_result PaletteBuilder_exclude_needs_support(PaletteBuilder* self);

typedef struct PaletteBuilder_exclude_transparent_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_transparent_result;
PaletteBuilder_exclude_transparent_result PaletteBuilder_exclude_transparent(PaletteBuilder* self);

typedef struct PaletteBuilder_exclude_light_sources_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_light_sources_result;
PaletteBuilder_exclude_light_sources_result PaletteBuilder_exclude_light_sources(PaletteBuilder* self);

typedef struct PaletteBuilder_survival_only_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_survival_only_result;
PaletteBuilder_survival_only_result PaletteBuilder_survival_only(PaletteBuilder* self);

typedef struct PaletteBuilder_exclude_keyword_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_keyword_result;
PaletteBuilder_exclude_keyword_result PaletteBuilder_exclude_keyword(PaletteBuilder* self, DiplomatStringView keyword);

typedef struct PaletteBuilder_include_keyword_result {union { NucleationError err;}; bool is_ok;} PaletteBuilder_include_keyword_result;
PaletteBuilder_include_keyword_result PaletteBuilder_include_keyword(PaletteBuilder* self, DiplomatStringView keyword);

typedef struct PaletteBuilder_build_result {union {Palette* ok; NucleationError err;}; bool is_ok;} PaletteBuilder_build_result;
PaletteBuilder_build_result PaletteBuilder_build(PaletteBuilder* self);

void PaletteBuilder_destroy(PaletteBuilder* self);





#endif // PaletteBuilder_H
