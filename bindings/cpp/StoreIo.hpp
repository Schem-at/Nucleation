#ifndef StoreIo_HPP
#define StoreIo_HPP

#include "StoreIo.d.hpp"

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

    typedef struct StoreIo_open_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_open_result;
    StoreIo_open_result StoreIo_open(diplomat::capi::DiplomatStringView uri);

    typedef struct StoreIo_save_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_save_result;
    StoreIo_save_result StoreIo_save(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView uri, diplomat::capi::DiplomatStringView version);

    typedef struct StoreIo_export_settings_schema_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_export_settings_schema_result;
    StoreIo_export_settings_schema_result StoreIo_export_settings_schema(diplomat::capi::DiplomatStringView format, diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_import_settings_schema_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_import_settings_schema_result;
    StoreIo_import_settings_schema_result StoreIo_import_settings_schema(diplomat::capi::DiplomatStringView format, diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_supported_import_formats_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_supported_import_formats_result;
    StoreIo_supported_import_formats_result StoreIo_supported_import_formats(diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_supported_export_formats_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_supported_export_formats_result;
    StoreIo_supported_export_formats_result StoreIo_supported_export_formats(diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_format_versions_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_format_versions_result;
    StoreIo_format_versions_result StoreIo_format_versions(diplomat::capi::DiplomatStringView format, diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_default_format_version_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} StoreIo_default_format_version_result;
    StoreIo_default_format_version_result StoreIo_default_format_version(diplomat::capi::DiplomatStringView format, diplomat::capi::DiplomatWrite* write);

    void StoreIo_destroy(StoreIo* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> StoreIo::open(std::string_view uri) {
    auto result = diplomat::capi::StoreIo_open({uri.data(), uri.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> StoreIo::save(const Schematic& schematic, std::string_view uri, std::string_view version) {
    auto result = diplomat::capi::StoreIo_save(schematic.AsFFI(),
        {uri.data(), uri.size()},
        {version.data(), version.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> StoreIo::export_settings_schema(std::string_view format) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::StoreIo_export_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> StoreIo::export_settings_schema_write(std::string_view format, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::StoreIo_export_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> StoreIo::import_settings_schema(std::string_view format) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::StoreIo_import_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> StoreIo::import_settings_schema_write(std::string_view format, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::StoreIo_import_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> StoreIo::supported_import_formats() {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::StoreIo_supported_import_formats(&write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> StoreIo::supported_import_formats_write(W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::StoreIo_supported_import_formats(&write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> StoreIo::supported_export_formats() {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::StoreIo_supported_export_formats(&write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> StoreIo::supported_export_formats_write(W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::StoreIo_supported_export_formats(&write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> StoreIo::format_versions(std::string_view format) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::StoreIo_format_versions({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> StoreIo::format_versions_write(std::string_view format, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::StoreIo_format_versions({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> StoreIo::default_format_version(std::string_view format) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::StoreIo_default_format_version({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> StoreIo::default_format_version_write(std::string_view format, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::StoreIo_default_format_version({format.data(), format.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::StoreIo* StoreIo::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::StoreIo*>(this);
}

inline diplomat::capi::StoreIo* StoreIo::AsFFI() {
    return reinterpret_cast<diplomat::capi::StoreIo*>(this);
}

inline const StoreIo* StoreIo::FromFFI(const diplomat::capi::StoreIo* ptr) {
    return reinterpret_cast<const StoreIo*>(ptr);
}

inline StoreIo* StoreIo::FromFFI(diplomat::capi::StoreIo* ptr) {
    return reinterpret_cast<StoreIo*>(ptr);
}

inline void StoreIo::operator delete(void* ptr) {
    diplomat::capi::StoreIo_destroy(reinterpret_cast<diplomat::capi::StoreIo*>(ptr));
}


#endif // StoreIo_HPP
