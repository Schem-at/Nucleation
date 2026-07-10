#ifndef NUCLEATION_Store_HPP
#define NUCLEATION_Store_HPP

#include "Store.d.hpp"

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

    typedef struct Store_open_result {union {nucleation::capi::Store* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Store_open_result;
    Store_open_result Store_open(nucleation::diplomat::capi::DiplomatStringView url);

    typedef struct Store_get_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Store_get_b64_result;
    Store_get_b64_result Store_get_b64(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Store_put_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Store_put_result;
    Store_put_result Store_put(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Store_exists_result {union {bool ok; nucleation::capi::NucleationError err;}; bool is_ok;} Store_exists_result;
    Store_exists_result Store_exists(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView key);

    typedef struct Store_delete_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Store_delete_result;
    Store_delete_result Store_delete(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView key);

    typedef struct Store_list_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Store_list_result;
    Store_list_result Store_list(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView prefix, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Store_put_if_absent_result {union {bool ok; nucleation::capi::NucleationError err;}; bool is_ok;} Store_put_if_absent_result;
    Store_put_if_absent_result Store_put_if_absent(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Store_list_paginated_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Store_list_paginated_result;
    Store_list_paginated_result Store_list_paginated(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView prefix, nucleation::diplomat::capi::DiplomatStringView after, uint32_t limit, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Store_health_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Store_health_result;
    Store_health_result Store_health(const nucleation::capi::Store* self);

    typedef struct Store_open_schematic_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Store_open_schematic_result;
    Store_open_schematic_result Store_open_schematic(const nucleation::capi::Store* self, nucleation::diplomat::capi::DiplomatStringView key);

    typedef struct Store_save_schematic_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Store_save_schematic_result;
    Store_save_schematic_result Store_save_schematic(const nucleation::capi::Store* self, const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatStringView version);

    void Store_destroy(Store* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Store>, nucleation::NucleationError> nucleation::Store::open(std::string_view url) {
    auto result = nucleation::capi::Store_open({url.data(), url.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Store>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Store>>(std::unique_ptr<nucleation::Store>(nucleation::Store::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Store>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Store::get_b64(std::string_view key) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Store_get_b64(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Store::get_b64_write(std::string_view key, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Store_get_b64(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Store::put(std::string_view key, nucleation::diplomat::span<const uint8_t> data) const {
    auto result = nucleation::capi::Store_put(this->AsFFI(),
        {key.data(), key.size()},
        {data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<bool, nucleation::NucleationError> nucleation::Store::exists(std::string_view key) const {
    auto result = nucleation::capi::Store_exists(this->AsFFI(),
        {key.data(), key.size()});
    return result.is_ok ? nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Ok<bool>(result.ok)) : nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Store::delete_(std::string_view key) const {
    auto result = nucleation::capi::Store_delete(this->AsFFI(),
        {key.data(), key.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Store::list(std::string_view prefix) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Store_list(this->AsFFI(),
        {prefix.data(), prefix.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Store::list_write(std::string_view prefix, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Store_list(this->AsFFI(),
        {prefix.data(), prefix.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<bool, nucleation::NucleationError> nucleation::Store::put_if_absent(std::string_view key, nucleation::diplomat::span<const uint8_t> data) const {
    auto result = nucleation::capi::Store_put_if_absent(this->AsFFI(),
        {key.data(), key.size()},
        {data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Ok<bool>(result.ok)) : nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Store::list_paginated(std::string_view prefix, std::string_view after, uint32_t limit) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Store_list_paginated(this->AsFFI(),
        {prefix.data(), prefix.size()},
        {after.data(), after.size()},
        limit,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Store::list_paginated_write(std::string_view prefix, std::string_view after, uint32_t limit, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Store_list_paginated(this->AsFFI(),
        {prefix.data(), prefix.size()},
        {after.data(), after.size()},
        limit,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Store::health() const {
    auto result = nucleation::capi::Store_health(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Store::open_schematic(std::string_view key) const {
    auto result = nucleation::capi::Store_open_schematic(this->AsFFI(),
        {key.data(), key.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Store::save_schematic(const nucleation::Schematic& schematic, std::string_view key, std::string_view version) const {
    auto result = nucleation::capi::Store_save_schematic(this->AsFFI(),
        schematic.AsFFI(),
        {key.data(), key.size()},
        {version.data(), version.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Store* nucleation::Store::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Store*>(this);
}

inline nucleation::capi::Store* nucleation::Store::AsFFI() {
    return reinterpret_cast<nucleation::capi::Store*>(this);
}

inline const nucleation::Store* nucleation::Store::FromFFI(const nucleation::capi::Store* ptr) {
    return reinterpret_cast<const nucleation::Store*>(ptr);
}

inline nucleation::Store* nucleation::Store::FromFFI(nucleation::capi::Store* ptr) {
    return reinterpret_cast<nucleation::Store*>(ptr);
}

inline void nucleation::Store::operator delete(void* ptr) {
    nucleation::capi::Store_destroy(reinterpret_cast<nucleation::capi::Store*>(ptr));
}


#endif // NUCLEATION_Store_HPP
