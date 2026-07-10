#ifndef InterpolationSpace_D_H
#define InterpolationSpace_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum InterpolationSpace {
  InterpolationSpace_Rgb = 0,
  InterpolationSpace_Oklab = 1,
} InterpolationSpace;

typedef struct InterpolationSpace_option {union { InterpolationSpace ok; }; bool is_ok; } InterpolationSpace_option;



#endif // InterpolationSpace_D_H
