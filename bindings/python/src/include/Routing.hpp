#ifndef NUCLEATION_Routing_HPP
#define NUCLEATION_Routing_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Routing_route_net_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Routing_route_net_result;
    Routing_route_net_result Routing_route_net(nucleation::capi::Schematic* schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, nucleation::diplomat::capi::DiplomatStringView label, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Routing_drc_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Routing_drc_result;
    Routing_drc_result Routing_drc(const nucleation::capi::Schematic* schematic, bool check_decay, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Routing_sta_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Routing_sta_result;
    Routing_sta_result Routing_sta(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView netlist_json, nucleation::diplomat::capi::DiplomatWrite* write);

    void Routing_destroy(Routing* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Routing::route_net(nucleation::Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Routing_route_net(schematic.AsFFI(),
        sx,
        sy,
        sz,
        dx,
        dy,
        dz,
        {label.data(), label.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Routing::route_net_write(nucleation::Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Routing_route_net(schematic.AsFFI(),
        sx,
        sy,
        sz,
        dx,
        dy,
        dz,
        {label.data(), label.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Routing::drc(const nucleation::Schematic& schematic, bool check_decay) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Routing_drc(schematic.AsFFI(),
        check_decay,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Routing::drc_write(const nucleation::Schematic& schematic, bool check_decay, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Routing_drc(schematic.AsFFI(),
        check_decay,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Routing::sta(const nucleation::Schematic& schematic, std::string_view netlist_json) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Routing_sta(schematic.AsFFI(),
        {netlist_json.data(), netlist_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Routing::sta_write(const nucleation::Schematic& schematic, std::string_view netlist_json, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Routing_sta(schematic.AsFFI(),
        {netlist_json.data(), netlist_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Routing* nucleation::Routing::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Routing*>(this);
}

inline nucleation::capi::Routing* nucleation::Routing::AsFFI() {
    return reinterpret_cast<nucleation::capi::Routing*>(this);
}

inline const nucleation::Routing* nucleation::Routing::FromFFI(const nucleation::capi::Routing* ptr) {
    return reinterpret_cast<const nucleation::Routing*>(ptr);
}

inline nucleation::Routing* nucleation::Routing::FromFFI(nucleation::capi::Routing* ptr) {
    return reinterpret_cast<nucleation::Routing*>(ptr);
}

inline void nucleation::Routing::operator delete(void* ptr) {
    nucleation::capi::Routing_destroy(reinterpret_cast<nucleation::capi::Routing*>(ptr));
}


#endif // NUCLEATION_Routing_HPP
