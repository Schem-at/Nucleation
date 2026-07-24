#ifndef SdfCellMode_D_H
#define SdfCellMode_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum SdfCellMode {
  SdfCellMode_F1 = 0,
  SdfCellMode_F2 = 1,
  SdfCellMode_F2MinusF1 = 2,
  SdfCellMode_CellValue = 3,
} SdfCellMode;

typedef struct SdfCellMode_option {union { SdfCellMode ok; }; bool is_ok; } SdfCellMode_option;



#endif // SdfCellMode_D_H
