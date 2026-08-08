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
