#ifndef NUCLEATION_TickSimulation_HPP
#define NUCLEATION_TickSimulation_HPP

#include "TickSimulation.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "TickSettleMode.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    void TickSimulation_last_error_detail(nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_max_volume(void);

    typedef struct TickSimulation_from_snbt_result {union {nucleation::capi::TickSimulation* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_from_snbt_result;
    TickSimulation_from_snbt_result TickSimulation_from_snbt(nucleation::diplomat::capi::DiplomatStringView snbt, nucleation::capi::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, nucleation::diplomat::capi::DiplomatStringView extra_states);

    typedef struct TickSimulation_from_schematic_result {union {nucleation::capi::TickSimulation* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_from_schematic_result;
    TickSimulation_from_schematic_result TickSimulation_from_schematic(const nucleation::capi::Schematic* schematic, nucleation::capi::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, nucleation::diplomat::capi::DiplomatStringView extra_states);

    typedef struct TickSimulation_from_blocks_result {union {nucleation::capi::TickSimulation* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_from_blocks_result;
    TickSimulation_from_blocks_result TickSimulation_from_blocks(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, nucleation::diplomat::capi::DiplomatStringView palette, nucleation::diplomat::capi::DiplomatU16View cells, uint16_t air_index, nucleation::capi::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z);

    typedef struct TickSimulation_eval_flight_batch_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_eval_flight_batch_result;
    TickSimulation_eval_flight_batch_result TickSimulation_eval_flight_batch(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, nucleation::diplomat::capi::DiplomatStringView palette, nucleation::diplomat::capi::DiplomatU16View cells, uint16_t air_index, nucleation::diplomat::capi::DiplomatI32View kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_set_rng_seed(nucleation::capi::TickSimulation* self, int64_t seed);

    void TickSimulation_step(nucleation::capi::TickSimulation* self);

    void TickSimulation_run(nucleation::capi::TickSimulation* self, uint32_t ticks);

    bool TickSimulation_run_until_quiescent(nucleation::capi::TickSimulation* self, uint32_t budget);

    uint32_t TickSimulation_tick_count(const nucleation::capi::TickSimulation* self);

    bool TickSimulation_is_quiescent(const nucleation::capi::TickSimulation* self);

    void TickSimulation_use_block(nucleation::capi::TickSimulation* self, int32_t x, int32_t y, int32_t z);

    typedef struct TickSimulation_place_block_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_place_block_result;
    TickSimulation_place_block_result TickSimulation_place_block(nucleation::capi::TickSimulation* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView state);

    void TickSimulation_get_block(const nucleation::capi::TickSimulation* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_checkpoint(nucleation::capi::TickSimulation* self);

    typedef struct TickSimulation_restore_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_restore_result;
    TickSimulation_restore_result TickSimulation_restore(nucleation::capi::TickSimulation* self, uint32_t id);

    void TickSimulation_gametest_snbt(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_block_entity_audit_json(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_record_updates(nucleation::capi::TickSimulation* self, bool on);

    void TickSimulation_clear_updates(nucleation::capi::TickSimulation* self);

    void TickSimulation_record_timeline(nucleation::capi::TickSimulation* self);

    void TickSimulation_stop_timeline(nucleation::capi::TickSimulation* self);

    void TickSimulation_timeline_activity_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_timeline_cycles_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct TickSimulation_animation_timeline_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_animation_timeline_json_result;
    TickSimulation_animation_timeline_json_result TickSimulation_animation_timeline_json(const nucleation::capi::TickSimulation* self, uint32_t start_tick, uint32_t end_tick, float tick_ms, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct TickSimulation_selection_schematic_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_selection_schematic_b64_result;
    TickSimulation_selection_schematic_b64_result TickSimulation_selection_schematic_b64(const nucleation::capi::TickSimulation* self, uint32_t start_tick, uint32_t end_tick, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_updates_count(const nucleation::capi::TickSimulation* self);

    void TickSimulation_updates_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_updates_json_between(const nucleation::capi::TickSimulation* self, uint32_t from_tick, uint32_t to_tick, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_updates_heat_json(const nucleation::capi::TickSimulation* self, uint32_t from_tick, uint32_t to_tick, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_updates_wave_json(const nucleation::capi::TickSimulation* self, uint32_t tick, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_moving_blocks_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    bool TickSimulation_clear_changes(nucleation::capi::TickSimulation* self);

    void TickSimulation_changes_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_changes_json_from(const nucleation::capi::TickSimulation* self, uint32_t start, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_item_entities_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_motion_semantics(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_piston_retract_contacts(const nucleation::capi::TickSimulation* self);

    void TickSimulation_events_summary_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_non_air_count(const nucleation::capi::TickSimulation* self);

    double TickSimulation_non_air_center_x(const nucleation::capi::TickSimulation* self);

    int32_t TickSimulation_non_air_min_x(const nucleation::capi::TickSimulation* self);

    int32_t TickSimulation_non_air_max_x(const nucleation::capi::TickSimulation* self);

    uint32_t TickSimulation_changes_count(const nucleation::capi::TickSimulation* self);

    void TickSimulation_world_snapshot_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_machine_graph_json(const nucleation::capi::TickSimulation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct TickSimulation_machine_graph_batch_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TickSimulation_machine_graph_batch_json_result;
    TickSimulation_machine_graph_batch_json_result TickSimulation_machine_graph_batch_json(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, nucleation::diplomat::capi::DiplomatStringView palette, nucleation::diplomat::capi::DiplomatU16View cells, uint16_t air_index, nucleation::diplomat::capi::DiplomatWrite* write);

    void TickSimulation_destroy(TickSimulation* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::string nucleation::TickSimulation::last_error_detail() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_last_error_detail(&write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::last_error_detail_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_last_error_detail(&write);
}

inline uint32_t nucleation::TickSimulation::max_volume() {
    auto result = nucleation::capi::TickSimulation_max_volume();
    return result;
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError> nucleation::TickSimulation::from_snbt(std::string_view snbt, nucleation::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, std::string_view extra_states) {
    auto result = nucleation::capi::TickSimulation_from_snbt({snbt.data(), snbt.size()},
        settle.AsFFI(),
        origin_x,
        origin_y,
        origin_z,
        {extra_states.data(), extra_states.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TickSimulation>>(std::unique_ptr<nucleation::TickSimulation>(nucleation::TickSimulation::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError> nucleation::TickSimulation::from_schematic(const nucleation::Schematic& schematic, nucleation::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, std::string_view extra_states) {
    auto result = nucleation::capi::TickSimulation_from_schematic(schematic.AsFFI(),
        settle.AsFFI(),
        origin_x,
        origin_y,
        origin_z,
        {extra_states.data(), extra_states.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TickSimulation>>(std::unique_ptr<nucleation::TickSimulation>(nucleation::TickSimulation::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError> nucleation::TickSimulation::from_blocks(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, nucleation::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z) {
    auto result = nucleation::capi::TickSimulation_from_blocks(bx,
        by,
        bz,
        travel,
        x_off,
        {palette.data(), palette.size()},
        {cells.data(), cells.size()},
        air_index,
        settle.AsFFI(),
        origin_x,
        origin_y,
        origin_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TickSimulation>>(std::unique_ptr<nucleation::TickSimulation>(nucleation::TickSimulation::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::TickSimulation::eval_flight_batch(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, nucleation::diplomat::span<const int32_t> kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::TickSimulation_eval_flight_batch(bx,
        by,
        bz,
        travel,
        x_off,
        {palette.data(), palette.size()},
        {cells.data(), cells.size()},
        air_index,
        {kicks.data(), kicks.size()},
        eval_ticks,
        seed,
        must_move_by_tick,
        need_period,
        early_exit,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TickSimulation::eval_flight_batch_write(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, nucleation::diplomat::span<const int32_t> kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::TickSimulation_eval_flight_batch(bx,
        by,
        bz,
        travel,
        x_off,
        {palette.data(), palette.size()},
        {cells.data(), cells.size()},
        air_index,
        {kicks.data(), kicks.size()},
        eval_ticks,
        seed,
        must_move_by_tick,
        need_period,
        early_exit,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::TickSimulation::set_rng_seed(int64_t seed) {
    nucleation::capi::TickSimulation_set_rng_seed(this->AsFFI(),
        seed);
}

inline void nucleation::TickSimulation::step() {
    nucleation::capi::TickSimulation_step(this->AsFFI());
}

inline void nucleation::TickSimulation::run(uint32_t ticks) {
    nucleation::capi::TickSimulation_run(this->AsFFI(),
        ticks);
}

inline bool nucleation::TickSimulation::run_until_quiescent(uint32_t budget) {
    auto result = nucleation::capi::TickSimulation_run_until_quiescent(this->AsFFI(),
        budget);
    return result;
}

inline uint32_t nucleation::TickSimulation::tick_count() const {
    auto result = nucleation::capi::TickSimulation_tick_count(this->AsFFI());
    return result;
}

inline bool nucleation::TickSimulation::is_quiescent() const {
    auto result = nucleation::capi::TickSimulation_is_quiescent(this->AsFFI());
    return result;
}

inline void nucleation::TickSimulation::use_block(int32_t x, int32_t y, int32_t z) {
    nucleation::capi::TickSimulation_use_block(this->AsFFI(),
        x,
        y,
        z);
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TickSimulation::place_block(int32_t x, int32_t y, int32_t z, std::string_view state) {
    auto result = nucleation::capi::TickSimulation_place_block(this->AsFFI(),
        x,
        y,
        z,
        {state.data(), state.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::TickSimulation::get_block(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_get_block(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::get_block_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_get_block(this->AsFFI(),
        x,
        y,
        z,
        &write);
}

inline uint32_t nucleation::TickSimulation::checkpoint() {
    auto result = nucleation::capi::TickSimulation_checkpoint(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TickSimulation::restore(uint32_t id) {
    auto result = nucleation::capi::TickSimulation_restore(this->AsFFI(),
        id);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::TickSimulation::gametest_snbt(const nucleation::Schematic& schematic) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_gametest_snbt(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::gametest_snbt_write(const nucleation::Schematic& schematic, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_gametest_snbt(schematic.AsFFI(),
        &write);
}

inline std::string nucleation::TickSimulation::block_entity_audit_json(const nucleation::Schematic& schematic) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_block_entity_audit_json(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::block_entity_audit_json_write(const nucleation::Schematic& schematic, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_block_entity_audit_json(schematic.AsFFI(),
        &write);
}

inline void nucleation::TickSimulation::record_updates(bool on) {
    nucleation::capi::TickSimulation_record_updates(this->AsFFI(),
        on);
}

inline void nucleation::TickSimulation::clear_updates() {
    nucleation::capi::TickSimulation_clear_updates(this->AsFFI());
}

inline void nucleation::TickSimulation::record_timeline() {
    nucleation::capi::TickSimulation_record_timeline(this->AsFFI());
}

inline void nucleation::TickSimulation::stop_timeline() {
    nucleation::capi::TickSimulation_stop_timeline(this->AsFFI());
}

inline std::string nucleation::TickSimulation::timeline_activity_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_timeline_activity_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::timeline_activity_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_timeline_activity_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::TickSimulation::timeline_cycles_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_timeline_cycles_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::timeline_cycles_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_timeline_cycles_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::TickSimulation::animation_timeline_json(uint32_t start_tick, uint32_t end_tick, float tick_ms) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::TickSimulation_animation_timeline_json(this->AsFFI(),
        start_tick,
        end_tick,
        tick_ms,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TickSimulation::animation_timeline_json_write(uint32_t start_tick, uint32_t end_tick, float tick_ms, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::TickSimulation_animation_timeline_json(this->AsFFI(),
        start_tick,
        end_tick,
        tick_ms,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::TickSimulation::selection_schematic_b64(uint32_t start_tick, uint32_t end_tick) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::TickSimulation_selection_schematic_b64(this->AsFFI(),
        start_tick,
        end_tick,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TickSimulation::selection_schematic_b64_write(uint32_t start_tick, uint32_t end_tick, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::TickSimulation_selection_schematic_b64(this->AsFFI(),
        start_tick,
        end_tick,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::TickSimulation::updates_count() const {
    auto result = nucleation::capi::TickSimulation_updates_count(this->AsFFI());
    return result;
}

inline std::string nucleation::TickSimulation::updates_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_updates_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::updates_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_updates_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::TickSimulation::updates_json_between(uint32_t from_tick, uint32_t to_tick) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_updates_json_between(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::updates_json_between_write(uint32_t from_tick, uint32_t to_tick, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_updates_json_between(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
}

inline std::string nucleation::TickSimulation::updates_heat_json(uint32_t from_tick, uint32_t to_tick) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_updates_heat_json(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::updates_heat_json_write(uint32_t from_tick, uint32_t to_tick, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_updates_heat_json(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
}

inline std::string nucleation::TickSimulation::updates_wave_json(uint32_t tick) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_updates_wave_json(this->AsFFI(),
        tick,
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::updates_wave_json_write(uint32_t tick, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_updates_wave_json(this->AsFFI(),
        tick,
        &write);
}

inline std::string nucleation::TickSimulation::moving_blocks_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_moving_blocks_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::moving_blocks_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_moving_blocks_json(this->AsFFI(),
        &write);
}

inline bool nucleation::TickSimulation::clear_changes() {
    auto result = nucleation::capi::TickSimulation_clear_changes(this->AsFFI());
    return result;
}

inline std::string nucleation::TickSimulation::changes_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_changes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::changes_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_changes_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::TickSimulation::changes_json_from(uint32_t start) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_changes_json_from(this->AsFFI(),
        start,
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::changes_json_from_write(uint32_t start, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_changes_json_from(this->AsFFI(),
        start,
        &write);
}

inline std::string nucleation::TickSimulation::item_entities_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_item_entities_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::item_entities_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_item_entities_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::TickSimulation::motion_semantics() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_motion_semantics(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::motion_semantics_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_motion_semantics(this->AsFFI(),
        &write);
}

inline uint32_t nucleation::TickSimulation::piston_retract_contacts() const {
    auto result = nucleation::capi::TickSimulation_piston_retract_contacts(this->AsFFI());
    return result;
}

inline std::string nucleation::TickSimulation::events_summary_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_events_summary_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::events_summary_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_events_summary_json(this->AsFFI(),
        &write);
}

inline uint32_t nucleation::TickSimulation::non_air_count() const {
    auto result = nucleation::capi::TickSimulation_non_air_count(this->AsFFI());
    return result;
}

inline double nucleation::TickSimulation::non_air_center_x() const {
    auto result = nucleation::capi::TickSimulation_non_air_center_x(this->AsFFI());
    return result;
}

inline int32_t nucleation::TickSimulation::non_air_min_x() const {
    auto result = nucleation::capi::TickSimulation_non_air_min_x(this->AsFFI());
    return result;
}

inline int32_t nucleation::TickSimulation::non_air_max_x() const {
    auto result = nucleation::capi::TickSimulation_non_air_max_x(this->AsFFI());
    return result;
}

inline uint32_t nucleation::TickSimulation::changes_count() const {
    auto result = nucleation::capi::TickSimulation_changes_count(this->AsFFI());
    return result;
}

inline std::string nucleation::TickSimulation::world_snapshot_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_world_snapshot_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::world_snapshot_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_world_snapshot_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::TickSimulation::machine_graph_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TickSimulation_machine_graph_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TickSimulation::machine_graph_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TickSimulation_machine_graph_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::TickSimulation::machine_graph_batch_json(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::TickSimulation_machine_graph_batch_json(bx,
        by,
        bz,
        travel,
        x_off,
        {palette.data(), palette.size()},
        {cells.data(), cells.size()},
        air_index,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TickSimulation::machine_graph_batch_json_write(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::TickSimulation_machine_graph_batch_json(bx,
        by,
        bz,
        travel,
        x_off,
        {palette.data(), palette.size()},
        {cells.data(), cells.size()},
        air_index,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::TickSimulation* nucleation::TickSimulation::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::TickSimulation*>(this);
}

inline nucleation::capi::TickSimulation* nucleation::TickSimulation::AsFFI() {
    return reinterpret_cast<nucleation::capi::TickSimulation*>(this);
}

inline const nucleation::TickSimulation* nucleation::TickSimulation::FromFFI(const nucleation::capi::TickSimulation* ptr) {
    return reinterpret_cast<const nucleation::TickSimulation*>(ptr);
}

inline nucleation::TickSimulation* nucleation::TickSimulation::FromFFI(nucleation::capi::TickSimulation* ptr) {
    return reinterpret_cast<nucleation::TickSimulation*>(ptr);
}

inline void nucleation::TickSimulation::operator delete(void* ptr) {
    nucleation::capi::TickSimulation_destroy(reinterpret_cast<nucleation::capi::TickSimulation*>(ptr));
}


#endif // NUCLEATION_TickSimulation_HPP
