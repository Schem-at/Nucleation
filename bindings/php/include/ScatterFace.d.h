#ifndef ScatterFace_D_H
#define ScatterFace_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum ScatterFace {
  ScatterFace_Top = 0,
  ScatterFace_Bottom = 1,
  ScatterFace_North = 2,
  ScatterFace_South = 3,
  ScatterFace_East = 4,
  ScatterFace_West = 5,
} ScatterFace;

typedef struct ScatterFace_option {union { ScatterFace ok; }; bool is_ok; } ScatterFace_option;



#endif // ScatterFace_D_H
