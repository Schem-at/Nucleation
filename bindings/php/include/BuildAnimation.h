#ifndef BuildAnimation_H
#define BuildAnimation_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "AnimationEffect.d.h"
#include "Brush.d.h"
#include "NucleationError.d.h"
#include "RenderConfig.d.h"
#include "Shape.d.h"

#include "BuildAnimation.d.h"






BuildAnimation* BuildAnimation_create(DiplomatStringView name);

void BuildAnimation_set_default_effect(BuildAnimation* self, const AnimationEffect* effect);

BuildAnimation* BuildAnimation_with_effect(BuildAnimation* self, const AnimationEffect* effect);

void BuildAnimation_set_step_ms(BuildAnimation* self, float step_ms);

void BuildAnimation_set_stagger_total_ms(BuildAnimation* self, float total_ms);

void BuildAnimation_clear_stagger(BuildAnimation* self);

void BuildAnimation_set_stagger_offset_ms(BuildAnimation* self, float offset_ms);

typedef struct BuildAnimation_set_loop_period_ms_result {union { NucleationError err;}; bool is_ok;} BuildAnimation_set_loop_period_ms_result;
BuildAnimation_set_loop_period_ms_result BuildAnimation_set_loop_period_ms(BuildAnimation* self, float period_ms);

void BuildAnimation_clear_loop_period(BuildAnimation* self);

typedef struct BuildAnimation_begin_group_result {union { NucleationError err;}; bool is_ok;} BuildAnimation_begin_group_result;
BuildAnimation_begin_group_result BuildAnimation_begin_group(BuildAnimation* self);

typedef struct BuildAnimation_begin_keyed_group_result {union { NucleationError err;}; bool is_ok;} BuildAnimation_begin_keyed_group_result;
BuildAnimation_begin_keyed_group_result BuildAnimation_begin_keyed_group(BuildAnimation* self, float key);

typedef struct BuildAnimation_end_group_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} BuildAnimation_end_group_result;
BuildAnimation_end_group_result BuildAnimation_end_group(BuildAnimation* self);

typedef struct BuildAnimation_set_block_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} BuildAnimation_set_block_result;
BuildAnimation_set_block_result BuildAnimation_set_block(BuildAnimation* self, int32_t x, int32_t y, int32_t z, DiplomatStringView block);

typedef struct BuildAnimation_fill_along_parameter_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} BuildAnimation_fill_along_parameter_result;
BuildAnimation_fill_along_parameter_result BuildAnimation_fill_along_parameter(BuildAnimation* self, const Shape* shape, const Brush* brush, uint32_t group_count);

typedef struct BuildAnimation_add_armor_stand_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} BuildAnimation_add_armor_stand_result;
BuildAnimation_add_armor_stand_result BuildAnimation_add_armor_stand(BuildAnimation* self, double x, double y, double z, float yaw, DiplomatStringView armor_material);

void BuildAnimation_animate_camera(BuildAnimation* self, const AnimationEffect* effect, float offset_ms);

uint32_t BuildAnimation_frame_count(const BuildAnimation* self, double fps, float hold_ms);

typedef struct BuildAnimation_render_gif_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} BuildAnimation_render_gif_result;
BuildAnimation_render_gif_result BuildAnimation_render_gif(const BuildAnimation* self, DiplomatU8View pack_zip, const RenderConfig* config, DiplomatStringView path, double fps, float hold_ms);

typedef struct BuildAnimation_render_frames_result {union {uint32_t ok; NucleationError err;}; bool is_ok;} BuildAnimation_render_frames_result;
BuildAnimation_render_frames_result BuildAnimation_render_frames(const BuildAnimation* self, DiplomatU8View pack_zip, const RenderConfig* config, DiplomatStringView prefix, double fps, float hold_ms);

typedef struct BuildAnimation_save_to_file_result {union { NucleationError err;}; bool is_ok;} BuildAnimation_save_to_file_result;
BuildAnimation_save_to_file_result BuildAnimation_save_to_file(const BuildAnimation* self, DiplomatStringView path);

uint32_t BuildAnimation_group_count(const BuildAnimation* self);

float BuildAnimation_duration_ms(const BuildAnimation* self);

void BuildAnimation_destroy(BuildAnimation* self);





#endif // BuildAnimation_H
