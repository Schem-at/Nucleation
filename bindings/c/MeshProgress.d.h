#ifndef MeshProgress_D_H
#define MeshProgress_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "MeshPhase.d.h"




typedef struct MeshProgress {
  MeshPhase phase;
  uint32_t current;
  uint32_t total;
} MeshProgress;

typedef struct MeshProgress_option {union { MeshProgress ok; }; bool is_ok; } MeshProgress_option;



#endif // MeshProgress_D_H
