#ifndef FieldNoiseBasis_D_H
#define FieldNoiseBasis_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldNoiseBasis {
  FieldNoiseBasis_ValueNoise = 0,
  FieldNoiseBasis_Perlin = 1,
} FieldNoiseBasis;

typedef struct FieldNoiseBasis_option {union { FieldNoiseBasis ok; }; bool is_ok; } FieldNoiseBasis_option;



#endif // FieldNoiseBasis_D_H
