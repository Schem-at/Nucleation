#ifndef CircuitBuilder_H
#define CircuitBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "IoType.d.h"
#include "LayoutFunction.d.h"
#include "NucleationError.d.h"
#include "Schematic.d.h"
#include "SortStrategy.d.h"
#include "TypedCircuitExecutor.d.h"

#include "CircuitBuilder.d.h"






CircuitBuilder* CircuitBuilder_create(const Schematic* schematic);

typedef struct CircuitBuilder_from_insign_result {union {CircuitBuilder* ok; NucleationError err;}; bool is_ok;} CircuitBuilder_from_insign_result;
CircuitBuilder_from_insign_result CircuitBuilder_from_insign(const Schematic* schematic);

typedef struct CircuitBuilder_with_input_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_result;
CircuitBuilder_with_input_result CircuitBuilder_with_input(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View region_positions);

typedef struct CircuitBuilder_with_input_sorted_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_sorted_result;
CircuitBuilder_with_input_sorted_result CircuitBuilder_with_input_sorted(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View region_positions, const SortStrategy* sort);

typedef struct CircuitBuilder_with_input_auto_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_auto_result;
CircuitBuilder_with_input_auto_result CircuitBuilder_with_input_auto(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View region_positions);

typedef struct CircuitBuilder_with_input_auto_sorted_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_auto_sorted_result;
CircuitBuilder_with_input_auto_sorted_result CircuitBuilder_with_input_auto_sorted(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View region_positions, const SortStrategy* sort);

typedef struct CircuitBuilder_with_output_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_result;
CircuitBuilder_with_output_result CircuitBuilder_with_output(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View region_positions);

typedef struct CircuitBuilder_with_output_sorted_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_sorted_result;
CircuitBuilder_with_output_sorted_result CircuitBuilder_with_output_sorted(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View region_positions, const SortStrategy* sort);

typedef struct CircuitBuilder_with_output_auto_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_auto_result;
CircuitBuilder_with_output_auto_result CircuitBuilder_with_output_auto(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View region_positions);

typedef struct CircuitBuilder_with_output_auto_sorted_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_auto_sorted_result;
CircuitBuilder_with_output_auto_sorted_result CircuitBuilder_with_output_auto_sorted(CircuitBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View region_positions, const SortStrategy* sort);

typedef struct CircuitBuilder_with_options_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_options_result;
CircuitBuilder_with_options_result CircuitBuilder_with_options(CircuitBuilder* self, bool optimize, bool io_only);

typedef struct CircuitBuilder_with_state_mode_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_with_state_mode_result;
CircuitBuilder_with_state_mode_result CircuitBuilder_with_state_mode(CircuitBuilder* self, DiplomatStringView mode);

typedef struct CircuitBuilder_validate_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_validate_result;
CircuitBuilder_validate_result CircuitBuilder_validate(const CircuitBuilder* self);

typedef struct CircuitBuilder_build_result {union {TypedCircuitExecutor* ok; NucleationError err;}; bool is_ok;} CircuitBuilder_build_result;
CircuitBuilder_build_result CircuitBuilder_build(CircuitBuilder* self);

typedef struct CircuitBuilder_build_validated_result {union {TypedCircuitExecutor* ok; NucleationError err;}; bool is_ok;} CircuitBuilder_build_validated_result;
CircuitBuilder_build_validated_result CircuitBuilder_build_validated(CircuitBuilder* self);

uint32_t CircuitBuilder_input_count(const CircuitBuilder* self);

uint32_t CircuitBuilder_output_count(const CircuitBuilder* self);

typedef struct CircuitBuilder_input_names_json_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_input_names_json_result;
CircuitBuilder_input_names_json_result CircuitBuilder_input_names_json(const CircuitBuilder* self, DiplomatWrite* write);

typedef struct CircuitBuilder_output_names_json_result {union { NucleationError err;}; bool is_ok;} CircuitBuilder_output_names_json_result;
CircuitBuilder_output_names_json_result CircuitBuilder_output_names_json(const CircuitBuilder* self, DiplomatWrite* write);

void CircuitBuilder_destroy(CircuitBuilder* self);





#endif // CircuitBuilder_H
