#ifndef ItemModelResult_HPP
#define ItemModelResult_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct ItemModelResult_create_result {union {diplomat::capi::ItemModelResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_create_result;
    ItemModelResult_create_result ItemModelResult_create(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::ItemModelConfig* config);

    typedef struct ItemModelResult_model_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_model_json_result;
    ItemModelResult_model_json_result ItemModelResult_model_json(const diplomat::capi::ItemModelResult* self, diplomat::capi::DiplomatWrite* write);

    typedef struct ItemModelResult_element_count_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_element_count_result;
    ItemModelResult_element_count_result ItemModelResult_element_count(const diplomat::capi::ItemModelResult* self);

    typedef struct ItemModelResult_texture_count_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_texture_count_result;
    ItemModelResult_texture_count_result ItemModelResult_texture_count(const diplomat::capi::ItemModelResult* self);

    typedef struct ItemModelResult_plane_count_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_plane_count_result;
    ItemModelResult_plane_count_result ItemModelResult_plane_count(const diplomat::capi::ItemModelResult* self);

    typedef struct ItemModelResult_dimensions_result {union {diplomat::capi::Dimensions ok; diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_dimensions_result;
    ItemModelResult_dimensions_result ItemModelResult_dimensions(const diplomat::capi::ItemModelResult* self);

    typedef struct ItemModelResult_scale_result {union {diplomat::capi::ItemScale ok; diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_scale_result;
    ItemModelResult_scale_result ItemModelResult_scale(const diplomat::capi::ItemModelResult* self);

    typedef struct ItemModelResult_to_resource_pack_zip_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_to_resource_pack_zip_b64_result;
    ItemModelResult_to_resource_pack_zip_b64_result ItemModelResult_to_resource_pack_zip_b64(const diplomat::capi::ItemModelResult* self, diplomat::capi::DiplomatWrite* write);

    typedef struct ItemModelResult_add_to_pack_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelResult_add_to_pack_result;
    ItemModelResult_add_to_pack_result ItemModelResult_add_to_pack(diplomat::capi::ItemModelResult* self, const diplomat::capi::ItemModelPackBuilder* builder);

    void ItemModelResult_destroy(ItemModelResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<ItemModelResult>, NucleationError> ItemModelResult::create(const Schematic& schematic, const ResourcePack& pack, const ItemModelConfig& config) {
    auto result = diplomat::capi::ItemModelResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<ItemModelResult>, NucleationError>(diplomat::Ok<std::unique_ptr<ItemModelResult>>(std::unique_ptr<ItemModelResult>(ItemModelResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ItemModelResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> ItemModelResult::model_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::ItemModelResult_model_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> ItemModelResult::model_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::ItemModelResult_model_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> ItemModelResult::element_count() const {
    auto result = diplomat::capi::ItemModelResult_element_count(this->AsFFI());
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> ItemModelResult::texture_count() const {
    auto result = diplomat::capi::ItemModelResult_texture_count(this->AsFFI());
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> ItemModelResult::plane_count() const {
    auto result = diplomat::capi::ItemModelResult_plane_count(this->AsFFI());
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<Dimensions, NucleationError> ItemModelResult::dimensions() const {
    auto result = diplomat::capi::ItemModelResult_dimensions(this->AsFFI());
    return result.is_ok ? diplomat::result<Dimensions, NucleationError>(diplomat::Ok<Dimensions>(Dimensions::FromFFI(result.ok))) : diplomat::result<Dimensions, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<ItemScale, NucleationError> ItemModelResult::scale() const {
    auto result = diplomat::capi::ItemModelResult_scale(this->AsFFI());
    return result.is_ok ? diplomat::result<ItemScale, NucleationError>(diplomat::Ok<ItemScale>(ItemScale::FromFFI(result.ok))) : diplomat::result<ItemScale, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> ItemModelResult::to_resource_pack_zip_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::ItemModelResult_to_resource_pack_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> ItemModelResult::to_resource_pack_zip_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::ItemModelResult_to_resource_pack_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ItemModelResult::add_to_pack(const ItemModelPackBuilder& builder) {
    auto result = diplomat::capi::ItemModelResult_add_to_pack(this->AsFFI(),
        builder.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::ItemModelResult* ItemModelResult::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ItemModelResult*>(this);
}

inline diplomat::capi::ItemModelResult* ItemModelResult::AsFFI() {
    return reinterpret_cast<diplomat::capi::ItemModelResult*>(this);
}

inline const ItemModelResult* ItemModelResult::FromFFI(const diplomat::capi::ItemModelResult* ptr) {
    return reinterpret_cast<const ItemModelResult*>(ptr);
}

inline ItemModelResult* ItemModelResult::FromFFI(diplomat::capi::ItemModelResult* ptr) {
    return reinterpret_cast<ItemModelResult*>(ptr);
}

inline void ItemModelResult::operator delete(void* ptr) {
    diplomat::capi::ItemModelResult_destroy(reinterpret_cast<diplomat::capi::ItemModelResult*>(ptr));
}


#endif // ItemModelResult_HPP
