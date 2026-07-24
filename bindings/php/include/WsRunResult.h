#ifndef WsRunResult_H
#define WsRunResult_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "BlockPos.d.h"
#include "NucleationError.d.h"
#include "WsPartitionHints.d.h"
#include "WsProfile.d.h"
#include "WsSegmentJob.d.h"

#include "WsRunResult.d.h"






typedef struct WsRunResult_run_dir_result {union {WsRunResult* ok; NucleationError err;}; bool is_ok;} WsRunResult_run_dir_result;
WsRunResult_run_dir_result WsRunResult_run_dir(const WsSegmentJob* job, const WsPartitionHints* hints, const WsProfile* profile, DiplomatStringView world_dir);

uint64_t WsRunResult_builds(const WsRunResult* self);

uint64_t WsRunResult_tier_confident(const WsRunResult* self);

uint64_t WsRunResult_tier_probable(const WsRunResult* self);

uint64_t WsRunResult_tier_debris(const WsRunResult* self);

uint64_t WsRunResult_cross_tile(const WsRunResult* self);

uint64_t WsRunResult_largest_block_count(const WsRunResult* self);

uint32_t WsRunResult_build_count(const WsRunResult* self);

typedef struct WsRunResult_stable_id_hex_result {union { NucleationError err;}; bool is_ok;} WsRunResult_stable_id_hex_result;
WsRunResult_stable_id_hex_result WsRunResult_stable_id_hex(const WsRunResult* self, uint32_t index, DiplomatWrite* write);

typedef struct WsRunResult_fingerprint_hex_result {union { NucleationError err;}; bool is_ok;} WsRunResult_fingerprint_hex_result;
WsRunResult_fingerprint_hex_result WsRunResult_fingerprint_hex(const WsRunResult* self, uint32_t index, DiplomatWrite* write);

typedef struct WsRunResult_tier_of_result {union {uint8_t ok; NucleationError err;}; bool is_ok;} WsRunResult_tier_of_result;
WsRunResult_tier_of_result WsRunResult_tier_of(const WsRunResult* self, uint32_t index);

typedef struct WsRunResult_block_count_of_result {union {uint64_t ok; NucleationError err;}; bool is_ok;} WsRunResult_block_count_of_result;
WsRunResult_block_count_of_result WsRunResult_block_count_of(const WsRunResult* self, uint32_t index);

typedef struct WsRunResult_bbox_min_of_result {union {BlockPos ok; NucleationError err;}; bool is_ok;} WsRunResult_bbox_min_of_result;
WsRunResult_bbox_min_of_result WsRunResult_bbox_min_of(const WsRunResult* self, uint32_t index);

typedef struct WsRunResult_bbox_max_of_result {union {BlockPos ok; NucleationError err;}; bool is_ok;} WsRunResult_bbox_max_of_result;
WsRunResult_bbox_max_of_result WsRunResult_bbox_max_of(const WsRunResult* self, uint32_t index);

typedef struct WsRunResult_write_schem_to_result {union { NucleationError err;}; bool is_ok;} WsRunResult_write_schem_to_result;
WsRunResult_write_schem_to_result WsRunResult_write_schem_to(const WsRunResult* self, uint32_t index, DiplomatStringView path);

void WsRunResult_destroy(WsRunResult* self);





#endif // WsRunResult_H
