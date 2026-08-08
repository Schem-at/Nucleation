#ifndef TickSimulation_HPP
#define TickSimulation_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    void TickSimulation_last_error_detail(diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_max_volume(void);

    typedef struct TickSimulation_from_snbt_result {union {diplomat::capi::TickSimulation* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_from_snbt_result;
    TickSimulation_from_snbt_result TickSimulation_from_snbt(diplomat::capi::DiplomatStringView snbt, diplomat::capi::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, diplomat::capi::DiplomatStringView extra_states);

    typedef struct TickSimulation_from_schematic_result {union {diplomat::capi::TickSimulation* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_from_schematic_result;
    TickSimulation_from_schematic_result TickSimulation_from_schematic(const diplomat::capi::Schematic* schematic, diplomat::capi::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, diplomat::capi::DiplomatStringView extra_states);

    typedef struct TickSimulation_from_blocks_result {union {diplomat::capi::TickSimulation* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_from_blocks_result;
    TickSimulation_from_blocks_result TickSimulation_from_blocks(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, diplomat::capi::DiplomatStringView palette, diplomat::capi::DiplomatU16View cells, uint16_t air_index, diplomat::capi::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z);

    typedef struct TickSimulation_eval_flight_batch_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_eval_flight_batch_result;
    TickSimulation_eval_flight_batch_result TickSimulation_eval_flight_batch(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, diplomat::capi::DiplomatStringView palette, diplomat::capi::DiplomatU16View cells, uint16_t air_index, diplomat::capi::DiplomatI32View kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_set_rng_seed(diplomat::capi::TickSimulation* self, int64_t seed);

    void TickSimulation_step(diplomat::capi::TickSimulation* self);

    void TickSimulation_run(diplomat::capi::TickSimulation* self, uint32_t ticks);

    bool TickSimulation_run_until_quiescent(diplomat::capi::TickSimulation* self, uint32_t budget);

    uint32_t TickSimulation_tick_count(const diplomat::capi::TickSimulation* self);

    bool TickSimulation_is_quiescent(const diplomat::capi::TickSimulation* self);

    void TickSimulation_use_block(diplomat::capi::TickSimulation* self, int32_t x, int32_t y, int32_t z);

    typedef struct TickSimulation_place_block_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_place_block_result;
    TickSimulation_place_block_result TickSimulation_place_block(diplomat::capi::TickSimulation* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView state);

    void TickSimulation_get_block(const diplomat::capi::TickSimulation* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_checkpoint(diplomat::capi::TickSimulation* self);

    typedef struct TickSimulation_restore_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_restore_result;
    TickSimulation_restore_result TickSimulation_restore(diplomat::capi::TickSimulation* self, uint32_t id);

    void TickSimulation_gametest_snbt(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_block_entity_audit_json(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_record_updates(diplomat::capi::TickSimulation* self, bool on);

    void TickSimulation_clear_updates(diplomat::capi::TickSimulation* self);

    void TickSimulation_record_timeline(diplomat::capi::TickSimulation* self);

    void TickSimulation_stop_timeline(diplomat::capi::TickSimulation* self);

    void TickSimulation_timeline_activity_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_timeline_cycles_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    typedef struct TickSimulation_animation_timeline_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_animation_timeline_json_result;
    TickSimulation_animation_timeline_json_result TickSimulation_animation_timeline_json(const diplomat::capi::TickSimulation* self, uint32_t start_tick, uint32_t end_tick, float tick_ms, diplomat::capi::DiplomatWrite* write);

    typedef struct TickSimulation_selection_schematic_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_selection_schematic_b64_result;
    TickSimulation_selection_schematic_b64_result TickSimulation_selection_schematic_b64(const diplomat::capi::TickSimulation* self, uint32_t start_tick, uint32_t end_tick, diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_updates_count(const diplomat::capi::TickSimulation* self);

    void TickSimulation_updates_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_updates_json_between(const diplomat::capi::TickSimulation* self, uint32_t from_tick, uint32_t to_tick, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_updates_heat_json(const diplomat::capi::TickSimulation* self, uint32_t from_tick, uint32_t to_tick, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_updates_wave_json(const diplomat::capi::TickSimulation* self, uint32_t tick, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_moving_blocks_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_clear_changes(diplomat::capi::TickSimulation* self);

    void TickSimulation_changes_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_item_entities_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_motion_semantics(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_piston_retract_contacts(const diplomat::capi::TickSimulation* self);

    void TickSimulation_events_summary_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    uint32_t TickSimulation_non_air_count(const diplomat::capi::TickSimulation* self);

    double TickSimulation_non_air_center_x(const diplomat::capi::TickSimulation* self);

    int32_t TickSimulation_non_air_min_x(const diplomat::capi::TickSimulation* self);

    int32_t TickSimulation_non_air_max_x(const diplomat::capi::TickSimulation* self);

    uint32_t TickSimulation_changes_count(const diplomat::capi::TickSimulation* self);

    void TickSimulation_world_snapshot_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_machine_graph_json(const diplomat::capi::TickSimulation* self, diplomat::capi::DiplomatWrite* write);

    typedef struct TickSimulation_machine_graph_batch_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TickSimulation_machine_graph_batch_json_result;
    TickSimulation_machine_graph_batch_json_result TickSimulation_machine_graph_batch_json(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, diplomat::capi::DiplomatStringView palette, diplomat::capi::DiplomatU16View cells, uint16_t air_index, diplomat::capi::DiplomatWrite* write);

    void TickSimulation_destroy(TickSimulation* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::string TickSimulation::last_error_detail() {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_last_error_detail(&write);
    return output;
}
template<typename W>
inline void TickSimulation::last_error_detail_write(W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_last_error_detail(&write);
}

inline uint32_t TickSimulation::max_volume() {
    auto result = diplomat::capi::TickSimulation_max_volume();
    return result;
}

inline diplomat::result<std::unique_ptr<TickSimulation>, NucleationError> TickSimulation::from_snbt(std::string_view snbt, TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, std::string_view extra_states) {
    auto result = diplomat::capi::TickSimulation_from_snbt({snbt.data(), snbt.size()},
        settle.AsFFI(),
        origin_x,
        origin_y,
        origin_z,
        {extra_states.data(), extra_states.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<TickSimulation>, NucleationError>(diplomat::Ok<std::unique_ptr<TickSimulation>>(std::unique_ptr<TickSimulation>(TickSimulation::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TickSimulation>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<TickSimulation>, NucleationError> TickSimulation::from_schematic(const Schematic& schematic, TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, std::string_view extra_states) {
    auto result = diplomat::capi::TickSimulation_from_schematic(schematic.AsFFI(),
        settle.AsFFI(),
        origin_x,
        origin_y,
        origin_z,
        {extra_states.data(), extra_states.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<TickSimulation>, NucleationError>(diplomat::Ok<std::unique_ptr<TickSimulation>>(std::unique_ptr<TickSimulation>(TickSimulation::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TickSimulation>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<TickSimulation>, NucleationError> TickSimulation::from_blocks(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, diplomat::span<const uint16_t> cells, uint16_t air_index, TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z) {
    auto result = diplomat::capi::TickSimulation_from_blocks(bx,
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
    return result.is_ok ? diplomat::result<std::unique_ptr<TickSimulation>, NucleationError>(diplomat::Ok<std::unique_ptr<TickSimulation>>(std::unique_ptr<TickSimulation>(TickSimulation::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TickSimulation>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> TickSimulation::eval_flight_batch(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, diplomat::span<const uint16_t> cells, uint16_t air_index, diplomat::span<const int32_t> kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::TickSimulation_eval_flight_batch(bx,
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
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> TickSimulation::eval_flight_batch_write(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, diplomat::span<const uint16_t> cells, uint16_t air_index, diplomat::span<const int32_t> kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::TickSimulation_eval_flight_batch(bx,
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
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void TickSimulation::set_rng_seed(int64_t seed) {
    diplomat::capi::TickSimulation_set_rng_seed(this->AsFFI(),
        seed);
}

inline void TickSimulation::step() {
    diplomat::capi::TickSimulation_step(this->AsFFI());
}

inline void TickSimulation::run(uint32_t ticks) {
    diplomat::capi::TickSimulation_run(this->AsFFI(),
        ticks);
}

inline bool TickSimulation::run_until_quiescent(uint32_t budget) {
    auto result = diplomat::capi::TickSimulation_run_until_quiescent(this->AsFFI(),
        budget);
    return result;
}

inline uint32_t TickSimulation::tick_count() const {
    auto result = diplomat::capi::TickSimulation_tick_count(this->AsFFI());
    return result;
}

inline bool TickSimulation::is_quiescent() const {
    auto result = diplomat::capi::TickSimulation_is_quiescent(this->AsFFI());
    return result;
}

inline void TickSimulation::use_block(int32_t x, int32_t y, int32_t z) {
    diplomat::capi::TickSimulation_use_block(this->AsFFI(),
        x,
        y,
        z);
}

inline diplomat::result<std::monostate, NucleationError> TickSimulation::place_block(int32_t x, int32_t y, int32_t z, std::string_view state) {
    auto result = diplomat::capi::TickSimulation_place_block(this->AsFFI(),
        x,
        y,
        z,
        {state.data(), state.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string TickSimulation::get_block(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_get_block(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::get_block_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_get_block(this->AsFFI(),
        x,
        y,
        z,
        &write);
}

inline uint32_t TickSimulation::checkpoint() {
    auto result = diplomat::capi::TickSimulation_checkpoint(this->AsFFI());
    return result;
}

inline diplomat::result<std::monostate, NucleationError> TickSimulation::restore(uint32_t id) {
    auto result = diplomat::capi::TickSimulation_restore(this->AsFFI(),
        id);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string TickSimulation::gametest_snbt(const Schematic& schematic) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_gametest_snbt(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::gametest_snbt_write(const Schematic& schematic, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_gametest_snbt(schematic.AsFFI(),
        &write);
}

inline std::string TickSimulation::block_entity_audit_json(const Schematic& schematic) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_block_entity_audit_json(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::block_entity_audit_json_write(const Schematic& schematic, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_block_entity_audit_json(schematic.AsFFI(),
        &write);
}

inline void TickSimulation::record_updates(bool on) {
    diplomat::capi::TickSimulation_record_updates(this->AsFFI(),
        on);
}

inline void TickSimulation::clear_updates() {
    diplomat::capi::TickSimulation_clear_updates(this->AsFFI());
}

inline void TickSimulation::record_timeline() {
    diplomat::capi::TickSimulation_record_timeline(this->AsFFI());
}

inline void TickSimulation::stop_timeline() {
    diplomat::capi::TickSimulation_stop_timeline(this->AsFFI());
}

inline std::string TickSimulation::timeline_activity_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_timeline_activity_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::timeline_activity_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_timeline_activity_json(this->AsFFI(),
        &write);
}

inline std::string TickSimulation::timeline_cycles_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_timeline_cycles_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::timeline_cycles_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_timeline_cycles_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> TickSimulation::animation_timeline_json(uint32_t start_tick, uint32_t end_tick, float tick_ms) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::TickSimulation_animation_timeline_json(this->AsFFI(),
        start_tick,
        end_tick,
        tick_ms,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> TickSimulation::animation_timeline_json_write(uint32_t start_tick, uint32_t end_tick, float tick_ms, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::TickSimulation_animation_timeline_json(this->AsFFI(),
        start_tick,
        end_tick,
        tick_ms,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> TickSimulation::selection_schematic_b64(uint32_t start_tick, uint32_t end_tick) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::TickSimulation_selection_schematic_b64(this->AsFFI(),
        start_tick,
        end_tick,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> TickSimulation::selection_schematic_b64_write(uint32_t start_tick, uint32_t end_tick, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::TickSimulation_selection_schematic_b64(this->AsFFI(),
        start_tick,
        end_tick,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t TickSimulation::updates_count() const {
    auto result = diplomat::capi::TickSimulation_updates_count(this->AsFFI());
    return result;
}

inline std::string TickSimulation::updates_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_updates_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::updates_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_updates_json(this->AsFFI(),
        &write);
}

inline std::string TickSimulation::updates_json_between(uint32_t from_tick, uint32_t to_tick) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_updates_json_between(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::updates_json_between_write(uint32_t from_tick, uint32_t to_tick, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_updates_json_between(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
}

inline std::string TickSimulation::updates_heat_json(uint32_t from_tick, uint32_t to_tick) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_updates_heat_json(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::updates_heat_json_write(uint32_t from_tick, uint32_t to_tick, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_updates_heat_json(this->AsFFI(),
        from_tick,
        to_tick,
        &write);
}

inline std::string TickSimulation::updates_wave_json(uint32_t tick) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_updates_wave_json(this->AsFFI(),
        tick,
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::updates_wave_json_write(uint32_t tick, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_updates_wave_json(this->AsFFI(),
        tick,
        &write);
}

inline std::string TickSimulation::moving_blocks_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_moving_blocks_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::moving_blocks_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_moving_blocks_json(this->AsFFI(),
        &write);
}

inline void TickSimulation::clear_changes() {
    diplomat::capi::TickSimulation_clear_changes(this->AsFFI());
}

inline std::string TickSimulation::changes_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_changes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::changes_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_changes_json(this->AsFFI(),
        &write);
}

inline std::string TickSimulation::item_entities_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_item_entities_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::item_entities_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_item_entities_json(this->AsFFI(),
        &write);
}

inline std::string TickSimulation::motion_semantics() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_motion_semantics(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::motion_semantics_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_motion_semantics(this->AsFFI(),
        &write);
}

inline uint32_t TickSimulation::piston_retract_contacts() const {
    auto result = diplomat::capi::TickSimulation_piston_retract_contacts(this->AsFFI());
    return result;
}

inline std::string TickSimulation::events_summary_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_events_summary_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::events_summary_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_events_summary_json(this->AsFFI(),
        &write);
}

inline uint32_t TickSimulation::non_air_count() const {
    auto result = diplomat::capi::TickSimulation_non_air_count(this->AsFFI());
    return result;
}

inline double TickSimulation::non_air_center_x() const {
    auto result = diplomat::capi::TickSimulation_non_air_center_x(this->AsFFI());
    return result;
}

inline int32_t TickSimulation::non_air_min_x() const {
    auto result = diplomat::capi::TickSimulation_non_air_min_x(this->AsFFI());
    return result;
}

inline int32_t TickSimulation::non_air_max_x() const {
    auto result = diplomat::capi::TickSimulation_non_air_max_x(this->AsFFI());
    return result;
}

inline uint32_t TickSimulation::changes_count() const {
    auto result = diplomat::capi::TickSimulation_changes_count(this->AsFFI());
    return result;
}

inline std::string TickSimulation::world_snapshot_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_world_snapshot_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::world_snapshot_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_world_snapshot_json(this->AsFFI(),
        &write);
}

inline std::string TickSimulation::machine_graph_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TickSimulation_machine_graph_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TickSimulation::machine_graph_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TickSimulation_machine_graph_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> TickSimulation::machine_graph_batch_json(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, diplomat::span<const uint16_t> cells, uint16_t air_index) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::TickSimulation_machine_graph_batch_json(bx,
        by,
        bz,
        travel,
        x_off,
        {palette.data(), palette.size()},
        {cells.data(), cells.size()},
        air_index,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> TickSimulation::machine_graph_batch_json_write(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, diplomat::span<const uint16_t> cells, uint16_t air_index, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::TickSimulation_machine_graph_batch_json(bx,
        by,
        bz,
        travel,
        x_off,
        {palette.data(), palette.size()},
        {cells.data(), cells.size()},
        air_index,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::TickSimulation* TickSimulation::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::TickSimulation*>(this);
}

inline diplomat::capi::TickSimulation* TickSimulation::AsFFI() {
    return reinterpret_cast<diplomat::capi::TickSimulation*>(this);
}

inline const TickSimulation* TickSimulation::FromFFI(const diplomat::capi::TickSimulation* ptr) {
    return reinterpret_cast<const TickSimulation*>(ptr);
}

inline TickSimulation* TickSimulation::FromFFI(diplomat::capi::TickSimulation* ptr) {
    return reinterpret_cast<TickSimulation*>(ptr);
}

inline void TickSimulation::operator delete(void* ptr) {
    diplomat::capi::TickSimulation_destroy(reinterpret_cast<diplomat::capi::TickSimulation*>(ptr));
}


#endif // TickSimulation_HPP
