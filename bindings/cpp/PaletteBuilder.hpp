#ifndef PaletteBuilder_HPP
#define PaletteBuilder_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::PaletteBuilder* PaletteBuilder_create(void);

    typedef struct PaletteBuilder_exclude_falling_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_falling_result;
    PaletteBuilder_exclude_falling_result PaletteBuilder_exclude_falling(diplomat::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_tile_entities_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_tile_entities_result;
    PaletteBuilder_exclude_tile_entities_result PaletteBuilder_exclude_tile_entities(diplomat::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_full_blocks_only_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_full_blocks_only_result;
    PaletteBuilder_full_blocks_only_result PaletteBuilder_full_blocks_only(diplomat::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_needs_support_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_needs_support_result;
    PaletteBuilder_exclude_needs_support_result PaletteBuilder_exclude_needs_support(diplomat::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_transparent_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_transparent_result;
    PaletteBuilder_exclude_transparent_result PaletteBuilder_exclude_transparent(diplomat::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_light_sources_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_light_sources_result;
    PaletteBuilder_exclude_light_sources_result PaletteBuilder_exclude_light_sources(diplomat::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_survival_only_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_survival_only_result;
    PaletteBuilder_survival_only_result PaletteBuilder_survival_only(diplomat::capi::PaletteBuilder* self);

    typedef struct PaletteBuilder_exclude_keyword_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_keyword_result;
    PaletteBuilder_exclude_keyword_result PaletteBuilder_exclude_keyword(diplomat::capi::PaletteBuilder* self, diplomat::capi::DiplomatStringView keyword);

    typedef struct PaletteBuilder_include_keyword_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_include_keyword_result;
    PaletteBuilder_include_keyword_result PaletteBuilder_include_keyword(diplomat::capi::PaletteBuilder* self, diplomat::capi::DiplomatStringView keyword);

    typedef struct PaletteBuilder_tag_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_tag_result;
    PaletteBuilder_tag_result PaletteBuilder_tag(diplomat::capi::PaletteBuilder* self, diplomat::capi::DiplomatStringView t);

    typedef struct PaletteBuilder_exclude_tag_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_exclude_tag_result;
    PaletteBuilder_exclude_tag_result PaletteBuilder_exclude_tag(diplomat::capi::PaletteBuilder* self, diplomat::capi::DiplomatStringView t);

    typedef struct PaletteBuilder_kind_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_kind_result;
    PaletteBuilder_kind_result PaletteBuilder_kind(diplomat::capi::PaletteBuilder* self, diplomat::capi::DiplomatStringView k);

    typedef struct PaletteBuilder_build_result {union {diplomat::capi::Palette* ok; diplomat::capi::NucleationError err;}; bool is_ok;} PaletteBuilder_build_result;
    PaletteBuilder_build_result PaletteBuilder_build(diplomat::capi::PaletteBuilder* self);

    void PaletteBuilder_destroy(PaletteBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<PaletteBuilder> PaletteBuilder::create() {
    auto result = diplomat::capi::PaletteBuilder_create();
    return std::unique_ptr<PaletteBuilder>(PaletteBuilder::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::exclude_falling() {
    auto result = diplomat::capi::PaletteBuilder_exclude_falling(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::exclude_tile_entities() {
    auto result = diplomat::capi::PaletteBuilder_exclude_tile_entities(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::full_blocks_only() {
    auto result = diplomat::capi::PaletteBuilder_full_blocks_only(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::exclude_needs_support() {
    auto result = diplomat::capi::PaletteBuilder_exclude_needs_support(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::exclude_transparent() {
    auto result = diplomat::capi::PaletteBuilder_exclude_transparent(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::exclude_light_sources() {
    auto result = diplomat::capi::PaletteBuilder_exclude_light_sources(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::survival_only() {
    auto result = diplomat::capi::PaletteBuilder_survival_only(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::exclude_keyword(std::string_view keyword) {
    auto result = diplomat::capi::PaletteBuilder_exclude_keyword(this->AsFFI(),
        {keyword.data(), keyword.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::include_keyword(std::string_view keyword) {
    auto result = diplomat::capi::PaletteBuilder_include_keyword(this->AsFFI(),
        {keyword.data(), keyword.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::tag(std::string_view t) {
    auto result = diplomat::capi::PaletteBuilder_tag(this->AsFFI(),
        {t.data(), t.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::exclude_tag(std::string_view t) {
    auto result = diplomat::capi::PaletteBuilder_exclude_tag(this->AsFFI(),
        {t.data(), t.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> PaletteBuilder::kind(std::string_view k) {
    auto result = diplomat::capi::PaletteBuilder_kind(this->AsFFI(),
        {k.data(), k.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Palette>, NucleationError> PaletteBuilder::build() {
    auto result = diplomat::capi::PaletteBuilder_build(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Palette>, NucleationError>(diplomat::Ok<std::unique_ptr<Palette>>(std::unique_ptr<Palette>(Palette::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Palette>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::PaletteBuilder* PaletteBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::PaletteBuilder*>(this);
}

inline diplomat::capi::PaletteBuilder* PaletteBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::PaletteBuilder*>(this);
}

inline const PaletteBuilder* PaletteBuilder::FromFFI(const diplomat::capi::PaletteBuilder* ptr) {
    return reinterpret_cast<const PaletteBuilder*>(ptr);
}

inline PaletteBuilder* PaletteBuilder::FromFFI(diplomat::capi::PaletteBuilder* ptr) {
    return reinterpret_cast<PaletteBuilder*>(ptr);
}

inline void PaletteBuilder::operator delete(void* ptr) {
    diplomat::capi::PaletteBuilder_destroy(reinterpret_cast<diplomat::capi::PaletteBuilder*>(ptr));
}


#endif // PaletteBuilder_HPP
