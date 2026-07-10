#ifndef RegionBounds_D_H
#define RegionBounds_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct RegionBounds {
  int32_t min_x;
  int32_t min_y;
  int32_t min_z;
  int32_t max_x;
  int32_t max_y;
  int32_t max_z;
} RegionBounds;

typedef struct RegionBounds_option {union { RegionBounds ok; }; bool is_ok; } RegionBounds_option;
typedef struct DiplomatRegionBoundsView {
  const RegionBounds* data;
  size_t len;
} DiplomatRegionBoundsView;

typedef struct DiplomatRegionBoundsViewMut {
  RegionBounds* data;
  size_t len;
} DiplomatRegionBoundsViewMut;




#endif // RegionBounds_D_H
