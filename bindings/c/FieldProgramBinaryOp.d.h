#ifndef FieldProgramBinaryOp_D_H
#define FieldProgramBinaryOp_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldProgramBinaryOp {
  FieldProgramBinaryOp_Add = 0,
  FieldProgramBinaryOp_Sub = 1,
  FieldProgramBinaryOp_Mul = 2,
  FieldProgramBinaryOp_Div = 3,
  FieldProgramBinaryOp_Min = 4,
  FieldProgramBinaryOp_Max = 5,
  FieldProgramBinaryOp_Pow = 6,
  FieldProgramBinaryOp_Atan2 = 7,
  FieldProgramBinaryOp_Lt = 8,
  FieldProgramBinaryOp_Le = 9,
  FieldProgramBinaryOp_Gt = 10,
  FieldProgramBinaryOp_Ge = 11,
  FieldProgramBinaryOp_Eq = 12,
  FieldProgramBinaryOp_Dot = 13,
  FieldProgramBinaryOp_Cross = 14,
  FieldProgramBinaryOp_Scale = 15,
} FieldProgramBinaryOp;

typedef struct FieldProgramBinaryOp_option {union { FieldProgramBinaryOp ok; }; bool is_ok; } FieldProgramBinaryOp_option;



#endif // FieldProgramBinaryOp_D_H
