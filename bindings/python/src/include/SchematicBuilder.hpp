#ifndef NUCLEATION_SchematicBuilder_HPP
#define NUCLEATION_SchematicBuilder_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::SchematicBuilder* SchematicBuilder_create(void);

    typedef struct SchematicBuilder_from_template_result {union {nucleation::capi::SchematicBuilder* ok; nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_from_template_result;
    SchematicBuilder_from_template_result SchematicBuilder_from_template(nucleation::diplomat::capi::DiplomatStringView template_);

    typedef struct SchematicBuilder_name_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_name_result;
    SchematicBuilder_name_result SchematicBuilder_name(nucleation::capi::SchematicBuilder* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct SchematicBuilder_map_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_map_result;
    SchematicBuilder_map_result SchematicBuilder_map(nucleation::capi::SchematicBuilder* self, nucleation::diplomat::capi::DiplomatStringView ch, nucleation::diplomat::capi::DiplomatStringView block);

    typedef struct SchematicBuilder_layers_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_layers_result;
    SchematicBuilder_layers_result SchematicBuilder_layers(nucleation::capi::SchematicBuilder* self, nucleation::diplomat::capi::DiplomatStringView layers_json);

    typedef struct SchematicBuilder_layer_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_layer_result;
    SchematicBuilder_layer_result SchematicBuilder_layer(nucleation::capi::SchematicBuilder* self, nucleation::diplomat::capi::DiplomatStringView rows_json);

    typedef struct SchematicBuilder_palette_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_palette_result;
    SchematicBuilder_palette_result SchematicBuilder_palette(nucleation::capi::SchematicBuilder* self, nucleation::diplomat::capi::DiplomatStringView pairs_json);

    typedef struct SchematicBuilder_offset_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_offset_result;
    SchematicBuilder_offset_result SchematicBuilder_offset(nucleation::capi::SchematicBuilder* self, int32_t x, int32_t y, int32_t z);

    typedef struct SchematicBuilder_use_standard_palette_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_use_standard_palette_result;
    SchematicBuilder_use_standard_palette_result SchematicBuilder_use_standard_palette(nucleation::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_use_minimal_palette_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_use_minimal_palette_result;
    SchematicBuilder_use_minimal_palette_result SchematicBuilder_use_minimal_palette(nucleation::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_use_compact_palette_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_use_compact_palette_result;
    SchematicBuilder_use_compact_palette_result SchematicBuilder_use_compact_palette(nucleation::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_validate_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_validate_result;
    SchematicBuilder_validate_result SchematicBuilder_validate(const nucleation::capi::SchematicBuilder* self);

    typedef struct SchematicBuilder_to_template_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_to_template_result;
    SchematicBuilder_to_template_result SchematicBuilder_to_template(const nucleation::capi::SchematicBuilder* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct SchematicBuilder_build_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} SchematicBuilder_build_result;
    SchematicBuilder_build_result SchematicBuilder_build(nucleation::capi::SchematicBuilder* self);

    void SchematicBuilder_destroy(SchematicBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::SchematicBuilder> nucleation::SchematicBuilder::create() {
    auto result = nucleation::capi::SchematicBuilder_create();
    return std::unique_ptr<nucleation::SchematicBuilder>(nucleation::SchematicBuilder::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::SchematicBuilder>, nucleation::NucleationError> nucleation::SchematicBuilder::from_template(std::string_view template_) {
    auto result = nucleation::capi::SchematicBuilder_from_template({template_.data(), template_.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::SchematicBuilder>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::SchematicBuilder>>(std::unique_ptr<nucleation::SchematicBuilder>(nucleation::SchematicBuilder::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::SchematicBuilder>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::name(std::string_view name) {
    auto result = nucleation::capi::SchematicBuilder_name(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::map(std::string_view ch, std::string_view block) {
    auto result = nucleation::capi::SchematicBuilder_map(this->AsFFI(),
        {ch.data(), ch.size()},
        {block.data(), block.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::layers(std::string_view layers_json) {
    auto result = nucleation::capi::SchematicBuilder_layers(this->AsFFI(),
        {layers_json.data(), layers_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::layer(std::string_view rows_json) {
    auto result = nucleation::capi::SchematicBuilder_layer(this->AsFFI(),
        {rows_json.data(), rows_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::palette(std::string_view pairs_json) {
    auto result = nucleation::capi::SchematicBuilder_palette(this->AsFFI(),
        {pairs_json.data(), pairs_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::offset(int32_t x, int32_t y, int32_t z) {
    auto result = nucleation::capi::SchematicBuilder_offset(this->AsFFI(),
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::use_standard_palette() {
    auto result = nucleation::capi::SchematicBuilder_use_standard_palette(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::use_minimal_palette() {
    auto result = nucleation::capi::SchematicBuilder_use_minimal_palette(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::use_compact_palette() {
    auto result = nucleation::capi::SchematicBuilder_use_compact_palette(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::validate() const {
    auto result = nucleation::capi::SchematicBuilder_validate(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::SchematicBuilder::to_template() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::SchematicBuilder_to_template(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicBuilder::to_template_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::SchematicBuilder_to_template(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::SchematicBuilder::build() {
    auto result = nucleation::capi::SchematicBuilder_build(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::SchematicBuilder* nucleation::SchematicBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::SchematicBuilder*>(this);
}

inline nucleation::capi::SchematicBuilder* nucleation::SchematicBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::SchematicBuilder*>(this);
}

inline const nucleation::SchematicBuilder* nucleation::SchematicBuilder::FromFFI(const nucleation::capi::SchematicBuilder* ptr) {
    return reinterpret_cast<const nucleation::SchematicBuilder*>(ptr);
}

inline nucleation::SchematicBuilder* nucleation::SchematicBuilder::FromFFI(nucleation::capi::SchematicBuilder* ptr) {
    return reinterpret_cast<nucleation::SchematicBuilder*>(ptr);
}

inline void nucleation::SchematicBuilder::operator delete(void* ptr) {
    nucleation::capi::SchematicBuilder_destroy(reinterpret_cast<nucleation::capi::SchematicBuilder*>(ptr));
}


#endif // NUCLEATION_SchematicBuilder_HPP
