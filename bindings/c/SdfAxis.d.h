#ifndef SdfAxis_D_H
#define SdfAxis_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum SdfAxis {
  SdfAxis_X = 0,
  SdfAxis_Y = 1,
  SdfAxis_Z = 2,
} SdfAxis;

typedef struct SdfAxis_option {union { SdfAxis ok; }; bool is_ok; } SdfAxis_option;



#endif // SdfAxis_D_H
