#ifndef SchematicBuilder_HPP
#define SchematicBuilder_HPP

#include "SchematicBuilder.d.hpp"

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

    diplomat::capi::SchematicBuilder* SchematicBuilder_create(void);

    typedef struct SchematicBuilder_from_template_result {union {diplomat::capi::SchematicBuilder* ok; diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_from_template_result;
    SchematicBuilder_from_template_result SchematicBuilder_from_template(diplomat::capi::DiplomatStringView template_);

    typedef struct SchematicBuilder_name_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_name_result;
    SchematicBuilder_name_result SchematicBuilder_name(diplomat::capi::SchematicBuilder* self, diplomat::capi::DiplomatStringView name);

    typedef struct SchematicBuilder_map_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_map_result;
    SchematicBuilder_map_result SchematicBuilder_map(diplomat::capi::SchematicBuilder* self, diplomat::capi::DiplomatStringView ch, diplomat::capi::DiplomatStringView block);

    typedef struct SchematicBuilder_layers_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_layers_result;
    SchematicBuilder_layers_result SchematicBuilder_layers(diplomat::capi::SchematicBuilder* self, diplomat::capi::DiplomatStringView layers_json);

    typedef struct SchematicBuilder_layer_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_layer_result;
    SchematicBuilder_layer_result SchematicBuilder_layer(diplomat::capi::SchematicBuilder* self, diplomat::capi::DiplomatStringView rows_json);

    typedef struct SchematicBuilder_palette_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_palette_result;
    SchematicBuilder_palette_result SchematicBuilder_palette(diplomat::capi::SchematicBuilder* self, diplomat::capi::DiplomatStringView pairs_json);

    typedef struct SchematicBuilder_offset_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_offset_result;
    SchematicBuilder_offset_result SchematicBuilder_offset(diplomat::capi::SchematicBuilder* self, int32_t x, int32_t y, int32_t z);

    typedef struct SchematicBuilder_use_standard_palette_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_use_standard_palette_result;
    SchematicBuilder_use_standard_palette_result SchematicBuilder_use_standard_palette(diplomat::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_use_minimal_palette_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_use_minimal_palette_result;
    SchematicBuilder_use_minimal_palette_result SchematicBuilder_use_minimal_palette(diplomat::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_use_compact_palette_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_use_compact_palette_result;
    SchematicBuilder_use_compact_palette_result SchematicBuilder_use_compact_palette(diplomat::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_validate_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_validate_result;
    SchematicBuilder_validate_result SchematicBuilder_validate(const diplomat::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_to_template_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_to_template_result;
    SchematicBuilder_to_template_result SchematicBuilder_to_template(const diplomat::capi::SchematicBuilder* self, diplomat::capi::DiplomatWrite* write);

    typedef struct SchematicBuilder_build_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_build_result;
    SchematicBuilder_build_result SchematicBuilder_build(diplomat::capi::SchematicBuilder* self);

    void SchematicBuilder_destroy(SchematicBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<SchematicBuilder> SchematicBuilder::create() {
    auto result = diplomat::capi::SchematicBuilder_create();
    return std::unique_ptr<SchematicBuilder>(SchematicBuilder::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<SchematicBuilder>, NucleationError> SchematicBuilder::from_template(std::string_view template_) {
    auto result = diplomat::capi::SchematicBuilder_from_template({template_.data(), template_.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<SchematicBuilder>, NucleationError>(diplomat::Ok<std::unique_ptr<SchematicBuilder>>(std::unique_ptr<SchematicBuilder>(SchematicBuilder::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<SchematicBuilder>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::name(std::string_view name) {
    auto result = diplomat::capi::SchematicBuilder_name(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::map(std::string_view ch, std::string_view block) {
    auto result = diplomat::capi::SchematicBuilder_map(this->AsFFI(),
        {ch.data(), ch.size()},
        {block.data(), block.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::layers(std::string_view layers_json) {
    auto result = diplomat::capi::SchematicBuilder_layers(this->AsFFI(),
        {layers_json.data(), layers_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::layer(std::string_view rows_json) {
    auto result = diplomat::capi::SchematicBuilder_layer(this->AsFFI(),
        {rows_json.data(), rows_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::palette(std::string_view pairs_json) {
    auto result = diplomat::capi::SchematicBuilder_palette(this->AsFFI(),
        {pairs_json.data(), pairs_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::offset(int32_t x, int32_t y, int32_t z) {
    auto result = diplomat::capi::SchematicBuilder_offset(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::use_standard_palette() {
    auto result = diplomat::capi::SchematicBuilder_use_standard_palette(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::use_minimal_palette() {
    auto result = diplomat::capi::SchematicBuilder_use_minimal_palette(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::use_compact_palette() {
    auto result = diplomat::capi::SchematicBuilder_use_compact_palette(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::validate() const {
    auto result = diplomat::capi::SchematicBuilder_validate(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> SchematicBuilder::to_template() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::SchematicBuilder_to_template(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> SchematicBuilder::to_template_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::SchematicBuilder_to_template(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> SchematicBuilder::build() {
    auto result = diplomat::capi::SchematicBuilder_build(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::SchematicBuilder* SchematicBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::SchematicBuilder*>(this);
}

inline diplomat::capi::SchematicBuilder* SchematicBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::SchematicBuilder*>(this);
}

inline const SchematicBuilder* SchematicBuilder::FromFFI(const diplomat::capi::SchematicBuilder* ptr) {
    return reinterpret_cast<const SchematicBuilder*>(ptr);
}

inline SchematicBuilder* SchematicBuilder::FromFFI(diplomat::capi::SchematicBuilder* ptr) {
    return reinterpret_cast<SchematicBuilder*>(ptr);
}

inline void SchematicBuilder::operator delete(void* ptr) {
    diplomat::capi::SchematicBuilder_destroy(reinterpret_cast<diplomat::capi::SchematicBuilder*>(ptr));
}


#endif // SchematicBuilder_HPP
