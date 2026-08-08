#ifndef TickSimulation_H
#define TickSimulation_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"
#include "TickSettleMode.d.h"

#include "TickSimulation.d.h"






void TickSimulation_last_error_detail(DiplomatWrite* write);

uint32_t TickSimulation_max_volume(void);

typedef struct TickSimulation_from_snbt_result {union {TickSimulation* ok; NucleationError err;}; bool is_ok;} TickSimulation_from_snbt_result;
TickSimulation_from_snbt_result TickSimulation_from_snbt(DiplomatStringView snbt, TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, DiplomatStringView extra_states);

typedef struct TickSimulation_from_schematic_result {union {TickSimulation* ok; NucleationError err;}; bool is_ok;} TickSimulation_from_schematic_result;
TickSimulation_from_schematic_result TickSimulation_from_schematic(const Schematic* schematic, TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, DiplomatStringView extra_states);

typedef struct TickSimulation_from_blocks_result {union {TickSimulation* ok; NucleationError err;}; bool is_ok;} TickSimulation_from_blocks_result;
TickSimulation_from_blocks_result TickSimulation_from_blocks(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, DiplomatStringView palette, DiplomatU16View cells, uint16_t air_index, TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z);

typedef struct TickSimulation_eval_flight_batch_result {union { NucleationError err;}; bool is_ok;} TickSimulation_eval_flight_batch_result;
TickSimulation_eval_flight_batch_result TickSimulation_eval_flight_batch(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, DiplomatStringView palette, DiplomatU16View cells, uint16_t air_index, DiplomatI32View kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit, DiplomatWrite* write);

void TickSimulation_set_rng_seed(TickSimulation* self, int64_t seed);

void TickSimulation_step(TickSimulation* self);

void TickSimulation_run(TickSimulation* self, uint32_t ticks);

bool TickSimulation_run_until_quiescent(TickSimulation* self, uint32_t budget);

uint32_t TickSimulation_tick_count(const TickSimulation* self);

bool TickSimulation_is_quiescent(const TickSimulation* self);

void TickSimulation_use_block(TickSimulation* self, int32_t x, int32_t y, int32_t z);

typedef struct TickSimulation_place_block_result {union { NucleationError err;}; bool is_ok;} TickSimulation_place_block_result;
TickSimulation_place_block_result TickSimulation_place_block(TickSimulation* self, int32_t x, int32_t y, int32_t z, DiplomatStringView state);

void TickSimulation_get_block(const TickSimulation* self, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct TickSimulation_read_probes_result {union { NucleationError err;}; bool is_ok;} TickSimulation_read_probes_result;
TickSimulation_read_probes_result TickSimulation_read_probes(const TickSimulation* self, DiplomatStringView positions_json, DiplomatWrite* write);

void TickSimulation_conduction_trace(const TickSimulation* self, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

uint32_t TickSimulation_bake_to(const TickSimulation* self, Schematic* schematic);

uint32_t TickSimulation_checkpoint(TickSimulation* self);

typedef struct TickSimulation_restore_result {union { NucleationError err;}; bool is_ok;} TickSimulation_restore_result;
TickSimulation_restore_result TickSimulation_restore(TickSimulation* self, uint32_t id);

void TickSimulation_gametest_snbt(const Schematic* schematic, DiplomatWrite* write);

void TickSimulation_block_entity_audit_json(const Schematic* schematic, DiplomatWrite* write);

void TickSimulation_record_updates(TickSimulation* self, bool on);

void TickSimulation_clear_updates(TickSimulation* self);

void TickSimulation_record_timeline(TickSimulation* self);

void TickSimulation_stop_timeline(TickSimulation* self);

void TickSimulation_timeline_activity_json(const TickSimulation* self, DiplomatWrite* write);

void TickSimulation_timeline_cycles_json(const TickSimulation* self, DiplomatWrite* write);

typedef struct TickSimulation_animation_timeline_json_result {union { NucleationError err;}; bool is_ok;} TickSimulation_animation_timeline_json_result;
TickSimulation_animation_timeline_json_result TickSimulation_animation_timeline_json(const TickSimulation* self, uint32_t start_tick, uint32_t end_tick, float tick_ms, DiplomatWrite* write);

typedef struct TickSimulation_selection_schematic_b64_result {union { NucleationError err;}; bool is_ok;} TickSimulation_selection_schematic_b64_result;
TickSimulation_selection_schematic_b64_result TickSimulation_selection_schematic_b64(const TickSimulation* self, uint32_t start_tick, uint32_t end_tick, DiplomatWrite* write);

uint32_t TickSimulation_updates_count(const TickSimulation* self);

void TickSimulation_updates_json(const TickSimulation* self, DiplomatWrite* write);

void TickSimulation_updates_json_between(const TickSimulation* self, uint32_t from_tick, uint32_t to_tick, DiplomatWrite* write);

void TickSimulation_updates_heat_json(const TickSimulation* self, uint32_t from_tick, uint32_t to_tick, DiplomatWrite* write);

void TickSimulation_updates_wave_json(const TickSimulation* self, uint32_t tick, DiplomatWrite* write);

void TickSimulation_moving_blocks_json(const TickSimulation* self, DiplomatWrite* write);

bool TickSimulation_clear_changes(TickSimulation* self);

void TickSimulation_changes_json(const TickSimulation* self, DiplomatWrite* write);

void TickSimulation_changes_json_from(const TickSimulation* self, uint32_t start, DiplomatWrite* write);

void TickSimulation_item_entities_json(const TickSimulation* self, DiplomatWrite* write);

void TickSimulation_motion_semantics(const TickSimulation* self, DiplomatWrite* write);

uint32_t TickSimulation_piston_retract_contacts(const TickSimulation* self);

void TickSimulation_events_summary_json(const TickSimulation* self, DiplomatWrite* write);

uint32_t TickSimulation_non_air_count(const TickSimulation* self);

double TickSimulation_non_air_center_x(const TickSimulation* self);

int32_t TickSimulation_non_air_min_x(const TickSimulation* self);

int32_t TickSimulation_non_air_max_x(const TickSimulation* self);

uint32_t TickSimulation_changes_count(const TickSimulation* self);

void TickSimulation_world_snapshot_json(const TickSimulation* self, DiplomatWrite* write);

void TickSimulation_machine_graph_json(const TickSimulation* self, DiplomatWrite* write);

typedef struct TickSimulation_machine_graph_batch_json_result {union { NucleationError err;}; bool is_ok;} TickSimulation_machine_graph_batch_json_result;
TickSimulation_machine_graph_batch_json_result TickSimulation_machine_graph_batch_json(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, DiplomatStringView palette, DiplomatU16View cells, uint16_t air_index, DiplomatWrite* write);

void TickSimulation_destroy(TickSimulation* self);





#endif // TickSimulation_H
