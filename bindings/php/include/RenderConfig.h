#ifndef RenderConfig_H
#define RenderConfig_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "RenderConfig.d.h"






RenderConfig* RenderConfig_create(uint32_t width, uint32_t height);

void RenderConfig_set_yaw(RenderConfig* self, float yaw);

void RenderConfig_set_pitch(RenderConfig* self, float pitch);

void RenderConfig_set_zoom(RenderConfig* self, float zoom);

void RenderConfig_set_sphere_fit(RenderConfig* self, bool sphere_fit);

void RenderConfig_set_fov(RenderConfig* self, float fov);

typedef struct RenderConfig_set_directional_light_result {union { NucleationError err;}; bool is_ok;} RenderConfig_set_directional_light_result;
RenderConfig_set_directional_light_result RenderConfig_set_directional_light(RenderConfig* self, float x, float y, float z, float intensity);

typedef struct RenderConfig_set_ambient_light_result {union { NucleationError err;}; bool is_ok;} RenderConfig_set_ambient_light_result;
RenderConfig_set_ambient_light_result RenderConfig_set_ambient_light(RenderConfig* self, float ambient);

void RenderConfig_set_background(RenderConfig* self, float r, float g, float b, float a);

void RenderConfig_clear_background(RenderConfig* self);

void RenderConfig_set_grid(RenderConfig* self, int32_t half_extent, int32_t spacing, float plane_y, bool show_axes, float red, float green, float blue, float alpha);

void RenderConfig_set_fitted_grid(RenderConfig* self, int32_t margin, int32_t spacing, float plane_y, bool show_axes, float red, float green, float blue, float alpha);

void RenderConfig_clear_grid(RenderConfig* self);

void RenderConfig_set_orthographic(RenderConfig* self, bool orthographic);

void RenderConfig_set_isometric(RenderConfig* self);

void RenderConfig_destroy(RenderConfig* self);





#endif // RenderConfig_H
