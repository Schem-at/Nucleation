#ifndef NUCLEATION_ItemModelResult_HPP
#define NUCLEATION_ItemModelResult_HPP

#include "ItemModelResult.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Dimensions.hpp"
#include "ItemModelConfig.hpp"
#include "ItemModelPackBuilder.hpp"
#include "ItemScale.hpp"
#include "NucleationError.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct ItemModelResult_create_result {union {nucleation::capi::ItemModelResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_create_result;
    ItemModelResult_create_result ItemModelResult_create(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::ItemModelConfig* config);

    typedef struct ItemModelResult_model_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_model_json_result;
    ItemModelResult_model_json_result ItemModelResult_model_json(const nucleation::capi::ItemModelResult* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct ItemModelResult_element_count_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_element_count_result;
    ItemModelResult_element_count_result ItemModelResult_element_count(const nucleation::capi::ItemModelResult* self);

    typedef struct ItemModelResult_texture_count_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_texture_count_result;
    ItemModelResult_texture_count_result ItemModelResult_texture_count(const nucleation::capi::ItemModelResult* self);

    typedef struct ItemModelResult_plane_count_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_plane_count_result;
    ItemModelResult_plane_count_result ItemModelResult_plane_count(const nucleation::capi::ItemModelResult* self);

    typedef struct ItemModelResult_dimensions_result {union {nucleation::capi::Dimensions ok; nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_dimensions_result;
    ItemModelResult_dimensions_result ItemModelResult_dimensions(const nucleation::capi::ItemModelResult* self);

    typedef struct ItemModelResult_scale_result {union {nucleation::capi::ItemScale ok; nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_scale_result;
    ItemModelResult_scale_result ItemModelResult_scale(const nucleation::capi::ItemModelResult* self);

    typedef struct ItemModelResult_to_resource_pack_zip_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_to_resource_pack_zip_b64_result;
    ItemModelResult_to_resource_pack_zip_b64_result ItemModelResult_to_resource_pack_zip_b64(const nucleation::capi::ItemModelResult* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct ItemModelResult_add_to_pack_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelResult_add_to_pack_result;
    ItemModelResult_add_to_pack_result ItemModelResult_add_to_pack(nucleation::capi::ItemModelResult* self, const nucleation::capi::ItemModelPackBuilder* builder);

    void ItemModelResult_destroy(ItemModelResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelResult>, nucleation::NucleationError> nucleation::ItemModelResult::create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::ItemModelConfig& config) {
    auto result = nucleation::capi::ItemModelResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ItemModelResult>>(std::unique_ptr<nucleation::ItemModelResult>(nucleation::ItemModelResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::ItemModelResult::model_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::ItemModelResult_model_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ItemModelResult::model_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::ItemModelResult_model_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::ItemModelResult::element_count() const {
    auto result = nucleation::capi::ItemModelResult_element_count(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::ItemModelResult::texture_count() const {
    auto result = nucleation::capi::ItemModelResult_texture_count(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::ItemModelResult::plane_count() const {
    auto result = nucleation::capi::ItemModelResult_plane_count(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::Dimensions, nucleation::NucleationError> nucleation::ItemModelResult::dimensions() const {
    auto result = nucleation::capi::ItemModelResult_dimensions(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::Dimensions, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::Dimensions>(nucleation::Dimensions::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::Dimensions, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::ItemScale, nucleation::NucleationError> nucleation::ItemModelResult::scale() const {
    auto result = nucleation::capi::ItemModelResult_scale(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::ItemScale, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::ItemScale>(nucleation::ItemScale::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::ItemScale, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::ItemModelResult::to_resource_pack_zip_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::ItemModelResult_to_resource_pack_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ItemModelResult::to_resource_pack_zip_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::ItemModelResult_to_resource_pack_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ItemModelResult::add_to_pack(const nucleation::ItemModelPackBuilder& builder) {
    auto result = nucleation::capi::ItemModelResult_add_to_pack(this->AsFFI(),
        builder.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::ItemModelResult* nucleation::ItemModelResult::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ItemModelResult*>(this);
}

inline nucleation::capi::ItemModelResult* nucleation::ItemModelResult::AsFFI() {
    return reinterpret_cast<nucleation::capi::ItemModelResult*>(this);
}

inline const nucleation::ItemModelResult* nucleation::ItemModelResult::FromFFI(const nucleation::capi::ItemModelResult* ptr) {
    return reinterpret_cast<const nucleation::ItemModelResult*>(ptr);
}

inline nucleation::ItemModelResult* nucleation::ItemModelResult::FromFFI(nucleation::capi::ItemModelResult* ptr) {
    return reinterpret_cast<nucleation::ItemModelResult*>(ptr);
}

inline void nucleation::ItemModelResult::operator delete(void* ptr) {
    nucleation::capi::ItemModelResult_destroy(reinterpret_cast<nucleation::capi::ItemModelResult*>(ptr));
}


#endif // NUCLEATION_ItemModelResult_HPP
