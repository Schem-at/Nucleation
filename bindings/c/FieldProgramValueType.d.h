#ifndef FieldProgramValueType_D_H
#define FieldProgramValueType_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldProgramValueType {
  FieldProgramValueType_Scalar = 0,
  FieldProgramValueType_Vec3 = 1,
  FieldProgramValueType_Bool = 2,
} FieldProgramValueType;

typedef struct FieldProgramValueType_option {union { FieldProgramValueType ok; }; bool is_ok; } FieldProgramValueType_option;



#endif // FieldProgramValueType_D_H
