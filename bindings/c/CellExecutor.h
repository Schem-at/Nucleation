#ifndef CellExecutor_H
#define CellExecutor_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"
#include "Value.d.h"

#include "CellExecutor.d.h"






typedef struct CellExecutor_for_schematic_result {union {CellExecutor* ok; NucleationError err;}; bool is_ok;} CellExecutor_for_schematic_result;
CellExecutor_for_schematic_result CellExecutor_for_schematic(const Schematic* schematic);

typedef struct CellExecutor_set_input_result {union { NucleationError err;}; bool is_ok;} CellExecutor_set_input_result;
CellExecutor_set_input_result CellExecutor_set_input(CellExecutor* self, DiplomatStringView name, const Value* value);

bool CellExecutor_settle(CellExecutor* self, uint32_t budget);

typedef struct CellExecutor_read_output_result {union {Value* ok; NucleationError err;}; bool is_ok;} CellExecutor_read_output_result;
CellExecutor_read_output_result CellExecutor_read_output(CellExecutor* self, DiplomatStringView name);

typedef struct CellExecutor_reset_result {union { NucleationError err;}; bool is_ok;} CellExecutor_reset_result;
CellExecutor_reset_result CellExecutor_reset(CellExecutor* self);

void CellExecutor_destroy(CellExecutor* self);





#endif // CellExecutor_H
