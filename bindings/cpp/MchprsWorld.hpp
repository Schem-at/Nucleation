#ifndef MchprsWorld_HPP
#define MchprsWorld_HPP

#include "MchprsWorld.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "RedstoneGraph.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct MchprsWorld_create_result {union {diplomat::capi::MchprsWorld* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MchprsWorld_create_result;
    MchprsWorld_create_result MchprsWorld_create(const diplomat::capi::Schematic* schematic);

    typedef struct MchprsWorld_create_with_options_result {union {diplomat::capi::MchprsWorld* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MchprsWorld_create_with_options_result;
    MchprsWorld_create_with_options_result MchprsWorld_create_with_options(const diplomat::capi::Schematic* schematic, bool optimize, bool io_only);

    typedef struct MchprsWorld_create_with_custom_io_result {union {diplomat::capi::MchprsWorld* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MchprsWorld_create_with_custom_io_result;
    MchprsWorld_create_with_custom_io_result MchprsWorld_create_with_custom_io(const diplomat::capi::Schematic* schematic, bool optimize, bool io_only, diplomat::capi::DiplomatI32View custom_io_positions);

    typedef struct MchprsWorld_simulate_use_block_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MchprsWorld_simulate_use_block_result;
    MchprsWorld_simulate_use_block_result MchprsWorld_simulate_use_block(const diplomat::capi::Schematic* schematic, uint32_t ticks, diplomat::capi::DiplomatI32View events_xyz);

    void MchprsWorld_tick(diplomat::capi::MchprsWorld* self, uint32_t ticks);

    void MchprsWorld_flush(diplomat::capi::MchprsWorld* self);

    void MchprsWorld_set_lever_power(diplomat::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z, bool powered);

    bool MchprsWorld_get_lever_power(const diplomat::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    bool MchprsWorld_is_lit(const diplomat::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_set_signal_strength(diplomat::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z, uint8_t strength);

    uint8_t MchprsWorld_get_signal_strength(const diplomat::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_on_use_block(diplomat::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_sync_to_schematic(diplomat::capi::MchprsWorld* self);

    diplomat::capi::Schematic* MchprsWorld_get_schematic(const diplomat::capi::MchprsWorld* self);

    uint8_t MchprsWorld_get_redstone_power(const diplomat::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_check_custom_io_changes(diplomat::capi::MchprsWorld* self);

    void MchprsWorld_poll_custom_io_changes_json(diplomat::capi::MchprsWorld* self, diplomat::capi::DiplomatWrite* write);

    void MchprsWorld_peek_custom_io_changes_json(const diplomat::capi::MchprsWorld* self, diplomat::capi::DiplomatWrite* write);

    void MchprsWorld_clear_custom_io_changes(diplomat::capi::MchprsWorld* self);

    typedef struct MchprsWorld_export_graph_result {union {diplomat::capi::RedstoneGraph* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MchprsWorld_export_graph_result;
    MchprsWorld_export_graph_result MchprsWorld_export_graph(const diplomat::capi::MchprsWorld* self);

    typedef struct MchprsWorld_export_graph_structural_result {union {diplomat::capi::RedstoneGraph* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MchprsWorld_export_graph_structural_result;
    MchprsWorld_export_graph_structural_result MchprsWorld_export_graph_structural(const diplomat::capi::MchprsWorld* self);

    void MchprsWorld_destroy(MchprsWorld* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError> MchprsWorld::create(const Schematic& schematic) {
    auto result = diplomat::capi::MchprsWorld_create(schematic.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError>(diplomat::Ok<std::unique_ptr<MchprsWorld>>(std::unique_ptr<MchprsWorld>(MchprsWorld::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError> MchprsWorld::create_with_options(const Schematic& schematic, bool optimize, bool io_only) {
    auto result = diplomat::capi::MchprsWorld_create_with_options(schematic.AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError>(diplomat::Ok<std::unique_ptr<MchprsWorld>>(std::unique_ptr<MchprsWorld>(MchprsWorld::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError> MchprsWorld::create_with_custom_io(const Schematic& schematic, bool optimize, bool io_only, diplomat::span<const int32_t> custom_io_positions) {
    auto result = diplomat::capi::MchprsWorld_create_with_custom_io(schematic.AsFFI(),
        optimize,
        io_only,
        {custom_io_positions.data(), custom_io_positions.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError>(diplomat::Ok<std::unique_ptr<MchprsWorld>>(std::unique_ptr<MchprsWorld>(MchprsWorld::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> MchprsWorld::simulate_use_block(const Schematic& schematic, uint32_t ticks, diplomat::span<const int32_t> events_xyz) {
    auto result = diplomat::capi::MchprsWorld_simulate_use_block(schematic.AsFFI(),
        ticks,
        {events_xyz.data(), events_xyz.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void MchprsWorld::tick(uint32_t ticks) {
    diplomat::capi::MchprsWorld_tick(this->AsFFI(),
        ticks);
}

inline void MchprsWorld::flush() {
    diplomat::capi::MchprsWorld_flush(this->AsFFI());
}

inline void MchprsWorld::set_lever_power(int32_t x, int32_t y, int32_t z, bool powered) {
    diplomat::capi::MchprsWorld_set_lever_power(this->AsFFI(),
        x,
        y,
        z,
        powered);
}

inline bool MchprsWorld::get_lever_power(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::MchprsWorld_get_lever_power(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline bool MchprsWorld::is_lit(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::MchprsWorld_is_lit(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void MchprsWorld::set_signal_strength(int32_t x, int32_t y, int32_t z, uint8_t strength) {
    diplomat::capi::MchprsWorld_set_signal_strength(this->AsFFI(),
        x,
        y,
        z,
        strength);
}

inline uint8_t MchprsWorld::get_signal_strength(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::MchprsWorld_get_signal_strength(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void MchprsWorld::on_use_block(int32_t x, int32_t y, int32_t z) {
    diplomat::capi::MchprsWorld_on_use_block(this->AsFFI(),
        x,
        y,
        z);
}

inline void MchprsWorld::sync_to_schematic() {
    diplomat::capi::MchprsWorld_sync_to_schematic(this->AsFFI());
}

inline std::unique_ptr<Schematic> MchprsWorld::get_schematic() const {
    auto result = diplomat::capi::MchprsWorld_get_schematic(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline uint8_t MchprsWorld::get_redstone_power(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::MchprsWorld_get_redstone_power(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void MchprsWorld::check_custom_io_changes() {
    diplomat::capi::MchprsWorld_check_custom_io_changes(this->AsFFI());
}

inline std::string MchprsWorld::poll_custom_io_changes_json() {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::MchprsWorld_poll_custom_io_changes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void MchprsWorld::poll_custom_io_changes_json_write(W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::MchprsWorld_poll_custom_io_changes_json(this->AsFFI(),
        &write);
}

inline std::string MchprsWorld::peek_custom_io_changes_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::MchprsWorld_peek_custom_io_changes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void MchprsWorld::peek_custom_io_changes_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::MchprsWorld_peek_custom_io_changes_json(this->AsFFI(),
        &write);
}

inline void MchprsWorld::clear_custom_io_changes() {
    diplomat::capi::MchprsWorld_clear_custom_io_changes(this->AsFFI());
}

inline diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError> MchprsWorld::export_graph() const {
    auto result = diplomat::capi::MchprsWorld_export_graph(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError>(diplomat::Ok<std::unique_ptr<RedstoneGraph>>(std::unique_ptr<RedstoneGraph>(RedstoneGraph::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError> MchprsWorld::export_graph_structural() const {
    auto result = diplomat::capi::MchprsWorld_export_graph_structural(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError>(diplomat::Ok<std::unique_ptr<RedstoneGraph>>(std::unique_ptr<RedstoneGraph>(RedstoneGraph::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::MchprsWorld* MchprsWorld::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MchprsWorld*>(this);
}

inline diplomat::capi::MchprsWorld* MchprsWorld::AsFFI() {
    return reinterpret_cast<diplomat::capi::MchprsWorld*>(this);
}

inline const MchprsWorld* MchprsWorld::FromFFI(const diplomat::capi::MchprsWorld* ptr) {
    return reinterpret_cast<const MchprsWorld*>(ptr);
}

inline MchprsWorld* MchprsWorld::FromFFI(diplomat::capi::MchprsWorld* ptr) {
    return reinterpret_cast<MchprsWorld*>(ptr);
}

inline void MchprsWorld::operator delete(void* ptr) {
    diplomat::capi::MchprsWorld_destroy(reinterpret_cast<diplomat::capi::MchprsWorld*>(ptr));
}


#endif // MchprsWorld_HPP
