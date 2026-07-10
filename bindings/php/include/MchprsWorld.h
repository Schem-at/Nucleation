#ifndef MchprsWorld_H
#define MchprsWorld_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "RedstoneGraph.d.h"
#include "Schematic.d.h"

#include "MchprsWorld.d.h"






typedef struct MchprsWorld_create_result {union {MchprsWorld* ok; NucleationError err;}; bool is_ok;} MchprsWorld_create_result;
MchprsWorld_create_result MchprsWorld_create(const Schematic* schematic);

typedef struct MchprsWorld_create_with_options_result {union {MchprsWorld* ok; NucleationError err;}; bool is_ok;} MchprsWorld_create_with_options_result;
MchprsWorld_create_with_options_result MchprsWorld_create_with_options(const Schematic* schematic, bool optimize, bool io_only);

typedef struct MchprsWorld_create_with_custom_io_result {union {MchprsWorld* ok; NucleationError err;}; bool is_ok;} MchprsWorld_create_with_custom_io_result;
MchprsWorld_create_with_custom_io_result MchprsWorld_create_with_custom_io(const Schematic* schematic, bool optimize, bool io_only, DiplomatI32View custom_io_positions);

typedef struct MchprsWorld_simulate_use_block_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} MchprsWorld_simulate_use_block_result;
MchprsWorld_simulate_use_block_result MchprsWorld_simulate_use_block(const Schematic* schematic, uint32_t ticks, DiplomatI32View events_xyz);

void MchprsWorld_tick(MchprsWorld* self, uint32_t ticks);

void MchprsWorld_flush(MchprsWorld* self);

void MchprsWorld_set_lever_power(MchprsWorld* self, int32_t x, int32_t y, int32_t z, bool powered);

bool MchprsWorld_get_lever_power(const MchprsWorld* self, int32_t x, int32_t y, int32_t z);

bool MchprsWorld_is_lit(const MchprsWorld* self, int32_t x, int32_t y, int32_t z);

void MchprsWorld_set_signal_strength(MchprsWorld* self, int32_t x, int32_t y, int32_t z, uint8_t strength);

uint8_t MchprsWorld_get_signal_strength(const MchprsWorld* self, int32_t x, int32_t y, int32_t z);

void MchprsWorld_on_use_block(MchprsWorld* self, int32_t x, int32_t y, int32_t z);

void MchprsWorld_sync_to_schematic(MchprsWorld* self);

Schematic* MchprsWorld_get_schematic(const MchprsWorld* self);

uint8_t MchprsWorld_get_redstone_power(const MchprsWorld* self, int32_t x, int32_t y, int32_t z);

void MchprsWorld_check_custom_io_changes(MchprsWorld* self);

void MchprsWorld_poll_custom_io_changes_json(MchprsWorld* self, DiplomatWrite* write);

void MchprsWorld_peek_custom_io_changes_json(const MchprsWorld* self, DiplomatWrite* write);

void MchprsWorld_clear_custom_io_changes(MchprsWorld* self);

typedef struct MchprsWorld_export_graph_result {union {RedstoneGraph* ok; NucleationError err;}; bool is_ok;} MchprsWorld_export_graph_result;
MchprsWorld_export_graph_result MchprsWorld_export_graph(const MchprsWorld* self);

typedef struct MchprsWorld_export_graph_structural_result {union {RedstoneGraph* ok; NucleationError err;}; bool is_ok;} MchprsWorld_export_graph_structural_result;
MchprsWorld_export_graph_structural_result MchprsWorld_export_graph_structural(const MchprsWorld* self);

void MchprsWorld_destroy(MchprsWorld* self);





#endif // MchprsWorld_H
