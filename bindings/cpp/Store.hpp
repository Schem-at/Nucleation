#ifndef Store_HPP
#define Store_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Store_open_result {union {diplomat::capi::Store* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Store_open_result;
    Store_open_result Store_open(diplomat::capi::DiplomatStringView url);

    typedef struct Store_get_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Store_get_b64_result;
    Store_get_b64_result Store_get_b64(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatWrite* write);

    typedef struct Store_put_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Store_put_result;
    Store_put_result Store_put(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatU8View data);

    typedef struct Store_exists_result {union {bool ok; diplomat::capi::NucleationError err;}; bool is_ok;} Store_exists_result;
    Store_exists_result Store_exists(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView key);

    typedef struct Store_delete_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Store_delete_result;
    Store_delete_result Store_delete(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView key);

    typedef struct Store_list_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Store_list_result;
    Store_list_result Store_list(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView prefix, diplomat::capi::DiplomatWrite* write);

    typedef struct Store_put_if_absent_result {union {bool ok; diplomat::capi::NucleationError err;}; bool is_ok;} Store_put_if_absent_result;
    Store_put_if_absent_result Store_put_if_absent(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatU8View data);

    typedef struct Store_list_paginated_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Store_list_paginated_result;
    Store_list_paginated_result Store_list_paginated(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView prefix, diplomat::capi::DiplomatStringView after, uint32_t limit, diplomat::capi::DiplomatWrite* write);

    typedef struct Store_health_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Store_health_result;
    Store_health_result Store_health(const diplomat::capi::Store* self);

    typedef struct Store_open_schematic_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Store_open_schematic_result;
    Store_open_schematic_result Store_open_schematic(const diplomat::capi::Store* self, diplomat::capi::DiplomatStringView key);

    typedef struct Store_save_schematic_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Store_save_schematic_result;
    Store_save_schematic_result Store_save_schematic(const diplomat::capi::Store* self, const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatStringView version);

    void Store_destroy(Store* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Store>, NucleationError> Store::open(std::string_view url) {
    auto result = diplomat::capi::Store_open({url.data(), url.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Store>, NucleationError>(diplomat::Ok<std::unique_ptr<Store>>(std::unique_ptr<Store>(Store::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Store>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Store::get_b64(std::string_view key) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Store_get_b64(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Store::get_b64_write(std::string_view key, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Store_get_b64(this->AsFFI(),
        {key.data(), key.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Store::put(std::string_view key, diplomat::span<const uint8_t> data) const {
    auto result = diplomat::capi::Store_put(this->AsFFI(),
        {key.data(), key.size()},
        {data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<bool, NucleationError> Store::exists(std::string_view key) const {
    auto result = diplomat::capi::Store_exists(this->AsFFI(),
        {key.data(), key.size()});
    return result.is_ok ? diplomat::result<bool, NucleationError>(diplomat::Ok<bool>(result.ok)) : diplomat::result<bool, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Store::delete_(std::string_view key) const {
    auto result = diplomat::capi::Store_delete(this->AsFFI(),
        {key.data(), key.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Store::list(std::string_view prefix) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Store_list(this->AsFFI(),
        {prefix.data(), prefix.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Store::list_write(std::string_view prefix, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Store_list(this->AsFFI(),
        {prefix.data(), prefix.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<bool, NucleationError> Store::put_if_absent(std::string_view key, diplomat::span<const uint8_t> data) const {
    auto result = diplomat::capi::Store_put_if_absent(this->AsFFI(),
        {key.data(), key.size()},
        {data.data(), data.size()});
    return result.is_ok ? diplomat::result<bool, NucleationError>(diplomat::Ok<bool>(result.ok)) : diplomat::result<bool, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Store::list_paginated(std::string_view prefix, std::string_view after, uint32_t limit) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Store_list_paginated(this->AsFFI(),
        {prefix.data(), prefix.size()},
        {after.data(), after.size()},
        limit,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Store::list_paginated_write(std::string_view prefix, std::string_view after, uint32_t limit, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Store_list_paginated(this->AsFFI(),
        {prefix.data(), prefix.size()},
        {after.data(), after.size()},
        limit,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Store::health() const {
    auto result = diplomat::capi::Store_health(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Store::open_schematic(std::string_view key) const {
    auto result = diplomat::capi::Store_open_schematic(this->AsFFI(),
        {key.data(), key.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Store::save_schematic(const Schematic& schematic, std::string_view key, std::string_view version) const {
    auto result = diplomat::capi::Store_save_schematic(this->AsFFI(),
        schematic.AsFFI(),
        {key.data(), key.size()},
        {version.data(), version.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Store* Store::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Store*>(this);
}

inline diplomat::capi::Store* Store::AsFFI() {
    return reinterpret_cast<diplomat::capi::Store*>(this);
}

inline const Store* Store::FromFFI(const diplomat::capi::Store* ptr) {
    return reinterpret_cast<const Store*>(ptr);
}

inline Store* Store::FromFFI(diplomat::capi::Store* ptr) {
    return reinterpret_cast<Store*>(ptr);
}

inline void Store::operator delete(void* ptr) {
    diplomat::capi::Store_destroy(reinterpret_cast<diplomat::capi::Store*>(ptr));
}


#endif // Store_HPP
