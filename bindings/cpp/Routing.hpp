#ifndef Routing_HPP
#define Routing_HPP

#include "Routing.d.hpp"

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

    typedef struct Routing_route_net_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Routing_route_net_result;
    Routing_route_net_result Routing_route_net(diplomat::capi::Schematic* schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, diplomat::capi::DiplomatStringView label, diplomat::capi::DiplomatWrite* write);

    typedef struct Routing_route_all_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Routing_route_all_result;
    Routing_route_all_result Routing_route_all(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView nets_json, diplomat::capi::DiplomatWrite* write);

    typedef struct Routing_lvs_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Routing_lvs_result;
    Routing_lvs_result Routing_lvs(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView intent_json, diplomat::capi::DiplomatWrite* write);

    typedef struct Routing_drc_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Routing_drc_result;
    Routing_drc_result Routing_drc(const diplomat::capi::Schematic* schematic, bool check_decay, diplomat::capi::DiplomatWrite* write);

    typedef struct Routing_sta_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Routing_sta_result;
    Routing_sta_result Routing_sta(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView netlist_json, diplomat::capi::DiplomatWrite* write);

    void Routing_destroy(Routing* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::string, NucleationError> Routing::route_net(Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Routing_route_net(schematic.AsFFI(),
        sx,
        sy,
        sz,
        dx,
        dy,
        dz,
        {label.data(), label.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Routing::route_net_write(Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Routing_route_net(schematic.AsFFI(),
        sx,
        sy,
        sz,
        dx,
        dy,
        dz,
        {label.data(), label.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Routing::route_all(Schematic& schematic, std::string_view nets_json) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Routing_route_all(schematic.AsFFI(),
        {nets_json.data(), nets_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Routing::route_all_write(Schematic& schematic, std::string_view nets_json, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Routing_route_all(schematic.AsFFI(),
        {nets_json.data(), nets_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Routing::lvs(const Schematic& schematic, std::string_view intent_json) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Routing_lvs(schematic.AsFFI(),
        {intent_json.data(), intent_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Routing::lvs_write(const Schematic& schematic, std::string_view intent_json, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Routing_lvs(schematic.AsFFI(),
        {intent_json.data(), intent_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Routing::drc(const Schematic& schematic, bool check_decay) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Routing_drc(schematic.AsFFI(),
        check_decay,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Routing::drc_write(const Schematic& schematic, bool check_decay, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Routing_drc(schematic.AsFFI(),
        check_decay,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Routing::sta(const Schematic& schematic, std::string_view netlist_json) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Routing_sta(schematic.AsFFI(),
        {netlist_json.data(), netlist_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Routing::sta_write(const Schematic& schematic, std::string_view netlist_json, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Routing_sta(schematic.AsFFI(),
        {netlist_json.data(), netlist_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Routing* Routing::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Routing*>(this);
}

inline diplomat::capi::Routing* Routing::AsFFI() {
    return reinterpret_cast<diplomat::capi::Routing*>(this);
}

inline const Routing* Routing::FromFFI(const diplomat::capi::Routing* ptr) {
    return reinterpret_cast<const Routing*>(ptr);
}

inline Routing* Routing::FromFFI(diplomat::capi::Routing* ptr) {
    return reinterpret_cast<Routing*>(ptr);
}

inline void Routing::operator delete(void* ptr) {
    diplomat::capi::Routing_destroy(reinterpret_cast<diplomat::capi::Routing*>(ptr));
}


#endif // Routing_HPP
