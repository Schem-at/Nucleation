#ifndef SchematicBuilder_H
#define SchematicBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "SchematicBuilder.d.h"






SchematicBuilder* SchematicBuilder_create(void);

typedef struct SchematicBuilder_from_template_result {union {SchematicBuilder* ok; NucleationError err;}; bool is_ok;} SchematicBuilder_from_template_result;
SchematicBuilder_from_template_result SchematicBuilder_from_template(DiplomatStringView template);

typedef struct SchematicBuilder_name_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_name_result;
SchematicBuilder_name_result SchematicBuilder_name(SchematicBuilder* self, DiplomatStringView name);

typedef struct SchematicBuilder_map_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_map_result;
SchematicBuilder_map_result SchematicBuilder_map(SchematicBuilder* self, DiplomatStringView ch, DiplomatStringView block);

typedef struct SchematicBuilder_layers_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_layers_result;
SchematicBuilder_layers_result SchematicBuilder_layers(SchematicBuilder* self, DiplomatStringView layers_json);

typedef struct SchematicBuilder_layer_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_layer_result;
SchematicBuilder_layer_result SchematicBuilder_layer(SchematicBuilder* self, DiplomatStringView rows_json);

typedef struct SchematicBuilder_palette_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_palette_result;
SchematicBuilder_palette_result SchematicBuilder_palette(SchematicBuilder* self, DiplomatStringView pairs_json);

typedef struct SchematicBuilder_offset_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_offset_result;
SchematicBuilder_offset_result SchematicBuilder_offset(SchematicBuilder* self, int32_t x, int32_t y, int32_t z);

typedef struct SchematicBuilder_use_standard_palette_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_use_standard_palette_result;
SchematicBuilder_use_standard_palette_result SchematicBuilder_use_standard_palette(SchematicBuilder* self);

typedef struct SchematicBuilder_use_minimal_palette_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_use_minimal_palette_result;
SchematicBuilder_use_minimal_palette_result SchematicBuilder_use_minimal_palette(SchematicBuilder* self);

typedef struct SchematicBuilder_use_compact_palette_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_use_compact_palette_result;
SchematicBuilder_use_compact_palette_result SchematicBuilder_use_compact_palette(SchematicBuilder* self);

typedef struct SchematicBuilder_validate_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_validate_result;
SchematicBuilder_validate_result SchematicBuilder_validate(const SchematicBuilder* self);

typedef struct SchematicBuilder_to_template_result {union { NucleationError err;}; bool is_ok;} SchematicBuilder_to_template_result;
SchematicBuilder_to_template_result SchematicBuilder_to_template(const SchematicBuilder* self, DiplomatWrite* write);

typedef struct SchematicBuilder_build_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} SchematicBuilder_build_result;
SchematicBuilder_build_result SchematicBuilder_build(SchematicBuilder* self);

void SchematicBuilder_destroy(SchematicBuilder* self);





#endif // SchematicBuilder_H
