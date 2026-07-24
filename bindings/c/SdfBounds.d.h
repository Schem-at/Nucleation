#ifndef SdfBounds_D_H
#define SdfBounds_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct SdfBounds {
  float min_x;
  float min_y;
  float min_z;
  float max_x;
  float max_y;
  float max_z;
} SdfBounds;

typedef struct SdfBounds_option {union { SdfBounds ok; }; bool is_ok; } SdfBounds_option;



#endif // SdfBounds_D_H
