#ifndef Nbt_HPP
#define Nbt_HPP

#include "Nbt.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Nbt_text_build_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Nbt_text_build_result;
    Nbt_text_build_result Nbt_text_build(diplomat::capi::DiplomatStringView s, diplomat::capi::DiplomatStringView color, int32_t bold, int32_t italic, diplomat::capi::DiplomatWrite* write);

    typedef struct Nbt_chest_build_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Nbt_chest_build_result;
    Nbt_chest_build_result Nbt_chest_build(diplomat::capi::DiplomatStringView items_json, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    typedef struct Nbt_sign_build_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Nbt_sign_build_result;
    Nbt_sign_build_result Nbt_sign_build(diplomat::capi::DiplomatStringView front_json, diplomat::capi::DiplomatStringView back_json, diplomat::capi::DiplomatStringView color, bool glowing, bool waxed, diplomat::capi::DiplomatWrite* write);

    void Nbt_destroy(Nbt* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::string, NucleationError> Nbt::text_build(std::string_view s, std::string_view color, int32_t bold, int32_t italic) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Nbt_text_build({s.data(), s.size()},
        {color.data(), color.size()},
        bold,
        italic,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Nbt::text_build_write(std::string_view s, std::string_view color, int32_t bold, int32_t italic, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Nbt_text_build({s.data(), s.size()},
        {color.data(), color.size()},
        bold,
        italic,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Nbt::chest_build(std::string_view items_json, std::string_view name) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Nbt_chest_build({items_json.data(), items_json.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Nbt::chest_build_write(std::string_view items_json, std::string_view name, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Nbt_chest_build({items_json.data(), items_json.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Nbt::sign_build(std::string_view front_json, std::string_view back_json, std::string_view color, bool glowing, bool waxed) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Nbt_sign_build({front_json.data(), front_json.size()},
        {back_json.data(), back_json.size()},
        {color.data(), color.size()},
        glowing,
        waxed,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Nbt::sign_build_write(std::string_view front_json, std::string_view back_json, std::string_view color, bool glowing, bool waxed, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Nbt_sign_build({front_json.data(), front_json.size()},
        {back_json.data(), back_json.size()},
        {color.data(), color.size()},
        glowing,
        waxed,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Nbt* Nbt::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Nbt*>(this);
}

inline diplomat::capi::Nbt* Nbt::AsFFI() {
    return reinterpret_cast<diplomat::capi::Nbt*>(this);
}

inline const Nbt* Nbt::FromFFI(const diplomat::capi::Nbt* ptr) {
    return reinterpret_cast<const Nbt*>(ptr);
}

inline Nbt* Nbt::FromFFI(diplomat::capi::Nbt* ptr) {
    return reinterpret_cast<Nbt*>(ptr);
}

inline void Nbt::operator delete(void* ptr) {
    diplomat::capi::Nbt_destroy(reinterpret_cast<diplomat::capi::Nbt*>(ptr));
}


#endif // Nbt_HPP
