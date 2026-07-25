#ifndef WsSegmentJob_H
#define WsSegmentJob_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "WsSegmentJob.d.h"






typedef struct WsSegmentJob_create_result {union {WsSegmentJob* ok; NucleationError err;}; bool is_ok;} WsSegmentJob_create_result;
WsSegmentJob_create_result WsSegmentJob_create(uint32_t cell_size, uint32_t closing_radius, uint64_t min_cluster_blocks, DiplomatStringView source_id, DiplomatStringView snapshot_id, int32_t min_y, int32_t max_y, int64_t extracted_at, float match_iou, bool hard_cut);

void WsSegmentJob_destroy(WsSegmentJob* self);





#endif // WsSegmentJob_H
