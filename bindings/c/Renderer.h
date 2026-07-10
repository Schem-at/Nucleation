#ifndef Renderer_H
#define Renderer_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "RenderConfig.d.h"
#include "Schematic.d.h"

#include "Renderer.d.h"






typedef struct Renderer_render_pixels_b64_result {union { NucleationError err;}; bool is_ok;} Renderer_render_pixels_b64_result;
Renderer_render_pixels_b64_result Renderer_render_pixels_b64(const Schematic* schematic, DiplomatU8View pack_zip, const RenderConfig* config, DiplomatWrite* write);

typedef struct Renderer_render_png_b64_result {union { NucleationError err;}; bool is_ok;} Renderer_render_png_b64_result;
Renderer_render_png_b64_result Renderer_render_png_b64(const Schematic* schematic, DiplomatU8View pack_zip, const RenderConfig* config, DiplomatWrite* write);

typedef struct Renderer_render_to_file_result {union { NucleationError err;}; bool is_ok;} Renderer_render_to_file_result;
Renderer_render_to_file_result Renderer_render_to_file(const Schematic* schematic, DiplomatU8View pack_zip, const RenderConfig* config, DiplomatStringView path);

void Renderer_destroy(Renderer* self);





#endif // Renderer_H
