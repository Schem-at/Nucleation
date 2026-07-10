#ifndef NucleationError_D_H
#define NucleationError_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum NucleationError {
  NucleationError_NullArgument = 0,
  NucleationError_InvalidArgument = 1,
  NucleationError_Parse = 2,
  NucleationError_Serialize = 3,
  NucleationError_Io = 4,
  NucleationError_Lock = 5,
  NucleationError_Store = 6,
  NucleationError_Mesh = 7,
  NucleationError_Render = 8,
  NucleationError_Simulation = 9,
  NucleationError_AlreadyConsumed = 10,
  NucleationError_NotFound = 11,
} NucleationError;

typedef struct NucleationError_option {union { NucleationError ok; }; bool is_ok; } NucleationError_option;



#endif // NucleationError_D_H
