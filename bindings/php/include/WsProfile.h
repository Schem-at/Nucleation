#ifndef WsProfile_H
#define WsProfile_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "WsProfile.d.h"






typedef struct WsProfile_derive_from_dir_result {union {WsProfile* ok; NucleationError err;}; bool is_ok;} WsProfile_derive_from_dir_result;
WsProfile_derive_from_dir_result WsProfile_derive_from_dir(DiplomatStringView world_dir, int32_t min_y, int32_t max_y, uint32_t sample, float coverage);

int32_t WsProfile_band_min(const WsProfile* self);

int32_t WsProfile_band_max(const WsProfile* self);

uint32_t WsProfile_palette_len(const WsProfile* self);

typedef struct WsProfile_write_palette_json_result {union { NucleationError err;}; bool is_ok;} WsProfile_write_palette_json_result;
WsProfile_write_palette_json_result WsProfile_write_palette_json(const WsProfile* self, DiplomatWrite* write);

void WsProfile_destroy(WsProfile* self);





#endif // WsProfile_H
