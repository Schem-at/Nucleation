#ifndef LayoutFunction_H
#define LayoutFunction_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "LayoutFunction.d.h"






LayoutFunction* LayoutFunction_one_to_one(void);

LayoutFunction* LayoutFunction_packed4(void);

typedef struct LayoutFunction_custom_result {union {LayoutFunction* ok; NucleationError err;}; bool is_ok;} LayoutFunction_custom_result;
LayoutFunction_custom_result LayoutFunction_custom(DiplomatU32View mapping);

LayoutFunction* LayoutFunction_row_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

LayoutFunction* LayoutFunction_column_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

LayoutFunction* LayoutFunction_scanline(uint32_t width, uint32_t height, uint32_t bits_per_pixel);

void LayoutFunction_destroy(LayoutFunction* self);





#endif // LayoutFunction_H
