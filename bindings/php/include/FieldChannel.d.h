#ifndef FieldChannel_D_H
#define FieldChannel_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldChannel {
  FieldChannel_WorldX = 0,
  FieldChannel_WorldY = 1,
  FieldChannel_WorldZ = 2,
  FieldChannel_Distance = 3,
  FieldChannel_DepthBelowSurface = 4,
  FieldChannel_Slope = 5,
  FieldChannel_NormalY = 6,
  FieldChannel_Curvature = 7,
} FieldChannel;

typedef struct FieldChannel_option {union { FieldChannel ok; }; bool is_ok; } FieldChannel_option;



#endif // FieldChannel_D_H
