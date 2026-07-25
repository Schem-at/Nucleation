#ifndef GeneratedChunkCoverage_D_H
#define GeneratedChunkCoverage_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum GeneratedChunkCoverage {
  GeneratedChunkCoverage_Complete = 0,
  GeneratedChunkCoverage_Partial = 1,
  GeneratedChunkCoverage_Outside = 2,
} GeneratedChunkCoverage;

typedef struct GeneratedChunkCoverage_option {union { GeneratedChunkCoverage ok; }; bool is_ok; } GeneratedChunkCoverage_option;



#endif // GeneratedChunkCoverage_D_H
