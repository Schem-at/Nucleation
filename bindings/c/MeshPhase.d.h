#ifndef MeshPhase_D_H
#define MeshPhase_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum MeshPhase {
  MeshPhase_BuildingAtlas = 0,
  MeshPhase_MeshingChunks = 1,
  MeshPhase_Complete = 2,
  MeshPhase_Failed = 3,
} MeshPhase;

typedef struct MeshPhase_option {union { MeshPhase ok; }; bool is_ok; } MeshPhase_option;



#endif // MeshPhase_D_H
