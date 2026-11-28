#ifndef MaterialCondition_H
#define MaterialCondition_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Field.d.h"
#include "FieldChannel.d.h"
#include "NucleationError.d.h"

#include "MaterialCondition.d.h"






MaterialCondition* MaterialCondition_always(void);

MaterialCondition* MaterialCondition_and_with(const MaterialCondition* self, const MaterialCondition* other);

MaterialCondition* MaterialCondition_or_with(const MaterialCondition* self, const MaterialCondition* other);

MaterialCondition* MaterialCondition_negated(const MaterialCondition* self);

MaterialCondition* MaterialCondition_tag(uint16_t material_id);

typedef struct MaterialCondition_channel_range_result {union {MaterialCondition* ok; NucleationError err;}; bool is_ok;} MaterialCondition_channel_range_result;
MaterialCondition_channel_range_result MaterialCondition_channel_range(FieldChannel channel, bool has_min, float min, bool has_max, float max);

typedef struct MaterialCondition_field_range_result {union {MaterialCondition* ok; NucleationError err;}; bool is_ok;} MaterialCondition_field_range_result;
MaterialCondition_field_range_result MaterialCondition_field_range(const Field* field, bool has_min, float min, bool has_max, float max);

typedef struct MaterialCondition_y_range_result {union {MaterialCondition* ok; NucleationError err;}; bool is_ok;} MaterialCondition_y_range_result;
MaterialCondition_y_range_result MaterialCondition_y_range(bool has_min, float min, bool has_max, float max);

MaterialCondition* MaterialCondition_depth_below_surface(bool has_min, int32_t min, bool has_max, int32_t max);

typedef struct MaterialCondition_slope_range_result {union {MaterialCondition* ok; NucleationError err;}; bool is_ok;} MaterialCondition_slope_range_result;
MaterialCondition_slope_range_result MaterialCondition_slope_range(bool has_min, float min, bool has_max, float max);

typedef struct MaterialCondition_curvature_range_result {union {MaterialCondition* ok; NucleationError err;}; bool is_ok;} MaterialCondition_curvature_range_result;
MaterialCondition_curvature_range_result MaterialCondition_curvature_range(bool has_min, float min, bool has_max, float max);

typedef struct MaterialCondition_normal_dot_result {union {MaterialCondition* ok; NucleationError err;}; bool is_ok;} MaterialCondition_normal_dot_result;
MaterialCondition_normal_dot_result MaterialCondition_normal_dot(float dx, float dy, float dz, bool has_min, float min, bool has_max, float max);

MaterialCondition* MaterialCondition_exposed_face(uint8_t mask, bool require_all);

typedef struct MaterialCondition_chance_result {union {MaterialCondition* ok; NucleationError err;}; bool is_ok;} MaterialCondition_chance_result;
MaterialCondition_chance_result MaterialCondition_chance(float probability, int32_t seed);

void MaterialCondition_destroy(MaterialCondition* self);





#endif // MaterialCondition_H
