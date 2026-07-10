#ifndef NUCLEATION_Nbt_HPP
#define NUCLEATION_Nbt_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Nbt_text_build_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Nbt_text_build_result;
    Nbt_text_build_result Nbt_text_build(nucleation::diplomat::capi::DiplomatStringView s, nucleation::diplomat::capi::DiplomatStringView color, int32_t bold, int32_t italic, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Nbt_chest_build_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Nbt_chest_build_result;
    Nbt_chest_build_result Nbt_chest_build(nucleation::diplomat::capi::DiplomatStringView items_json, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Nbt_sign_build_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Nbt_sign_build_result;
    Nbt_sign_build_result Nbt_sign_build(nucleation::diplomat::capi::DiplomatStringView front_json, nucleation::diplomat::capi::DiplomatStringView back_json, nucleation::diplomat::capi::DiplomatStringView color, bool glowing, bool waxed, nucleation::diplomat::capi::DiplomatWrite* write);

    void Nbt_destroy(Nbt* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Nbt::text_build(std::string_view s, std::string_view color, int32_t bold, int32_t italic) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Nbt_text_build({s.data(), s.size()},
        {color.data(), color.size()},
        bold,
        italic,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Nbt::text_build_write(std::string_view s, std::string_view color, int32_t bold, int32_t italic, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Nbt_text_build({s.data(), s.size()},
        {color.data(), color.size()},
        bold,
        italic,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Nbt::chest_build(std::string_view items_json, std::string_view name) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Nbt_chest_build({items_json.data(), items_json.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Nbt::chest_build_write(std::string_view items_json, std::string_view name, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Nbt_chest_build({items_json.data(), items_json.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Nbt::sign_build(std::string_view front_json, std::string_view back_json, std::string_view color, bool glowing, bool waxed) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Nbt_sign_build({front_json.data(), front_json.size()},
        {back_json.data(), back_json.size()},
        {color.data(), color.size()},
        glowing,
        waxed,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Nbt::sign_build_write(std::string_view front_json, std::string_view back_json, std::string_view color, bool glowing, bool waxed, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Nbt_sign_build({front_json.data(), front_json.size()},
        {back_json.data(), back_json.size()},
        {color.data(), color.size()},
        glowing,
        waxed,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Nbt* nucleation::Nbt::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Nbt*>(this);
}

inline nucleation::capi::Nbt* nucleation::Nbt::AsFFI() {
    return reinterpret_cast<nucleation::capi::Nbt*>(this);
}

inline const nucleation::Nbt* nucleation::Nbt::FromFFI(const nucleation::capi::Nbt* ptr) {
    return reinterpret_cast<const nucleation::Nbt*>(ptr);
}

inline nucleation::Nbt* nucleation::Nbt::FromFFI(nucleation::capi::Nbt* ptr) {
    return reinterpret_cast<nucleation::Nbt*>(ptr);
}

inline void nucleation::Nbt::operator delete(void* ptr) {
    nucleation::capi::Nbt_destroy(reinterpret_cast<nucleation::capi::Nbt*>(ptr));
}


#endif // NUCLEATION_Nbt_HPP
