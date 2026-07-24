#ifndef SdfNormal_D_H
#define SdfNormal_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct SdfNormal {
  float x;
  float y;
  float z;
} SdfNormal;

typedef struct SdfNormal_option {union { SdfNormal ok; }; bool is_ok; } SdfNormal_option;



#endif // SdfNormal_D_H
