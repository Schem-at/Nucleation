#ifndef NUCLEATION_MchprsWorld_HPP
#define NUCLEATION_MchprsWorld_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct MchprsWorld_create_result {union {nucleation::capi::MchprsWorld* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MchprsWorld_create_result;
    MchprsWorld_create_result MchprsWorld_create(const nucleation::capi::Schematic* schematic);

    typedef struct MchprsWorld_create_with_options_result {union {nucleation::capi::MchprsWorld* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MchprsWorld_create_with_options_result;
    MchprsWorld_create_with_options_result MchprsWorld_create_with_options(const nucleation::capi::Schematic* schematic, bool optimize, bool io_only);

    typedef struct MchprsWorld_create_with_custom_io_result {union {nucleation::capi::MchprsWorld* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MchprsWorld_create_with_custom_io_result;
    MchprsWorld_create_with_custom_io_result MchprsWorld_create_with_custom_io(const nucleation::capi::Schematic* schematic, bool optimize, bool io_only, nucleation::diplomat::capi::DiplomatI32View custom_io_positions);

    typedef struct MchprsWorld_simulate_use_block_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MchprsWorld_simulate_use_block_result;
    MchprsWorld_simulate_use_block_result MchprsWorld_simulate_use_block(const nucleation::capi::Schematic* schematic, uint32_t ticks, nucleation::diplomat::capi::DiplomatI32View events_xyz);

    void MchprsWorld_tick(nucleation::capi::MchprsWorld* self, uint32_t ticks);

    void MchprsWorld_flush(nucleation::capi::MchprsWorld* self);

    void MchprsWorld_set_lever_power(nucleation::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z, bool powered);

    bool MchprsWorld_get_lever_power(const nucleation::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    bool MchprsWorld_is_lit(const nucleation::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_set_signal_strength(nucleation::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z, uint8_t strength);

    uint8_t MchprsWorld_get_signal_strength(const nucleation::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_on_use_block(nucleation::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_sync_to_schematic(nucleation::capi::MchprsWorld* self);

    nucleation::capi::Schematic* MchprsWorld_get_schematic(const nucleation::capi::MchprsWorld* self);

    uint8_t MchprsWorld_get_redstone_power(const nucleation::capi::MchprsWorld* self, int32_t x, int32_t y, int32_t z);

    void MchprsWorld_check_custom_io_changes(nucleation::capi::MchprsWorld* self);

    void MchprsWorld_poll_custom_io_changes_json(nucleation::capi::MchprsWorld* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void MchprsWorld_peek_custom_io_changes_json(const nucleation::capi::MchprsWorld* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void MchprsWorld_clear_custom_io_changes(nucleation::capi::MchprsWorld* self);

    typedef struct MchprsWorld_export_graph_result {union {nucleation::capi::RedstoneGraph* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MchprsWorld_export_graph_result;
    MchprsWorld_export_graph_result MchprsWorld_export_graph(const nucleation::capi::MchprsWorld* self);

    typedef struct MchprsWorld_export_graph_structural_result {union {nucleation::capi::RedstoneGraph* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MchprsWorld_export_graph_structural_result;
    MchprsWorld_export_graph_structural_result MchprsWorld_export_graph_structural(const nucleation::capi::MchprsWorld* self);

    void MchprsWorld_destroy(MchprsWorld* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError> nucleation::MchprsWorld::create(const nucleation::Schematic& schematic) {
    auto result = nucleation::capi::MchprsWorld_create(schematic.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MchprsWorld>>(std::unique_ptr<nucleation::MchprsWorld>(nucleation::MchprsWorld::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError> nucleation::MchprsWorld::create_with_options(const nucleation::Schematic& schematic, bool optimize, bool io_only) {
    auto result = nucleation::capi::MchprsWorld_create_with_options(schematic.AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MchprsWorld>>(std::unique_ptr<nucleation::MchprsWorld>(nucleation::MchprsWorld::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError> nucleation::MchprsWorld::create_with_custom_io(const nucleation::Schematic& schematic, bool optimize, bool io_only, nucleation::diplomat::span<const int32_t> custom_io_positions) {
    auto result = nucleation::capi::MchprsWorld_create_with_custom_io(schematic.AsFFI(),
        optimize,
        io_only,
        {custom_io_positions.data(), custom_io_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MchprsWorld>>(std::unique_ptr<nucleation::MchprsWorld>(nucleation::MchprsWorld::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MchprsWorld>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::MchprsWorld::simulate_use_block(const nucleation::Schematic& schematic, uint32_t ticks, nucleation::diplomat::span<const int32_t> events_xyz) {
    auto result = nucleation::capi::MchprsWorld_simulate_use_block(schematic.AsFFI(),
        ticks,
        {events_xyz.data(), events_xyz.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::MchprsWorld::tick(uint32_t ticks) {
    nucleation::capi::MchprsWorld_tick(this->AsFFI(),
        ticks);
}

inline void nucleation::MchprsWorld::flush() {
    nucleation::capi::MchprsWorld_flush(this->AsFFI());
}

inline void nucleation::MchprsWorld::set_lever_power(int32_t x, int32_t y, int32_t z, bool powered) {
    nucleation::capi::MchprsWorld_set_lever_power(this->AsFFI(),
        x,
        y,
        z,
        powered);
}

inline bool nucleation::MchprsWorld::get_lever_power(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::MchprsWorld_get_lever_power(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline bool nucleation::MchprsWorld::is_lit(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::MchprsWorld_is_lit(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void nucleation::MchprsWorld::set_signal_strength(int32_t x, int32_t y, int32_t z, uint8_t strength) {
    nucleation::capi::MchprsWorld_set_signal_strength(this->AsFFI(),
        x,
        y,
        z,
        strength);
}

inline uint8_t nucleation::MchprsWorld::get_signal_strength(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::MchprsWorld_get_signal_strength(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void nucleation::MchprsWorld::on_use_block(int32_t x, int32_t y, int32_t z) {
    nucleation::capi::MchprsWorld_on_use_block(this->AsFFI(),
        x,
        y,
        z);
}

inline void nucleation::MchprsWorld::sync_to_schematic() {
    nucleation::capi::MchprsWorld_sync_to_schematic(this->AsFFI());
}

inline std::unique_ptr<nucleation::Schematic> nucleation::MchprsWorld::get_schematic() const {
    auto result = nucleation::capi::MchprsWorld_get_schematic(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline uint8_t nucleation::MchprsWorld::get_redstone_power(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::MchprsWorld_get_redstone_power(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline void nucleation::MchprsWorld::check_custom_io_changes() {
    nucleation::capi::MchprsWorld_check_custom_io_changes(this->AsFFI());
}

inline std::string nucleation::MchprsWorld::poll_custom_io_changes_json() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::MchprsWorld_poll_custom_io_changes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::MchprsWorld::poll_custom_io_changes_json_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::MchprsWorld_poll_custom_io_changes_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::MchprsWorld::peek_custom_io_changes_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::MchprsWorld_peek_custom_io_changes_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::MchprsWorld::peek_custom_io_changes_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::MchprsWorld_peek_custom_io_changes_json(this->AsFFI(),
        &write);
}

inline void nucleation::MchprsWorld::clear_custom_io_changes() {
    nucleation::capi::MchprsWorld_clear_custom_io_changes(this->AsFFI());
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError> nucleation::MchprsWorld::export_graph() const {
    auto result = nucleation::capi::MchprsWorld_export_graph(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::RedstoneGraph>>(std::unique_ptr<nucleation::RedstoneGraph>(nucleation::RedstoneGraph::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError> nucleation::MchprsWorld::export_graph_structural() const {
    auto result = nucleation::capi::MchprsWorld_export_graph_structural(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::RedstoneGraph>>(std::unique_ptr<nucleation::RedstoneGraph>(nucleation::RedstoneGraph::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::MchprsWorld* nucleation::MchprsWorld::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MchprsWorld*>(this);
}

inline nucleation::capi::MchprsWorld* nucleation::MchprsWorld::AsFFI() {
    return reinterpret_cast<nucleation::capi::MchprsWorld*>(this);
}

inline const nucleation::MchprsWorld* nucleation::MchprsWorld::FromFFI(const nucleation::capi::MchprsWorld* ptr) {
    return reinterpret_cast<const nucleation::MchprsWorld*>(ptr);
}

inline nucleation::MchprsWorld* nucleation::MchprsWorld::FromFFI(nucleation::capi::MchprsWorld* ptr) {
    return reinterpret_cast<nucleation::MchprsWorld*>(ptr);
}

inline void nucleation::MchprsWorld::operator delete(void* ptr) {
    nucleation::capi::MchprsWorld_destroy(reinterpret_cast<nucleation::capi::MchprsWorld*>(ptr));
}


#endif // NUCLEATION_MchprsWorld_HPP
