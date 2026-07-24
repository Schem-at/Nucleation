#ifndef VideoConfig_H
#define VideoConfig_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "VideoConfig.d.h"






typedef struct VideoConfig_prores_4444_result {union {VideoConfig* ok; NucleationError err;}; bool is_ok;} VideoConfig_prores_4444_result;
VideoConfig_prores_4444_result VideoConfig_prores_4444(double fps);

typedef struct VideoConfig_h264_result {union {VideoConfig* ok; NucleationError err;}; bool is_ok;} VideoConfig_h264_result;
VideoConfig_h264_result VideoConfig_h264(double fps);

typedef struct VideoConfig_set_ffmpeg_path_result {union { NucleationError err;}; bool is_ok;} VideoConfig_set_ffmpeg_path_result;
VideoConfig_set_ffmpeg_path_result VideoConfig_set_ffmpeg_path(VideoConfig* self, DiplomatStringView path);

void VideoConfig_destroy(VideoConfig* self);





#endif // VideoConfig_H
