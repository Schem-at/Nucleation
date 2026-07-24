#ifndef FieldProgramDistanceKind_D_H
#define FieldProgramDistanceKind_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldProgramDistanceKind {
  FieldProgramDistanceKind_Exact = 0,
  FieldProgramDistanceKind_LowerBound = 1,
  FieldProgramDistanceKind_Estimate = 2,
  FieldProgramDistanceKind_Implicit = 3,
} FieldProgramDistanceKind;

typedef struct FieldProgramDistanceKind_option {union { FieldProgramDistanceKind ok; }; bool is_ok; } FieldProgramDistanceKind_option;



#endif // FieldProgramDistanceKind_D_H
