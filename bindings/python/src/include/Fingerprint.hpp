#ifndef NUCLEATION_Fingerprint_HPP
#define NUCLEATION_Fingerprint_HPP

#include "Fingerprint.d.hpp"

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

    typedef struct Fingerprint_compute_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Fingerprint_compute_result;
    Fingerprint_compute_result Fingerprint_compute(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView preset, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Fingerprint_signature_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Fingerprint_signature_json_result;
    Fingerprint_signature_json_result Fingerprint_signature_json(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView preset, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Fingerprint_footprint_distance_result {union {float ok; nucleation::capi::NucleationError err;}; bool is_ok;} Fingerprint_footprint_distance_result;
    Fingerprint_footprint_distance_result Fingerprint_footprint_distance(const nucleation::capi::Schematic* a, const nucleation::capi::Schematic* b, nucleation::diplomat::capi::DiplomatStringView preset);

    typedef struct Fingerprint_footprint_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Fingerprint_footprint_json_result;
    Fingerprint_footprint_json_result Fingerprint_footprint_json(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView preset, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Fingerprint_is_duplicate_result {union {bool ok; nucleation::capi::NucleationError err;}; bool is_ok;} Fingerprint_is_duplicate_result;
    Fingerprint_is_duplicate_result Fingerprint_is_duplicate(const nucleation::capi::Schematic* a, const nucleation::capi::Schematic* b, nucleation::diplomat::capi::DiplomatStringView preset);

    void Fingerprint_destroy(Fingerprint* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Fingerprint::compute(const nucleation::Schematic& schematic, std::string_view preset) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Fingerprint_compute(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Fingerprint::compute_write(const nucleation::Schematic& schematic, std::string_view preset, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Fingerprint_compute(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Fingerprint::signature_json(const nucleation::Schematic& schematic, std::string_view preset) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Fingerprint_signature_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Fingerprint::signature_json_write(const nucleation::Schematic& schematic, std::string_view preset, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Fingerprint_signature_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<float, nucleation::NucleationError> nucleation::Fingerprint::footprint_distance(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset) {
    auto result = nucleation::capi::Fingerprint_footprint_distance(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()});
    return result.is_ok ? nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Ok<float>(result.ok)) : nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Fingerprint::footprint_json(const nucleation::Schematic& schematic, std::string_view preset) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Fingerprint_footprint_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Fingerprint::footprint_json_write(const nucleation::Schematic& schematic, std::string_view preset, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Fingerprint_footprint_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<bool, nucleation::NucleationError> nucleation::Fingerprint::is_duplicate(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset) {
    auto result = nucleation::capi::Fingerprint_is_duplicate(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()});
    return result.is_ok ? nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Ok<bool>(result.ok)) : nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Fingerprint* nucleation::Fingerprint::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Fingerprint*>(this);
}

inline nucleation::capi::Fingerprint* nucleation::Fingerprint::AsFFI() {
    return reinterpret_cast<nucleation::capi::Fingerprint*>(this);
}

inline const nucleation::Fingerprint* nucleation::Fingerprint::FromFFI(const nucleation::capi::Fingerprint* ptr) {
    return reinterpret_cast<const nucleation::Fingerprint*>(ptr);
}

inline nucleation::Fingerprint* nucleation::Fingerprint::FromFFI(nucleation::capi::Fingerprint* ptr) {
    return reinterpret_cast<nucleation::Fingerprint*>(ptr);
}

inline void nucleation::Fingerprint::operator delete(void* ptr) {
    nucleation::capi::Fingerprint_destroy(reinterpret_cast<nucleation::capi::Fingerprint*>(ptr));
}


#endif // NUCLEATION_Fingerprint_HPP
