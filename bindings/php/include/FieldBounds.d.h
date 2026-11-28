#ifndef FieldBounds_D_H
#define FieldBounds_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct FieldBounds {
  float min_x;
  float min_y;
  float min_z;
  float max_x;
  float max_y;
  float max_z;
} FieldBounds;

typedef struct FieldBounds_option {union { FieldBounds ok; }; bool is_ok; } FieldBounds_option;



#endif // FieldBounds_D_H
