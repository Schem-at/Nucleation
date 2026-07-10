#ifndef BlockPos_D_H
#define BlockPos_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct BlockPos {
  int32_t x;
  int32_t y;
  int32_t z;
} BlockPos;

typedef struct BlockPos_option {union { BlockPos ok; }; bool is_ok; } BlockPos_option;
typedef struct DiplomatBlockPosView {
  const BlockPos* data;
  size_t len;
} DiplomatBlockPosView;

typedef struct DiplomatBlockPosViewMut {
  BlockPos* data;
  size_t len;
} DiplomatBlockPosViewMut;




#endif // BlockPos_D_H
