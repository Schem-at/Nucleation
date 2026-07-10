#ifndef OutputCondition_H
#define OutputCondition_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "Value.d.h"

#include "OutputCondition.d.h"






OutputCondition* OutputCondition_equals(const Value* value);

OutputCondition* OutputCondition_not_equals(const Value* value);

OutputCondition* OutputCondition_greater_than(const Value* value);

OutputCondition* OutputCondition_less_than(const Value* value);

OutputCondition* OutputCondition_bitwise_and(uint32_t mask);

void OutputCondition_destroy(OutputCondition* self);





#endif // OutputCondition_H
