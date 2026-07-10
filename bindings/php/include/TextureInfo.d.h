#ifndef TextureInfo_D_H
#define TextureInfo_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct TextureInfo {
  uint32_t width;
  uint32_t height;
  bool animated;
  uint32_t frame_count;
} TextureInfo;

typedef struct TextureInfo_option {union { TextureInfo ok; }; bool is_ok; } TextureInfo_option;



#endif // TextureInfo_D_H
