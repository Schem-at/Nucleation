#ifndef SortStrategy_H
#define SortStrategy_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "SortStrategy.d.h"






SortStrategy* SortStrategy_yxz(void);

SortStrategy* SortStrategy_xyz(void);

SortStrategy* SortStrategy_zyx(void);

SortStrategy* SortStrategy_y_desc_xz(void);

SortStrategy* SortStrategy_x_desc_yz(void);

SortStrategy* SortStrategy_z_desc_yx(void);

SortStrategy* SortStrategy_descending(void);

SortStrategy* SortStrategy_distance_from(int32_t x, int32_t y, int32_t z);

SortStrategy* SortStrategy_distance_from_desc(int32_t x, int32_t y, int32_t z);

SortStrategy* SortStrategy_preserve(void);

SortStrategy* SortStrategy_reverse(void);

typedef struct SortStrategy_from_string_result {union {SortStrategy* ok; NucleationError err;}; bool is_ok;} SortStrategy_from_string_result;
SortStrategy_from_string_result SortStrategy_from_string(DiplomatStringView s);

void SortStrategy_name(const SortStrategy* self, DiplomatWrite* write);

void SortStrategy_destroy(SortStrategy* self);





#endif // SortStrategy_H
