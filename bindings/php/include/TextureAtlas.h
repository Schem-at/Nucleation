#ifndef TextureAtlas_H
#define TextureAtlas_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "MeshConfig.d.h"
#include "NucleationError.d.h"
#include "ResourcePack.d.h"
#include "Schematic.d.h"

#include "TextureAtlas.d.h"






typedef struct TextureAtlas_build_global_result {union {TextureAtlas* ok; NucleationError err;}; bool is_ok;} TextureAtlas_build_global_result;
TextureAtlas_build_global_result TextureAtlas_build_global(const Schematic* schematic, const ResourcePack* pack, const MeshConfig* config);

uint32_t TextureAtlas_width(const TextureAtlas* self);

uint32_t TextureAtlas_height(const TextureAtlas* self);

void TextureAtlas_rgba_data_b64(const TextureAtlas* self, DiplomatWrite* write);

void TextureAtlas_destroy(TextureAtlas* self);





#endif // TextureAtlas_H
