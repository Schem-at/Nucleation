#ifndef ExecutionMode_H
#define ExecutionMode_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "OutputCondition.d.h"

#include "ExecutionMode.d.h"






ExecutionMode* ExecutionMode_fixed_ticks(uint32_t ticks);

typedef struct ExecutionMode_until_condition_result {union {ExecutionMode* ok; NucleationError err;}; bool is_ok;} ExecutionMode_until_condition_result;
ExecutionMode_until_condition_result ExecutionMode_until_condition(DiplomatStringView output_name, const OutputCondition* condition, uint32_t max_ticks, uint32_t check_interval);

ExecutionMode* ExecutionMode_until_change(uint32_t max_ticks, uint32_t check_interval);

ExecutionMode* ExecutionMode_until_stable(uint32_t stable_ticks, uint32_t max_ticks);

void ExecutionMode_destroy(ExecutionMode* self);





#endif // ExecutionMode_H
