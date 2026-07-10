#ifndef TypedCircuitExecutor_H
#define TypedCircuitExecutor_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "ExecutionMode.d.h"
#include "IoLayout.d.h"
#include "MchprsWorld.d.h"
#include "NucleationError.d.h"
#include "Schematic.d.h"
#include "Value.d.h"

#include "TypedCircuitExecutor.d.h"






typedef struct TypedCircuitExecutor_from_layout_result {union {TypedCircuitExecutor* ok; NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_layout_result;
TypedCircuitExecutor_from_layout_result TypedCircuitExecutor_from_layout(const MchprsWorld* world, const IoLayout* layout);

typedef struct TypedCircuitExecutor_from_layout_with_options_result {union {TypedCircuitExecutor* ok; NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_layout_with_options_result;
TypedCircuitExecutor_from_layout_with_options_result TypedCircuitExecutor_from_layout_with_options(const MchprsWorld* world, const IoLayout* layout, bool optimize, bool io_only);

typedef struct TypedCircuitExecutor_from_insign_result {union {TypedCircuitExecutor* ok; NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_insign_result;
TypedCircuitExecutor_from_insign_result TypedCircuitExecutor_from_insign(const Schematic* schematic);

typedef struct TypedCircuitExecutor_from_insign_with_options_result {union {TypedCircuitExecutor* ok; NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_insign_with_options_result;
TypedCircuitExecutor_from_insign_with_options_result TypedCircuitExecutor_from_insign_with_options(const Schematic* schematic, bool optimize, bool io_only);

typedef struct TypedCircuitExecutor_set_state_mode_result {union { NucleationError err;}; bool is_ok;} TypedCircuitExecutor_set_state_mode_result;
TypedCircuitExecutor_set_state_mode_result TypedCircuitExecutor_set_state_mode(TypedCircuitExecutor* self, DiplomatStringView mode);

typedef struct TypedCircuitExecutor_reset_result {union { NucleationError err;}; bool is_ok;} TypedCircuitExecutor_reset_result;
TypedCircuitExecutor_reset_result TypedCircuitExecutor_reset(TypedCircuitExecutor* self);

void TypedCircuitExecutor_tick(TypedCircuitExecutor* self, uint32_t ticks);

void TypedCircuitExecutor_flush(TypedCircuitExecutor* self);

typedef struct TypedCircuitExecutor_set_input_result {union { NucleationError err;}; bool is_ok;} TypedCircuitExecutor_set_input_result;
TypedCircuitExecutor_set_input_result TypedCircuitExecutor_set_input(TypedCircuitExecutor* self, DiplomatStringView name, const Value* value);

typedef struct TypedCircuitExecutor_read_output_result {union {Value* ok; NucleationError err;}; bool is_ok;} TypedCircuitExecutor_read_output_result;
TypedCircuitExecutor_read_output_result TypedCircuitExecutor_read_output(TypedCircuitExecutor* self, DiplomatStringView name);

typedef struct TypedCircuitExecutor_execute_result {union { NucleationError err;}; bool is_ok;} TypedCircuitExecutor_execute_result;
TypedCircuitExecutor_execute_result TypedCircuitExecutor_execute(TypedCircuitExecutor* self, DiplomatStringView inputs_json, const ExecutionMode* mode, DiplomatWrite* write);

void TypedCircuitExecutor_input_names_json(const TypedCircuitExecutor* self, DiplomatWrite* write);

void TypedCircuitExecutor_output_names_json(const TypedCircuitExecutor* self, DiplomatWrite* write);

void TypedCircuitExecutor_layout_info_json(const TypedCircuitExecutor* self, DiplomatWrite* write);

Schematic* TypedCircuitExecutor_sync_to_schematic(TypedCircuitExecutor* self);

void TypedCircuitExecutor_destroy(TypedCircuitExecutor* self);





#endif // TypedCircuitExecutor_H
