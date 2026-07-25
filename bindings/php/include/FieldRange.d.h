#ifndef FieldRange_D_H
#define FieldRange_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct FieldRange {
  float min;
  float max;
} FieldRange;

typedef struct FieldRange_option {union { FieldRange ok; }; bool is_ok; } FieldRange_option;



#endif // FieldRange_D_H
