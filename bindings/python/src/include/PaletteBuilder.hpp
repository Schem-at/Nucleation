#ifndef NUCLEATION_PaletteBuilder_HPP
#define NUCLEATION_PaletteBuilder_HPP

#include "PaletteBuilder.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Palette.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::PaletteBuilder* PaletteBuilder_create(void);

    typedef struct PaletteBuilder_exclude_falling_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_falling_result;
    PaletteBuilder_exclude_falling_result PaletteBuilder_exclude_falling(nucleation::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_tile_entities_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_tile_entities_result;
    PaletteBuilder_exclude_tile_entities_result PaletteBuilder_exclude_tile_entities(nucleation::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_full_blocks_only_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_full_blocks_only_result;
    PaletteBuilder_full_blocks_only_result PaletteBuilder_full_blocks_only(nucleation::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_needs_support_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_needs_support_result;
    PaletteBuilder_exclude_needs_support_result PaletteBuilder_exclude_needs_support(nucleation::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_transparent_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_transparent_result;
    PaletteBuilder_exclude_transparent_result PaletteBuilder_exclude_transparent(nucleation::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_light_sources_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_light_sources_result;
    PaletteBuilder_exclude_light_sources_result PaletteBuilder_exclude_light_sources(nucleation::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_survival_only_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_survival_only_result;
    PaletteBuilder_survival_only_result PaletteBuilder_survival_only(nucleation::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_keyword_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_keyword_result;
    PaletteBuilder_exclude_keyword_result PaletteBuilder_exclude_keyword(nucleation::capi::PaletteBuilder* self, nucleation::diplomat::capi::DiplomatStringView keyword);

    typedef struct PaletteBuilder_include_keyword_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_include_keyword_result;
    PaletteBuilder_include_keyword_result PaletteBuilder_include_keyword(nucleation::capi::PaletteBuilder* self, nucleation::diplomat::capi::DiplomatStringView keyword);

    typedef struct PaletteBuilder_tag_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_tag_result;
    PaletteBuilder_tag_result PaletteBuilder_tag(nucleation::capi::PaletteBuilder* self, nucleation::diplomat::capi::DiplomatStringView t);

    typedef struct PaletteBuilder_exclude_tag_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_tag_result;
    PaletteBuilder_exclude_tag_result PaletteBuilder_exclude_tag(nucleation::capi::PaletteBuilder* self, nucleation::diplomat::capi::DiplomatStringView t);

    typedef struct PaletteBuilder_kind_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_kind_result;
    PaletteBuilder_kind_result PaletteBuilder_kind(nucleation::capi::PaletteBuilder* self, nucleation::diplomat::capi::DiplomatStringView k);

    typedef struct PaletteBuilder_build_result {union {nucleation::capi::Palette* ok; nucleation::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_build_result;
    PaletteBuilder_build_result PaletteBuilder_build(nucleation::capi::PaletteBuilder* self);

    void PaletteBuilder_destroy(PaletteBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::PaletteBuilder> nucleation::PaletteBuilder::create() {
    auto result = nucleation::capi::PaletteBuilder_create();
    return std::unique_ptr<nucleation::PaletteBuilder>(nucleation::PaletteBuilder::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::exclude_falling() {
    auto result = nucleation::capi::PaletteBuilder_exclude_falling(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::exclude_tile_entities() {
    auto result = nucleation::capi::PaletteBuilder_exclude_tile_entities(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::full_blocks_only() {
    auto result = nucleation::capi::PaletteBuilder_full_blocks_only(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::exclude_needs_support() {
    auto result = nucleation::capi::PaletteBuilder_exclude_needs_support(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::exclude_transparent() {
    auto result = nucleation::capi::PaletteBuilder_exclude_transparent(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::exclude_light_sources() {
    auto result = nucleation::capi::PaletteBuilder_exclude_light_sources(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::survival_only() {
    auto result = nucleation::capi::PaletteBuilder_survival_only(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::exclude_keyword(std::string_view keyword) {
    auto result = nucleation::capi::PaletteBuilder_exclude_keyword(this->AsFFI(),
        {keyword.data(), keyword.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::include_keyword(std::string_view keyword) {
    auto result = nucleation::capi::PaletteBuilder_include_keyword(this->AsFFI(),
        {keyword.data(), keyword.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::tag(std::string_view t) {
    auto result = nucleation::capi::PaletteBuilder_tag(this->AsFFI(),
        {t.data(), t.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::exclude_tag(std::string_view t) {
    auto result = nucleation::capi::PaletteBuilder_exclude_tag(this->AsFFI(),
        {t.data(), t.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::PaletteBuilder::kind(std::string_view k) {
    auto result = nucleation::capi::PaletteBuilder_kind(this->AsFFI(),
        {k.data(), k.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError> nucleation::PaletteBuilder::build() {
    auto result = nucleation::capi::PaletteBuilder_build(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Palette>>(std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::PaletteBuilder* nucleation::PaletteBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::PaletteBuilder*>(this);
}

inline nucleation::capi::PaletteBuilder* nucleation::PaletteBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::PaletteBuilder*>(this);
}

inline const nucleation::PaletteBuilder* nucleation::PaletteBuilder::FromFFI(const nucleation::capi::PaletteBuilder* ptr) {
    return reinterpret_cast<const nucleation::PaletteBuilder*>(ptr);
}

inline nucleation::PaletteBuilder* nucleation::PaletteBuilder::FromFFI(nucleation::capi::PaletteBuilder* ptr) {
    return reinterpret_cast<nucleation::PaletteBuilder*>(ptr);
}

inline void nucleation::PaletteBuilder::operator delete(void* ptr) {
    nucleation::capi::PaletteBuilder_destroy(reinterpret_cast<nucleation::capi::PaletteBuilder*>(ptr));
}


#endif // NUCLEATION_PaletteBuilder_HPP
