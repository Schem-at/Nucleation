#ifndef NUCLEATION_Blocks_HPP
#define NUCLEATION_Blocks_HPP

#include "Blocks.d.hpp"

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

    typedef struct Blocks_get_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Blocks_get_json_result;
    Blocks_get_json_result Blocks_get_json(nucleation::diplomat::capi::DiplomatStringView id, nucleation::diplomat::capi::DiplomatWrite* write);

    void Blocks_ids_json(nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_by_tag_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Blocks_by_tag_json_result;
    Blocks_by_tag_json_result Blocks_by_tag_json(nucleation::diplomat::capi::DiplomatStringView tag, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_by_kind_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Blocks_by_kind_json_result;
    Blocks_by_kind_json_result Blocks_by_kind_json(nucleation::diplomat::capi::DiplomatStringView kind, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_variants_of_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Blocks_variants_of_json_result;
    Blocks_variants_of_json_result Blocks_variants_of_json(nucleation::diplomat::capi::DiplomatStringView base_id, nucleation::diplomat::capi::DiplomatWrite* write);

    void Blocks_tags_json(nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_states_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Blocks_states_json_result;
    Blocks_states_json_result Blocks_states_json(nucleation::diplomat::capi::DiplomatStringView id, nucleation::diplomat::capi::DiplomatWrite* write);

    size_t Blocks_count(void);

    void Blocks_destroy(Blocks* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Blocks::get_json(std::string_view id) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Blocks_get_json({id.data(), id.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Blocks::get_json_write(std::string_view id, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Blocks_get_json({id.data(), id.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Blocks::ids_json() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Blocks_ids_json(&write);
    return output;
}
template<typename W>
inline void nucleation::Blocks::ids_json_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Blocks_ids_json(&write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Blocks::by_tag_json(std::string_view tag) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Blocks_by_tag_json({tag.data(), tag.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Blocks::by_tag_json_write(std::string_view tag, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Blocks_by_tag_json({tag.data(), tag.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Blocks::by_kind_json(std::string_view kind) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Blocks_by_kind_json({kind.data(), kind.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Blocks::by_kind_json_write(std::string_view kind, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Blocks_by_kind_json({kind.data(), kind.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Blocks::variants_of_json(std::string_view base_id) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Blocks_variants_of_json({base_id.data(), base_id.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Blocks::variants_of_json_write(std::string_view base_id, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Blocks_variants_of_json({base_id.data(), base_id.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Blocks::tags_json() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Blocks_tags_json(&write);
    return output;
}
template<typename W>
inline void nucleation::Blocks::tags_json_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Blocks_tags_json(&write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Blocks::states_json(std::string_view id) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Blocks_states_json({id.data(), id.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Blocks::states_json_write(std::string_view id, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Blocks_states_json({id.data(), id.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline size_t nucleation::Blocks::count() {
    auto result = nucleation::capi::Blocks_count();
    return result;
}

inline const nucleation::capi::Blocks* nucleation::Blocks::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Blocks*>(this);
}

inline nucleation::capi::Blocks* nucleation::Blocks::AsFFI() {
    return reinterpret_cast<nucleation::capi::Blocks*>(this);
}

inline const nucleation::Blocks* nucleation::Blocks::FromFFI(const nucleation::capi::Blocks* ptr) {
    return reinterpret_cast<const nucleation::Blocks*>(ptr);
}

inline nucleation::Blocks* nucleation::Blocks::FromFFI(nucleation::capi::Blocks* ptr) {
    return reinterpret_cast<nucleation::Blocks*>(ptr);
}

inline void nucleation::Blocks::operator delete(void* ptr) {
    nucleation::capi::Blocks_destroy(reinterpret_cast<nucleation::capi::Blocks*>(ptr));
}


#endif // NUCLEATION_Blocks_HPP
