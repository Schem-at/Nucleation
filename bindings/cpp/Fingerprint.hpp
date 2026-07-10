#ifndef Fingerprint_HPP
#define Fingerprint_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Fingerprint_compute_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Fingerprint_compute_result;
    Fingerprint_compute_result Fingerprint_compute(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView preset, diplomat::capi::DiplomatWrite* write);

    typedef struct Fingerprint_signature_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Fingerprint_signature_json_result;
    Fingerprint_signature_json_result Fingerprint_signature_json(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView preset, diplomat::capi::DiplomatWrite* write);

    typedef struct Fingerprint_footprint_distance_result {union {float ok; diplomat::capi::NucleationError err;}; bool is_ok;} Fingerprint_footprint_distance_result;
    Fingerprint_footprint_distance_result Fingerprint_footprint_distance(const diplomat::capi::Schematic* a, const diplomat::capi::Schematic* b, diplomat::capi::DiplomatStringView preset);

    typedef struct Fingerprint_footprint_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Fingerprint_footprint_json_result;
    Fingerprint_footprint_json_result Fingerprint_footprint_json(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView preset, diplomat::capi::DiplomatWrite* write);

    typedef struct Fingerprint_is_duplicate_result {union {bool ok; diplomat::capi::NucleationError err;}; bool is_ok;} Fingerprint_is_duplicate_result;
    Fingerprint_is_duplicate_result Fingerprint_is_duplicate(const diplomat::capi::Schematic* a, const diplomat::capi::Schematic* b, diplomat::capi::DiplomatStringView preset);

    void Fingerprint_destroy(Fingerprint* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::string, NucleationError> Fingerprint::compute(const Schematic& schematic, std::string_view preset) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Fingerprint_compute(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Fingerprint::compute_write(const Schematic& schematic, std::string_view preset, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Fingerprint_compute(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Fingerprint::signature_json(const Schematic& schematic, std::string_view preset) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Fingerprint_signature_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Fingerprint::signature_json_write(const Schematic& schematic, std::string_view preset, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Fingerprint_signature_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<float, NucleationError> Fingerprint::footprint_distance(const Schematic& a, const Schematic& b, std::string_view preset) {
    auto result = diplomat::capi::Fingerprint_footprint_distance(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()});
    return result.is_ok ? diplomat::result<float, NucleationError>(diplomat::Ok<float>(result.ok)) : diplomat::result<float, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Fingerprint::footprint_json(const Schematic& schematic, std::string_view preset) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Fingerprint_footprint_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Fingerprint::footprint_json_write(const Schematic& schematic, std::string_view preset, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Fingerprint_footprint_json(schematic.AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<bool, NucleationError> Fingerprint::is_duplicate(const Schematic& a, const Schematic& b, std::string_view preset) {
    auto result = diplomat::capi::Fingerprint_is_duplicate(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()});
    return result.is_ok ? diplomat::result<bool, NucleationError>(diplomat::Ok<bool>(result.ok)) : diplomat::result<bool, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Fingerprint* Fingerprint::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Fingerprint*>(this);
}

inline diplomat::capi::Fingerprint* Fingerprint::AsFFI() {
    return reinterpret_cast<diplomat::capi::Fingerprint*>(this);
}

inline const Fingerprint* Fingerprint::FromFFI(const diplomat::capi::Fingerprint* ptr) {
    return reinterpret_cast<const Fingerprint*>(ptr);
}

inline Fingerprint* Fingerprint::FromFFI(diplomat::capi::Fingerprint* ptr) {
    return reinterpret_cast<Fingerprint*>(ptr);
}

inline void Fingerprint::operator delete(void* ptr) {
    diplomat::capi::Fingerprint_destroy(reinterpret_cast<diplomat::capi::Fingerprint*>(ptr));
}


#endif // Fingerprint_HPP
