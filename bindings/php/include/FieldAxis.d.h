#ifndef FieldAxis_D_H
#define FieldAxis_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldAxis {
  FieldAxis_X = 0,
  FieldAxis_Y = 1,
  FieldAxis_Z = 2,
} FieldAxis;

typedef struct FieldAxis_option {union { FieldAxis ok; }; bool is_ok; } FieldAxis_option;



#endif // FieldAxis_D_H
