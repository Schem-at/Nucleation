#ifndef FieldSample_D_H
#define FieldSample_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct FieldSample {
  float distance;
  uint16_t material_id;
  uint16_t blend_material_id;
  float blend_weight;
} FieldSample;

typedef struct FieldSample_option {union { FieldSample ok; }; bool is_ok; } FieldSample_option;



#endif // FieldSample_D_H
