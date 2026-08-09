#ifndef NUCLEATION_Design_HPP
#define NUCLEATION_Design_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Design_create_result {union {nucleation::capi::Design* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_create_result;
    Design_create_result Design_create(nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct Design_for_schematic_result {union {nucleation::capi::Design* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_for_schematic_result;
    Design_for_schematic_result Design_for_schematic(nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::Schematic* base);

    typedef struct Design_add_cell_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_add_cell_result;
    Design_add_cell_result Design_add_cell(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::Schematic* cell, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_place_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_place_result;
    Design_place_result Design_place(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatStringView cell, int32_t x, int32_t y, int32_t z, int32_t rot_y);

    typedef struct Design_declare_input_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_declare_input_result;
    Design_declare_input_result Design_declare_input(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, nucleation::diplomat::capi::DiplomatStringView ty);

    typedef struct Design_declare_output_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_declare_output_result;
    Design_declare_output_result Design_declare_output(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, nucleation::diplomat::capi::DiplomatStringView ty);

    typedef struct Design_route_bus_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_route_bus_result;
    Design_route_bus_result Design_route_bus(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatStringView driver, nucleation::diplomat::capi::DiplomatStringView sinks_json, nucleation::diplomat::capi::DiplomatStringView gates_json, nucleation::diplomat::capi::DiplomatStringView style_json, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_route_bus_or_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_route_bus_or_result;
    Design_route_bus_or_result Design_route_bus_or(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatStringView drivers_json, nucleation::diplomat::capi::DiplomatStringView sinks_json, nucleation::diplomat::capi::DiplomatStringView gates_json, nucleation::diplomat::capi::DiplomatStringView style_json, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_set_block_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_set_block_result;
    Design_set_block_result Design_set_block(nucleation::capi::Design* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block);

    typedef struct Design_move_instance_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_move_instance_result;
    Design_move_instance_result Design_move_instance(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, int32_t x, int32_t y, int32_t z, int32_t rot_y, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_remove_instance_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_remove_instance_result;
    Design_remove_instance_result Design_remove_instance(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_reroute_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_reroute_result;
    Design_reroute_result Design_reroute(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_remove_bus_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_remove_bus_result;
    Design_remove_bus_result Design_remove_bus(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct Design_to_schem_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_to_schem_b64_result;
    Design_to_schem_b64_result Design_to_schem_b64(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_flatten_composite_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_flatten_composite_result;
    Design_flatten_composite_result Design_flatten_composite(const nucleation::capi::Design* self);

    typedef struct Design_instance_ports_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_instance_ports_result;
    Design_instance_ports_result Design_instance_ports(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_resolve_port_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_resolve_port_result;
    Design_resolve_port_result Design_resolve_port(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_add_gate_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_add_gate_result;
    Design_add_gate_result Design_add_gate(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView bus, nucleation::diplomat::capi::DiplomatStringView gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_move_gate_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_move_gate_result;
    Design_move_gate_result Design_move_gate(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView bus, nucleation::diplomat::capi::DiplomatStringView gate, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_set_bus_rule_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_set_bus_rule_result;
    Design_set_bus_rule_result Design_set_bus_rule(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView bus, nucleation::diplomat::capi::DiplomatStringView rule_json);

    typedef struct Design_bus_skew_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_bus_skew_result;
    Design_bus_skew_result Design_bus_skew(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_bus_state_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_bus_state_result;
    Design_bus_state_result Design_bus_state(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_rip_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_rip_result;
    Design_rip_result Design_rip(nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct Design_flatten_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_flatten_result;
    Design_flatten_result Design_flatten(const nucleation::capi::Design* self);

    typedef struct Design_check_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_check_result;
    Design_check_result Design_check(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_bake_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_bake_result;
    Design_bake_result Design_bake(const nucleation::capi::Design* self, uint32_t budget);

    typedef struct Design_to_nucm_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_to_nucm_b64_result;
    Design_to_nucm_b64_result Design_to_nucm_b64(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_from_nucm_result {union {nucleation::capi::Design* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_from_nucm_result;
    Design_from_nucm_result Design_from_nucm(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Design_save_nucm_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_save_nucm_result;
    Design_save_nucm_result Design_save_nucm(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Design_load_nucm_result {union {nucleation::capi::Design* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_load_nucm_result;
    Design_load_nucm_result Design_load_nucm(nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Design_to_litematic_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_to_litematic_b64_result;
    Design_to_litematic_b64_result Design_to_litematic_b64(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Design_from_litematic_result {union {nucleation::capi::Design* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_from_litematic_result;
    Design_from_litematic_result Design_from_litematic(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Design_export_litematic_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Design_export_litematic_result;
    Design_export_litematic_result Design_export_litematic(const nucleation::capi::Design* self, nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Design_import_litematic_result {union {nucleation::capi::Design* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Design_import_litematic_result;
    Design_import_litematic_result Design_import_litematic(nucleation::diplomat::capi::DiplomatStringView path);

    void Design_destroy(Design* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> nucleation::Design::create(std::string_view name) {
    auto result = nucleation::capi::Design_create({name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Design>>(std::unique_ptr<nucleation::Design>(nucleation::Design::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> nucleation::Design::for_schematic(std::string_view name, const nucleation::Schematic& base) {
    auto result = nucleation::capi::Design_for_schematic({name.data(), name.size()},
        base.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Design>>(std::unique_ptr<nucleation::Design>(nucleation::Design::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::add_cell(std::string_view name, const nucleation::Schematic& cell) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_add_cell(this->AsFFI(),
        {name.data(), name.size()},
        cell.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::add_cell_write(std::string_view name, const nucleation::Schematic& cell, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_add_cell(this->AsFFI(),
        {name.data(), name.size()},
        cell.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::place(std::string_view name, std::string_view cell, int32_t x, int32_t y, int32_t z, int32_t rot_y) {
    auto result = nucleation::capi::Design_place(this->AsFFI(),
        {name.data(), name.size()},
        {cell.data(), cell.size()},
        x,
        y,
        z,
        rot_y);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::declare_input(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty) {
    auto result = nucleation::capi::Design_declare_input(this->AsFFI(),
        {name.data(), name.size()},
        ax,
        ay,
        az,
        sx,
        sy,
        sz,
        width,
        {ty.data(), ty.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::declare_output(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty) {
    auto result = nucleation::capi::Design_declare_output(this->AsFFI(),
        {name.data(), name.size()},
        ax,
        ay,
        az,
        sx,
        sy,
        sz,
        width,
        {ty.data(), ty.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::route_bus(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_route_bus(this->AsFFI(),
        {name.data(), name.size()},
        {driver.data(), driver.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::route_bus_write(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_route_bus(this->AsFFI(),
        {name.data(), name.size()},
        {driver.data(), driver.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::route_bus_or(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_route_bus_or(this->AsFFI(),
        {name.data(), name.size()},
        {drivers_json.data(), drivers_json.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::route_bus_or_write(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_route_bus_or(this->AsFFI(),
        {name.data(), name.size()},
        {drivers_json.data(), drivers_json.size()},
        {sinks_json.data(), sinks_json.size()},
        {gates_json.data(), gates_json.size()},
        {style_json.data(), style_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::set_block(int32_t x, int32_t y, int32_t z, std::string_view block) {
    auto result = nucleation::capi::Design_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block.data(), block.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::move_instance(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_move_instance(this->AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z,
        rot_y,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::move_instance_write(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_move_instance(this->AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z,
        rot_y,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::remove_instance(std::string_view name) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_remove_instance(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::remove_instance_write(std::string_view name, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_remove_instance(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::reroute(std::string_view name) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_reroute(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::reroute_write(std::string_view name, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_reroute(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::remove_bus(std::string_view name) {
    auto result = nucleation::capi::Design_remove_bus(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::to_schem_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_to_schem_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::to_schem_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_to_schem_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Design::flatten_composite() const {
    auto result = nucleation::capi::Design_flatten_composite(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::instance_ports() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_instance_ports(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::instance_ports_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_instance_ports(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::resolve_port(std::string_view name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_resolve_port(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::resolve_port_write(std::string_view name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_resolve_port(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::add_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_add_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        sx,
        sy,
        sz,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::add_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_add_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        sx,
        sy,
        sz,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::move_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_move_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::move_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_move_gate(this->AsFFI(),
        {bus.data(), bus.size()},
        {gate.data(), gate.size()},
        x,
        y,
        z,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::set_bus_rule(std::string_view bus, std::string_view rule_json) {
    auto result = nucleation::capi::Design_set_bus_rule(this->AsFFI(),
        {bus.data(), bus.size()},
        {rule_json.data(), rule_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::bus_skew(std::string_view name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_bus_skew(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::bus_skew_write(std::string_view name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_bus_skew(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::bus_state(std::string_view name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_bus_state(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::bus_state_write(std::string_view name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_bus_state(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::rip(std::string_view name) {
    auto result = nucleation::capi::Design_rip(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Design::flatten() const {
    auto result = nucleation::capi::Design_flatten(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::check() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_check(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::check_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_check(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Design::bake(uint32_t budget) const {
    auto result = nucleation::capi::Design_bake(this->AsFFI(),
        budget);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::to_nucm_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_to_nucm_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::to_nucm_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_to_nucm_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> nucleation::Design::from_nucm(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Design_from_nucm({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Design>>(std::unique_ptr<nucleation::Design>(nucleation::Design::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::save_nucm(std::string_view path) const {
    auto result = nucleation::capi::Design_save_nucm(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> nucleation::Design::load_nucm(std::string_view path) {
    auto result = nucleation::capi::Design_load_nucm({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Design>>(std::unique_ptr<nucleation::Design>(nucleation::Design::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Design::to_litematic_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Design_to_litematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::to_litematic_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Design_to_litematic_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> nucleation::Design::from_litematic(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Design_from_litematic({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Design>>(std::unique_ptr<nucleation::Design>(nucleation::Design::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Design::export_litematic(std::string_view path) const {
    auto result = nucleation::capi::Design_export_litematic(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> nucleation::Design::import_litematic(std::string_view path) {
    auto result = nucleation::capi::Design_import_litematic({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Design>>(std::unique_ptr<nucleation::Design>(nucleation::Design::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Design* nucleation::Design::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Design*>(this);
}

inline nucleation::capi::Design* nucleation::Design::AsFFI() {
    return reinterpret_cast<nucleation::capi::Design*>(this);
}

inline const nucleation::Design* nucleation::Design::FromFFI(const nucleation::capi::Design* ptr) {
    return reinterpret_cast<const nucleation::Design*>(ptr);
}

inline nucleation::Design* nucleation::Design::FromFFI(nucleation::capi::Design* ptr) {
    return reinterpret_cast<nucleation::Design*>(ptr);
}

inline void nucleation::Design::operator delete(void* ptr) {
    nucleation::capi::Design_destroy(reinterpret_cast<nucleation::capi::Design*>(ptr));
}


#endif // NUCLEATION_Design_HPP
