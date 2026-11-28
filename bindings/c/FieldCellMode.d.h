#ifndef FieldCellMode_D_H
#define FieldCellMode_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldCellMode {
  FieldCellMode_F1 = 0,
  FieldCellMode_F2 = 1,
  FieldCellMode_F2MinusF1 = 2,
  FieldCellMode_CellValue = 3,
} FieldCellMode;

typedef struct FieldCellMode_option {union { FieldCellMode ok; }; bool is_ok; } FieldCellMode_option;



#endif // FieldCellMode_D_H
