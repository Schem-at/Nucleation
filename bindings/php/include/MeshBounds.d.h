#ifndef MeshBounds_D_H
#define MeshBounds_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct MeshBounds {
  float min_x;
  float min_y;
  float min_z;
  float max_x;
  float max_y;
  float max_z;
} MeshBounds;

typedef struct MeshBounds_option {union { MeshBounds ok; }; bool is_ok; } MeshBounds_option;



#endif // MeshBounds_D_H
