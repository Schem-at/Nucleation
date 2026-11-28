#ifndef FieldNormal_D_H
#define FieldNormal_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct FieldNormal {
  float x;
  float y;
  float z;
} FieldNormal;

typedef struct FieldNormal_option {union { FieldNormal ok; }; bool is_ok; } FieldNormal_option;



#endif // FieldNormal_D_H
