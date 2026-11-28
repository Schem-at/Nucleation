#ifndef FieldDistanceKind_D_H
#define FieldDistanceKind_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum FieldDistanceKind {
  FieldDistanceKind_Exact = 0,
  FieldDistanceKind_LowerBound = 1,
  FieldDistanceKind_Estimate = 2,
  FieldDistanceKind_Implicit = 3,
} FieldDistanceKind;

typedef struct FieldDistanceKind_option {union { FieldDistanceKind ok; }; bool is_ok; } FieldDistanceKind_option;



#endif // FieldDistanceKind_D_H
