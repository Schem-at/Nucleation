#ifndef FieldProgramUnaryOp_D_H
#define FieldProgramUnaryOp_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldProgramUnaryOp {
  FieldProgramUnaryOp_Neg = 0,
  FieldProgramUnaryOp_Abs = 1,
  FieldProgramUnaryOp_Sqrt = 2,
  FieldProgramUnaryOp_Log = 3,
  FieldProgramUnaryOp_Sin = 4,
  FieldProgramUnaryOp_Cos = 5,
  FieldProgramUnaryOp_Acos = 6,
  FieldProgramUnaryOp_VecX = 7,
  FieldProgramUnaryOp_VecY = 8,
  FieldProgramUnaryOp_VecZ = 9,
  FieldProgramUnaryOp_Length = 10,
  FieldProgramUnaryOp_Normalize = 11,
} FieldProgramUnaryOp;

typedef struct FieldProgramUnaryOp_option {union { FieldProgramUnaryOp ok; }; bool is_ok; } FieldProgramUnaryOp_option;



#endif // FieldProgramUnaryOp_D_H
