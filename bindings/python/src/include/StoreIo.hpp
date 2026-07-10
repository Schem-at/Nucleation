#ifndef NUCLEATION_StoreIo_HPP
#define NUCLEATION_StoreIo_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct StoreIo_open_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_open_result;
    StoreIo_open_result StoreIo_open(nucleation::diplomat::capi::DiplomatStringView uri);

    typedef struct StoreIo_save_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_save_result;
    StoreIo_save_result StoreIo_save(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView uri, nucleation::diplomat::capi::DiplomatStringView version);

    typedef struct StoreIo_export_settings_schema_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_export_settings_schema_result;
    StoreIo_export_settings_schema_result StoreIo_export_settings_schema(nucleation::diplomat::capi::DiplomatStringView format, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_import_settings_schema_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_import_settings_schema_result;
    StoreIo_import_settings_schema_result StoreIo_import_settings_schema(nucleation::diplomat::capi::DiplomatStringView format, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_supported_import_formats_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_supported_import_formats_result;
    StoreIo_supported_import_formats_result StoreIo_supported_import_formats(nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_supported_export_formats_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_supported_export_formats_result;
    StoreIo_supported_export_formats_result StoreIo_supported_export_formats(nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_format_versions_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_format_versions_result;
    StoreIo_format_versions_result StoreIo_format_versions(nucleation::diplomat::capi::DiplomatStringView format, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct StoreIo_default_format_version_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} StoreIo_default_format_version_result;
    StoreIo_default_format_version_result StoreIo_default_format_version(nucleation::diplomat::capi::DiplomatStringView format, nucleation::diplomat::capi::DiplomatWrite* write);

    void StoreIo_destroy(StoreIo* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::StoreIo::open(std::string_view uri) {
    auto result = nucleation::capi::StoreIo_open({uri.data(), uri.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::StoreIo::save(const nucleation::Schematic& schematic, std::string_view uri, std::string_view version) {
    auto result = nucleation::capi::StoreIo_save(schematic.AsFFI(),
        {uri.data(), uri.size()},
        {version.data(), version.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::StoreIo::export_settings_schema(std::string_view format) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::StoreIo_export_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::StoreIo::export_settings_schema_write(std::string_view format, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::StoreIo_export_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::StoreIo::import_settings_schema(std::string_view format) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::StoreIo_import_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::StoreIo::import_settings_schema_write(std::string_view format, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::StoreIo_import_settings_schema({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::StoreIo::supported_import_formats() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::StoreIo_supported_import_formats(&write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::StoreIo::supported_import_formats_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::StoreIo_supported_import_formats(&write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::StoreIo::supported_export_formats() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::StoreIo_supported_export_formats(&write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::StoreIo::supported_export_formats_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::StoreIo_supported_export_formats(&write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::StoreIo::format_versions(std::string_view format) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::StoreIo_format_versions({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::StoreIo::format_versions_write(std::string_view format, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::StoreIo_format_versions({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::StoreIo::default_format_version(std::string_view format) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::StoreIo_default_format_version({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::StoreIo::default_format_version_write(std::string_view format, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::StoreIo_default_format_version({format.data(), format.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::StoreIo* nucleation::StoreIo::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::StoreIo*>(this);
}

inline nucleation::capi::StoreIo* nucleation::StoreIo::AsFFI() {
    return reinterpret_cast<nucleation::capi::StoreIo*>(this);
}

inline const nucleation::StoreIo* nucleation::StoreIo::FromFFI(const nucleation::capi::StoreIo* ptr) {
    return reinterpret_cast<const nucleation::StoreIo*>(ptr);
}

inline nucleation::StoreIo* nucleation::StoreIo::FromFFI(nucleation::capi::StoreIo* ptr) {
    return reinterpret_cast<nucleation::StoreIo*>(ptr);
}

inline void nucleation::StoreIo::operator delete(void* ptr) {
    nucleation::capi::StoreIo_destroy(reinterpret_cast<nucleation::capi::StoreIo*>(ptr));
}


#endif // NUCLEATION_StoreIo_HPP
