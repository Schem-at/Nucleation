#ifndef Design_HPP
#define Design_HPP

#include "Design.d.hpp"

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
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Design_create_result {union {diplomat::capi::Design* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Design_create_result;
    Design_create_result Design_create(diplomat::capi::DiplomatStringView name);

    typedef struct Design_for_schematic_result {union {diplomat::capi::Design* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Design_for_schematic_result;
    Design_for_schematic_result Design_for_schematic(diplomat::capi::DiplomatStringView name, const diplomat::capi::Schematic* base);

    typedef struct Design_add_cell_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_add_cell_result;
    Design_add_cell_result Design_add_cell(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::Schematic* cell, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_place_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_place_result;
    Design_place_result Design_place(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatStringView cell, int32_t x, int32_t y, int32_t z, int32_t rot_y);

    typedef struct Design_declare_input_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_declare_input_result;
    Design_declare_input_result Design_declare_input(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, diplomat::capi::DiplomatStringView ty);

    typedef struct Design_declare_output_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_declare_output_result;
    Design_declare_output_result Design_declare_output(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, diplomat::capi::DiplomatStringView ty);

    typedef struct Design_route_bus_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_route_bus_result;
    Design_route_bus_result Design_route_bus(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatStringView driver, diplomat::capi::DiplomatStringView sinks_json, diplomat::capi::DiplomatStringView gates_json, diplomat::capi::DiplomatStringView style_json, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_route_bus_or_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_route_bus_or_result;
    Design_route_bus_or_result Design_route_bus_or(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatStringView drivers_json, diplomat::capi::DiplomatStringView sinks_json, diplomat::capi::DiplomatStringView gates_json, diplomat::capi::DiplomatStringView style_json, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_set_block_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_set_block_result;
    Design_set_block_result Design_set_block(diplomat::capi::Design* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block);

    typedef struct Design_move_instance_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_move_instance_result;
    Design_move_instance_result Design_move_instance(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, int32_t x, int32_t y, int32_t z, int32_t rot_y, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_add_gate_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_add_gate_result;
    Design_add_gate_result Design_add_gate(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView bus, diplomat::capi::DiplomatStringView gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_move_gate_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_move_gate_result;
    Design_move_gate_result Design_move_gate(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView bus, diplomat::capi::DiplomatStringView gate, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_set_bus_rule_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_set_bus_rule_result;
    Design_set_bus_rule_result Design_set_bus_rule(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView bus, diplomat::capi::DiplomatStringView rule_json);

    typedef struct Design_bus_skew_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_bus_skew_result;
    Design_bus_skew_result Design_bus_skew(const diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_bus_state_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_bus_state_result;
    Design_bus_state_result Design_bus_state(const diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_rip_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_rip_result;
    Design_rip_result Design_rip(diplomat::capi::Design* self, diplomat::capi::DiplomatStringView name);

    typedef struct Design_flatten_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Design_flatten_result;
    Design_flatten_result Design_flatten(const diplomat::capi::Design* self);

    typedef struct Design_check_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Design_check_result;
    Design_check_result Design_check(const diplomat::capi::Design* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Design_bake_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Design_bake_result;
    Design_bake_result Design_bake(const diplomat::capi::Design* self, uint32_t budget);

    void Design_destroy(Design* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Design>, NucleationError> Design::create(std::string_view name) {
    auto result = diplomat::capi::Design_create({name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Design>, NucleationError>(diplomat::Ok<std::unique_ptr<Design>>(std::unique_ptr<Design>(Design::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Design>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Design>, NucleationError> Design::for_schematic(std::string_view name, const Schematic& base) {
    auto result = diplomat::capi::Design_for_schematic({name.data(), name.size()},
        base.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Design>, NucleationError>(diplomat::Ok<std::unique_ptr<Design>>(std::unique_ptr<Design>(Design::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Design>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::add_cell(std::string_view name, const Schematic& cell) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_add_cell(this->AsFFI(),
        {name.data(), name.size()},
        cell.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::add_cell_write(std::string_view name, const Schematic& cell, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_add_cell(this->AsFFI(),
        {name.data(), name.size()},
        cell.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Design::place(std::string_view name, std::string_view cell, int32_t x, int32_t y, int32_t z, int32_t rot_y) {
    auto result = diplomat::capi::Design_place(this->AsFFI(),
        {name.data(), name.size()},
        {cell.data(), cell.size()},
        x,
        y,
        z,
        rot_y);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Design::declare_input(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty) {
    auto result = diplomat::capi::Design_declare_input(this->AsFFI(),
        {name.data(), name.size()},
        ax,
        ay,
        az,
        sx,
        sy,
        sz,
        width,
        {ty.data(), ty.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Design::declare_output(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty) {
    auto result = diplomat::capi::Design_declare_output(this->AsFFI(),
        {name.data(), name.size()},
        ax,
        ay,
        az,
        sx,
        sy,
        sz,
        width,
        {ty.data(), ty.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::route_bus(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_route_bus(this->AsFFI(),
        {name.data(), name.size()},
        {driver.data(), driver.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::route_bus_write(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_route_bus(this->AsFFI(),
        {name.data(), name.size()},
        {driver.data(), driver.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::route_bus_or(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_route_bus_or(this->AsFFI(),
        {name.data(), name.size()},
        {drivers_json.data(), drivers_json.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::route_bus_or_write(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_route_bus_or(this->AsFFI(),
        {name.data(), name.size()},
        {drivers_json.data(), drivers_json.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Design::set_block(int32_t x, int32_t y, int32_t z, std::string_view block) {
    auto result = diplomat::capi::Design_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block.data(), block.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::move_instance(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_move_instance(this->AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z,
        rot_y,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::move_instance_write(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_move_instance(this->AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z,
        rot_y,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::add_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_add_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        sx,
        sy,
        sz,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::add_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_add_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        sx,
        sy,
        sz,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::move_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_move_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::move_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_move_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Design::set_bus_rule(std::string_view bus, std::string_view rule_json) {
    auto result = diplomat::capi::Design_set_bus_rule(this->AsFFI(),
        {bus.data(), bus.size()},
        {rule_json.data(), rule_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::bus_skew(std::string_view name) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_bus_skew(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::bus_skew_write(std::string_view name, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_bus_skew(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::bus_state(std::string_view name) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_bus_state(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::bus_state_write(std::string_view name, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_bus_state(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Design::rip(std::string_view name) {
    auto result = diplomat::capi::Design_rip(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Design::flatten() const {
    auto result = diplomat::capi::Design_flatten(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Design::check() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Design_check(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Design::check_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Design_check(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Design::bake(uint32_t budget) const {
    auto result = diplomat::capi::Design_bake(this->AsFFI(),
        budget);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Design* Design::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Design*>(this);
}

inline diplomat::capi::Design* Design::AsFFI() {
    return reinterpret_cast<diplomat::capi::Design*>(this);
}

inline const Design* Design::FromFFI(const diplomat::capi::Design* ptr) {
    return reinterpret_cast<const Design*>(ptr);
}

inline Design* Design::FromFFI(diplomat::capi::Design* ptr) {
    return reinterpret_cast<Design*>(ptr);
}

inline void Design::operator delete(void* ptr) {
    diplomat::capi::Design_destroy(reinterpret_cast<diplomat::capi::Design*>(ptr));
}


#endif // Design_HPP
