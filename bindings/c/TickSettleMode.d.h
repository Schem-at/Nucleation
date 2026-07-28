#ifndef TickSettleMode_D_H
#define TickSettleMode_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum TickSettleMode {
  TickSettleMode_Placement = 0,
  TickSettleMode_Quiet = 1,
  TickSettleMode_InWorld = 2,
} TickSettleMode;

typedef struct TickSettleMode_option {union { TickSettleMode ok; }; bool is_ok; } TickSettleMode_option;



#endif // TickSettleMode_D_H
