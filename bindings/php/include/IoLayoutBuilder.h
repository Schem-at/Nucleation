#ifndef IoLayoutBuilder_H
#define IoLayoutBuilder_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "IoLayout.d.h"
#include "IoType.d.h"
#include "LayoutFunction.d.h"
#include "NucleationError.d.h"

#include "IoLayoutBuilder.d.h"






IoLayoutBuilder* IoLayoutBuilder_create(void);

typedef struct IoLayoutBuilder_add_input_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_result;
IoLayoutBuilder_add_input_result IoLayoutBuilder_add_input(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View positions);

typedef struct IoLayoutBuilder_add_output_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_result;
IoLayoutBuilder_add_output_result IoLayoutBuilder_add_output(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View positions);

typedef struct IoLayoutBuilder_add_input_auto_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_auto_result;
IoLayoutBuilder_add_input_auto_result IoLayoutBuilder_add_input_auto(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View positions);

typedef struct IoLayoutBuilder_add_output_auto_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_auto_result;
IoLayoutBuilder_add_output_auto_result IoLayoutBuilder_add_output_auto(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View positions);

typedef struct IoLayoutBuilder_add_input_from_region_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_from_region_result;
IoLayoutBuilder_add_input_from_region_result IoLayoutBuilder_add_input_from_region(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View region_positions);

typedef struct IoLayoutBuilder_add_input_from_region_auto_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_from_region_auto_result;
IoLayoutBuilder_add_input_from_region_auto_result IoLayoutBuilder_add_input_from_region_auto(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View region_positions);

typedef struct IoLayoutBuilder_add_output_from_region_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_from_region_result;
IoLayoutBuilder_add_output_from_region_result IoLayoutBuilder_add_output_from_region(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, const LayoutFunction* layout, DiplomatI32View region_positions);

typedef struct IoLayoutBuilder_add_output_from_region_auto_result {union { NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_from_region_auto_result;
IoLayoutBuilder_add_output_from_region_auto_result IoLayoutBuilder_add_output_from_region_auto(IoLayoutBuilder* self, DiplomatStringView name, const IoType* io_type, DiplomatI32View region_positions);

typedef struct IoLayoutBuilder_build_result {union {IoLayout* ok; NucleationError err;}; bool is_ok;} IoLayoutBuilder_build_result;
IoLayoutBuilder_build_result IoLayoutBuilder_build(IoLayoutBuilder* self);

void IoLayoutBuilder_destroy(IoLayoutBuilder* self);





#endif // IoLayoutBuilder_H
