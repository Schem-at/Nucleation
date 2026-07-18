#ifndef Blocks_HPP
#define Blocks_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Blocks_get_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Blocks_get_json_result;
    Blocks_get_json_result Blocks_get_json(diplomat::capi::DiplomatStringView id, diplomat::capi::DiplomatWrite* write);

    void Blocks_ids_json(diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_by_color_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Blocks_by_color_json_result;
    Blocks_by_color_json_result Blocks_by_color_json(uint8_t r, uint8_t g, uint8_t b, float max_distance, diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_by_tag_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Blocks_by_tag_json_result;
    Blocks_by_tag_json_result Blocks_by_tag_json(diplomat::capi::DiplomatStringView tag, diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_by_kind_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Blocks_by_kind_json_result;
    Blocks_by_kind_json_result Blocks_by_kind_json(diplomat::capi::DiplomatStringView kind, diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_variants_of_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Blocks_variants_of_json_result;
    Blocks_variants_of_json_result Blocks_variants_of_json(diplomat::capi::DiplomatStringView base_id, diplomat::capi::DiplomatWrite* write);

    void Blocks_tags_json(diplomat::capi::DiplomatWrite* write);

    typedef struct Blocks_states_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Blocks_states_json_result;
    Blocks_states_json_result Blocks_states_json(diplomat::capi::DiplomatStringView id, diplomat::capi::DiplomatWrite* write);

    size_t Blocks_count(void);

    void Blocks_destroy(Blocks* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::string, NucleationError> Blocks::get_json(std::string_view id) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Blocks_get_json({id.data(), id.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Blocks::get_json_write(std::string_view id, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Blocks_get_json({id.data(), id.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Blocks::ids_json() {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Blocks_ids_json(&write);
    return output;
}
template<typename W>
inline void Blocks::ids_json_write(W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Blocks_ids_json(&write);
}

inline diplomat::result<std::string, NucleationError> Blocks::by_color_json(uint8_t r, uint8_t g, uint8_t b, float max_distance) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Blocks_by_color_json(r,
        g,
        b,
        max_distance,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Blocks::by_color_json_write(uint8_t r, uint8_t g, uint8_t b, float max_distance, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Blocks_by_color_json(r,
        g,
        b,
        max_distance,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Blocks::by_tag_json(std::string_view tag) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Blocks_by_tag_json({tag.data(), tag.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Blocks::by_tag_json_write(std::string_view tag, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Blocks_by_tag_json({tag.data(), tag.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Blocks::by_kind_json(std::string_view kind) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Blocks_by_kind_json({kind.data(), kind.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Blocks::by_kind_json_write(std::string_view kind, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Blocks_by_kind_json({kind.data(), kind.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Blocks::variants_of_json(std::string_view base_id) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Blocks_variants_of_json({base_id.data(), base_id.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Blocks::variants_of_json_write(std::string_view base_id, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Blocks_variants_of_json({base_id.data(), base_id.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Blocks::tags_json() {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Blocks_tags_json(&write);
    return output;
}
template<typename W>
inline void Blocks::tags_json_write(W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Blocks_tags_json(&write);
}

inline diplomat::result<std::string, NucleationError> Blocks::states_json(std::string_view id) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Blocks_states_json({id.data(), id.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Blocks::states_json_write(std::string_view id, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Blocks_states_json({id.data(), id.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline size_t Blocks::count() {
    auto result = diplomat::capi::Blocks_count();
    return result;
}

inline const diplomat::capi::Blocks* Blocks::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Blocks*>(this);
}

inline diplomat::capi::Blocks* Blocks::AsFFI() {
    return reinterpret_cast<diplomat::capi::Blocks*>(this);
}

inline const Blocks* Blocks::FromFFI(const diplomat::capi::Blocks* ptr) {
    return reinterpret_cast<const Blocks*>(ptr);
}

inline Blocks* Blocks::FromFFI(diplomat::capi::Blocks* ptr) {
    return reinterpret_cast<Blocks*>(ptr);
}

inline void Blocks::operator delete(void* ptr) {
    diplomat::capi::Blocks_destroy(reinterpret_cast<diplomat::capi::Blocks*>(ptr));
}


#endif // Blocks_HPP
