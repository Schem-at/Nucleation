#ifndef Dimensions_D_H
#define Dimensions_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct Dimensions {
  int32_t x;
  int32_t y;
  int32_t z;
} Dimensions;

typedef struct Dimensions_option {union { Dimensions ok; }; bool is_ok; } Dimensions_option;
typedef struct DiplomatDimensionsView {
  const Dimensions* data;
  size_t len;
} DiplomatDimensionsView;

typedef struct DiplomatDimensionsViewMut {
  Dimensions* data;
  size_t len;
} DiplomatDimensionsViewMut;




#endif // Dimensions_D_H
