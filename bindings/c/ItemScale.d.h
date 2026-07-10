#ifndef ItemScale_D_H
#define ItemScale_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct ItemScale {
  float x;
  float y;
  float z;
} ItemScale;

typedef struct ItemScale_option {union { ItemScale ok; }; bool is_ok; } ItemScale_option;



#endif // ItemScale_D_H
